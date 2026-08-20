use std::{fmt, sync::Arc};

use miette::SourceSpan;
use snafu::{OptionExt as _, Snafu, ensure};

pub(super) use self::{find::*, replace::*};
use crate::{config::metadata::BadgeItem, parse::Spanned};

use super::ManifestFile;

mod find;
mod replace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ReplaceSpecifier {
    Title,
    Badge {
        name: Arc<str>,
        badges: Arc<[BadgeItem]>,
    },
    Rustdoc,
}

impl ReplaceSpecifier {
    fn from_str(
        specifier: Spanned<&str>,
        manifest: &ManifestFile,
    ) -> Result<Self, ParseMarkerError> {
        let group = match specifier.value {
            "title" => return Ok(Self::Title),
            "rustdoc" => return Ok(Self::Rustdoc),
            "badge" => specifier.end(),
            _ => specifier
                .strip_prefix_str("badge:")
                .context(UnknownReplaceSpecifierSnafu {
                    specifier: specifier.value,
                    span: specifier.source_span(),
                })?,
        };
        let badges = &manifest.value().config().badge.badges;
        let (name, badges) = badges.get_key_value(group.value).ok_or_else(|| {
            if group.value.is_empty() {
                NoDefaultBadgeConfiguredSnafu {
                    span: specifier.source_span(),
                }
                .build()
            } else {
                NoSuchBadgeGroupSnafu {
                    group: group.value,
                    span: group.source_span(),
                }
                .build()
            }
        })?;
        Ok(Self::Badge {
            name: Arc::clone(name),
            badges: Arc::clone(badges),
        })
    }
}

impl fmt::Display for ReplaceSpecifier {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Marker {
    Replace(ReplaceSpecifier),
    Start(ReplaceSpecifier),
    End,
}

const MAGIC: &str = "cargo-sync-rdme";

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Replace(replace) => write!(f, "<!-- {MAGIC} {replace} -->"),
            Self::Start(replace) => write!(f, "<!-- {MAGIC} {replace} [[ -->"),
            Self::End => write!(f, "<!-- {MAGIC} ]] -->"),
        }
    }
}

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(super) enum ParseMarkerError {
    #[snafu(display("unknown replacement specifier: {specifier}"))]
    UnknownReplaceSpecifier {
        specifier: String,
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
    #[snafu(display("no replacement specifier found"))]
    NoReplaceSpecifier {
        #[label]
        span: SourceSpan,
    },
}

impl Marker {
    pub(super) fn matches(
        text: Spanned<&str>,
        manifest: &ManifestFile,
    ) -> Result<Option<Spanned<Marker>>, ParseMarkerError> {
        let span = text.span;

        let Some(body) = Self::matches_marker(text)? else {
            return Ok(None);
        };

        // <replace> [[
        if let Some(replace) = body.strip_suffix_str("[[") {
            let replace = replace.trim();
            let replace = ReplaceSpecifier::from_str(replace, manifest)?;
            return Ok(Some(Spanned::new(Marker::Start(replace), span)));
        }

        if body == "]]" {
            return Ok(Some(Spanned::new(Marker::End, span)));
        }

        let replace = ReplaceSpecifier::from_str(body, manifest)?;
        Ok(Some(Spanned::new(Marker::Replace(replace), span)))
    }

    fn matches_marker(text: Spanned<&str>) -> Result<Option<Spanned<&str>>, ParseMarkerError> {
        // <!-- cargo-sync-rdme <body> -->
        let Some(text) = trim_comment(text) else {
            return Ok(None);
        };

        ensure!(
            text != MAGIC,
            NoReplaceSpecifierSnafu {
                span: text.source_span(),
            }
        );
        let Some((head, body)) = text.split_once_fn(char::is_whitespace) else {
            return Ok(None);
        };
        Ok((head == MAGIC).then_some(body))
    }
}

fn trim_comment(text: Spanned<&str>) -> Option<Spanned<&str>> {
    let text = text
        .trim()
        .strip_prefix_str("<!--")?
        .trim_start()
        .strip_suffix_str("-->")?
        .trim_end();
    Some(text)
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;
    use std::assert_matches;

    use super::*;

    #[test]
    fn matches() {
        fn ok(text: &str) -> Option<Marker> {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            let text = Spanned::new(text, (0..text.len()).into());
            let marker = Marker::matches(text, &manifest).unwrap()?;
            assert_eq!(marker.span, text.span);
            Some(marker.value)
        }
        fn err_kind(text: &str) -> String {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            let text = Spanned::new(text, (0..text.len()).into());
            match Marker::matches(text, &manifest).unwrap_err() {
                ParseMarkerError::UnknownReplaceSpecifier { specifier: s, .. } => s,
                e => panic!("unexpected: {e}"),
            }
        }
        fn err_norep(text: &str) {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            let text = Spanned::new(text, (0..text.len()).into());
            match Marker::matches(text, &manifest).unwrap_err() {
                ParseMarkerError::NoReplaceSpecifier { .. } => {}
                e => panic!("unexpected: {e}"),
            }
        }

        assert_eq!(ok(""), None);
        assert_eq!(ok("<!-- cargo-sync-rdmexxx -->"), None);

        assert_eq!(
            ok("<!-- cargo-sync-rdme title -->"),
            Some(Marker::Replace(ReplaceSpecifier::Title))
        );
        assert_matches!(
            ok("<!-- cargo-sync-rdme badge [[ -->"),
            Some(Marker::Start(ReplaceSpecifier::Badge { name, .. })) if name.is_empty()
        );
        assert_matches!(
            ok("<!-- cargo-sync-rdme badge[[-->"),
            Some(Marker::Start(ReplaceSpecifier::Badge { name, ..})) if name.is_empty()
        );
        assert_eq!(ok("<!-- cargo-sync-rdme ]] -->"), Some(Marker::End));

        err_norep("<!-- cargo-sync-rdme  -->");
        assert_eq!(err_kind("<!-- cargo-sync-rdme title [ -->"), "title [");
        assert_eq!(err_kind("<!-- cargo-sync-rdme ] -->"), "]");
    }
}
