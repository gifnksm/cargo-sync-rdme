use std::{fmt, str::FromStr};

use miette::SourceSpan;

pub(super) use self::{find::*, replace::*};
use crate::traits::StrSpanExt;

mod find;
mod replace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Replace {
    Title,
    Badge,
    Rustdoc,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ParseReplaceError {
    #[error("unknown replace specifier: {0:?}")]
    UnknownReplace(String),
}

impl FromStr for Replace {
    type Err = ParseReplaceError;

    fn from_str(s: &str) -> Result<Self, ParseReplaceError> {
        match s {
            "title" => Ok(Self::Title),
            "badge" => Ok(Self::Badge),
            "rustdoc" => Ok(Self::Rustdoc),
            _ => Err(Self::Err::UnknownReplace(s.into())),
        }
    }
}

impl fmt::Display for Replace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Replace {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Badge => "badge",
            Self::Rustdoc => "rustdoc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Marker {
    Replace(Replace),
    Start(Replace),
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

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(super) enum ParseMarkerError {
    #[error("{err}")]
    ParseReplace {
        err: ParseReplaceError,
        #[label]
        span: SourceSpan,
    },
    #[error("no replace specifier found")]
    NoReplace {
        #[label]
        span: SourceSpan,
    },
}

impl From<(ParseReplaceError, SourceSpan)> for ParseMarkerError {
    fn from((err, span): (ParseReplaceError, SourceSpan)) -> Self {
        Self::ParseReplace { err, span }
    }
}

impl Marker {
    pub(super) fn matches(text: (&str, SourceSpan)) -> Result<Option<Marker>, ParseMarkerError> {
        let body = opt_try!(Self::matches_marker(text)?);

        // <replace> [[
        if let Some(replace) = body.strip_suffix_str("[[") {
            let replace = replace.trim();
            let replace = replace.parse()?;
            return Ok(Some(Marker::Start(replace.0)));
        }

        if body.0 == "]]" {
            return Ok(Some(Marker::End));
        }

        let replace = body.parse()?;
        Ok(Some(Marker::Replace(replace.0)))
    }

    fn matches_marker(
        text: (&str, SourceSpan),
    ) -> Result<Option<(&str, SourceSpan)>, ParseMarkerError> {
        // <!-- cargo-sync-rdme <body> -->
        let text = opt_try!(trim_comment(text));

        if text.0 == MAGIC {
            return Err(ParseMarkerError::NoReplace { span: text.1 });
        }
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
    use super::*;

    #[test]
    fn matches() {
        fn ok(s: &str) -> Option<Marker> {
            let span = SourceSpan::from(0..s.len());
            Marker::matches((s, span)).unwrap()
        }
        fn err_kind(s: &str) -> String {
            let span = SourceSpan::from(0..s.len());
            match Marker::matches((s, span)).unwrap_err() {
                ParseMarkerError::ParseReplace {
                    err: ParseReplaceError::UnknownReplace(s),
                    ..
                } => s,
                e => panic!("unexpected: {e}"),
            }
        }
        fn err_norep(s: &str) {
            let span = SourceSpan::from(0..s.len());
            match Marker::matches((s, span)).unwrap_err() {
                ParseMarkerError::NoReplace { .. } => {}
                e => panic!("unexpected: {e}"),
            }
        }

        assert_eq!(ok(""), None);
        assert_eq!(ok("<!-- cargo-sync-rdmexxx -->"), None);

        assert_eq!(
            ok("<!-- cargo-sync-rdme title -->"),
            Some(Marker::Replace(Replace::Title))
        );
        assert_eq!(
            ok("<!-- cargo-sync-rdme badge [[ -->"),
            Some(Marker::Start(Replace::Badge))
        );
        assert_eq!(
            ok("<!-- cargo-sync-rdme badge[[-->"),
            Some(Marker::Start(Replace::Badge))
        );
        assert_eq!(ok("<!-- cargo-sync-rdme ]] -->"), Some(Marker::End));

        err_norep("<!-- cargo-sync-rdme  -->");
        assert_eq!(err_kind("<!-- cargo-sync-rdme title [ -->"), "title [");
        assert_eq!(err_kind("<!-- cargo-sync-rdme ] -->"), "]");
    }
}
