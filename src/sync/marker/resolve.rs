use std::{fmt, sync::Arc};

use miette::{Diagnostic, SourceSpan};
use snafu::Snafu;

use crate::{
    config::metadata::BadgeItem,
    parse::Spanned,
    sync::{
        ManifestFile,
        marker::{
            MAGIC,
            parse::{Marker, ReplaceSpecifier},
        },
    },
    traits::RangeExt as _,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ResolvedMarker {
    Replace(ResolvedReplaceSpecifier),
    Start(ResolvedReplaceSpecifier),
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) enum ResolvedReplaceSpecifier {
    Title,
    Badge {
        name: Arc<str>,
        badges: Arc<[BadgeItem]>,
    },
    Rustdoc,
}

impl fmt::Display for ResolvedMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Replace(specifier) => write!(f, "<!-- {MAGIC} {specifier} -->"),
            Self::Start(specifier) => write!(f, "<!-- {MAGIC} {specifier} [[ -->"),
            Self::End => write!(f, "<!-- {MAGIC} ]] -->"),
        }
    }
}

impl fmt::Display for ResolvedReplaceSpecifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Title => write!(f, "title"),
            Self::Badge { name, .. } => {
                if name.is_empty() {
                    write!(f, "badge")
                } else {
                    write!(f, "badge:{name}")
                }
            }
            Self::Rustdoc => write!(f, "rustdoc"),
        }
    }
}

#[derive(Debug, Snafu, Diagnostic)]
pub(super) enum ResolveMarkerError {
    #[snafu(display("unknown marker kind: {kind}"))]
    UnknownMarkerKind {
        kind: String,
        #[label]
        span: SourceSpan,
    },
    #[snafu(display("marker kind `{kind}` cannot have group suffix `:{group}`"))]
    UnexpectedGroupForSpecifier {
        kind: String,
        group: String,
        #[label]
        span: SourceSpan,
    },
    #[snafu(display(
        "default badge group is not configured in the package manifest: package.metadata.cargo-sync-rdme.badge.badges"
    ))]
    NoDefaultBadgeConfigured {
        #[label]
        span: SourceSpan,
    },
    #[snafu(display(
        "badge group not found in the package manifest: package.metadata.cargo-sync-rdme.badge.badges-{group}"
    ))]
    NoSuchBadgeGroup {
        group: String,
        #[label]
        span: SourceSpan,
    },
}

pub(super) fn resolve_marker(
    marker: Spanned<Marker<'_>>,
    manifest: &ManifestFile,
) -> Result<Spanned<ResolvedMarker>, ResolveMarkerError> {
    match marker.value {
        Marker::Replace(specifier) => {
            let specifier = resolve_specifier(specifier, manifest)?;
            Ok(Spanned::new(
                ResolvedMarker::Replace(specifier),
                marker.span,
            ))
        }
        Marker::Start(specifier) => {
            let specifier = resolve_specifier(specifier, manifest)?;
            Ok(Spanned::new(ResolvedMarker::Start(specifier), marker.span))
        }
        Marker::End => Ok(Spanned::new(ResolvedMarker::End, marker.span)),
    }
}

