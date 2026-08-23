use std::{fmt, sync::Arc};

use miette::NamedSource;
use snafu::{Snafu, ensure};

use crate::{
    config::manifest::package::metadata::badge::BadgeItem,
    parse::Spanned,
    sync::{
        ManifestFile,
        contents::Contents,
        marker::resolve::{ResolveMarkerError, Resolver},
    },
    text_file::{PackageTextFile, PackageTextFileDisplayPath},
};

mod parse;
mod resolve;
mod scan;

const MAGIC: &str = "cargo-sync-rdme";

#[derive(Debug, Snafu, miette::Diagnostic)]
#[snafu(display(
    "failed to parse `<!-- {MAGIC} ... -->` markers in markdown file for package `{package}`: {markdown}",
    package = markdown.package, markdown = markdown.path,
))]
pub(crate) struct ParseMarkersError {
    markdown: PackageTextFileDisplayPath,
    #[source_code]
    source_code: NamedSource<Arc<str>>,
    #[related]
    errors: Vec<ResolveMarkerError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ResolvedReplaceSpecifier {
    Title,
    Badge {
        group: Option<Arc<str>>,
        badges: Arc<[BadgeItem]>,
    },
    Rustdoc,
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

pub(super) fn parse_markers(
    markdown: &PackageTextFile<'_>,
    manifest: &ManifestFile,
) -> Result<Vec<Spanned<ResolvedReplaceSpecifier>>, Box<ParseMarkersError>> {
    let mut resolver = Resolver::new(markdown.text(), manifest);
    let mut specifiers = vec![];
    let mut errors = vec![];

    while let Some(res) = resolver.try_next().transpose() {
        match res {
            Ok(specifier) => specifiers.push(specifier),
            Err(err) => errors.push(err),
        }
    }

    ensure!(
        errors.is_empty(),
        ParseMarkersSnafu {
            markdown,
            source_code: markdown.to_named_source(),
            errors
        }
    );

    Ok(specifiers)
}

pub(super) fn make_marked_contents(contents: &Contents) -> String {
    let specifier = contents.specifier();
    let text = contents.text();
    if text.is_empty() {
        format!("<!-- {MAGIC} {specifier} -->")
    } else {
        format!("<!-- {MAGIC} {specifier} [[ -->\n{text}<!-- {MAGIC} ]] -->")
    }
}
