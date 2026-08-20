use std::fmt::{self, Display};

use miette::{Diagnostic, SourceSpan};
use snafu::{Snafu, ensure};

use crate::{parse::Spanned, sync::marker::MAGIC, traits::RangeExt as _};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Marker<'a> {
    Replace(Spanned<ReplaceSpecifier<'a>>),
    Start(Spanned<ReplaceSpecifier<'a>>),
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ReplaceSpecifier<'a> {
    pub(super) kind: Spanned<&'a str>,
    pub(super) group: Option<Spanned<&'a str>>,
}

impl Display for ReplaceSpecifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind;
        if let Some(group) = self.group {
            write!(f, "{kind}:{group}")
        } else {
            write!(f, "{kind}")
        }
    }
}

#[derive(Debug, Snafu, Diagnostic)]
pub(crate) enum ParseMarkerError {
    #[snafu(display("no replacement specifier found"))]
    NoReplaceSpecifier {
        #[label]
        span: SourceSpan,
    },
    #[snafu(display("empty marker kind"))]
    EmptyMarkerKind {
        #[label]
        span: SourceSpan,
    },
    #[snafu(display("empty group name"))]
    EmptyGroupName {
        #[label]
        span: SourceSpan,
    },
}

pub(super) fn parse_marker(
    html: Spanned<&str>,
) -> Result<Option<Spanned<Marker<'_>>>, ParseMarkerError> {
    let html_span = html.span;

    let Some(comment_body) = trim_comment(html) else {
        return Ok(None);
    };
    let Some(marker_body) = trim_magic(comment_body) else {
        return Ok(None);
    };

    ensure!(
        !marker_body.value.is_empty(),
        NoReplaceSpecifierSnafu {
            span: html_span.to_span(),
        }
    );

    if let Some(specifier) = marker_body.strip_suffix_str("[[") {
        let Some(specifier) = parse_specifier(specifier)? else {
            return Err(NoReplaceSpecifierSnafu {
                span: html_span.to_span(),
            }
            .build());
        };
        return Ok(Some(Spanned::new(Marker::Start(specifier), html_span)));
    }

    if marker_body == "]]" {
        return Ok(Some(Spanned::new(Marker::End, html_span)));
    }

    let Some(specifier) = parse_specifier(marker_body)? else {
        return Err(NoReplaceSpecifierSnafu {
            span: html_span.to_span(),
        }
        .build());
    };
    Ok(Some(Spanned::new(Marker::Replace(specifier), html_span)))
}

fn parse_specifier(
    marker_body: Spanned<&str>,
) -> Result<Option<Spanned<ReplaceSpecifier<'_>>>, ParseMarkerError> {
    let specifier = marker_body.trim();

    if specifier.value.is_empty() {
        return Ok(None);
    }

    let (kind, group) = match specifier.split_once_char(':') {
        Some((kind, group)) => (kind, Some(group)),
        None => (specifier, None),
    };
    let kind = kind.trim();
    let group = group.map(|g| g.trim());

    if kind.value.is_empty() {
        return Err(EmptyMarkerKindSnafu {
            span: specifier.source_span(),
        }
        .build());
    }
    if group.is_some_and(|group| group.value.is_empty()) {
        return Err(EmptyGroupNameSnafu {
            span: specifier.source_span(),
        }
        .build());
    }

    Ok(Some(Spanned::new(
        ReplaceSpecifier { kind, group },
        specifier.span,
    )))
}

fn trim_magic(comment_body: Spanned<&str>) -> Option<Spanned<&str>> {
    let (head, tail) = comment_body
        .split_once_fn(char::is_whitespace)
        .unwrap_or((comment_body, comment_body.end()));
    if head != MAGIC {
        return None;
    }
    Some(tail.trim())
}

