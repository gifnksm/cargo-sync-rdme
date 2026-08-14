use std::{borrow::Cow, env, ffi::OsStr, path::Path, process::Command};

use cargo_metadata::Metadata;

pub(crate) fn command_path() -> Cow<'static, OsStr> {
    if let Some(cargo) = env::var_os("CARGO") {
        cargo.into()
    } else {
        OsStr::new("cargo").into()
    }
}

pub(crate) fn command() -> Command {
    Command::new(command_path())
}

pub(crate) fn metadata(manifest_path: Option<&Path>) -> cargo_metadata::Result<Metadata> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }
    cmd.cargo_path(&command_path());
    cmd.exec()
}
