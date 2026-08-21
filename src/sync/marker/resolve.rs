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
        group: Option<Arc<str>>,
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
            Self::Badge { group, .. } => {
                if let Some(name) = group {
                    write!(f, "badge:{name}")
                } else {
                    write!(f, "badge")
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

    let badge = &manifest.value().config().badge;
    if let Some(group) = group {
        let (group, badges) = badge.groups.get_key_value(group.value).ok_or_else(|| {
            NoSuchBadgeGroupSnafu {
                group: group.value,
                span: group.source_span(),
            }
            .build()
        })?;
        Ok(ResolvedReplaceSpecifier::Badge {
            group: Some(Arc::clone(group)),
            badges: Arc::clone(badges),
        })
    } else {
        let badges = badge.default.as_ref().ok_or_else(|| {
            NoDefaultBadgeConfiguredSnafu {
                span: specifier.source_span(),
            }
            .build()
        })?;
        Ok(ResolvedReplaceSpecifier::Badge {
            group: None,
            badges: Arc::clone(badges),
        })
    }
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

    impl ResolvedMarker {
        #[track_caller]
        fn into_replace(self) -> ResolvedReplaceSpecifier {
            let Self::Replace(specifier) = self else {
                panic!("unexpected marker: {self:?}");
            };
            specifier
        }
    }

    impl ResolvedReplaceSpecifier {
        #[track_caller]
        fn into_title(self) {
            let Self::Title = self else {
                panic!("unexpected replace specifier: {self:?}");
            };
        }

        #[track_caller]
        fn into_rustdoc(self) {
            let Self::Rustdoc = self else {
                panic!("unexpected replace specifier: {self:?}");
            };
        }

        #[track_caller]
        fn into_badge(self) -> (Option<Arc<str>>, Arc<[BadgeItem]>) {
            let Self::Badge { group, badges } = self else {
                panic!("unexpected replace specifier: {self:?}");
            };
            (group, badges)
        }
    }

    impl ResolveMarkerError {
        #[track_caller]
        fn into_unknown_marker_kind(self) -> (String, SourceSpan) {
            let Self::UnknownMarkerKind { kind, span } = self else {
                panic!("unexpected error: {self:?}");
            };
            (kind, span)
        }
        #[track_caller]
        fn into_unexpected_group_for_specifier(self) -> (String, String, SourceSpan) {
            let Self::UnexpectedGroupForSpecifier { kind, group, span } = self else {
                panic!("unexpected error: {self:?}");
            };
            (kind, group, span)
        }

        #[track_caller]
        fn into_no_default_badge_configured(self) -> SourceSpan {
            let Self::NoDefaultBadgeConfigured { span } = self else {
                panic!("unexpected error: {self:?}");
            };
            span
        }

        #[track_caller]
        fn into_no_such_badge_group(self) -> (String, SourceSpan) {
            let Self::NoSuchBadgeGroup { group, span } = self else {
                panic!("unexpected error: {self:?}");
            };
            (group, span)
        }
    }

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
        resolved.value.into_replace().into_title();
        source.assert_span(resolved.span, "<!-- cargo-sync-rdme title -->");

        let source = Spanned::from_str("<!-- cargo-sync-rdme rustdoc -->");
        let resolved = resolve(source, CONFIG).unwrap();
        resolved.value.into_replace().into_rustdoc();
        source.assert_span(resolved.span, "<!-- cargo-sync-rdme rustdoc -->");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge -->");
        let resolved = resolve(source, CONFIG).unwrap();
        let (group, _badges) = resolved.value.into_replace().into_badge();
        assert!(group.is_none());
        source.assert_span(resolved.span, "<!-- cargo-sync-rdme badge -->");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge:foo -->");
        let resolved = resolve(source, CONFIG).unwrap();
        let (group, _badges) = resolved.value.into_replace().into_badge();
        assert_eq!(group.as_deref().unwrap(), "foo");
        source.assert_span(resolved.span, "<!-- cargo-sync-rdme badge:foo -->");
    }

    #[test]
    fn resolve_marker_rejects_invalid_markers() {
        let source = Spanned::from_str("<!-- cargo-sync-rdme unknown -->");
        let (kind, span) = resolve(source, CONFIG)
            .unwrap_err()
            .into_unknown_marker_kind();
        assert_eq!(kind, "unknown");
        source.assert_source_span(span, "unknown");

        let source = Spanned::from_str("<!-- cargo-sync-rdme title:foo -->");
        let (kind, group, span) = resolve(source, CONFIG)
            .unwrap_err()
            .into_unexpected_group_for_specifier();
        assert_eq!(kind, "title");
        assert_eq!(group, "foo");
        source.assert_source_span(span, "title:foo");

        let source = Spanned::from_str("<!-- cargo-sync-rdme rustdoc:foo -->");
        let (kind, group, span) = resolve(source, CONFIG)
            .unwrap_err()
            .into_unexpected_group_for_specifier();
        assert_eq!(kind, "rustdoc");
        assert_eq!(group, "foo");
        source.assert_source_span(span, "rustdoc:foo");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge:bar -->");
        let (group, span) = resolve(source, CONFIG)
            .unwrap_err()
            .into_no_such_badge_group();
        assert_eq!(group, "bar");
        source.assert_source_span(span, "bar");

        let source = Spanned::from_str("<!-- cargo-sync-rdme badge -->");
        let span = resolve(source, "")
            .unwrap_err()
            .into_no_default_badge_configured();
        source.assert_source_span(span, "badge");
    }
}
