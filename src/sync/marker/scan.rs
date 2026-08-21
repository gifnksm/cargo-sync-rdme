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
    use indoc::indoc;
    use similar_asserts::assert_eq;

    use crate::config::Manifest;

    use super::*;

    impl ScanError {
        #[track_caller]
        pub(crate) fn into_unexpected_end_marker(self) -> SourceSpan {
            let Self::UnexpectedEndMarker { span } = self else {
                panic!("unexpected error: {self:?}");
            };
            span
        }

        #[track_caller]
        pub(crate) fn into_no_corresponding_end_marker(self) -> SourceSpan {
            let Self::NoCorrespondingEndMarker { start_span } = self else {
                panic!("unexpected error: {self:?}");
            };
            start_span
        }

        #[track_caller]
        pub(crate) fn into_nested_marker(self) -> (SourceSpan, SourceSpan) {
            let Self::NestedMarker {
                nested_span,
                previous_span,
            } = self
            else {
                panic!("unexpected error: {self:?}");
            };
            (nested_span, previous_span)
        }
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
        let source = indoc! {"
            Good morning, world!
            <!-- cargo-sync-rdme title -->
            Good afternoon, world!
              <!--  cargo-sync-rdme badge  -->
            Good evening, world!
            <!--cargo-sync-rdme rustdoc  -->
            Good night, world!
        "};
        let source = Spanned::from_str(source);
        let config = indoc! {"
            [package.metadata.cargo-sync-rdme.badge.badges]
        "};
        let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
        let mut scanner = Scanner::new(&manifest, source.value);

        let marker = scanner.try_next().unwrap().unwrap();
        source.assert_span(marker.span, "<!-- cargo-sync-rdme title -->");
        marker.value.into_title();

        let marker = scanner.try_next().unwrap().unwrap();
        source.assert_span(marker.span, "<!--  cargo-sync-rdme badge  -->");
        let (group, badges) = marker.value.into_badge();
        assert!(group.is_none());
        assert_eq!(*badges, []);

        let marker = scanner.try_next().unwrap().unwrap();
        source.assert_span(marker.span, "<!--cargo-sync-rdme rustdoc  -->");
        marker.value.into_rustdoc();

        assert!(scanner.try_next().unwrap().is_none());
    }

    #[test]
    fn start_and_end_marker() {
        let source = indoc! {"
            Good morning, world!
              <!-- cargo-sync-rdme rustdoc [[ -->
            Good afternoon, world!
            # Heading!
              <!-- cargo-sync-rdme ]] -->
            Good evening, world!
        "};
        let source = Spanned::from_str(source);
        let manifest = ManifestFile::dummy(Manifest::default());
        let mut scanner = Scanner::new(&manifest, source.value);

        let marker = scanner.try_next().unwrap().unwrap();
        let span_source = &source.value[marker.span];
        assert!(span_source.starts_with("<!-- cargo-sync-rdme rustdoc [[ -->"));
        assert!(span_source.ends_with("<!-- cargo-sync-rdme ]] -->"));
        marker.value.into_rustdoc();
    }

    #[test]
    fn unexpected_end_marker() {
        let source = indoc! {"
            Good morning, world!
            <!-- cargo-sync-rdme ]] -->
            Bad end of the world!
        "};

        let source = Spanned::from_str(source);
        let manifest = ManifestFile::dummy(Manifest::default());
        let mut scanner = Scanner::new(&manifest, source.value);

        let span = scanner.try_next().unwrap_err().into_unexpected_end_marker();
        source.assert_source_span(span, "<!-- cargo-sync-rdme ]] -->");
    }

    #[test]
    fn no_corresponding_end_marker() {
        let source = indoc! {"
            Good morning, world!
            <!-- cargo-sync-rdme rustdoc [[ -->
            Bad end of the world!
        "};

        let source = Spanned::from_str(source);
        let manifest = ManifestFile::dummy(Manifest::default());
        let mut scanner = Scanner::new(&manifest, source.value);

        let span = scanner
            .try_next()
            .unwrap_err()
            .into_no_corresponding_end_marker();
        source.assert_source_span(span, "<!-- cargo-sync-rdme rustdoc [[ -->");
    }

    #[test]
    fn nested_marker() {
        let source = indoc! {"
            Good morning, world!
            <!-- cargo-sync-rdme rustdoc [[ -->
            Good afternoon, world!
            <!-- cargo-sync-rdme title   [[ -->
            Bad nested, world!
            <!-- cargo-sync-rdme ]] -->
            Good evening, world!
            <!-- cargo-sync-rdme   ]] -->
            Good night, world!
        "};
        let source = Spanned::from_str(source);
        let manifest = ManifestFile::dummy(Manifest::default());
        let mut scanner = Scanner::new(&manifest, source.value);

        let (nested_span, previous_span) = scanner.try_next().unwrap_err().into_nested_marker();
        source.assert_source_span(nested_span, "<!-- cargo-sync-rdme title   [[ -->");
        source.assert_source_span(previous_span, "<!-- cargo-sync-rdme rustdoc [[ -->");
    }
}
