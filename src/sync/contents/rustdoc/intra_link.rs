use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
};

use pulldown_cmark::{
    BrokenLink, BrokenLinkCallback, CowStr, Event, LinkType, Options, Parser, RefDefs, Tag,
    TextMergeStream,
};
use rustdoc_types::{Id, Item};
use unicase::UniCase;

use crate::sync::contents::rustdoc::document::IntraLinkResolver;

#[derive(Debug)]
pub(super) struct LinkMappingConfig<'map> {
    pub(super) mappings: &'map HashMap<String, String>,
}

impl<'map> LinkMappingConfig<'map> {
    pub(super) fn build_mapper<'doc>(
        &self,
        resolver: &IntraLinkResolver<'_>,
        item: &'doc Item,
    ) -> Option<LinkMapper<'doc, 'map>> {
        let docs = item.docs.as_deref()?;
        let url_map = item
            .links
            .iter()
            .map(|(name, id)| (name.as_str(), resolve_link(resolver, self, name, *id)))
            .collect();
        Some(LinkMapper { docs, url_map })
    }
}

#[derive(Debug)]
enum ResolvedLink<'map> {
    Mapped(CowStr<'map>),
    IntraDocResolved { url: String, title: String },
}

impl<'map> ResolvedLink<'map> {
    fn is_intra_doc_resolved(&self) -> bool {
        matches!(self, Self::IntraDocResolved { .. })
    }

    fn url(&self) -> CowStr<'map> {
        match self {
            Self::Mapped(url) => url.clone(),
            Self::IntraDocResolved { url, .. } => url.clone().into(),
        }
    }

    fn url_as_str(&self) -> &str {
        match self {
            Self::Mapped(url) => url.as_ref(),
            Self::IntraDocResolved { url, .. } => url.as_ref(),
        }
    }

    fn title(&self) -> CowStr<'map> {
        match self {
            Self::Mapped(_) => "".into(),
            Self::IntraDocResolved { title, .. } => title.clone().into(),
        }
    }
}

#[tracing::instrument(skip_all, fields(name = name, id = ?id))]
fn resolve_link<'map>(
    resolver: &IntraLinkResolver<'_>,
    config: &LinkMappingConfig<'map>,
    name: &str,
    id: Id,
) -> Option<ResolvedLink<'map>> {
    if let Some(url) = config.mappings.get(name) {
        return Some(ResolvedLink::Mapped(url.as_str().into()));
    }
    let Some((url, title)) = resolver
        .resolve_link(id)
        .map(|target| (target.build_url(), target.build_title()))
    else {
        tracing::warn!("failed to resolve link");
        return None;
    };
    Some(ResolvedLink::IntraDocResolved { url, title })
}

#[derive(Debug)]
pub(super) struct LinkMapper<'doc, 'map> {
    docs: &'doc str,
    url_map: HashMap<&'doc str, Option<ResolvedLink<'map>>>,
}

impl<'url, 'input> BrokenLinkCallback<'input> for &LinkMapper<'_, 'url>
where
    'url: 'input,
{
    fn handle_broken_link(
        &mut self,
        link: BrokenLink<'input>,
    ) -> Option<(CowStr<'input>, CowStr<'input>)> {
        let resolved = self.url_map.get(&*link.reference)?.as_ref()?;
        Some((resolved.url(), resolved.title()))
    }
}

impl LinkMapper<'_, '_> {
    pub(super) fn build_parser(&self, options: Options) -> impl Iterator<Item = Event<'_>> {
        let parser = Parser::new_with_broken_link_callback(self.docs, options, Some(self));
        let label_registry = LabelRegistry::from_reference_definitions(
            parser.reference_definitions(),
            &self.url_map,
        );
        let stream = TextMergeStream::new(parser);
        EventStream {
            url_map: &self.url_map,
            label_registry,
            stream,
            events: VecDeque::new(),
        }
    }
}

