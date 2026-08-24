use miette::SourceSpan;
use snafu::{OptionExt as _, Snafu};

use crate::{
    source::Spanned,
    sync::marker::parse::{self, Marker, MarkerParser, ReplaceSpecifier},
    traits::RangeExt as _,
};

#[expect(clippy::enum_variant_names)]
#[derive(Debug, Snafu, miette::Diagnostic)]
pub(super) enum ScanError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    ParseMarker {
        #[snafu(source)]
        #[diagnostic_source]
        source: parse::ParseMarkerError,
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
pub(super) struct Chunk<'a> {
    pub(super) specifier: Spanned<ReplaceSpecifier<'a>>,
}

#[derive(Debug)]
pub(super) struct Scanner<'a> {
    parser: MarkerParser<'a>,
}

impl<'a> Scanner<'a> {
    pub(super) fn new(markdown: &'a str) -> Self {
        let parser = MarkerParser::new(markdown);
        Self { parser }
    }

    pub(super) fn try_next(&mut self) -> Result<Option<Spanned<Chunk<'a>>>, ScanError> {
        let Some(start_marker) = self.next_marker()? else {
            return Ok(None);
        };
        let start_span = start_marker.span;
        let specifier = match start_marker.value {
            Marker::Replace(specifier) => {
                return Ok(Some(Spanned::new(Chunk { specifier }, start_span)));
            }
            Marker::Start(specifier) => specifier,
            Marker::End => {
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
            Marker::End => Ok(Some(Spanned::new(
                Chunk { specifier },
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

impl<'a> Scanner<'a> {
    fn next_marker(&mut self) -> Result<Option<Spanned<Marker<'a>>>, ScanError> {
        let Some(marker) = self.parser.try_next()? else {
            return Ok(None);
        };
        Ok(Some(marker))
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

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
        let mut markers = Scanner::new(input);
        assert!(markers.try_next().unwrap().is_none());
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
        let mut scanner = Scanner::new(source.value);

        let chunk = scanner.try_next().unwrap().unwrap();
        source.assert_span(chunk.span, "<!-- cargo-sync-rdme title -->");
        let specifier = chunk.value.specifier;
        source.assert_span(specifier.span, "title");
        source.assert_spanned_str(specifier.value.kind, "title");
        assert!(specifier.value.group.is_none());

        let chunk = scanner.try_next().unwrap().unwrap();
        source.assert_span(chunk.span, "<!--  cargo-sync-rdme badge  -->");
        let specifier = chunk.value.specifier;
        source.assert_span(specifier.span, "badge");
        source.assert_spanned_str(specifier.value.kind, "badge");
        assert!(specifier.value.group.is_none());

        let chunk = scanner.try_next().unwrap().unwrap();
        source.assert_span(chunk.span, "<!--cargo-sync-rdme rustdoc  -->");
        let specifier = chunk.value.specifier;
        source.assert_span(specifier.span, "rustdoc");
        source.assert_spanned_str(specifier.value.kind, "rustdoc");
        assert!(specifier.value.group.is_none());

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
        let mut scanner = Scanner::new(source.value);

        let chunk = scanner.try_next().unwrap().unwrap();
        let span_source = &source.value[chunk.span];
        assert!(span_source.starts_with("<!-- cargo-sync-rdme rustdoc [[ -->"));
        assert!(span_source.ends_with("<!-- cargo-sync-rdme ]] -->"));
        let specifier = chunk.value.specifier;
        source.assert_span(specifier.span, "rustdoc");
        source.assert_spanned_str(specifier.value.kind, "rustdoc");
        assert!(specifier.value.group.is_none());
    }

    #[test]
    fn unexpected_end_marker() {
        let source = indoc! {"
            Good morning, world!
            <!-- cargo-sync-rdme ]] -->
            Bad end of the world!
        "};

        let source = Spanned::from_str(source);
        let mut scanner = Scanner::new(source.value);

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
        let mut scanner = Scanner::new(source.value);

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
        let mut scanner = Scanner::new(source.value);

        let (nested_span, previous_span) = scanner.try_next().unwrap_err().into_nested_marker();
        source.assert_source_span(nested_span, "<!-- cargo-sync-rdme title   [[ -->");
        source.assert_source_span(previous_span, "<!-- cargo-sync-rdme rustdoc [[ -->");
    }
}
