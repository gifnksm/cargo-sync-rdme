use std::process::ExitStatus;

use cargo_metadata::{Metadata, Package};
use rustdoc_types::Crate;

use crate::{
    with_source::{ReadFileError, WithSource},
    App,
};

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
}

pub(super) fn create(app: &App, workspace: &Metadata, package: &Package) -> CreateResult<String> {
    run_rustdoc(app, package)?;

    let output_file = workspace
        .target_directory
        .join("doc")
        .join(format!("{}.json", package.name.replace('-', "_")));

    let doc_with_source: WithSource<Crate> = WithSource::from_json("rustdoc output", &output_file)?;
    let doc = doc_with_source.value();

    let mut root_doc = doc
        .index
        .get(&doc.root)
        .and_then(|root| root.docs.as_ref())
        .cloned();

    if let Some(root_doc) = &mut root_doc {
        if !root_doc.is_empty() && !root_doc.ends_with('\n') {
            root_doc.push('\n');
        }
    }

    Ok(root_doc.unwrap_or_default())
}

fn run_rustdoc(app: &App, package: &Package) -> CreateResult<()> {
    let mut command = app.toolchain.cargo_command();
    command
        .args(["rustdoc"])
        .args(["--package", &package.name])
        .args(app.feature.cargo_args())
        .args(["--", "-Z", "unstable-options", "--output-format", "json"]);

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
