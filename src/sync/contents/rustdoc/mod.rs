use std::process::ExitStatus;

use cargo_metadata::{Metadata, Package, PackageName};
use pulldown_cmark::Options;

use crate::{
    App,
    sync::{
        ManifestFile,
        contents::rustdoc::{document::RustdocDocument, intra_link::LinkMappingConfig},
    },
    with_source::{ReadFileError, WithSource},
};

mod code_block;
mod document;
mod heading;
mod intra_link;

type CreateResult<T> = Result<T, CreateRustdocError>;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(in super::super) enum CreateRustdocError {
    #[error("failed to create rustdoc process")]
    Spawn(#[source] std::io::Error),
    #[error("rustdoc exited with non-zero status code: {0}")]
    Exit(ExitStatus),
    #[error(transparent)]
    #[diagnostic(transparent)]
    ReadFileError(#[from] ReadFileError),
    #[error("package {package_name} does not have a root item")]
    RootNotFound { package_name: PackageName },
    #[error("package {package_name} does not have a crate-level documentation")]
    RootDocNotFound { package_name: PackageName },
}

pub(super) fn create(
    app: &App,
    manifest: &ManifestFile,
    workspace: &Metadata,
    package: &Package,
) -> CreateResult<String> {
    let config = manifest.value().config();
    let local_html_root_url = config
        .rustdoc
        .html_root_url
        .clone()
        .unwrap_or_else(|| format!("https://docs.rs/{}/{}", package.name, package.version));
    let mapping_config = LinkMappingConfig::new(&config.rustdoc.mappings, &local_html_root_url);

    run_rustdoc(app, package)?;

    let output_file = workspace
        .target_directory
        .join("doc")
        .join(format!("{}.json", package.name.replace('-', "_")));

    let doc = WithSource::from_json("rustdoc output", output_file)?.into_value();
    let doc = RustdocDocument::new(doc);
    let root = doc
        .root_item()
        .ok_or_else(|| CreateRustdocError::RootNotFound {
            package_name: package.name.clone(),
        })?;

    let resolver = doc.intra_link_resolver();
    let mapper = mapping_config
        .build_mapper(&resolver, root)
        .ok_or_else(|| CreateRustdocError::RootDocNotFound {
            package_name: package.name.clone(),
        })?;

    let events = mapper.build_parser(main_body_opts());
    let events = heading::convert(events);
    let events = code_block::convert(events);

    let mut buf = String::new();
    pulldown_cmark_to_cmark::cmark(events, &mut buf).unwrap();
    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }
    Ok(buf)
}

fn run_rustdoc(app: &App, package: &Package) -> CreateResult<()> {
    let mut command = app.toolchain.cargo_command();
    command
        .args(["rustdoc", "--package", &package.name])
        .args(app.feature.cargo_args())
        .args([
            "-Zrustdoc-map",
            "--",
            "--document-private-items",
            "-Zunstable-options",
            "--output-format=json",
        ]);

    tracing::info!(
        "executing {}{}",
        command.get_program().to_string_lossy(),
        command.get_args().fold(String::new(), |mut s, a| {
            s.push(' ');
            s.push_str(a.to_string_lossy().as_ref());
            s
        })
    );

    let status = command.status().map_err(CreateRustdocError::Spawn)?;
    if !status.success() {
        return Err(CreateRustdocError::Exit(status));
    }
    Ok(())
}

// Same options as rustdoc uses for the main body of the crate-level documentation.
// <https://github.com/rust-lang/rust/blob/153ecc4f74035b709bb3e1eb9546f1d934865042/compiler/rustc_resolve/src/rustdoc.rs#L250-L257>
// These extensions are also explicitly documented in the rustdoc book:
// <https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html#markdown>.
fn main_body_opts() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        // Keep smart punctuation enabled so synced Markdown matches rustdoc's
        // rendered output more closely, including on renderers like GitHub that
        // do not apply those substitutions themselves.
        | Options::ENABLE_SMART_PUNCTUATION
}
