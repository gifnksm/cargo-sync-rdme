use std::sync::Arc;

use miette::{NamedSource, SourceSpan};
use pulldown_cmark::{Event, OffsetIter, Options, Parser};
use snafu::{OptionExt as _, Snafu, ensure};

use crate::{
    parse::Spanned,
    sync::{
        ManifestFile, MarkdownPath,
        marker::{MAGIC, ResolveMarkerError, parse, resolve},
    },
    traits::RangeExt as _,
};

use super::{super::MarkdownFile, ResolvedMarker, ResolvedReplaceSpecifier};

pub(in crate::sync) fn scan_all(
    markdown: &MarkdownFile<'_>,
    manifest: &ManifestFile,
) -> Result<Vec<Spanned<ResolvedReplaceSpecifier>>, Box<ScanAllError>> {
    let scanner = Scanner::new(manifest, &markdown.text);
    let mut markers = vec![];
    let mut errors = vec![];
    for res in scanner {
        match res {
            Ok(marker) => markers.push(marker),
            Err(err) => errors.push(err),
        }
    }

    ensure!(
        errors.is_empty(),
        ScanAllSnafu {
            markdown,
            source_code: markdown.to_named_source(),
            errors
        }
    );

    Ok(markers)
}

#[derive(Debug, Snafu, miette::Diagnostic)]
#[snafu(display(
    "failed to parse `<!-- {MAGIC} ... -->` markers in markdown file for package `{package}`: {markdown}",
    package = markdown.package, markdown = markdown.path,
))]
pub(crate) struct ScanAllError {
    markdown: MarkdownPath,
    #[source_code]
    source_code: NamedSource<Arc<str>>,
    #[related]
    errors: Vec<ScanError>,
}

#[expect(clippy::enum_variant_names)]
#[derive(Debug, Snafu, miette::Diagnostic)]
enum ScanError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    ParseMarker {
        #[snafu(source)]
        #[diagnostic_source]
        source: parse::ParseMarkerError,
    },
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    ResolveMarker {
        #[snafu(source)]
        #[diagnostic_source]
        source: ResolveMarkerError,
    },
    #[snafu(display("unexpected end marker"))]
    UnexpectedEndMarker {
        #[label = "the end marker defined here"]
        span: SourceSpan,
    },
    #[snafu(display("no corresponding end marker found"))]
    NoCorrespondingEndMarker {
        #[label = "the start marker defined here"]
        start_span: SourceSpan,
    },
    #[snafu(display("nested markers are not allowed"))]
    NestedMarker {
        #[label = "the nested marker defined here"]
        nested_span: SourceSpan,
        #[label = "the previous marker starts here"]
        previous_span: SourceSpan,
    },
}

#[derive(Debug)]
struct Scanner<'manifest, 'markdown> {
    manifest: &'manifest ManifestFile,
    markdown: &'markdown str,
    parser: OffsetIter<'markdown>,
}

