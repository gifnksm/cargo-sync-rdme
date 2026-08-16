use std::process::ExitStatus;

use cargo_metadata::{Metadata, Package, PackageName};
use pulldown_cmark::Options;
use snafu::{OptionExt as _, ResultExt as _, Snafu, ensure};

use crate::{
    cargo,
    sync::{
        ManifestFile, SyncOptions,
        contents::rustdoc::{
            document::{BuildUrlOptions, RustdocDocument},
            intra_link::LinkMappingConfig,
        },
    },
    with_source::{ReadFileError, WithSource},
};

mod code_block;
mod document;
mod heading;
mod intra_link;

type CreateResult<T> = Result<T, CreateRustdocError>;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(in super::super) enum CreateRustdocError {
    #[snafu(display("failed to start rustdoc"))]
    StartRustdocProcess {
        #[snafu(source)]
        source: std::io::Error,
    },
    #[snafu(display("rustdoc exited with non-zero status code: {status}"))]
    NonZeroExitStatus { status: ExitStatus },
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    ReadFileError {
        #[snafu(source)]
        #[diagnostic_source]
        source: ReadFileError,
    },
    #[snafu(display("package {package_name} does not have a root item"))]
    RootNotFound { package_name: PackageName },
    #[snafu(display("package {package_name} does not have crate-level documentation"))]
    RootDocNotFound { package_name: PackageName },
    #[snafu(display("failed to determine the Rust toolchain version"))]
    DetermineToolchain {
        #[snafu(source)]
        #[diagnostic_source]
        source: cargo::ToolchainError,
    },
}

pub(super) fn create(
    manifest: &ManifestFile,
    workspace: &Metadata,
    package: &Package,
    options: &SyncOptions<'_>,
) -> CreateResult<String> {
    let config = manifest.value().config();
    let local_html_root_url = config
        .rustdoc
        .html_root_url
        .clone()
        .unwrap_or_else(|| format!("https://docs.rs/{}/{}", package.name, package.version));
    let expected_toolchain = cargo::toolchain(None).context(DetermineToolchainSnafu)?;
    let rustdoc_toolchain =
        cargo::toolchain(Some(options.toolchain)).context(DetermineToolchainSnafu)?;
    let build_url_options = BuildUrlOptions {
        local_html_root_url: &local_html_root_url,
        expected_toolchain,
        rustdoc_toolchain,
    };

    let mapping_config = LinkMappingConfig {
        mappings: &config.rustdoc.mappings,
    };

    run_rustdoc(package, options)?;

    let output_file = workspace
        .target_directory
        .join("doc")
        .join(format!("{}.json", package.name.replace('-', "_")));

    let doc = WithSource::from_json("rustdoc output", output_file)?.into_value();
    let doc = RustdocDocument::new(doc);
    let root = doc.root_item().with_context(|| RootNotFoundSnafu {
        package_name: package.name.clone(),
    })?;

    let resolver = doc.intra_link_resolver(&build_url_options);
    let mapper = mapping_config
        .build_mapper(&resolver, root)
        .with_context(|| RootDocNotFoundSnafu {
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

fn run_rustdoc(package: &Package, options: &SyncOptions<'_>) -> CreateResult<()> {
    let mut command = cargo::command_for_build_doc(options.toolchain);
    command
        .args(["rustdoc", "--package", &package.name])
        .args(cargo::feature_args(options.feature))
        .args([
            "-Zrustdoc-map",
            "--",
            "--document-private-items",
            "-Zunstable-options",
            "--output-format=json",
        ]);

    tracing::info!(
        "executing rustdoc command: {}{}",
        command.get_program().to_string_lossy(),
        command.get_args().fold(String::new(), |mut s, a| {
            s.push(' ');
            s.push_str(a.to_string_lossy().as_ref());
            s
        })
    );

    let status = command.status().context(StartRustdocProcessSnafu)?;
    ensure!(status.success(), NonZeroExitStatusSnafu { status });
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