fn resolve_specifier(
    specifier: Spanned<ReplaceSpecifier<'_>>,
    manifest: &ManifestFile,
) -> Result<ResolvedReplaceSpecifier, ResolveMarkerError> {
    let kind = specifier.value.kind;
    let group = specifier.value.group;
    match (kind.value, group) {
        ("title", None) => return Ok(ResolvedReplaceSpecifier::Title),
        ("rustdoc", None) => return Ok(ResolvedReplaceSpecifier::Rustdoc),
        ("title" | "rustdoc", Some(group)) => {
            return Err(ResolveMarkerError::UnexpectedGroupForSpecifier {
                kind: kind.value.to_string(),
                group: group.value.to_string(),
                span: specifier.span.to_span(),
            });
        }
        ("badge", _) => {}
        _ => {
            return Err(ResolveMarkerError::UnknownMarkerKind {
                kind: kind.value.to_string(),
                span: specifier.span.to_span(),
            });
        }
    }

    let badges = &manifest.value().config().badge.badges;
    let (name, badges) = if let Some(group) = group {
        badges.get_key_value(group.value).ok_or_else(|| {
            NoSuchBadgeGroupSnafu {
                group: group.value,
                span: group.source_span(),
            }
            .build()
        })?
    } else {
        badges.get_key_value("").ok_or_else(|| {
            NoDefaultBadgeConfiguredSnafu {
                span: specifier.source_span(),
            }
            .build()
        })?
    };

    Ok(ResolvedReplaceSpecifier::Badge {
        name: Arc::clone(name),
        badges: Arc::clone(badges),
    })
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use crate::sync::marker::parse;

    use super::*;

    static CONFIG: &str = indoc::indoc! {"
        [package.metadata.cargo-sync-rdme.badge.badges]
        [package.metadata.cargo-sync-rdme.badge.badges-foo]
    "};

    fn resolve(
        source: Spanned<&str>,
        config: &str,
    ) -> Result<Spanned<ResolvedMarker>, ResolveMarkerError> {
        let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
        let marker = parse::parse_marker(source).unwrap().unwrap();
        resolve_marker(marker, &manifest)
    }

    #[test]
    fn resolve_marker_resolves_valid_markers() {
        let source = Spanned::from_str("<!-- cargo-sync-rdme title -->");
        let resolved = resolve(source, CONFIG).unwrap();
        let ResolvedMarker::Replace(ResolvedReplaceSpecifier::Title) = resolved.value else {
            panic!("unexpected: {resolved:?}");
        };
        source.assert_span(resolved, "<!-- cargo-sync-rdme title -->");

        let source = Spanned::from_str("<!-- cargo-sync-rdme rustdoc -->");
        let resolved = resolve(source, CONFIG).unwrap();
        let ResolvedMarker::Replace(ResolvedReplaceSpecifier::Rustdoc) = resolved.value else {
            panic!("unexpected: {resolved:?}");
        };
        source.assert_span(resolved, "<!-- cargo-sync-rdme rustdoc -->");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge -->");
        let resolved = resolve(source, CONFIG).unwrap();
        let ResolvedMarker::Replace(ResolvedReplaceSpecifier::Badge { name, .. }) = &resolved.value
        else {
            panic!("unexpected: {resolved:?}");
        };
        assert!(name.is_empty());
        source.assert_span(resolved, "<!-- cargo-sync-rdme badge -->");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge:foo -->");
        let resolved = resolve(source, CONFIG).unwrap();
        let ResolvedMarker::Replace(ResolvedReplaceSpecifier::Badge { name, .. }) = &resolved.value
        else {
            panic!("unexpected: {resolved:?}");
        };
        assert_eq!(name.as_ref(), "foo");
        source.assert_span(resolved, "<!-- cargo-sync-rdme badge:foo -->");
    }

    #[test]
    fn resolve_marker_rejects_invalid_markers() {
        let source = Spanned::from_str("<!-- cargo-sync-rdme unknown -->");
        let err = resolve(source, CONFIG).unwrap_err();
        let ResolveMarkerError::UnknownMarkerKind { kind, span } = err else {
            panic!("unexpected: {err:?}");
        };
        assert_eq!(kind, "unknown");
        source.assert_source_span(span, "unknown");

        let source = Spanned::from_str("<!-- cargo-sync-rdme title:foo -->");
        let err = resolve(source, CONFIG).unwrap_err();
        let ResolveMarkerError::UnexpectedGroupForSpecifier { kind, group, span } = err else {
            panic!("unexpected: {err:?}");
        };
        assert_eq!(kind, "title");
        assert_eq!(group, "foo");
        source.assert_source_span(span, "title:foo");

        let source = Spanned::from_str("<!-- cargo-sync-rdme rustdoc:foo -->");
        let err = resolve(source, CONFIG).unwrap_err();
        let ResolveMarkerError::UnexpectedGroupForSpecifier { kind, group, span } = err else {
            panic!("unexpected: {err:?}");
        };
        assert_eq!(kind, "rustdoc");
        assert_eq!(group, "foo");
        source.assert_source_span(span, "rustdoc:foo");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge:bar -->");
        let err = resolve(source, CONFIG).unwrap_err();
        let ResolveMarkerError::NoSuchBadgeGroup { group, span } = err else {
            panic!("unexpected: {err:?}");
        };
        assert_eq!(group, "bar");
        source.assert_source_span(span, "bar");
    }
}