#[derive(Debug)]
struct EventStream<'input, 'mapper, 'doc, 'map> {
    url_map: &'mapper HashMap<&'doc str, Option<ResolvedLink<'map>>>,
    label_registry: LabelRegistry,
    stream: TextMergeStream<'input, Parser<'input, &'mapper LinkMapper<'doc, 'map>>>,
    events: VecDeque<Event<'input>>,
}

impl<'input, 'map> Iterator for EventStream<'input, '_, '_, 'map>
where
    'map: 'input,
{
    type Item = Event<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.events.pop_front() {
            return Some(event);
        }
        let mut event = self.stream.next()?;
        let mut ns_prefix = None;
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
                    let updated_id = self
                        .label_registry
                        .allocate_label(id, dest_url, title)
                        .0
                        .into_inner();
                    if *id != updated_id {
                        *id = updated_id.into_static();
                    }
                }
                LinkType::CollapsedUnknown => {
                    *link_type = LinkType::Collapsed;
                    self.strip_namespace_prefix_from_label(id, &mut ns_prefix);
                    let updated_id = self
                        .label_registry
                        .allocate_label(id, dest_url, title)
                        .0
                        .into_inner();
                    if *id != updated_id {
                        *id = updated_id.into_static();
                        *link_type = LinkType::Reference;
                    }
                }
                LinkType::ShortcutUnknown => {
                    *link_type = LinkType::Shortcut;
                    self.strip_namespace_prefix_from_label(id, &mut ns_prefix);
                    let updated_id = self
                        .label_registry
                        .allocate_label(id, dest_url, title)
                        .0
                        .into_inner();
                    if *id != updated_id {
                        *id = updated_id.into_static();
                        *link_type = LinkType::Reference;
                    }
                }
                LinkType::Inline => {
                    if let Some(Some(resolved)) = self.url_map.get(dest_url.as_ref()) {
                        *link_type = LinkType::Reference;
                        let label = reference_label_from_link_destination(dest_url);
                        if title.is_empty() {
                            *title = resolved.title();
                        }
                        *id = self
                            .label_registry
                            .allocate_label(&label, resolved.url_as_str(), title)
                            .0
                            .into_inner()
                            .into_static();
                        *dest_url = resolved.url();
                    }
                }
                LinkType::Reference | LinkType::Collapsed | LinkType::Shortcut => {
                    if let Some(Some(resolved)) = self.url_map.get(dest_url.as_ref()) {
                        if title.is_empty() {
                            *title = resolved.title();
                        }
                        *dest_url = resolved.url();
                    }
                }
                LinkType::Autolink | LinkType::Email => {}
                LinkType::WikiLink { .. } => unreachable!(),
            }
        }
        if let Some(ns_prefix) = ns_prefix
            && let Some(mut next) = self.stream.next()
        {
            if let Event::Code(text) | Event::Text(text) = &mut next
                && let Some(new_text) = text.strip_prefix(&ns_prefix)
            {
                *text = new_text.to_owned().into();
            }
            self.events.push_back(next);
        }
        Some(event)
    }
}

