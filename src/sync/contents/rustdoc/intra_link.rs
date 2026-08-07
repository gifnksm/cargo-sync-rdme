use std::{
    borrow::Cow,
    collections::{HashMap, hash_map},
    rc::Rc,
};

use pulldown_cmark::{BrokenLink, CowStr, Event, Options, Tag};
use rustdoc_types::{Crate, Id, Item, ItemEnum, ItemKind, ItemSummary};

trait CowStrExt<'a> {
    fn as_str(&'a self) -> &'a str;
}

impl<'a> CowStrExt<'a> for CowStr<'a> {
    fn as_str(&'a self) -> &'a str {
        match self {
            CowStr::Boxed(s) => s,
            CowStr::Borrowed(s) => s,
            CowStr::Inlined(s) => s,
        }
    }
}

#[derive(Debug)]
pub(super) struct Parser<B, M> {
    broken_link_callback: B,
    iterator_map: M,
}

type BrokenLinkPair<'a> = (CowStr<'a>, CowStr<'a>);

impl Parser<(), ()> {
    pub(super) fn new<'doc>(
        doc: &'doc Crate,
        item: &'doc Item,
        local_html_root_url: &'doc str,
        mappings: &'doc HashMap<String, String>,
    ) -> Parser<
        impl FnMut(BrokenLink<'_>) -> Option<BrokenLinkPair<'doc>>,
        impl FnMut(Event<'doc>) -> Option<Event<'doc>>,
    > {
        let resolver = LinkResolver::new(doc, local_html_root_url, mappings);
        let url_map = Rc::new(resolve_all_links(&resolver, &item.links));

        let broken_link_callback = {
            let url_map = Rc::clone(&url_map);
            move |link: BrokenLink<'_>| {
                let url = url_map.get(link.reference.as_str())?.as_ref()?;
                Some((url.to_owned().into(), "".into()))
            }
        };
        let iterator_map = move |event| convert_link(&url_map, event);

        Parser {
            broken_link_callback,
            iterator_map,
        }
    }
}

impl<'doc, B, M> Parser<B, M>
where
    B: FnMut(BrokenLink<'_>) -> Option<BrokenLinkPair<'doc>> + 'doc,
    M: FnMut(Event<'doc>) -> Option<Event<'doc>> + 'doc,
{
    pub(super) fn events<'b>(&'b mut self, doc: &'doc str) -> impl Iterator<Item = Event<'doc>> + 'b
    where
        'doc: 'b,
    {
        pulldown_cmark::Parser::new_with_broken_link_callback(
            doc,
            Options::all(),
            Some(&mut self.broken_link_callback),
        )
        .filter_map(&mut self.iterator_map)
    }
}

fn resolve_all_links<'doc>(
    resolver: &LinkResolver<'doc>,
    links: &'doc HashMap<String, Id>,
) -> HashMap<&'doc str, Option<String>> {
    links
        .iter()
        .map(move |(name, id)| (name.as_str(), resolver.resolve_link(name, *id)))
        .collect()
}

fn convert_link<'a>(
    url_map: &HashMap<&str, Option<String>>,
    mut event: Event<'a>,
) -> Option<Event<'a>> {
    if let Event::Start(Tag::Link { dest_url: url, .. }) = &mut event
        && let Some(full_url) = url_map.get(url.as_ref())
    {
        *url = full_url.as_ref()?.to_owned().into();
    }
    Some(event)
}

type CrateId = u32;

#[derive(Debug)]
struct LinkResolver<'doc> {
    doc: &'doc Crate,
    local_html_root_url: &'doc str,
    mappings: &'doc HashMap<String, String>,
    per_crate_resolved_paths: HashMap<CrateId, HashMap<&'doc [String], (Id, &'doc ItemSummary)>>,
    fallback_resolved_path: HashMap<&'doc [String], (CrateId, Id, &'doc ItemSummary)>,
}

