use std::{
    borrow::Borrow,
    io::{self, BufReader},
    process::{ExitStatus, Stdio},
};

use cargo_metadata::{Message, PackageName, camino::Utf8PathBuf};
use miette::Diagnostic;
use snafu::{ResultExt as _, Snafu, ensure};
use tracing::Level;

use crate::{
    cargo,
    source::{DeserializeAsJsonError, SourceFileLoader, SourceFilePath},
    sync::{PackageSyncContext, contents::rustdoc::document::RustdocDocument},
    traits::CommandExt as _,
};

#[derive(Debug, Snafu, Diagnostic)]
pub(in crate::sync) enum BuildRustdocError {
    #[snafu(display("failed to start rustdoc for package `{package}`: {commandline}"))]
    StartRustdocProcess {
        package: PackageName,
        commandline: String,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to read rustdoc output for package `{package}`: {commandline}"))]
    ReadRustdocOutput {
        package: PackageName,
        commandline: String,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display(
        "failed to wait for rustdoc completion for package `{package}`: {commandline}"
    ))]
    WaitRustdocProcess {
        package: PackageName,
        commandline: String,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display(
        "rustdoc exited with status `{status}` for package `{package}`: {commandline}"
    ))]
    NonZeroExitStatus {
        package: PackageName,
        commandline: String,
        status: ExitStatus,
    },
    #[snafu(display(
        "rustdoc did not produce any JSON output files for package `{package}`: {commandline}"
    ))]
    NoRustdocJsonFiles {
        package: PackageName,
        commandline: String,
    },
    #[snafu(display(
        "rustdoc produced multiple JSON output files for package `{package}`: {commandline}\nfiles: {files:?}"
    ))]
    MultipleRustdocJsonFiles {
        package: PackageName,
        commandline: String,
        files: Vec<Utf8PathBuf>,
    },
    #[snafu(display("failed to read rustdoc JSON output file for package `{package}`: {path}", path = json.workspace_path))]
    ReadRustdocJson {
        package: PackageName,
        json: SourceFilePath,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to parse rustdoc JSON output file for package `{package}`: {path}", path = json.workspace_path))]
    ParseRustdocJson {
        package: PackageName,
        json: SourceFilePath,
        #[snafu(source)]
        source: DeserializeAsJsonError,
    },
}

impl Borrow<dyn Diagnostic> for Box<BuildRustdocError> {
    fn borrow(&self) -> &(dyn Diagnostic + 'static) {
        self.as_ref()
    }
}

pub(super) fn build_rustdoc(
    cx: &PackageSyncContext<'_>,
) -> Result<RustdocDocument, Box<BuildRustdocError>> {
    let json_path = run_rustdoc(cx)?;
    let json_file_loader = SourceFileLoader::from_path(cx.workspace, &json_path);
    let json_file = json_file_loader
        .load()
        .with_context(|_source| ReadRustdocJsonSnafu {
            package: cx,
            json: &json_file_loader,
        })?;
    let doc = json_file
        .deserialize_as_json()
        .with_context(|_source| ParseRustdocJsonSnafu {
            package: cx,
            json: &json_file_loader,
        })?;
    let doc = RustdocDocument::new(doc);
    Ok(doc)
}

fn run_rustdoc(cx: &PackageSyncContext<'_>) -> Result<Utf8PathBuf, Box<BuildRustdocError>> {
    let config = &cx.config.rustdoc;
    let mut command =
        cargo::command_for_build_doc(config.toolchain.as_deref(), cx.install_toolchain);

    match cx.verbosity {
        Some(Level::TRACE) => _ = command.arg("-vv"),
        Some(Level::DEBUG) => _ = command.arg("-v"),
        Some(Level::INFO) => {}
        _ => _ = command.arg("-q"),
    }

    command.arg("rustdoc");
    command.arg("-Zunstable-options");
    command.flag_value("--message-format", "json-render-diagnostics");
    // `--output-format=json` must be passed to Cargo, not forwarded to rustdoc.
    // Put it before `--`.
    // If passed after `--`, rustdoc still writes the JSON file, but Cargo does not
    // treat it as the documented artifact, so `compiler-artifact.filenames` is
    // empty and the output path cannot be discovered from the message stream.
    command.flag_value("--output-format", "json");
    // Pass `-Zrustdoc-map` so Cargo provides documentation URLs for
    // external crates that do not define `#![doc(html_root_url = ...)]`.
    // `cargo-sync-rdme` reads those URLs from
    // `external_crates[*].html_root_url` when generating links to
    // external items.
    command.arg("-Zrustdoc-map");

    command.args(["--package", &cx.package.name]);

    if let Some(features) = &config.features {
        for feature in features {
            command.args(["--features", feature]);
        }
    }
    if let Some(true) = config.all_features {
        command.arg("--all-features");
    }
    if let Some(true) = config.no_default_features {
        command.arg("--no-default-features");
    }

    command.arg("--");
    command.arg("--document-private-items");

    command.stdout(Stdio::piped());

    let commandline = command.commandline();
    tracing::debug!("executing rustdoc command: {commandline}");
    let mut child = command
        .spawn()
        .with_context(|_source| StartRustdocProcessSnafu {
            package: cx,
            commandline: &commandline,
        })?;

    let stdout = BufReader::new(child.stdout.take().unwrap());
    let mut json_filenames = vec![];
    for message in Message::parse_stream(stdout) {
        let message = message.with_context(|_source| ReadRustdocOutputSnafu {
            package: cx,
            commandline: &commandline,
        })?;
        if let Message::CompilerArtifact(artifact) = message
            && artifact.package_id == cx.package.id
        {
            json_filenames.extend(
                artifact
                    .filenames
                    .into_iter()
                    .filter(|f| f.extension().is_some_and(|e| e == "json")),
            );
        }
    }
    let status = child
        .wait()
        .with_context(|_source| WaitRustdocProcessSnafu {
            package: cx,
            commandline: &commandline,
        })?;
    ensure!(
        status.success(),
        NonZeroExitStatusSnafu {
            package: cx,
            commandline: &commandline,
            status,
        }
    );

    ensure!(
        json_filenames.len() <= 1,
        MultipleRustdocJsonFilesSnafu {
            package: cx,
            commandline: &commandline,
            files: json_filenames.clone(),
        }
    );
    let Some(output_file) = json_filenames.pop() else {
        return Err(NoRustdocJsonFilesSnafu {
            package: cx,
            commandline: &commandline,
        }
        .build()
        .into());
    };

    Ok(output_file)
}
