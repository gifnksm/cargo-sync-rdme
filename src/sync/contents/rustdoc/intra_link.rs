use std::{borrow::Cow, collections::HashMap, fmt::Debug};

use pulldown_cmark::{
    BrokenLink, BrokenLinkCallback, CowStr, Event, LinkType, Options, Parser, RefDefs, Tag,
};
use rustdoc_types::{Id, Item};
use unicase::UniCase;

use crate::sync::contents::rustdoc::document::IntraLinkResolver;

#[derive(Debug)]
pub(super) struct LinkMappingConfig<'map, 'url> {
    mappings: &'map HashMap<String, String>,
    local_html_root_url: &'url str,
}

impl<'map, 'url> LinkMappingConfig<'map, 'url> {
    pub(super) fn new(
        mappings: &'map HashMap<String, String>,
        local_html_root_url: &'url str,
    ) -> Self {
        Self {
            mappings,
            local_html_root_url,
        }
    }

    pub(super) fn build_mapper<'doc>(
        &self,
        resolver: &IntraLinkResolver<'_>,
        item: &'doc Item,
    ) -> Option<LinkMapper<'doc, 'map>> {
        let docs = item.docs.as_deref()?;
        let map = item
            .links
            .iter()
            .map(|(name, id)| (name.as_str(), resolve_link(resolver, self, name, *id)))
            .collect();
        Some(LinkMapper { docs, map })
    }
}

#[tracing::instrument(skip_all, fields(name = name, id = ?id))]
fn resolve_link<'map>(
    resolver: &IntraLinkResolver<'_>,
    config: &LinkMappingConfig<'map, '_>,
    name: &str,
    id: Id,
) -> Option<Cow<'map, str>> {
    if let Some(url) = config.mappings.get(name) {
        return Some(url.into());
    }
    let Some(url) = resolver
        .resolve_link(id)
        .and_then(|target| target.build_url(config.local_html_root_url))
    else {
        tracing::warn!("failed to resolve link");
        return None;
    };
    Some(url.into())
}

#[derive(Debug)]
pub(super) struct LinkMapper<'doc, 'map> {
    docs: &'doc str,
    map: HashMap<&'doc str, Option<Cow<'map, str>>>,
}

impl<'url, 'input> BrokenLinkCallback<'input> for &LinkMapper<'_, 'url>
where
    'url: 'input,
{
    fn handle_broken_link(
        &mut self,
        link: BrokenLink<'input>,
    ) -> Option<(CowStr<'input>, CowStr<'input>)> {
        let target = self.map.get(&*link.reference)?.as_ref()?;
        Some((target.clone().into(), "".into()))
    }
}

impl LinkMapper<'_, '_> {
    pub(super) fn build_parser(&self, options: Options) -> impl Iterator<Item = Event<'_>> {
        let parser = Parser::new_with_broken_link_callback(self.docs, options, Some(self));
        let mut refs =
            LabelRegistry::from_reference_definitions(parser.reference_definitions(), &self.map);
        parser.map(move |mut event| {
            if let Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) = &mut event
            {
                match link_type {
                    LinkType::ReferenceUnknown => {
                        *link_type = LinkType::Reference;
                        let updated_id = refs.allocate_label(id, dest_url, title).0.into_inner();
                        if *id != updated_id {
                            *id = updated_id.into_static();
                        }
                    }
                    LinkType::CollapsedUnknown => {
                        *link_type = LinkType::Collapsed;
                        let updated_id = refs.allocate_label(id, dest_url, title).0.into_inner();
                        if *id != updated_id {
                            *id = updated_id.into_static();
                            *link_type = LinkType::Reference;
                        }
                    }
                    LinkType::ShortcutUnknown => {
                        *link_type = LinkType::Shortcut;
                        let updated_id = refs.allocate_label(id, dest_url, title).0.into_inner();
                        if *id != updated_id {
                            *id = updated_id.into_static();
                            *link_type = LinkType::Reference;
                        }
                    }
                    LinkType::Inline => {
                        if let Some(Some(new_dest_url)) = self.map.get(dest_url.as_ref()) {
                            *link_type = LinkType::Reference;
                            *id = refs
                                .allocate_label(dest_url, new_dest_url, title)
                                .0
                                .into_inner()
                                .into_static();
                            *dest_url = new_dest_url.clone().into();
                        }
                    }
                    LinkType::Reference | LinkType::Collapsed | LinkType::Shortcut => {
                        if let Some(Some(new_dest_url)) = self.map.get(dest_url.as_ref()) {
                            *dest_url = new_dest_url.clone().into();
                        }
                    }
                    LinkType::Autolink | LinkType::Email => {}
                    LinkType::WikiLink { .. } => unreachable!(),
                }
            }
            event
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct LinkLabel<'a>(UniCase<CowStr<'a>>);

impl Debug for LinkLabel<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0.as_ref(), f)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct LinkTarget<'a> {
    url: CowStr<'a>,
    title: CowStr<'a>,
}

impl Debug for LinkTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinkTarget")
            .field("url", &self.url.as_ref())
            .field("title", &self.title.as_ref())
            .finish()
    }
}

#[derive(Debug, Default)]
struct LabelRegistry {
    map: HashMap<LinkLabel<'static>, LinkTarget<'static>>,
}

impl LinkTarget<'_> {
    fn into_static(self) -> LinkTarget<'static> {
        LinkTarget {
            url: self.url.into_static(),
            title: self.title.into_static(),
        }
    }
}