impl<'doc> LinkResolver<'doc> {
    fn new(
        doc: &'doc Crate,
        local_html_root_url: &'doc str,
        mappings: &'doc HashMap<String, String>,
    ) -> Self {
        let mut per_crate_resolved_paths = HashMap::new();
        let mut fallback_resolved_path = HashMap::new();
        for (id, summary) in &doc.paths {
            let crate_id = summary.crate_id;
            let path = summary.path.as_slice();
            // TODO: Handle the case where there are multiple items with the same path but different kinds.
            //
            // Rust has different namespaces for different kinds of items, so there can be multiple items with the same path but different kinds,
            // such as `std::i32` (primitive type and module) or `std::clone::Clone` (trait and derive).
            // For now, we will just use the one with smaller ID, and emit a debug log if we find another one with the same path.
            match per_crate_resolved_paths
                .entry(crate_id)
                .or_insert_with(HashMap::new)
                .entry(path)
            {
                hash_map::Entry::Vacant(e) => {
                    e.insert((*id, summary));
                }
                hash_map::Entry::Occupied(mut e) => {
                    let (existing_id, existing_summary) = e.get();
                    tracing::debug!(
                        crate = display_crate(doc, crate_id).as_ref(),
                        path = path.join("::"),
                        existing = ?(existing_id, existing_summary.kind),
                        new = ?(*id, summary.kind),
                        "multiple items with the same path, using the one with smaller ID",
                    );
                    if *existing_id > *id {
                        e.insert((*id, summary));
                    }
                }
            }
            match fallback_resolved_path.entry(path) {
                hash_map::Entry::Vacant(e) => {
                    e.insert((crate_id, *id, summary));
                }
                hash_map::Entry::Occupied(mut e) => {
                    let (existing_crate_id, existing_id, existing_summary) = e.get();
                    tracing::debug!(
                        path = path.join("::"),
                        existing = ?(display_crate(doc, *existing_crate_id).as_ref(), existing_id, existing_summary.kind),
                        new = ?(display_crate(doc, crate_id).as_ref(), id.0, summary.kind),
                        "multiple items with the same path in different crates, using the one with smaller crate ID and smaller item ID",
                    );
                    if *existing_crate_id > crate_id
                        || (*existing_crate_id == crate_id && existing_id > id)
                    {
                        e.insert((crate_id, *id, summary));
                    }
                }
            }
        }
        Self {
            doc,
            local_html_root_url,
            mappings,
            per_crate_resolved_paths,
            fallback_resolved_path,
        }
    }

    #[tracing::instrument(skip_all, fields(name = name, id = ?id))]
    fn resolve_link(&self, name: &str, id: Id) -> Option<String> {
        if let Some(path) = self.mappings.get(name) {
            return Some(path.clone());
        }
        let Some(url) = self.build_url_for_item(id) else {
            tracing::warn!("failed to resolve link");
            return None;
        };
        Some(url)
    }

    fn build_url_for_item(&self, id: Id) -> Option<String> {
        let summary = self.doc.paths.get(&id)?;
        let target = self.build_link_target(id, summary)?;
        let relative_path = target.build_relative_path();
        let mut url = self.base_url(summary.crate_id)?.to_owned();
        if !url.ends_with('/') {
            url.push('/');
        }
        url.push_str(&relative_path);
        Some(url)
    }

