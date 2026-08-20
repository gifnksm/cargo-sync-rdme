use std::fmt;

use cargo_metadata::{Metadata, Package};
use snafu::{Snafu, ensure};

use crate::sync::SyncOptions;

use super::{ManifestFile, marker::ResolvedReplaceSpecifier};

mod badge;
mod rustdoc;
mod title;

pub(super) fn create_all(
    replaces: impl IntoIterator<Item = ResolvedReplaceSpecifier>,
    manifest: &ManifestFile,
    workspace: &Metadata,
    package: &Package,
    options: &SyncOptions<'_>,
) -> Result<Vec<Contents>, CreateAllContentsError> {
    let mut contents = vec![];
    let mut errors = vec![];
    for replace in replaces {
        let res = replace.create_content(manifest, workspace, package, options);
        match res {
            Ok(c) => contents.push(c),
            Err(err) => errors.push(err),
        }
    }

    ensure!(errors.is_empty(), CreateAllContentsSnafu { errors });

    Ok(contents)
}

#[derive(Debug, Snafu, miette::Diagnostic)]
#[snafu(display("failed to create replacement contents"))]
pub(crate) struct CreateAllContentsError {
    #[related]
    errors: Vec<CreateContentsError>,
}

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(super) enum CreateContentsError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    CreateBadge {
        #[snafu(source)]
        #[diagnostic_source]
        source: badge::CreateAllBadgesError,
    },
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    CreateRustdoc {
        #[snafu(source)]
        #[diagnostic_source]
        source: rustdoc::CreateRustdocError,
    },
}

#[derive(Debug, Clone)]
pub(super) struct Contents {
    text: String,
}

impl ResolvedReplaceSpecifier {
    fn create_content(
        self,
        manifest: &ManifestFile,
        workspace: &Metadata,
        package: &Package,
        options: &SyncOptions<'_>,
    ) -> Result<Contents, CreateContentsError> {
        let text = match self {
            ResolvedReplaceSpecifier::Title => title::create(package),
            ResolvedReplaceSpecifier::Badge { group: _, badges } => {
                badge::create_all(&badges, manifest, workspace, package)?
            }
            ResolvedReplaceSpecifier::Rustdoc => {
                rustdoc::create(manifest, workspace, package, options)?
            }
        };

        assert!(text.is_empty() || text.ends_with('\n'));

        Ok(Contents { text })
    }
}

impl Contents {
    pub(super) fn text(&self) -> &str {
        &self.text
    }
}

struct Escape<'s>(&'s str, &'s [char]);

impl fmt::Display for Escape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.0;
        while let Some(idx) = s.find(self.1) {
            f.write_str(&s[..idx])?;
            write!(f, r"\{}", s.as_bytes()[idx] as char)?;
            s = &s[idx + 1..];
        }
        f.write_str(s)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use similar_asserts::assert_eq;

    #[test]
    fn escape() {
        let need_escape = [
            '\\', '`', '*', '_', '{', '}', '[', ']', '(', ')', '>', '#', '+', '-', '.', '!',
        ];

        assert_eq!(Escape(r"foo", &need_escape).to_string(), r"foo");
        assert_eq!(Escape(r"`foobar", &need_escape).to_string(), r"\`foobar");
        assert_eq!(Escape(r"foo*bar", &need_escape).to_string(), r"foo\*bar");
        assert_eq!(Escape(r"foobar_", &need_escape).to_string(), r"foobar\_");
        assert_eq!(
            Escape(r"`foo*bar_", &need_escape).to_string(),
            r"\`foo\*bar\_"
        );
        assert_eq!(
            Escape(r"\foo\bar\", &need_escape).to_string(),
            r"\\foo\\bar\\"
        );
        assert_eq!(Escape(r"*", &need_escape).to_string(), r"\*");
    }
}
