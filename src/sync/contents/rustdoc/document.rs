use std::{
    borrow::Cow,
    collections::{HashMap, hash_map},
};

use rustdoc_types::{Crate, Id, Item, ItemEnum, ItemKind, ItemSummary};

type CrateId = u32;
const LOCAL_CRATE_ID: CrateId = 0;

#[derive(Debug)]
pub(super) struct RustdocDocument {
    doc: Crate,
}

impl RustdocDocument {
    pub(super) fn new(doc: Crate) -> Self {
        Self { doc }
    }

    pub(super) fn intra_link_resolver(&self) -> IntraLinkResolver<'_> {
        IntraLinkResolver::new(&self.doc)
    }

    pub(super) fn root_item(&self) -> Option<&Item> {
        self.doc.index.get(&self.doc.root)
    }
}

#[derive(Debug, Clone, Copy)]
struct FunctionKind {
    is_method: bool,
    has_body: bool,
}

#[derive(Debug)]
pub(super) struct IntraLinkResolver<'doc> {
    doc: &'doc Crate,
    per_crate_resolved_paths: HashMap<CrateId, HashMap<&'doc [String], (Id, &'doc ItemSummary)>>,
    fallback_resolved_paths: HashMap<&'doc [String], (CrateId, Id, &'doc ItemSummary)>,
}

impl<'doc> IntraLinkResolver<'doc> {
    fn new(doc: &'doc Crate) -> Self {
        let mut per_crate_resolved_paths = HashMap::new();
        let mut fallback_resolved_paths = HashMap::new();
        for (id, summary) in &doc.paths {
            let crate_id = summary.crate_id;
            let crate_ = LinkTargetCrate::new(doc, crate_id);
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
                        crate = crate_.display_name().as_ref(),
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
            match fallback_resolved_paths.entry(path) {
                hash_map::Entry::Vacant(e) => {
                    e.insert((crate_id, *id, summary));
                }
                hash_map::Entry::Occupied(mut e) => {
                    let (existing_crate_id, existing_id, existing_summary) = e.get();
                    let existing_crate = LinkTargetCrate::new(doc, *existing_crate_id);
                    tracing::debug!(
                        path = path.join("::"),
                        existing = ?(existing_crate.display_name().as_ref(), existing_id, existing_summary.kind),
                        new = ?(crate_.display_name().as_ref(), id.0, summary.kind),
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
            per_crate_resolved_paths,
            fallback_resolved_paths,
        }
    }

    pub(super) fn resolve_link(&self, id: Id) -> Option<LinkTarget<'doc>> {
        let summary = self.doc.paths.get(&id)?;
        let path = self.build_link_target_path(id, summary)?;
        let crate_ = LinkTargetCrate::new(self.doc, summary.crate_id);
        Some(LinkTarget { crate_, path })
    }

    fn build_link_target_path_from_path(
        &self,
        crate_id: CrateId,
        path: &[String],
    ) -> Option<LinkTargetPath<'doc>> {
        let (id, summary) = self.find_path_summary(crate_id, path)?;
        self.build_link_target_path(id, summary)
    }

    fn build_container_link_target_parts(
        &self,
        crate_id: CrateId,
        path: &'doc [String],
        kind: ItemKind,
    ) -> Option<(LinkTargetPath<'doc>, &'doc String)> {
        let [container_path @ .., item] = path else {
            return warn_unexpected_path_for_kind(kind, path);
        };
        let Some(container) = self.build_link_target_path_from_path(crate_id, container_path)
        else {
            return warn_missing_container_information(kind, path);
        };
        Some((container, item))
    }

    fn build_link_target_path(
        &self,
        id: Id,
        summary: &'doc ItemSummary,
    ) -> Option<LinkTargetPath<'doc>> {
        let crate_id = summary.crate_id;
        let path = summary.path.as_slice();
        let kind = summary.kind;
        #[expect(clippy::match_same_arms)]
        match kind {
            ItemKind::Module => Some(LinkTargetPath::module(path)),
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
                if let Some(c) = self.build_link_target_path_from_path(crate_id, container_path) {
                    return c.with_field(field);
                }
                // In some cases, rustdoc does not provide path information for enum variant.
                // To work around this, we fall back to the last two segments of the path as the variant and field names when the parent of parent is an enum.
                if let [path @ .., variant] = container_path
                    && let Some(c) = self.build_link_target_path_from_path(crate_id, path)
                {
                    return c.with_variant_field(variant, field);
                }
                warn_missing_container_information(kind, path)
            }
            ItemKind::Union => LinkItemKind::Union.with_path(path),
            ItemKind::Enum => LinkItemKind::Enum.with_path(path),
            ItemKind::Variant => {
                if let [module @ .., item, variant] = path {
                    return Some(LinkTargetPath::enum_variant(module, item, variant));
                }
                warn_unexpected_path_for_kind(kind, path)
            }
            ItemKind::Function => {
                let fn_kind = self.doc.index.get(&id).and_then(|item| match &item.inner {
                    ItemEnum::Function(f) => {
                        let is_method = f
                            .sig
                            .inputs
                            .first()
                            .is_some_and(|(name, _ty)| name == "self");
                        let has_body = f.has_body;
                        Some(FunctionKind {
                            is_method,
                            has_body,
                        })
                    }
                    _ => None,
                });
                let [container_path @ .., function] = path else {
                    return warn_unexpected_path_for_kind(kind, path);
                };
                // trait or impl method / associated function
                if let Some(c) = self.build_link_target_path_from_path(crate_id, container_path) {
                    return c.with_function(function, fn_kind);
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
                    self.build_container_link_target_parts(crate_id, path, kind)?;
                container.with_assoc_const(constant)
            }
            ItemKind::AssocType => {
                let (container, ty) =
                    self.build_container_link_target_parts(crate_id, path, kind)?;
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
        let crate_ = LinkTargetCrate::new(self.doc, crate_id);
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
        if let Some((found_crate_id, id, summary)) = self.fallback_resolved_paths.get(path).copied()
        {
            let found_crate = LinkTargetCrate::new(self.doc, found_crate_id);
            tracing::warn!(
                path = path.join("::"),
                expected = crate_.display_name().as_ref(),
                found = found_crate.display_name().as_ref(),
                kind = ?summary.kind,
                "path not found in expected crate; falling back to another crate with the same path",
            );
            return Some((id, summary));
        }

        None
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

#[derive(Debug)]
pub(super) struct LinkTarget<'doc> {
    crate_: LinkTargetCrate<'doc>,
    path: LinkTargetPath<'doc>,
}

impl LinkTarget<'_> {
    pub(super) fn build_url(&self, local_html_root_url: &str) -> Option<String> {
        let mut url = self.crate_.html_root_url(local_html_root_url)?.to_owned();
        if !url.ends_with('/') {
            url.push('/');
        }
        let relative_path = self.path.build_relative_path();
        url.push_str(&relative_path);
        Some(url)
    }

    pub(super) fn build_title(&self) -> String {
        format!("{} {}", self.path.kind_str(), self.path.display_path())
    }
}

#[derive(Debug)]
enum LinkTargetCrate<'doc> {
    Local {
        name: Option<&'doc str>,
    },
    External {
        id: CrateId,
        name: Option<&'doc str>,
        html_root_url: Option<&'doc str>,
    },
}

impl<'doc> LinkTargetCrate<'doc> {
    fn new(doc: &'doc Crate, id: CrateId) -> Self {
        if id == LOCAL_CRATE_ID {
            let name = doc
                .index
                .get(&doc.root)
                .and_then(|root| root.name.as_deref());
            return Self::Local { name };
        }
        let info = doc.external_crates.get(&id);
        let name = info.map(|info| info.name.as_ref());
        let html_root_url = info.and_then(|info| info.html_root_url.as_deref());
        Self::External {
            id,
            name,
            html_root_url,
        }
    }