impl Iterator for Scanner<'_, '_> {
    type Item = Result<Spanned<ResolvedReplaceSpecifier>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl<'manifest, 'markdown> Scanner<'manifest, 'markdown> {
    fn new(manifest: &'manifest ManifestFile, markdown: &'markdown str) -> Self {
        let parser = Parser::new_ext(markdown, Options::all()).into_offset_iter();
        Self {
            manifest,
            markdown,
            parser,
        }
    }

    fn try_next(&mut self) -> Result<Option<Spanned<ResolvedReplaceSpecifier>>, ScanError> {
        let Some(start_marker) = self.next_marker()? else {
            return Ok(None);
        };
        let start_span = start_marker.span;
        let specifier = match start_marker.value {
            ResolvedMarker::Replace(specifier) => {
                return Ok(Some(Spanned::new(specifier, start_span)));
            }
            ResolvedMarker::Start(specifier) => specifier,
            ResolvedMarker::End => {
                return Err(UnexpectedEndMarkerSnafu {
                    span: start_span.to_span(),
                }
                .build());
            }
        };
        let end_marker = self
            .next_marker()?
            .with_context(|| NoCorrespondingEndMarkerSnafu {
                start_span: start_span.to_span(),
            })?;
        let end_span = end_marker.span;
        match end_marker.value {
            ResolvedMarker::End => Ok(Some(Spanned::new(
                specifier,
                start_span.start..end_span.end,
            ))),
            _ => Err(NestedMarkerSnafu {
                nested_span: end_span.to_span(),
                previous_span: start_span.to_span(),
            }
            .build()),
        }
    }
}

impl Scanner<'_, '_> {
    fn next_marker(&mut self) -> Result<Option<Spanned<ResolvedMarker>>, ScanError> {
        for (event, range) in self.parser.by_ref() {
            if let Event::Html(_html) = event {
                // Use the original markdown slice for this HTML event instead of the
                // `Event::Html` payload. The payload may be normalized by pulldown-cmark,
                // which would make the parsed marker text diverge from the original source span
                // and break diagnostics.
                //
                // As of 2026-08-21, pulldown-cmark main contains an unreleased change that
                // replaces `\0` with `\u{FFFD}` in `Event::Html`:
                // <https://github.com/pulldown-cmark/pulldown-cmark/blob/07bae2459d90175b661d42b8acf207382e111ae5/pulldown-cmark/src/parse.rs#L2404-L2406>
                let html = &self.markdown[range.clone()];
                let html = Spanned::new(html, range);
                if let Some(marker) = parse::parse_marker(html.as_deref())? {
                    let marker = resolve::resolve_marker(marker, self.manifest)?;
                    return Ok(Some(marker));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::range::Range;

    use indoc::indoc;

    use crate::config::Manifest;

    use super::*;

    impl ScanError {
        #[track_caller]
        pub(crate) fn into_parse_marker(self) -> parse::ParseMarkerError {
            let Self::ParseMarker { source } = self else {
                panic!("unexpected error: {self:?}");
            };
            source
        }
    }

    fn line_ranges(lines: &[impl AsRef<str>]) -> Vec<Range<usize>> {
        lines
            .iter()
            .scan(0, |offset, line| {
                let line = line.as_ref();
                let range = Range::from(*offset..*offset + line.len() + 1);
                *offset = range.end;
                Some(range)
            })
            .collect()
    }

    #[test]
    fn no_markers() {
        let input = "Hello, world!";
        let manifest = ManifestFile::dummy(Manifest::default());
        let mut markers = Scanner::new(&manifest, input);
        assert!(markers.next().is_none());
    }

    #[test]
    fn replace_marker() {
        let lines = [
            "Good morning, world!",
            "<!-- cargo-sync-rdme title -->",
            "Good afternoon, world!",
            "<!--  cargo-sync-rdme badge  -->",
            "Good evening, world!",
            "<!--cargo-sync-rdme rustdoc  -->",
            "Good night, world!",
        ];
        let ranges = line_ranges(&lines);
        let input = lines.join("\n");

        let config = indoc! {"
            [package.metadata.cargo-sync-rdme.badge.badges]
        "};
        let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());

        let markers = Scanner::new(&manifest, &input)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            markers,
            [
                Spanned::new(ResolvedReplaceSpecifier::Title, ranges[1]),
                Spanned::new(
                    ResolvedReplaceSpecifier::Badge {
                        group: None,
                        badges: vec![].into()
                    },
                    ranges[3],
                ),
                Spanned::new(ResolvedReplaceSpecifier::Rustdoc, ranges[5])
            ]
        );
    }

    #[test]
    fn start_and_end_marker() {
        let lines = [
            "Good morning, world!",
            "<!-- cargo-sync-rdme title [[ -->",
            "Good afternoon, world!",
            "# Heading!",
            "<!-- cargo-sync-rdme ]] -->",
            "Good evening, world!",
        ];
        let ranges = line_ranges(&lines);
        let input = lines.join("\n");

        let config = indoc! {"
            [package.metadata.cargo-sync-rdme.badge.badges]
        "};

        let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
        let markers = Scanner::new(&manifest, &input)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            markers,
            [Spanned::new(
                ResolvedReplaceSpecifier::Title,
                ranges[1].start..ranges[4].end
            )]
        );
    }

    #[test]
    fn nul_character_conversion() {
        let lines = [
            "Good morning, world!",
            "<!-- cargo-sync-rdme title:\0 -->",
            "<!-- cargo-sync-rdme title:\0 -->",
            "Good afternoon, world!",
            "# Heading!",
            "<!-- cargo-sync-rdme title:\0 -->",
            "Good evening, world!",
        ];
        let source = lines.join("\n");
        let source = Spanned::from_str(&source);

        let config = indoc! {"
            [package.metadata.cargo-sync-rdme.badge.badges]
        "};

        let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
        let mut errors = Scanner::new(&manifest, source.value).map(|res| res.unwrap_err());

        let (token, expected, span) = errors
            .next()
            .unwrap()
            .into_parse_marker()
            .into_unexpected_token();
        assert_eq!(token, "\0");
        assert_eq!(expected, "group name");
        source.assert_source_span(span, "\0");

        let (token, expected, span) = errors
            .next()
            .unwrap()
            .into_parse_marker()
            .into_unexpected_token();
        assert_eq!(token, "\0");
        assert_eq!(expected, "group name");
        source.assert_source_span(span, "\0");

        let (token, expected, span) = errors
            .next()
            .unwrap()
            .into_parse_marker()
            .into_unexpected_token();
        assert_eq!(token, "\0");
        assert_eq!(expected, "group name");
        source.assert_source_span(span, "\0");

        assert!(errors.next().is_none());
    }
}
