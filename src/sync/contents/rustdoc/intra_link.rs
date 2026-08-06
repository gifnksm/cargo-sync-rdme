use std::{collections::HashMap, fmt::Write as _, rc::Rc};

use pulldown_cmark::{BrokenLink, CowStr, Event, Options, Tag};
use rustdoc_types::{Crate, Id, Item, ItemKind};

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
    pub(super) fn new<'a>(
        doc: &'a Crate,
        item: &'a Item,
        local_html_root_url: &str,
        mappings: &HashMap<String, String>,
    ) -> Parser<
        impl FnMut(BrokenLink<'_>) -> Option<BrokenLinkPair<'a>>,
        impl FnMut(Event<'a>) -> Option<Event<'a>>,
    > {
        let url_map = Rc::new(resolve_links(doc, item, local_html_root_url, mappings));

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

impl<'a, B, M> Parser<B, M>
where
    B: FnMut(BrokenLink<'_>) -> Option<BrokenLinkPair<'a>> + 'a,
    M: FnMut(Event<'a>) -> Option<Event<'a>> + 'a,
{
    pub(super) fn events<'b>(&'b mut self, doc: &'a str) -> impl Iterator<Item = Event<'a>> + 'b
    where
        'a: 'b,
    {
        pulldown_cmark::Parser::new_with_broken_link_callback(
            doc,
            Options::all(),
            Some(&mut self.broken_link_callback),
        )
        .filter_map(&mut self.iterator_map)
    }
}

fn resolve_links<'doc>(
    doc: &'doc Crate,
    item: &'doc Item,
    local_html_root_url: &str,
    mappings: &HashMap<String, String>,
) -> HashMap<&'doc str, Option<String>> {
    item.links
        .iter()
        .map(move |(name, id)| {
            if let Some(path) = mappings.get(name) {
                (name.as_str(), Some(path.clone()))
            } else {
                let url = id_to_url(doc, local_html_root_url, *id).or_else(|| {
                    tracing::warn!(?id, "failed to resolve link to `{name}`");
                    None
                });
                (name.as_str(), url)
            }
        })
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

fn id_to_url(doc: &Crate, local_html_root_url: &str, id: Id) -> Option<String> {
    let item = doc.paths.get(&id)?;
    let html_root_url = if item.crate_id == 0 {
        // local item
        local_html_root_url
    } else {
        // external item
        let external_crate = doc.external_crates.get(&item.crate_id)?;
        external_crate.html_root_url.as_ref()?
    };

    let mut url = html_root_url.trim_end_matches('/').to_owned();
    let mut join = |paths: &[String], args| {
        for path in paths {
            write!(&mut url, "/{path}").unwrap();
        }
        write!(&mut url, "/{args}").unwrap();
    };
    match (&item.kind, item.path.as_slice()) {
        (ItemKind::Module, ps) => join(ps, format_args!("index.html")),
        // (ItemKind::ExternCrate, [..]) => todo!(),
        // (ItemKind::Import, [..]) => todo!(),
        (ItemKind::Struct, [ps @ .., name]) => join(ps, format_args!("struct.{name}.html")),
        (ItemKind::StructField, [ps @ .., struct_name, field]) => join(
            ps,
            format_args!("struct.{struct_name}.html#structfield.{field}"),
        ),
        (ItemKind::Union, [ps @ .., name]) => join(ps, format_args!("union.{name}.html")),
        (ItemKind::Enum, [ps @ .., name]) => join(ps, format_args!("enum.{name}.html")),
        (ItemKind::Variant, [ps @ .., enum_name, variant_name]) => join(
            ps,
            format_args!("enum.{enum_name}.html#variant.{variant_name}"),
        ),
        (ItemKind::Function, [ps @ .., name]) => join(ps, format_args!("fn.{name}.html")),
        (ItemKind::TypeAlias, [ps @ .., name]) => join(ps, format_args!("type.{name}.html")),
        // (ItemKind::OpaqueTy, [..]) => todo!(),
        (ItemKind::Constant, [ps @ .., name]) => join(ps, format_args!("constant.{name}.html")),
        (ItemKind::Trait, [ps @ .., name]) => join(ps, format_args!("trait.{name}.html")),
        // (ItemKind::TraitAlias, [..]) => todo!(),
        // (ItemKind::Impl, [..]) => todo!(),
        (ItemKind::Static, [ps @ .., name]) => join(ps, format_args!("static.{name}.html")),
        // (ItemKind::ForeignType, [..]) => todo!(),
        (ItemKind::Macro, [ps @ .., name]) => join(ps, format_args!("macro.{name}.html")),
        (ItemKind::ProcAttribute, [ps @ .., name]) => join(ps, format_args!("attr.{name}.html")),
        (ItemKind::ProcDerive, [ps @ .., name]) => join(ps, format_args!("derive.{name}.html")),
        (ItemKind::AssocConst, [ps @ .., trait_name, const_name]) => join(
            ps,
            format_args!("trait.{trait_name}.html#associatedconstant.{const_name}"),
        ),
        (ItemKind::AssocType, [ps @ .., trait_name, type_name]) => join(
            ps,
            format_args!("trait.{trait_name}.html#associatedtype.{type_name}"),
        ),
        (ItemKind::Primitive, [ps @ .., name]) => join(ps, format_args!("primitive.{name}.html")),
        // (ItemKind::Keyword, [..]) => todo!(),
        (item, path) => {
            tracing::warn!(?item, ?path, "unexpected intra-doc link item & path found");
            return None;
        }
    }
    Some(url)
}