impl<'a> LinkLabel<'a> {
    fn new(label: &'a str) -> Self {
        Self(normalize_label(label))
    }

    fn into_static(self) -> LinkLabel<'static> {
        LinkLabel(UniCase::new(self.0.into_inner().into_static()))
    }
}

impl LabelRegistry {
    fn from_reference_definitions(
        defs: &RefDefs<'_>,
        url_map: &HashMap<&str, Option<Cow<'_, str>>>,
    ) -> Self {
        let mut map = HashMap::new();
        for (label, def) in defs.iter() {
            let label = LinkLabel::new(label);
            let url = match url_map.get(def.dest.as_ref()) {
                Some(Some(url)) => url.as_ref().into(),
                Some(None) | None => def.dest.clone(),
            };
            let title = def.title.clone().unwrap_or(CowStr::Borrowed(""));
            let target = LinkTarget { url, title };
            map.insert(label.clone().into_static(), target.clone().into_static());
        }
        Self { map }
    }

    fn allocate_label<'a>(
        &mut self,
        label: &'a CowStr<'a>,
        url: &str,
        title: &str,
    ) -> LinkLabel<'a> {
        let label = LinkLabel::new(label);
        let target = LinkTarget {
            url: url.into(),
            title: title.into(),
        };
        if let Some(existing) = self.map.get(&label) {
            if *existing == target {
                return label;
            }
        } else {
            self.map
                .insert(label.clone().into_static(), target.clone().into_static());
            return label;
        }
        let label_base = strip_code_span_backticks(label.0.as_ref());
        for i in 1.. {
            let new_label_text = format!("{label_base}@{i}");
            let new_label = LinkLabel::new(&new_label_text);
            if let Some(existing) = self.map.get(&new_label) {
                if *existing == target {
                    return new_label.into_static();
                }
                continue;
            }
            self.map.insert(
                new_label.clone().into_static(),
                target.clone().into_static(),
            );
            return new_label.into_static();
        }
        unreachable!()
    }
}

fn normalize_label(label: &str) -> UniCase<CowStr<'_>> {
    // The following is quoted from the CommonMark spec
    // <https://spec.commonmark.org/0.31.2/#matches>
    //
    // > To normalize a label, strip off the opening and closing brackets,
    // > perform the Unicode case fold, strip leading and trailing spaces,
    // > tabs, and line endings, and collapse consecutive internal spaces,
    // > tabs, and line endings to a single space.
    //

    // We do not need to strip brackets here, because we only get the label content.

    let label = label.trim_matches(is_whitespace);
    let label = collapse_consecutive_whitespace(label);
    let label = sanitize_label_for_cmark_to_cmark(label);

    // `pulldown-cmark` uses `UniCase` to perform Unicode case folding.
    UniCase::new(label)
}

// In the CommonMark spec, "spaces, tabs, and line endings" are defined as:
//
// * A space is `U+0020`
//   <https://spec.commonmark.org/0.31.2/#space>
// * A tab is `U+0009`
//   <https://spec.commonmark.org/0.31.2/#tab>
// * A line ending is a line feed (U+000A), a carriage return (U+000D) not followed by a line feed, or a carriage return and a following line feed.
//   <https://spec.commonmark.org/0.31.2/#line-ending>
//
// However, `pulldown-cmark` treats '\x09'..='\x0D' (HT, LF, FF, CR) and '\x20' as whitespace, which does not exactly match the CommonMark spec.
// <https://github.com/pulldown-cmark/pulldown-cmark/blob/2f251a6e9db0c550e2b9337881bb7d48fe27cdfc/pulldown-cmark/src/scanners.rs#L436-L438>
//
// In `cargo-sync-rdme`, we follow `pulldown-cmark`'s behavior.
// We cannot use `char::is_ascii_whitespace()` here because it does not treat U+000B (VT) as whitespace.
fn is_whitespace(c: char) -> bool {
    matches!(c, '\x09'..='\x0D' | '\x20')
}

fn is_whitespace_byte(b: u8) -> bool {
    matches!(b, b'\x09'..=b'\x0D' | b'\x20')
}

fn collapse_consecutive_whitespace(label: &str) -> CowStr<'_> {
    let has_consecutive_ws = label
        .as_bytes()
        .array_windows()
        .any(|[a, b]| is_whitespace_byte(*a) && is_whitespace_byte(*b));
    if !has_consecutive_ws {
        return label.into();
    }
    let mut collapsed = String::with_capacity(label.len());
    let mut prev_was_ws = false;
    for c in label.chars() {
        if is_whitespace(c) {
            if !prev_was_ws {
                collapsed.push(' ');
                prev_was_ws = true;
            }
        } else {
            collapsed.push(c);
            prev_was_ws = false;
        }
    }
    collapsed.into()
}

fn sanitize_label_for_cmark_to_cmark(label: CowStr<'_>) -> CowStr<'_> {
    // `pulldown-cmark-to-cmark` does not escape brackets in link labels, so we replace them with parentheses to avoid breaking the Markdown output.
    // <https://github.com/Byron/pulldown-cmark-to-cmark/issues/109>
    if label.contains('[') || label.contains(']') {
        let replaced = label.replace('[', "(").replace(']', ")");
        replaced.into()
    } else {
        label
    }
}

fn strip_code_span_backticks(label: &str) -> &str {
    label
        .strip_prefix('`')
        .and_then(|l| l.strip_suffix('`'))
        .unwrap_or(label)
}