    fn id(&self) -> CrateId {
        match self {
            Self::Local { .. } => LOCAL_CRATE_ID,
            Self::External { id, .. } => *id,
        }
    }

    fn name(&self) -> Option<&'doc str> {
        match self {
            Self::Local { name } | Self::External { name, .. } => *name,
        }
    }

    fn display_name(&self) -> Cow<'_, str> {
        self.name().map_or_else(
            || Cow::Owned(format!("<unknown crate #{}>", self.id())),
            Cow::Borrowed,
        )
    }

    fn html_root_url<'a>(&self, local_html_root_url: &'a str) -> Option<&'a str>
    where
        'doc: 'a,
    {
        match self {
            Self::Local { .. } => Some(local_html_root_url),
            Self::External { html_root_url, .. } => *html_root_url,
        }
    }
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
    fn with_path(self, path: &[String]) -> Option<LinkTargetPath<'_>> {
        let Some((item, module)) = path.split_last() else {
            return warn_unexpected_path_for_kind(self.as_item_kind(), path);
        };
        let kind = self;
        Some(LinkTargetPath::Item { kind, item, module })
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

    fn as_str(self) -> &'static str {
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
    RequiredAssocFn,
    ProvidedAssocFn,
    ImplementedMethod,
    ImplementedAssocFn,
    AssocConst,
    AssocType,
}

impl AnchorKind {
    fn as_item_kind(self) -> ItemKind {
        match self {
            Self::StructField | Self::EnumVariantField => ItemKind::StructField,
            Self::EnumVariant => ItemKind::Variant,
            Self::RequiredMethod
            | Self::ProvidedMethod
            | Self::RequiredAssocFn
            | Self::ProvidedAssocFn
            | Self::ImplementedMethod
            | Self::ImplementedAssocFn => ItemKind::Function,
            Self::AssocConst => ItemKind::AssocConst,
            Self::AssocType => ItemKind::AssocType,
        }
    }

