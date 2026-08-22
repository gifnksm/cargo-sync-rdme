use std::{
    ffi::OsString,
    io::{self, BufReader},
    process::{ExitStatus, Stdio},
};

use cargo_metadata::{Message, Package, PackageName, camino::Utf8PathBuf};
use pulldown_cmark::Options;
use snafu::{OptionExt as _, ResultExt as _, Snafu, ensure};
use tracing::Level;

use crate::{
    cargo,
    sync::{
        ManifestFile, SyncOptions,
        contents::rustdoc::{
            document::{BuildUrlOptions, RustdocDocument},
            intra_link::LinkMappingConfig,
        },
    },
    traits::CommandExt as _,
    with_source::{ReadFileError, WithSource},
};

mod code_block;
mod document;
mod heading;
mod intra_link;

type CreateResult<T> = Result<T, CreateRustdocError>;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(in super::super) enum CreateRustdocError {
    #[snafu(display("failed to start rustdoc: {}", commandline.display()))]
    StartRustdocProcess {
        commandline: OsString,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to read rustdoc output: {}", source))]
    ReadRustdocOutput {
        commandline: OsString,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to wait for rustdoc completion: {}", commandline.display()))]
    WaitRustdocProcess {
        commandline: OsString,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("rustdoc exited with status `{status}`: {}", commandline.display()))]
    NonZeroExitStatus {
        commandline: OsString,
        status: ExitStatus,
    },
    #[snafu(display("rustdoc did not produce any JSON output files: {}", commandline.display()))]
    NoRustdocJsonFiles { commandline: OsString },
    #[snafu(display("rustdoc produced multiple JSON output files: {}\nfiles: {files:?}", commandline.display()))]
    MultipleRustdocJsonFiles {
        commandline: OsString,
        files: Vec<Utf8PathBuf>,
    },
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

    let output_file = run_rustdoc(package, options)?;

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

fn run_rustdoc(package: &Package, options: &SyncOptions<'_>) -> CreateResult<Utf8PathBuf> {
    let mut command = cargo::command_for_build_doc(options.toolchain);
    match options.verbosity {
        Some(Level::TRACE) => _ = command.arg("-v"),
        Some(Level::DEBUG) => {}
        _ => _ = command.arg("-q"),
    }
    command
        .args(["rustdoc", "--package", &package.name])
        .args(cargo::feature_args(options.feature))
        .args([
            "--message-format=json-render-diagnostics",
            "-Zunstable-options",
            // `--output-format=json` must be passed to Cargo, not forwarded to rustdoc.
            // Put it before `--`.
            // If passed after `--`, rustdoc still writes the JSON file, but Cargo does not
            // treat it as the documented artifact, so `compiler-artifact.filenames` is
            // empty and the output path cannot be discovered from the message stream.
            "--output-format=json",
            // Pass `-Zrustdoc-map` so Cargo provides documentation URLs for
            // external crates that do not define `#![doc(html_root_url = ...)]`.
            // `cargo-sync-rdme` reads those URLs from
            // `external_crates[*].html_root_url` when generating links to
            // external items.
            "-Zrustdoc-map",
            "--",
            "--document-private-items",
        ])
        .stdout(Stdio::piped());

    let commandline = command.commandline();
    tracing::debug!("executing rustdoc command: {}", commandline.display());
    let mut child = command
        .spawn()
        .with_context(|_source| StartRustdocProcessSnafu {
            commandline: &commandline,
        })?;

    let stdout = BufReader::new(child.stdout.take().unwrap());
    let mut output_files = vec![];
    for message in Message::parse_stream(stdout) {
        let message = message.context(ReadRustdocOutputSnafu {
            commandline: &commandline,
        })?;
        if let Message::CompilerArtifact(artifact) = message
            && artifact.package_id == package.id
        {
            output_files.extend(
                artifact
                    .filenames
                    .into_iter()
                    .filter(|f| f.extension().is_some_and(|e| e == "json")),
            );
        }
    }
    let status = child.wait().context(WaitRustdocProcessSnafu {
        commandline: &commandline,
    })?;
    ensure!(
        status.success(),
        NonZeroExitStatusSnafu {
            commandline: &commandline,
            status,
        }
    );

    ensure!(
        output_files.len() <= 1,
        MultipleRustdocJsonFilesSnafu {
            commandline: &commandline,
            files: output_files.clone(),
        }
    );
    let Some(output_file) = output_files.pop() else {
        return Err(NoRustdocJsonFilesSnafu {
            commandline: &commandline,
        }
        .build());
    };

    Ok(output_file)
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
