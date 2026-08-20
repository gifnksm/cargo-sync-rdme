use std::{range::Range, sync::Arc};

use miette::{NamedSource, SourceSpan};
use pulldown_cmark::Event;
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

pub(in crate::sync) fn scan_all<'events>(
    markdown: &MarkdownFile<'_>,
    manifest: &ManifestFile,
    events: impl IntoIterator<Item = (Event<'events>, Range<usize>)> + 'events,
) -> Result<Vec<Spanned<ResolvedReplaceSpecifier>>, Box<ScanAllError>> {
    let events = events.into_iter();
    let it = Iter { manifest, events };
    let mut markers = vec![];
    let mut errors = vec![];
    for res in it {
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
struct Iter<'manifest, I> {
    manifest: &'manifest ManifestFile,
    events: I,
}

impl<'event, I> Iterator for Iter<'_, I>
where
    I: Iterator<Item = (Event<'event>, Range<usize>)>,
{
    type Item = Result<Spanned<ResolvedReplaceSpecifier>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

impl<'event, I> Iter<'_, I>
where
    I: Iterator<Item = (Event<'event>, Range<usize>)>,
{
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
                (start_span.start..end_span.end).into(),
            ))),
            _ => Err(NestedMarkerSnafu {
                nested_span: end_span.to_span(),
                previous_span: start_span.to_span(),
            }
            .build()),
        }
    }
}

impl<'event, I> Iter<'_, I>
where
    I: Iterator<Item = (Event<'event>, Range<usize>)>,
{
    fn next_marker(&mut self) -> Result<Option<Spanned<ResolvedMarker>>, ScanError> {
        for (event, range) in self.events.by_ref() {
            if let Event::Html(html) = event {
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
    use pulldown_cmark::Parser;
    use similar_asserts::assert_eq;

    use crate::config::Manifest;

    use super::*;

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
        let mut markers = Iter {
            manifest: &ManifestFile::dummy(Manifest::default()),
            events: Parser::new(input)
                .into_offset_iter()
                .map(|(event, range)| (event, Range::from(range))),
        };
        assert!(markers.next().is_none());
    }

    #[test]
    fn replace_marker() {
        let lines = [
            "Good morning, world!".to_string(),
            ResolvedMarker::Replace(ResolvedReplaceSpecifier::Title).to_string(),
            "Good afternoon, world!".to_string(),
            ResolvedMarker::Replace(ResolvedReplaceSpecifier::Badge {
                name: "".into(),
                badges: vec![].into(),
            })
            .to_string(),
            "Good evening, world!".to_string(),
            ResolvedMarker::Replace(ResolvedReplaceSpecifier::Rustdoc).to_string(),
            "Good night, world!".to_string(),
        ];
        let ranges = line_ranges(&lines);
        let input = lines.join("\n");

        let config = indoc::indoc! {"
            [package.metadata.cargo-sync-rdme.badge.badges]
        "};

        let mut markers = Iter {
            manifest: &ManifestFile::dummy(toml::from_str(config).unwrap()),
            events: Parser::new(&input)
                .into_offset_iter()
                .map(|(event, range)| (event, Range::from(range))),
        };
        assert_eq!(
            markers.next().unwrap().unwrap(),
            Spanned::new(ResolvedReplaceSpecifier::Title, ranges[1])
        );
        assert_eq!(
            markers.next().unwrap().unwrap(),
            Spanned::new(
                ResolvedReplaceSpecifier::Badge {
                    name: "".into(),
                    badges: vec![].into()
                },
                ranges[3],
            )
        );
        assert_eq!(
            markers.next().unwrap().unwrap(),
            Spanned::new(ResolvedReplaceSpecifier::Rustdoc, ranges[5])
        );
        assert!(markers.next().is_none());
    }

    #[test]
    fn replace_region() {
        let lines = [
            "Good morning, world!".to_string(),
            ResolvedMarker::Start(ResolvedReplaceSpecifier::Title).to_string(),
            "Good afternoon, world!".to_string(),
            "# Heading!".to_string(),
            ResolvedMarker::End.to_string(),
            "Good evening, world!".to_string(),
        ];
        let ranges = line_ranges(&lines);
        let input = lines.join("\n");

        let config = indoc::indoc! {"
            [package.metadata.cargo-sync-rdme.badge.badges]
        "};

        let mut markers = Iter {
            manifest: &ManifestFile::dummy(toml::from_str(config).unwrap()),
            events: Parser::new(&input)
                .into_offset_iter()
                .map(|(event, range)| (event, Range::from(range))),
        };
        assert_eq!(
            markers.next().unwrap().unwrap(),
            Spanned::new(
                ResolvedReplaceSpecifier::Title,
                (ranges[1].start..ranges[4].end).into()
            ),
        );
        assert!(markers.next().is_none());
    }
}