    fn namespace(self) -> &'static str {
        match self {
            Self::StructField => "structfield",
            Self::EnumVariant => "variant",
            Self::EnumVariantField => "field",
            Self::RequiredMethod | Self::RequiredAssocFn => "tymethod",
            Self::ProvidedMethod
            | Self::ProvidedAssocFn
            | Self::ImplementedMethod
            | Self::ImplementedAssocFn => "method",
            Self::AssocConst => "associatedconstant",
            Self::AssocType => "associatedtype",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::StructField | Self::EnumVariantField => "field",
            Self::EnumVariant => "variant",
            Self::RequiredMethod | Self::ProvidedMethod | Self::ImplementedMethod => "method",
            Self::RequiredAssocFn | Self::ProvidedAssocFn | Self::ImplementedAssocFn => {
                "associated function"
            }
            Self::AssocConst => "associated constant",
            Self::AssocType => "associated type",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum LinkTargetPath<'doc> {
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

impl<'doc> LinkTargetPath<'doc> {
    fn as_item_kind(self) -> ItemKind {
        match self {
            LinkTargetPath::Module { .. } => ItemKind::Module,
            LinkTargetPath::Item { kind, .. } => kind.as_item_kind(),
            LinkTargetPath::AnchoredItem {
                anchor: [(kind, _)],
                ..
            }
            | LinkTargetPath::NestedAnchoredItem {
                anchors: [_, (kind, _)],
                ..
            } => kind.as_item_kind(),
        }
    }

    fn kind_str(self) -> &'static str {
        match self {
            LinkTargetPath::Module { .. } => "mod",
            LinkTargetPath::Item { kind, .. } => kind.as_str(),
            LinkTargetPath::AnchoredItem {
                anchor: [(kind, _)],
                ..
            }
            | LinkTargetPath::NestedAnchoredItem {
                anchors: [_, (kind, _)],
                ..
            } => kind.as_str(),
        }
    }

    fn display_path(self) -> String {
        match self {
            LinkTargetPath::Module { module } => module.join("::"),
            LinkTargetPath::Item {
                kind: _,
                module,
                item,
            } => format!("{}::{item}", module.join("::")),
            LinkTargetPath::AnchoredItem {
                kind: _,
                module,
                item,
                anchor: [(_, a0)],
            } => format!("{}::{item}::{a0}", module.join("::")),
            LinkTargetPath::NestedAnchoredItem {
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

    fn with_function(self, function: &'doc str, fn_kind: Option<FunctionKind>) -> Option<Self> {
        let target = match self {
            Self::Module { module } => Some(Self::Item {
                kind: LinkItemKind::Function,
                module,
                item: function,
            }),
            Self::Item { kind, module, item } => {
                let anchor = match (kind, fn_kind) {
                    (LinkItemKind::Trait, Some(fn_kind)) => {
                        match (fn_kind.is_method, fn_kind.has_body) {
                            (true, true) => Some(AnchorKind::ProvidedMethod),
                            (true, false) => Some(AnchorKind::RequiredMethod),
                            (false, true) => Some(AnchorKind::ProvidedAssocFn),
                            (false, false) => Some(AnchorKind::RequiredAssocFn),
                        }
                    }
                    (LinkItemKind::Trait, None) => {
                        // In some cases, rustdoc does not provide enough information to determine the trait function kind.
                        // In those cases, we log a warning and default to `RequiredMethod`.
                        // <https://github.com/rust-lang/rust/issues/160662>
                        tracing::warn!(
                            path = format!("{}::{item}::{function}", module.join("::")),
                            "failed to determine the function kind, falling back to trait required method (this may be incorrect)",
                        );
                        Some(AnchorKind::RequiredMethod)
                    }
                    (kind, Some(fn_kind)) if kind.has_inherent_methods() => {
                        if fn_kind.is_method {
                            Some(AnchorKind::ImplementedMethod)
                        } else {
                            Some(AnchorKind::ImplementedAssocFn)
                        }
                    }
                    (kind, None) if kind.has_inherent_methods() => {
                        // In some cases, rustdoc does not provide information about whether a function is method or assoc fn.
                        // In those cases, we log a warning and default to `ImplementedMethod`.
                        // <https://github.com/rust-lang/rust/issues/160662>
                        tracing::warn!(
                            path = format!("{}::{item}::{function}", module.join("::")),
                            "failed to determine if the function is method or associated function, falling back to method (this may be incorrect)",
                        );
                        Some(AnchorKind::ImplementedMethod)
                    }
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

impl LinkTargetPath<'_> {
    fn build_relative_path(&self) -> String {
        match self {
            LinkTargetPath::Module { module } => {
                let module = module.join("/");
                format!("{module}/index.html")
            }
            LinkTargetPath::Item { kind, module, item } => {
                let module = module.join("/");
                let namespace = kind.namespace();
                format!("{module}/{namespace}.{item}.html")
            }
            LinkTargetPath::AnchoredItem {
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
            LinkTargetPath::NestedAnchoredItem {
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
