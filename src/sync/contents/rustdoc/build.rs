use std::{
    ffi::OsString,
    fs,
    io::{self, BufReader},
    process::{ExitStatus, Stdio},
    sync::Arc,
};

use cargo_metadata::{
    Message, Metadata, Package,
    camino::{Utf8Path, Utf8PathBuf},
};
use miette::{Diagnostic, NamedSource, SourceOffset, SourceSpan};
use rustdoc_types::Crate;
use snafu::{ResultExt as _, Snafu, ensure};
use tracing::Level;

use crate::{
    cargo,
    sync::{SyncOptions, contents::rustdoc::document::RustdocDocument},
    traits::CommandExt as _,
};

#[derive(Debug, Snafu, Diagnostic)]
pub(in crate::sync) enum BuildRustdocError {
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
    #[snafu(display("failed to read rustdoc JSON output file: {path}"))]
    ReadRustdocJson {
        path: Utf8PathBuf,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to parse rustdoc JSON output file: {path}"))]
    ParseRustdocJson {
        path: Utf8PathBuf,
        #[snafu(source)]
        source: serde_json::Error,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
        #[label]
        label: SourceSpan,
    },
}

pub(super) fn build_rustdoc(
    workspace: &Metadata,
    package: &Package,
    options: &SyncOptions<'_>,
) -> Result<RustdocDocument, BuildRustdocError> {
    let json_path = run_rustdoc(package, options)?;
    let json_file = JsonFile::new(workspace, &json_path)?;
    let doc = json_file.parse()?;
    let doc = RustdocDocument::new(doc);
    Ok(doc)
}

fn run_rustdoc(
    package: &Package,
    options: &SyncOptions<'_>,
) -> Result<Utf8PathBuf, BuildRustdocError> {
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
    let mut json_filenames = vec![];
    for message in Message::parse_stream(stdout) {
        let message = message.context(ReadRustdocOutputSnafu {
            commandline: &commandline,
        })?;
        if let Message::CompilerArtifact(artifact) = message
            && artifact.package_id == package.id
        {
            json_filenames.extend(
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
        json_filenames.len() <= 1,
        MultipleRustdocJsonFilesSnafu {
            commandline: &commandline,
            files: json_filenames.clone(),
        }
    );
    let Some(output_file) = json_filenames.pop() else {
        return Err(NoRustdocJsonFilesSnafu {
            commandline: &commandline,
        }
        .build());
    };

    Ok(output_file)
}

#[derive(Debug)]
struct JsonFile {
    relative_path: Utf8PathBuf,
    text: Arc<str>,
}

impl JsonFile {
    fn new(workspace: &Metadata, path: &Utf8Path) -> Result<Self, BuildRustdocError> {
        let relative_path = path
            .strip_prefix(&workspace.workspace_root)
            .unwrap_or(path)
            .to_owned();
        let text = fs::read_to_string(path)
            .with_context(|_source| ReadRustdocJsonSnafu {
                path: &relative_path,
            })?
            .into();
        Ok(Self {
            relative_path,
            text,
        })
    }

    fn parse(&self) -> Result<Crate, BuildRustdocError> {
        let doc = serde_json::from_str(&self.text).with_context(|source| {
            let path = &self.relative_path;
            let source_code = self.to_named_source();
            let offset = SourceOffset::from_location(&self.text, source.line(), source.column());
            let label = SourceSpan::new(offset, 1);
            ParseRustdocJsonSnafu {
                path,
                source_code,
                label,
            }
        })?;
        Ok(doc)
    }

    fn to_named_source(&self) -> NamedSource<Arc<str>> {
        NamedSource::new(&self.relative_path, Arc::clone(&self.text))
    }
}