fn trim_comment(html: Spanned<&str>) -> Option<Spanned<&str>> {
    let comment_body = html
        .trim()
        .strip_prefix_str("<!--")?
        .trim_start()
        .strip_suffix_str("-->")?
        .trim_end();
    Some(comment_body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_marker_parses_markers() {
        let source = Spanned::from_str("<!-- cargo-sync-rdme kind:group -->");
        let marker = parse_marker(source).unwrap().unwrap();
        source.assert_span(marker, "<!-- cargo-sync-rdme kind:group -->");
        let Marker::Replace(specifier) = marker.value else {
            panic!("expected replace marker, got {marker:?}");
        };
        source.assert_span(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        let source = Spanned::from_str("<!-- cargo-sync-rdme kind:group [[ -->");
        let marker = parse_marker(source).unwrap().unwrap();
        source.assert_span(marker, "<!-- cargo-sync-rdme kind:group [[ -->");
        let Marker::Start(specifier) = marker.value else {
            panic!("expected replace marker, got {marker:?}");
        };
        source.assert_span(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        let source = Spanned::from_str("<!-- cargo-sync-rdme ]] -->");
        let marker = parse_marker(source).unwrap().unwrap();
        source.assert_span(marker, "<!-- cargo-sync-rdme ]] -->");
        let Marker::End = marker.value else {
            panic!("expected end marker, got {marker:?}");
        };
    }

    #[test]
    fn parse_marker_rejects_invalid_markers() {
        let source = Spanned::from_str("<!-- cargo-sync-rdme -->");
        let err = parse_marker(source).unwrap_err();
        let ParseMarkerError::NoReplaceSpecifier { span } = err else {
            panic!("unexpected error: {err:?}");
        };
        source.assert_source_span(span, "<!-- cargo-sync-rdme -->");

        let source = Spanned::from_str("<!-- cargo-sync-rdme [[ -->");
        let err = parse_marker(source).unwrap_err();
        let ParseMarkerError::NoReplaceSpecifier { span } = err else {
            panic!("unexpected error: {err:?}");
        };
        source.assert_source_span(span, "<!-- cargo-sync-rdme [[ -->");
    }

    #[test]
    fn parse_marker_ignores_non_markers() {
        let source = Spanned::from_str("<!-- test -->");
        assert!(parse_marker(source).unwrap().is_none());
    }

    #[test]
    fn parse_specifier_parses_kind_and_group() {
        let source = Spanned::from_str("kind:group");
        let specifier = parse_specifier(source).unwrap().unwrap();
        source.assert_span(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        let source = Spanned::from_str(" kind:group ");
        let specifier = parse_specifier(source).unwrap().unwrap();
        source.assert_span(specifier, "kind:group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        let source = Spanned::from_str(" kind : group ");
        let specifier = parse_specifier(source).unwrap().unwrap();
        source.assert_span(specifier, "kind : group");
        source.assert_spanned_str(specifier.value.kind, "kind");
        source.assert_spanned_str(specifier.value.group.unwrap(), "group");

        let source = Spanned::from_str(" kind ");
        let specifier = parse_specifier(source).unwrap().unwrap();
        source.assert_span(specifier, "kind");
        source.assert_spanned_str(specifier.value.kind, "kind");
        assert!(specifier.value.group.is_none());

        let source = Spanned::from_str("  ");
        assert!(parse_specifier(source).unwrap().is_none());
    }

    #[test]
    fn parse_specifier_rejects_invalid_specifiers() {
        let source = Spanned::from_str(" kind: ");
        let err = parse_specifier(source).unwrap_err();
        let ParseMarkerError::EmptyGroupName { span } = err else {
            panic!("unexpected error: {err:?}");
        };
        source.assert_source_span(span, "kind:");

        let source = Spanned::from_str(" :group");
        let err = parse_specifier(source).unwrap_err();
        let ParseMarkerError::EmptyMarkerKind { span } = err else {
            panic!("unexpected error: {err:?}");
        };
        source.assert_source_span(span, ":group");
    }

    #[test]
    fn trim_magic_trims_magic() {
        let source = Spanned::from_str("cargo-sync-rdme   test  foo bar");
        let trimmed = trim_magic(source).unwrap();
        source.assert_spanned_str(trimmed, "test  foo bar");

        let source = Spanned::from_str("cargo-sync-rdme");
        let trimmed = trim_magic(source).unwrap();
        source.assert_spanned_str(trimmed, "");

        let source = Spanned::from_str("cargo-sync-rdmexxx");
        assert!(trim_magic(source).is_none());
    }

    #[test]
    fn trim_comment_trims_comment_tags() {
        let source = Spanned::from_str("<!-- test -->");
        let trimmed = trim_comment(source).unwrap();
        source.assert_spanned_str(trimmed, "test");

        let source = Spanned::from_str("<!--test-->");
        let trimmed = trim_comment(source).unwrap();
        source.assert_spanned_str(trimmed, "test");
    }

    #[test]
    fn trim_comment_ignores_non_comment() {
        let source = Spanned::from_str("test");
        assert!(trim_comment(source).is_none());
        let source = Spanned::from_str("<!--test");
        assert!(trim_comment(source).is_none());
    }
}