impl EventStream<'_, '_, '_, '_> {
    fn strip_namespace_prefix_from_label(
        &self,
        label: &mut CowStr<'_>,
        ns_prefix: &mut Option<String>,
    ) {
        if !self
            .url_map
            .get(label.as_ref())
            .and_then(Option::as_ref)
            .is_some_and(ResolvedLink::is_intra_doc_resolved)
        {
            return;
        }
        if let Some((ns, new_id)) = label.split_once('@') {
            if let Some(ns) = ns.strip_prefix('`') {
                *ns_prefix = Some(format!("{ns}@"));
                *label = format!("`{new_id}").into();
            } else {
                *ns_prefix = Some(format!("{ns}@"));
                *label = new_id.to_owned().into();
            }
        }
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
    // TODO: Revisit this once a pulldown-cmark release including
    // <https://github.com/pulldown-cmark/pulldown-cmark/pull/1092> is available.
    // In 0.13.4, `Parser::reference_definitions()` still returns `&RefDefs<'_>`,
    // so we have to own these entries as `'static` for now. After that fix is
    // released, this should be relaxable to `'input`.
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
        url_map: &HashMap<&str, Option<ResolvedLink<'_>>>,
    ) -> Self {
        let mut map = HashMap::new();
        for (label, def) in defs.iter() {
            let label = LinkLabel::new(label);
            let (url, title) = match url_map.get(def.dest.as_ref()) {
                Some(Some(resolved)) => (
                    resolved.url(),
                    def.title.clone().unwrap_or_else(|| resolved.title()),
                ),
                Some(None) | None => (
                    def.dest.clone(),
                    def.title.clone().unwrap_or(CowStr::Borrowed("")),
                ),
            };
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

fn strip_code_span_backticks(label: &str) -> &str {
    label
        .strip_prefix('`')
        .and_then(|l| l.strip_suffix('`'))
        .unwrap_or(label)
}

// Raw `[` and `]` are not valid in CommonMark reference-label source syntax, so
// labels that contain brackets must be written with escapes. `pulldown-cmark`
// keeps those escapes in the parsed label text instead of unescaping them.
//
// When we synthesize a label from an inline intra-doc link destination, we need
// to produce the same representation that `pulldown-cmark` would use for a
// parsed escaped label. Otherwise `pulldown-cmark-to-cmark` would emit an
// invalid reference definition containing raw brackets.
fn reference_label_from_link_destination(label: &str) -> CowStr<'_> {
    if !label.contains(['[', ']']) {
        return label.into();
    }
    let mut escaped = String::with_capacity(label.len());
    let mut chars = label.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                escaped.push(ch);
                if let Some(ch) = chars.next() {
                    escaped.push(ch);
                }
            }
            '[' | ']' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => {
                escaped.push(ch);
            }
        }
    }
    escaped.into()
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use similar_asserts::assert_eq;

    use super::*;

    fn render<const N: usize>(
        docs: &'static str,
        links: [(&'static str, Option<ResolvedLink<'static>>); N],
    ) -> String {
        let url_map = HashMap::from(links);
        let mapper = LinkMapper { docs, url_map };
        let events = mapper.build_parser(Options::empty());
        let mut output = String::new();
        pulldown_cmark_to_cmark::cmark(events, &mut output).unwrap();
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output
    }

    #[expect(clippy::unnecessary_wraps)]
    fn idr<'a>(url: &'a str, title: &'a str) -> Option<ResolvedLink<'a>> {
        let url = url.into();
        let title = title.into();
        Some(ResolvedLink::IntraDocResolved { url, title })
    }

    #[expect(clippy::unnecessary_wraps)]
    fn mapped(url: &str) -> Option<ResolvedLink<'_>> {
        Some(ResolvedLink::Mapped(url.into()))
    }

    #[test]
    fn strips_namespace_prefix_from_shortcut_link_text() {
        let docs = indoc! {"
            * [`struct@Struct`]
            * [enum@Enum]
        "};
        let links = [
            (
                "`struct@Struct`",
                idr(
                    "https://example.com/struct.Struct.html",
                    "struct example::Struct",
                ),
            ),
            (
                "enum@Enum",
                idr("https://example.com/enum.Enum.html", "enum example::Enum"),
            ),
        ];
        let expected = indoc! {r#"
            * [`Struct`]
            * [Enum]

            [`Struct`]: https://example.com/struct.Struct.html "struct example::Struct"
            [Enum]: https://example.com/enum.Enum.html "enum example::Enum"
        "#};

        let output = render(docs, links);
        assert_eq!(output, expected);
    }

    #[test]
    fn preserves_atmark_prefix_for_mapped_links() {
        let docs = indoc! {"
            * [`struct@Struct`]
            * [enum@Enum]
            * [`struct@Struct`][]
            * [enum@Enum][]
        "};
        let links = [
            (
                "`struct@Struct`",
                mapped("https://example.com/struct.Struct.html"),
            ),
            ("enum@Enum", mapped("https://example.com/enum.Enum.html")),
        ];
        let expected = indoc! {"
            * [`struct@Struct`]
            * [enum@Enum]
            * [`struct@Struct`][]
            * [enum@Enum][]

            [`struct@Struct`]: https://example.com/struct.Struct.html
            [enum@Enum]: https://example.com/enum.Enum.html
        "};

        let output = render(docs, links);
        assert_eq!(output, expected);
    }

    #[test]
    fn converts_to_explicit_reference_when_plain_label_already_exists() {
        let docs = indoc! {"
            * [`Struct`] and [`struct@Struct`]
            * [Enum] and [enum@Enum]

            [`Struct`]: https://example.com/other/struct.Struct.html
            [Enum]: https://example.com/other/enum.Enum.html
        "};
        let links = [
            (
                "`struct@Struct`",
                idr(
                    "https://example.com/struct.Struct.html",
                    "struct example::Struct",
                ),
            ),
            (
                "enum@Enum",
                idr("https://example.com/enum.Enum.html", "enum example::Enum"),
            ),
        ];

        let expected = indoc! {r#"
            * [`Struct`] and [`Struct`][Struct@1]
            * [Enum] and [Enum][Enum@1]

            [`Struct`]: https://example.com/other/struct.Struct.html
            [Struct@1]: https://example.com/struct.Struct.html "struct example::Struct"
            [Enum]: https://example.com/other/enum.Enum.html
            [Enum@1]: https://example.com/enum.Enum.html "enum example::Enum"
        "#};

        let output = render(docs, links);
        assert_eq!(output, expected);
    }

    #[test]
    fn preserves_existing_titles_and_fills_missing_titles_for_reference_and_inline_links() {
        let docs = indoc! {r#"
            * Inline:
              * [titled](Struct "custom title")
              * [untitled](Enum)
            * Reference:
              * [titled][struct-ref]
              * [untitled][enum-ref]
            * Reference Unknown:
              * [the struct][Struct]
              * [the enum][Enum]
            * Collapsed:
              * [struct-ref][]
              * [enum-ref][]
            * Collapsed Unknown:
              * [Struct][]
              * [Enum][]
            * Shortcut:
              * [struct-ref]
              * [enum-ref]
            * Shortcut Unknown:
              * [Struct]
              * [Enum]

            [struct-ref]: Struct "custom title"
            [enum-ref]: Enum
        "#};
        let links = [
            (
                "Struct",
                idr(
                    "https://example.com/struct.Struct.html",
                    "struct example::Struct",
                ),
            ),
            (
                "Enum",
                idr("https://example.com/enum.Enum.html", "enum example::Enum"),
            ),
        ];

        let expected = indoc! {r#"
            * Inline:
              * [titled][Struct]
              * [untitled][Enum]
            * Reference:
              * [titled][struct-ref]
              * [untitled][enum-ref]
            * Reference Unknown:
              * [the struct][Struct@1]
              * [the enum][Enum]
            * Collapsed:
              * [struct-ref][]
              * [enum-ref][]
            * Collapsed Unknown:
              * [Struct][Struct@1]
              * [Enum][]
            * Shortcut:
              * [struct-ref]
              * [enum-ref]
            * Shortcut Unknown:
              * [Struct][Struct@1]
              * [Enum]

            [Struct]: https://example.com/struct.Struct.html "custom title"
            [Enum]: https://example.com/enum.Enum.html "enum example::Enum"
            [struct-ref]: https://example.com/struct.Struct.html "custom title"
            [enum-ref]: https://example.com/enum.Enum.html "enum example::Enum"
            [Struct@1]: https://example.com/struct.Struct.html "struct example::Struct"
        "#};

        let output = render(docs, links);
        assert_eq!(output, expected);
    }
}
