use std::{fmt, sync::Arc};

use miette::SourceSpan;
use snafu::{OptionExt as _, Snafu, ensure};

pub(super) use self::{find::*, replace::*};
use crate::{config::metadata::BadgeItem, traits::StrSpanExt as _};

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

#[derive(Debug, Snafu)]
pub(super) enum ParseReplaceSpecifierError {
    #[snafu(display("unknown replacement specifier: {specifier:?}"))]
    UnknownReplaceSpecifier { specifier: String },
    #[snafu(display("badge group not found in the package manifest: package.metadata.cargo-sync-rdme.badge.badges{hyphen}{group}", hyphen = if group.is_empty() { "" } else { "-" }))]
    NoSuchBadgeGroup { group: String },
}

impl ReplaceSpecifier {
    fn from_str(
        specifier: &str,
        manifest: &ManifestFile,
    ) -> Result<Self, ParseReplaceSpecifierError> {
        let group = match specifier {
            "title" => return Ok(Self::Title),
            "rustdoc" => return Ok(Self::Rustdoc),
            "badge" => "",
            _ => specifier
                .strip_prefix("badge:")
                .context(UnknownReplaceSpecifierSnafu { specifier })?,
        };
        let badges = &manifest.value().config().badge.badges;
        let (name, badges) = badges
            .get_key_value(group)
            .context(NoSuchBadgeGroupSnafu { group })?;
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
    #[snafu(display("{source}"))]
    ParseReplace {
        #[snafu(source)]
        source: ParseReplaceSpecifierError,
        #[label]
        span: SourceSpan,
    },
    #[snafu(display("no replacement specifier found"))]
    NoReplaceSpecifier {
        #[label]
        span: SourceSpan,
    },
}

impl From<(ParseReplaceSpecifierError, SourceSpan)> for ParseMarkerError {
    fn from((err, span): (ParseReplaceSpecifierError, SourceSpan)) -> Self {
        Self::ParseReplace { source: err, span }
    }
}

impl Marker {
    pub(super) fn matches(
        text: (&str, SourceSpan),
        manifest: &ManifestFile,
    ) -> Result<Option<Marker>, ParseMarkerError> {
        let body = opt_try!(Self::matches_marker(text)?);

        // <replace> [[
        if let Some(replace) = body.strip_suffix_str("[[") {
            let replace = replace.trim();
            let replace =
                ReplaceSpecifier::from_str(replace.0, manifest).map_err(|err| (err, replace.1))?;
            return Ok(Some(Marker::Start(replace)));
        }

        if body.0 == "]]" {
            return Ok(Some(Marker::End));
        }

        let replace = ReplaceSpecifier::from_str(body.0, manifest).map_err(|err| (err, body.1))?;
        Ok(Some(Marker::Replace(replace)))
    }

    fn matches_marker(
        text: (&str, SourceSpan),
    ) -> Result<Option<(&str, SourceSpan)>, ParseMarkerError> {
        // <!-- cargo-sync-rdme <body> -->
        let text = opt_try!(trim_comment(text));

        ensure!(text.0 != MAGIC, NoReplaceSpecifierSnafu { span: text.1 });
        let (head, body) = opt_try!(text.split_once_fn(char::is_whitespace));
        Ok((head.0 == MAGIC).then_some(body))
    }
}

fn trim_comment(text: (&str, SourceSpan)) -> Option<(&str, SourceSpan)> {
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
            let span = SourceSpan::from(0..s.len());
            Marker::matches((s, span), &manifest).unwrap()
        }
        fn err_kind(s: &str) -> String {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            let span = SourceSpan::from(0..s.len());
            match Marker::matches((s, span), &manifest).unwrap_err() {
                ParseMarkerError::ParseReplace {
                    source: ParseReplaceSpecifierError::UnknownReplaceSpecifier { specifier: s },
                    ..
                } => s,
                e => panic!("unexpected: {e}"),
            }
        }
        fn err_norep(s: &str) {
            let config = indoc::indoc! {"
                [package.metadata.cargo-sync-rdme.badge.badges]
                [package.metadata.cargo-sync-rdme.badge.badges-foo]
            "};
            let manifest = ManifestFile::dummy(toml::from_str(config).unwrap());
            let span = SourceSpan::from(0..s.len());
            match Marker::matches((s, span), &manifest).unwrap_err() {
                ParseMarkerError::NoReplaceSpecifier { .. } => {}
                e @ ParseMarkerError::ParseReplace { .. } => panic!("unexpected: {e}"),
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
