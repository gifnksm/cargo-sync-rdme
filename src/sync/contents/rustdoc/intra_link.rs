use std::{collections::HashMap, fmt::Write, rc::Rc};

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

impl Parser<(), ()> {
    pub(super) fn new<'a>(
        doc: &'a Crate,
        item: &'a Item,
        local_html_root_url: &str,
    ) -> Parser<
        impl FnMut(BrokenLink<'_>) -> Option<(CowStr<'a>, CowStr<'a>)>,
        impl FnMut(Event<'a>) -> Event<'a>,
    > {
        let url_map = Rc::new(resolve_links(doc, item, local_html_root_url));

        let broken_link_callback = {
            let url_map = Rc::clone(&url_map);
            move |link: BrokenLink<'_>| {
                let url = url_map.get(link.reference.as_str())?;
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
    B: FnMut(BrokenLink<'_>) -> Option<(CowStr<'a>, CowStr<'a>)> + 'a,
    M: FnMut(Event<'a>) -> Event<'a> + 'a,
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
        .map(&mut self.iterator_map)
    }
}

fn resolve_links<'a>(
    doc: &Crate,
    item: &'a Item,
    local_html_root_url: &str,
) -> HashMap<&'a str, String> {
    item.links
        .iter()
        .filter_map(|(name, id)| {
            let url = id_to_url(doc, local_html_root_url, id).or_else(|| {
                tracing::warn!("failed to resolve link to `{}`", name);
                None
            })?;
            Some((name.as_str(), url))
        })
        .collect()
}

fn convert_link<'a>(url_map: &HashMap<&str, String>, mut event: Event<'a>) -> Event<'a> {
    match &mut event {
        Event::Start(Tag::Link(_link_type, url, _title))
        | Event::End(Tag::Link(_link_type, url, _title)) => {
            if let Some(full_url) = url_map.get(url.as_ref()) {
                *url = full_url.to_owned().into();
            }
        }
        _ => {}
    }
    event
}

fn id_to_url(doc: &Crate, local_html_root_url: &str, id: &Id) -> Option<String> {
    let item = doc.paths.get(id)?;
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
            write!(&mut url, "/{}", path).unwrap();
        }
        write!(&mut url, "/{}", args).unwrap();
    };
    match (&item.kind, item.path.as_slice()) {
        (ItemKind::Module, [ps @ ..]) => join(ps, format_args!("index.html")),
        // (ItemKind::ExternCrate, [..]) => todo!(),
        // (ItemKind::Import, [..]) => todo!(),
        (ItemKind::Struct, [ps @ .., name]) => join(ps, format_args!("struct.{name}.html")),
        // (ItemKind::StructField, [..]) => todo!(),
        (ItemKind::Union, [ps @ .., name]) => join(ps, format_args!("union.{name}.html")),
        (ItemKind::Enum, [ps @ .., name]) => join(ps, format_args!("enum.{name}.html")),
        (ItemKind::Variant, [ps @ .., name, variant]) => {
            join(ps, format_args!("enum.{name}.html#variant.{variant}"))
        }
        (ItemKind::Function, [ps @ .., name]) => join(ps, format_args!("fn.{name}.html")),
        (ItemKind::Typedef, [ps @ .., name]) => join(ps, format_args!("type.{name}.html")),
        // (ItemKind::OpaqueTy, [..]) => todo!(),
        (ItemKind::Constant, [ps @ .., name]) => join(ps, format_args!("constant.{name}.html")),
        (ItemKind::Trait, [ps @ .., name]) => join(ps, format_args!("trait.{name}.html")),
        // (ItemKind::TraitAlias, [..]) => todo!(),
        // (ItemKind::Method, [..]) => todo!(),
        // (ItemKind::Impl, [..]) => todo!(),
        (ItemKind::Static, [ps @ .., name]) => join(ps, format_args!("static.{name}.html")),
        // (ItemKind::ForeignType, [..]) => todo!(),
        (ItemKind::Macro, [ps @ .., name]) => join(ps, format_args!("macro.{name}.html")),
        (ItemKind::ProcAttribute, [ps @ .., name]) => join(ps, format_args!("attr.{name}.html")),
        (ItemKind::ProcDerive, [ps @ .., name]) => join(ps, format_args!("derive.{name}.html")),
        // (ItemKind::AssocConst, [..]) => todo!(),
        // (ItemKind::AssocType, [..]) => todo!(),
        (ItemKind::Primitive, [ps @ .., name]) => join(ps, format_args!("primitive.{name}.html")),
        // (ItemKind::Keyword, [..]) => todo!(),
        (item, path) => tracing::warn!(?item, ?path, "unexpected intra-doc link item & path found"),
    }
    Some(url)
}