    fn build_link_target_from_path(
        &self,
        crate_id: CrateId,
        path: &[String],
    ) -> Option<LinkTarget<'doc>> {
        let (id, summary) = self.find_path_summary(crate_id, path)?;
        self.build_link_target(id, summary)
    }

    fn expect_container_link_target(
        &self,
        crate_id: CrateId,
        path: &'doc [String],
        kind: ItemKind,
    ) -> Option<(LinkTarget<'doc>, &'doc String)> {
        let [container_path @ .., item] = path else {
            return warn_unexpected_path_for_kind(kind, path);
        };
        let Some(container) = self.build_link_target_from_path(crate_id, container_path) else {
            return warn_missing_container_information(kind, path);
        };
        Some((container, item))
    }

    fn build_link_target(&self, id: Id, summary: &'doc ItemSummary) -> Option<LinkTarget<'doc>> {
        let crate_id = summary.crate_id;
        let path = summary.path.as_slice();
        let kind = summary.kind;
        #[expect(clippy::match_same_arms)]
        match kind {
            ItemKind::Module => Some(LinkTarget::module(path)),
            // References to `extern crate` items are resolved to the crate root, which is already handled by the `Module` case above.
            ItemKind::ExternCrate => warn_not_supported_kind(kind, path),
            // References to `use` items (re-exported items) are resolved to the target of the `use`, which is already handled by the other cases above.
            ItemKind::Use => warn_not_supported_kind(kind, path),
            ItemKind::Struct => LinkItemKind::Struct.with_path(path),
            ItemKind::StructField => {
                let [container_path @ .., field] = path else {
                    return warn_unexpected_path_for_kind(kind, path);
                };
                // struct, union or enum variant field
                if let Some(c) = self.build_link_target_from_path(crate_id, container_path) {
                    return c.with_field(field);
                }
                // In some cases, rustdoc does not provide path information for enum variant.
                // To work around this, we fall back to the last two segments of the path as the variant and field names when the parent of parent is an enum.
                if let [path @ .., variant] = container_path
                    && let Some(c) = self.build_link_target_from_path(crate_id, path)
                {
                    return c.with_variant_field(variant, field);
                }
                warn_missing_container_information(kind, path)
            }
            ItemKind::Union => LinkItemKind::Union.with_path(path),
            ItemKind::Enum => LinkItemKind::Enum.with_path(path),
            ItemKind::Variant => {
                if let [module @ .., item, variant] = path {
                    return Some(LinkTarget::enum_variant(module, item, variant));
                }
                warn_unexpected_path_for_kind(kind, path)
            }
            ItemKind::Function => {
                let has_body = self.doc.index.get(&id).and_then(|item| match &item.inner {
                    ItemEnum::Function(f) => Some(f.has_body),
                    _ => None,
                });
                let [container_path @ .., function] = path else {
                    return warn_unexpected_path_for_kind(kind, path);
                };
                // trait or impl method / associated function
                if let Some(c) = self.build_link_target_from_path(crate_id, container_path) {
                    return c.with_function(function, has_body);
                }
                tracing::warn!(
                    path = path.join("::"),
                    ?kind,
                    "container information for the item is missing, falling back to free function",
                );
                LinkItemKind::Function.with_path(path)
            }
            ItemKind::TypeAlias => LinkItemKind::TypeAlias.with_path(path),
            ItemKind::Constant => LinkItemKind::Constant.with_path(path),
            ItemKind::Trait => LinkItemKind::Trait.with_path(path),
            // Trait aliases are unstable features (`trait_alias`), and we don't support them yet.
            // Tracking issue: <https://github.com/rust-lang/rust/issues/41517>
            ItemKind::TraitAlias => warn_not_supported_kind(kind, path),
            // Impl blocks have dedicated sections (`#implementations`), but there is no way to create an intra-doc link to it.
            ItemKind::Impl => warn_not_supported_kind(kind, path),
            ItemKind::Static => LinkItemKind::Static.with_path(path),
            // Extern types are unstable features (`extern_types`), and we don't support them yet.
            // Tracking issue: <https://github.com/rust-lang/rust/issues/43467>
            ItemKind::ExternType => warn_not_supported_kind(kind, path),
            ItemKind::Macro => LinkItemKind::Macro.with_path(path),
            ItemKind::ProcAttribute => LinkItemKind::ProcAttribute.with_path(path),
            ItemKind::ProcDerive => LinkItemKind::ProcDerive.with_path(path),
            ItemKind::AssocConst => {
                let (container, constant) =
                    self.expect_container_link_target(crate_id, path, kind)?;
                container.with_assoc_const(constant)
            }
            ItemKind::AssocType => {
                let (container, ty) = self.expect_container_link_target(crate_id, path, kind)?;
                container.with_assoc_type(ty)
            }
            ItemKind::Primitive => LinkItemKind::Primitive.with_path(path),
            // Keywords have dedicated pages in `std` and `core`, but there is no way to create an intra-doc link to it.
            ItemKind::Keyword => warn_not_supported_kind(kind, path),
            // Attributes do not have dedicated pages, and there is no way to create an intra-doc link to it.
            ItemKind::Attribute => warn_not_supported_kind(kind, path),
        }
    }

    fn find_path_summary(
        &self,
        crate_id: CrateId,
        path: &[String],
    ) -> Option<(Id, &'doc ItemSummary)> {
        if let Some((id, summary)) = self
            .per_crate_resolved_paths
            .get(&crate_id)
            .and_then(|crate_paths| crate_paths.get(path))
            .copied()
        {
            return Some((id, summary));
        }

        // For some reason, rustdoc sometimes has inconsistent crate IDs for the ancestor paths (e.g. `std` vs `core`), which causes the path to not be found in the expected crate.
        // To work around this, we fall back to an item with the same path in another crate, and log a warning.
        // <https://github.com/rust-lang/rust/issues/160665>
        if let Some((found_crate_id, id, summary)) = self.fallback_resolved_path.get(path).copied()
        {
            tracing::warn!(
                path = path.join("::"),
                expected = display_crate(self.doc, crate_id).as_ref(),
                found = display_crate(self.doc, found_crate_id).as_ref(),
                kind = ?summary.kind,
                "path not found in expected crate; falling back to another crate with the same path",
            );
            return Some((id, summary));
        }

        None
    }

    fn base_url(&self, crate_id: CrateId) -> Option<&'doc str> {
        if crate_id == 0 {
            return Some(self.local_html_root_url);
        }
        let external_crate = self.doc.external_crates.get(&crate_id)?;
        external_crate.html_root_url.as_deref()
    }
}

