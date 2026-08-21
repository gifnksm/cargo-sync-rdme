use std::sync::Arc;

use miette::{NamedSource, SourceSpan};
use snafu::{OptionExt as _, Snafu, ensure};

use crate::{
    parse::Spanned,
    sync::{
        ManifestFile, MarkdownPath,
        marker::{
            MAGIC, ResolveMarkerError,
            parse::{self, MarkerParser},
            resolve,
        },
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
    parser: MarkerParser<'markdown>,
}

impl Iterator for Scanner<'_, '_> {
    type Item = Result<Spanned<ResolvedReplaceSpecifier>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl<'manifest, 'markdown> Scanner<'manifest, 'markdown> {
    fn new(manifest: &'manifest ManifestFile, markdown: &'markdown str) -> Self {
        let parser = MarkerParser::new(markdown);
        Self { manifest, parser }
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
        let Some(marker) = self.parser.try_next()? else {
            return Ok(None);
        };
        let marker = resolve::resolve_marker(marker, self.manifest)?;
        Ok(Some(marker))
    }
}

#[cfg(test)]
mod tests {
    use std::range::Range;

    use indoc::indoc;
    use similar_asserts::assert_eq;

    use crate::config::Manifest;

    use super::*;

    fn line_ranges(lines: &[impl AsRef<str>]) -> Vec<Range<usize>> {
        lines
            .iter()
            .scan(0, |offset, line| {
                let line = line.as_ref();
                let range = Range::from(*offset..*offset + line.len());
                *offset = range.end + 1; // +1 for the newline character
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
}
