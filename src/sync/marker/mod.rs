use std::{fmt, range::Range, sync::Arc};

use miette::SourceSpan;
use snafu::{OptionExt as _, Snafu, ensure};

pub(super) use self::{find::*, replace::*};
use crate::{
    config::metadata::BadgeItem,
    traits::{RangeExt as _, StrSpanExt as _},
};

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
        specifier: (&str, Range<usize>),
        manifest: &ManifestFile,
    ) -> Result<Self, ParseMarkerError> {
        let group = match specifier.0 {
            "title" => return Ok(Self::Title),
            "rustdoc" => return Ok(Self::Rustdoc),
            "badge" => ("", (specifier.1.end..specifier.1.end).into()),
            _ => specifier
                .strip_prefix_str("badge:")
                .context(UnknownReplaceSpecifierSnafu {
                    specifier: specifier.0,
                    span: specifier.1.to_span(),
                })?,
        };
        let badges = &manifest.value().config().badge.badges;
        let (name, badges) = badges.get_key_value(group.0).ok_or_else(|| {
            if group.0.is_empty() {
                NoDefaultBadgeConfiguredSnafu {
                    span: specifier.1.to_span(),
                }
                .build()
            } else {
                NoSuchBadgeGroupSnafu {
                    group: group.0,
                    span: group.1.to_span(),
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
        text: (&str, Range<usize>),
        manifest: &ManifestFile,
    ) -> Result<Option<Marker>, ParseMarkerError> {
        let Some(body) = Self::matches_marker(text)? else {
            return Ok(None);
        };

        // <replace> [[
        if let Some(replace) = body.strip_suffix_str("[[") {
            let replace = replace.trim();
            let replace = ReplaceSpecifier::from_str(replace, manifest)?;
            return Ok(Some(Marker::Start(replace)));
        }

        if body.0 == "]]" {
            return Ok(Some(Marker::End));
        }

        let replace = ReplaceSpecifier::from_str(body, manifest)?;
        Ok(Some(Marker::Replace(replace)))
    }

    fn matches_marker(
        text: (&str, Range<usize>),
    ) -> Result<Option<(&str, Range<usize>)>, ParseMarkerError> {
        // <!-- cargo-sync-rdme <body> -->
        let Some(text) = trim_comment(text) else {
            return Ok(None);
        };

        ensure!(
            text.0 != MAGIC,
            NoReplaceSpecifierSnafu {
                span: text.1.to_span()
            }
        );
        let Some((head, body)) = text.split_once_fn(char::is_whitespace) else {
            return Ok(None);
        };
        Ok((head.0 == MAGIC).then_some(body))
    }
}

fn trim_comment(text: (&str, Range<usize>)) -> Option<(&str, Range<usize>)> {
    let body = text
        .trim()
        .strip_prefix_str("<!--")?
        .trim_start()
        .strip_suffix_str("-->")?
        .trim_end();
    Some(body)
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;
    use std::assert_matches;

    use super::*;

    #[test]
    fn matches() {
        fn ok(s: &str) -> Option<Marker> {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            Marker::matches((s, (0..s.len()).into()), &manifest).unwrap()
        }
        fn err_kind(s: &str) -> String {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            match Marker::matches((s, (0..s.len()).into()), &manifest).unwrap_err() {
                ParseMarkerError::UnknownReplaceSpecifier { specifier: s, .. } => s,
                e => panic!("unexpected: {e}"),
            }
        }
        fn err_norep(s: &str) {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            match Marker::matches((s, (0..s.len()).into()), &manifest).unwrap_err() {
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