fn warn_not_supported_kind<T>(kind: ItemKind, path: &[String]) -> Option<T> {
    tracing::warn!(
        path = path.join("::"),
        ?kind,
        "items of this kind are not supported yet"
    );
    None
}

fn warn_unexpected_path_for_kind<T>(kind: ItemKind, path: &[String]) -> Option<T> {
    tracing::warn!(
        path = path.join("::"),
        ?kind,
        "unexpected path for the item"
    );
    None
}

fn warn_missing_container_information<T>(kind: ItemKind, path: &[String]) -> Option<T> {
    tracing::warn!(
        path = path.join("::"),
        ?kind,
        "container information for the item is missing",
    );
    None
}

fn resolve_crate_name(doc: &Crate, crate_id: CrateId) -> Option<&str> {
    if crate_id == 0 {
        return doc.index.get(&doc.root)?.name.as_deref();
    }
    doc.external_crates.get(&crate_id).map(|c| c.name.as_str())
}

fn display_crate(doc: &Crate, crate_id: CrateId) -> Cow<'_, str> {
    resolve_crate_name(doc, crate_id).map_or_else(
        || Cow::Owned(format!("<unknown crate #{crate_id}>")),
        Cow::Borrowed,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkItemKind {
    Struct,
    Union,
    Enum,
    Function,
    TypeAlias,
    Constant,
    Trait,
    Static,
    Macro,
    ProcAttribute,
    ProcDerive,
    Primitive,
}

impl LinkItemKind {
    fn with_path(self, path: &[String]) -> Option<LinkTarget<'_>> {
        let Some((item, module)) = path.split_last() else {
            return warn_unexpected_path_for_kind(self.as_item_kind(), path);
        };
        let kind = self;
        Some(LinkTarget::Item { kind, item, module })
    }

    fn has_inherent_methods(self) -> bool {
        match self {
            LinkItemKind::Struct
            | LinkItemKind::Union
            | LinkItemKind::Enum
            | LinkItemKind::Primitive => true,
            LinkItemKind::Function
            | LinkItemKind::TypeAlias
            | LinkItemKind::Constant
            | LinkItemKind::Trait
            | LinkItemKind::Static
            | LinkItemKind::Macro
            | LinkItemKind::ProcAttribute
            | LinkItemKind::ProcDerive => false,
        }
    }

    fn has_assoc_items(self) -> bool {
        self == LinkItemKind::Trait || self.has_inherent_methods()
    }

    fn as_item_kind(self) -> ItemKind {
        match self {
            LinkItemKind::Struct => ItemKind::Struct,
            LinkItemKind::Union => ItemKind::Union,
            LinkItemKind::Enum => ItemKind::Enum,
            LinkItemKind::Function => ItemKind::Function,
            LinkItemKind::TypeAlias => ItemKind::TypeAlias,
            LinkItemKind::Constant => ItemKind::Constant,
            LinkItemKind::Trait => ItemKind::Trait,
            LinkItemKind::Static => ItemKind::Static,
            LinkItemKind::Macro => ItemKind::Macro,
            LinkItemKind::ProcAttribute => ItemKind::ProcAttribute,
            LinkItemKind::ProcDerive => ItemKind::ProcDerive,
            LinkItemKind::Primitive => ItemKind::Primitive,
        }
    }

    fn namespace(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Union => "union",
            Self::Enum => "enum",
            Self::Function => "fn",
            Self::TypeAlias => "type",
            Self::Constant => "constant",
            Self::Trait => "trait",
            Self::Static => "static",
            Self::Macro => "macro",
            Self::ProcAttribute => "attr",
            Self::ProcDerive => "derive",
            Self::Primitive => "primitive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnchorKind {
    StructField,
    EnumVariant,
    EnumVariantField,
    RequiredMethod,
    ProvidedMethod,
    ImplementedMethod,
    AssocConst,
    AssocType,
}

impl AnchorKind {
    fn as_item_kind(self) -> ItemKind {
        match self {
            AnchorKind::StructField | AnchorKind::EnumVariantField => ItemKind::StructField,
            AnchorKind::EnumVariant => ItemKind::Variant,
            AnchorKind::RequiredMethod
            | AnchorKind::ProvidedMethod
            | AnchorKind::ImplementedMethod => ItemKind::Function,
            AnchorKind::AssocConst => ItemKind::AssocConst,
            AnchorKind::AssocType => ItemKind::AssocType,
        }
    }

    fn namespace(self) -> &'static str {
        match self {
            Self::StructField => "structfield",
            Self::EnumVariant => "variant",
            Self::EnumVariantField => "field",
            Self::RequiredMethod => "tymethod",
            Self::ProvidedMethod | Self::ImplementedMethod => "method",
            Self::AssocConst => "associatedconstant",
            Self::AssocType => "associatedtype",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum LinkTarget<'doc> {
    Module {
        module: &'doc [String],
    },
    Item {
        kind: LinkItemKind,
        module: &'doc [String],
        item: &'doc str,
    },
    AnchoredItem {
        kind: LinkItemKind,
        module: &'doc [String],
        item: &'doc str,
        anchor: [(AnchorKind, &'doc str); 1],
    },
    NestedAnchoredItem {
        kind: LinkItemKind,
        module: &'doc [String],
        item: &'doc str,
        anchors: [(AnchorKind, &'doc str); 2],
    },
}

impl<'doc> LinkTarget<'doc> {
    fn as_item_kind(self) -> ItemKind {
        match self {
            LinkTarget::Module { .. } => ItemKind::Module,
            LinkTarget::Item { kind, .. } => kind.as_item_kind(),
            LinkTarget::AnchoredItem {
                anchor: [(kind, _)],
                ..
            }
            | LinkTarget::NestedAnchoredItem {
                anchors: [_, (kind, _)],
                ..
            } => kind.as_item_kind(),
        }
    }

    fn display_path(self) -> String {
        match self {
            LinkTarget::Module { module } => module.join("::"),
            LinkTarget::Item {
                kind: _,
                module,
                item,
            } => format!("{}::{item}", module.join("::")),
            LinkTarget::AnchoredItem {
                kind: _,
                module,
                item,
                anchor: [(_, a0)],
            } => format!("{}::{item}::{a0}", module.join("::")),
            LinkTarget::NestedAnchoredItem {
                kind: _,
                module,
                item,
                anchors: [(_, a0), (_, a1)],
            } => format!("{}::{item}::{a0}::{a1}", module.join("::")),
        }
    }

    fn warn_unexpected_container_for_item<T>(self, kind: ItemKind, item: &str) -> Option<T> {
        tracing::warn!(
            path = format!("{}::{item}", self.display_path()),
            container_kind = ?self.as_item_kind(),
            ?kind,
            "unexpected container kind for the item",
        );
        None
    }

    fn module(module: &'doc [String]) -> Self {
        Self::Module { module }
    }

    fn enum_variant(module: &'doc [String], item: &'doc str, variant: &'doc str) -> Self {
        Self::AnchoredItem {
            kind: LinkItemKind::Enum,
            module,
            item,
            anchor: [(AnchorKind::EnumVariant, variant)],
        }
    }

    fn with_field(self, field: &'doc str) -> Option<Self> {
        match self {
            Self::Item {
                kind: kind @ (LinkItemKind::Struct | LinkItemKind::Union),
                module,
                item,
            } => Some(Self::AnchoredItem {
                kind,
                module,
                item,
                anchor: [(AnchorKind::StructField, field)],
            }),
            Self::AnchoredItem {
                kind: kind @ LinkItemKind::Enum,
                module,
                item,
                anchor: [(AnchorKind::EnumVariant, variant)],
            } => Some(Self::NestedAnchoredItem {
                kind,
                module,
                item,
                anchors: [
                    (AnchorKind::EnumVariant, variant),
                    (AnchorKind::EnumVariantField, field),
                ],
            }),
            _ => self.warn_unexpected_container_for_item(ItemKind::StructField, field),
        }
    }

    fn with_variant_field(self, variant: &'doc str, field: &'doc str) -> Option<Self> {
        let Self::Item {
            kind: kind @ LinkItemKind::Enum,
            module,
            item,
        } = self
        else {
            tracing::warn!(
                path = format!("{}::{variant}::{field}", self.display_path()),
                container_kind = ?self.as_item_kind(),
                "unexpected container kind for the variant field",
            );
            return None;
        };
        Some(Self::NestedAnchoredItem {
            kind,
            module,
            item,
            anchors: [
                (AnchorKind::EnumVariant, variant),
                (AnchorKind::EnumVariantField, field),
            ],
        })
    }

    fn with_function(self, function: &'doc str, has_body: Option<bool>) -> Option<Self> {
        let target = match self {
            Self::Module { module } => Some(Self::Item {
                kind: LinkItemKind::Function,
                module,
                item: function,
            }),
            Self::Item { kind, module, item } => {
                let anchor = match (kind, has_body) {
                    (LinkItemKind::Trait, Some(true)) => Some(AnchorKind::ProvidedMethod),
                    (LinkItemKind::Trait, Some(false)) => Some(AnchorKind::RequiredMethod),
                    (LinkItemKind::Trait, None) => {
                        // In some cases, rustdoc does not provide information about whether a trait method is required or provided.
                        // In those cases, we log a warning and default to `RequiredMethod`.
                        // <https://github.com/rust-lang/rust/issues/160662>
                        tracing::warn!(
                            path = format!("{}::{item}::{function}", module.join("::")),
                            "failed to determine if trait method is required or provided",
                        );
                        Some(AnchorKind::RequiredMethod)
                    }
                    (kind, _) if kind.has_inherent_methods() => Some(AnchorKind::ImplementedMethod),
                    (_, _) => None,
                };
                anchor.map(|anchor| Self::AnchoredItem {
                    kind,
                    module,
                    item,
                    anchor: [(anchor, function)],
                })
            }
            _ => None,
        };
        let Some(target) = target else {
            return self.warn_unexpected_container_for_item(ItemKind::Function, function);
        };
        Some(target)
    }

    fn with_assoc_const(self, constant: &'doc str) -> Option<Self> {
        match self {
            Self::Item { kind, module, item } if kind.has_assoc_items() => {
                Some(Self::AnchoredItem {
                    kind,
                    module,
                    item,
                    anchor: [(AnchorKind::AssocConst, constant)],
                })
            }
            _ => self.warn_unexpected_container_for_item(ItemKind::AssocConst, constant),
        }
    }

    fn with_assoc_type(self, ty: &'doc str) -> Option<Self> {
        match self {
            Self::Item { kind, module, item } if kind.has_assoc_items() => {
                Some(Self::AnchoredItem {
                    kind,
                    module,
                    item,
                    anchor: [(AnchorKind::AssocType, ty)],
                })
            }
            _ => self.warn_unexpected_container_for_item(ItemKind::AssocType, ty),
        }
    }
}

impl LinkTarget<'_> {
    fn build_relative_path(&self) -> String {
        match self {
            LinkTarget::Module { module } => {
                let module = module.join("/");
                format!("{module}/index.html")
            }
            LinkTarget::Item { kind, module, item } => {
                let module = module.join("/");
                let namespace = kind.namespace();
                format!("{module}/{namespace}.{item}.html")
            }
            LinkTarget::AnchoredItem {
                kind,
                module,
                item,
                anchor: [(a0_kind, a0_name)],
            } => {
                let module = module.join("/");
                let namespace = kind.namespace();
                let a0_namespace = a0_kind.namespace();
                format!("{module}/{namespace}.{item}.html#{a0_namespace}.{a0_name}")
            }
            LinkTarget::NestedAnchoredItem {
                kind,
                module,
                item,
                anchors: [(a0_kind, a0_name), (a1_kind, a1_name)],
            } => {
                let module = module.join("/");
                let namespace = kind.namespace();
                let a0_namespace = a0_kind.namespace();
                let a1_namespace = a1_kind.namespace();
                format!(
                    "{module}/{namespace}.{item}.html#{a0_namespace}.{a0_name}.{a1_namespace}.{a1_name}"
                )
            }
        }
    }
}
