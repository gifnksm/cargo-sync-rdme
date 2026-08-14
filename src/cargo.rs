use std::{borrow::Cow, env, ffi::OsStr, iter, process::Command};

use cargo_metadata::{Metadata, Package};
use miette::{IntoDiagnostic as _, WrapErr as _, miette};

use crate::cli::{FeatureArgs, PackageArgs, RustdocToolchainArgs, WorkspaceArgs};

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

pub(crate) fn command_for_build_doc(args: &RustdocToolchainArgs) -> Command {
    let RustdocToolchainArgs {
        toolchain,
        install_toolchain,
    } = args;

    let Some(toolchain) = toolchain.as_ref() else {
        return command();
    };
    // Use `rustup run` instead of `cargo +toolchain ...` for two
    // reasons:
    // - `--install` keeps toolchain installation explicit (`cargo
    //   +toolchain` auto-installs unless opted out)
    // - `cargo +toolchain` has a known issue on Windows:
    //   https://github.com/rust-lang/rustup/issues/3036
    let mut command = Command::new("rustup");
    command.arg("run");
    if *install_toolchain {
        command.arg("--install");
    }
    command.args([toolchain, "cargo"]);
    command
}

pub(crate) fn metadata(args: &WorkspaceArgs) -> crate::Result<Metadata> {
    let WorkspaceArgs { manifest_path } = args;

    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.no_deps();
    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }
    cmd.cargo_path(&command_path());
    cmd.exec()
        .into_diagnostic()
        .wrap_err("failed to get package metadata")
}

pub(crate) fn select_packages<'meta>(
    meta: &'meta Metadata,
    args: &PackageArgs,
) -> crate::Result<Vec<&'meta Package>> {
    let PackageArgs {
        workspace,
        packages,
    } = args;

    if *workspace {
        return Ok(meta.workspace_packages());
    }

    let Some(names) = packages else {
        return Ok(meta.workspace_default_packages());
    };

    names
        .iter()
        .map(|name| {
            meta.packages
                .iter()
                .find(|pkg| *pkg.name == *name)
                .ok_or_else(|| miette!("package not found: {name}"))
        })
        .collect()
}

pub(crate) fn feature_args(args: &FeatureArgs) -> impl Iterator<Item = &str> {
    let FeatureArgs {
        features,
        all_features,
        no_default_features,
    } = args;

    iter::empty()
        .chain(all_features.then_some("--all-features"))
        .chain(features.iter().flat_map(|f| ["--features", f]))
        .chain(no_default_features.then_some("--no-default-features"))
}
