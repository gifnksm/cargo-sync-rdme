use std::sync::Arc;

use miette::{Diagnostic, SourceSpan};
use snafu::Snafu;

use crate::{
    config::Config,
    source::Spanned,
    sync::{
        PackageSyncContext,
        marker::{
            ResolvedReplaceSpecifier,
            parse::ReplaceSpecifier,
            scan::{ScanError, Scanner},
        },
    },
};

#[derive(Debug, Snafu, Diagnostic)]
pub(super) enum ResolveMarkerError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    Scan {
        #[snafu(source)]
        #[diagnostic_source]
        source: ScanError,
    },
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

#[derive(Debug)]
pub(super) struct Resolver<'cx, 'markdown> {
    cx: &'cx PackageSyncContext<'cx>,
    scanner: Scanner<'markdown>,
}

impl<'cx, 'markdown> Resolver<'cx, 'markdown> {
    pub(super) fn new(cx: &'cx PackageSyncContext<'cx>, markdown: &'markdown str) -> Self {
        Self {
            cx,
            scanner: Scanner::new(markdown),
        }
    }

    pub(super) fn try_next(
        &mut self,
    ) -> Result<Option<Spanned<ResolvedReplaceSpecifier>>, ResolveMarkerError> {
        let Some(chunk) = self.scanner.try_next()? else {
            return Ok(None);
        };
        let resolved = resolve_specifier(chunk.value.specifier, &self.cx.config)?;
        Ok(Some(Spanned::new(resolved, chunk.span)))
    }
}

pub(super) fn resolve_specifier(
    specifier: Spanned<ReplaceSpecifier<'_>>,
    config: &Config,
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
                span: specifier.source_span(),
            });
        }
        ("badge", _) => {}
        _ => {
            return Err(ResolveMarkerError::UnknownMarkerKind {
                kind: kind.value.to_string(),
                span: kind.source_span(),
            });
        }
    }

    let badge = &config.badge;
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

    use crate::{
        config::badge::item::BadgeItem, manifest::Manifest, source::SourceFile, sync::marker::parse,
    };

    use super::*;

    static CONFIG: &str = indoc::indoc! {r#"
        [package]
        name = "foo"
        version = "0.1.0"

        [package.metadata.cargo-sync-rdme.badge.badges]
        [package.metadata.cargo-sync-rdme.badge.badges-foo]
    "#};

    impl ResolvedReplaceSpecifier {
        #[track_caller]
        pub(crate) fn into_title(self) {
            let Self::Title = self else {
                panic!("unexpected replace specifier: {self:?}");
            };
        }

        #[track_caller]
        pub(crate) fn into_rustdoc(self) {
            let Self::Rustdoc = self else {
                panic!("unexpected replace specifier: {self:?}");
            };
        }

        #[track_caller]
        pub(crate) fn into_badge(self) -> (Option<Arc<str>>, Arc<[BadgeItem]>) {
            let Self::Badge { group, badges } = self else {
                panic!("unexpected replace specifier: {self:?}");
            };
            (group, badges)
        }
    }

    impl ResolveMarkerError {
        #[track_caller]
        pub(crate) fn into_unknown_marker_kind(self) -> (String, SourceSpan) {
            let Self::UnknownMarkerKind { kind, span } = self else {
                panic!("unexpected error: {self:?}");
            };
            (kind, span)
        }
        #[track_caller]
        pub(crate) fn into_unexpected_group_for_specifier(self) -> (String, String, SourceSpan) {
            let Self::UnexpectedGroupForSpecifier { kind, group, span } = self else {
                panic!("unexpected error: {self:?}");
            };
            (kind, group, span)
        }

        #[track_caller]
        pub(crate) fn into_no_default_badge_configured(self) -> SourceSpan {
            let Self::NoDefaultBadgeConfigured { span } = self else {
                panic!("unexpected error: {self:?}");
            };
            span
        }

        #[track_caller]
        pub(crate) fn into_no_such_badge_group(self) -> (String, SourceSpan) {
            let Self::NoSuchBadgeGroup { group, span } = self else {
                panic!("unexpected error: {self:?}");
            };
            (group, span)
        }
    }

    fn resolve(
        source: Spanned<&str>,
        config: &str,
    ) -> Result<ResolvedReplaceSpecifier, ResolveMarkerError> {
        let source_file = SourceFile::new_for_test("Cargo.toml", config);
        let manifest = Manifest::new_for_test(&source_file).unwrap();
        let config = manifest.package_config().unwrap().unwrap_or_default();
        let (specifier, _rest) = parse::parse_specifier(source).unwrap().unwrap();
        resolve_specifier(specifier, &config)
    }

    #[test]
    fn resolve_marker_resolves_valid_markers() {
        let source = Spanned::from_str("title");
        let resolved = resolve(source, CONFIG).unwrap();
        resolved.into_title();

        let source = Spanned::from_str("rustdoc");
        let resolved = resolve(source, CONFIG).unwrap();
        resolved.into_rustdoc();

        let source = Spanned::from_str("badge");
        let resolved = resolve(source, CONFIG).unwrap();
        let (group, _badges) = resolved.into_badge();
        assert!(group.is_none());

        let source = Spanned::from_str("badge:foo");
        let resolved = resolve(source, CONFIG).unwrap();
        let (group, _badges) = resolved.into_badge();
        assert_eq!(group.as_deref().unwrap(), "foo");
    }

    #[test]
    fn resolve_marker_rejects_invalid_markers() {
        let source = Spanned::from_str("unknown");
        let (kind, span) = resolve(source, CONFIG)
            .unwrap_err()
            .into_unknown_marker_kind();
        assert_eq!(kind, "unknown");
        source.assert_source_span(span, "unknown");

        let source = Spanned::from_str("title:foo");
        let (kind, group, span) = resolve(source, CONFIG)
            .unwrap_err()
            .into_unexpected_group_for_specifier();
        assert_eq!(kind, "title");
        assert_eq!(group, "foo");
        source.assert_source_span(span, "title:foo");

        let source = Spanned::from_str("rustdoc:foo");
        let (kind, group, span) = resolve(source, CONFIG)
            .unwrap_err()
            .into_unexpected_group_for_specifier();
        assert_eq!(kind, "rustdoc");
        assert_eq!(group, "foo");
        source.assert_source_span(span, "rustdoc:foo");

        let source = Spanned::from_str("badge:bar");
        let (group, span) = resolve(source, CONFIG)
            .unwrap_err()
            .into_no_such_badge_group();
        assert_eq!(group, "bar");
        source.assert_source_span(span, "bar");

        let source = Spanned::from_str("badge");
        let span = resolve(source, "")
            .unwrap_err()
            .into_no_default_badge_configured();
        source.assert_source_span(span, "badge");
    }
}
