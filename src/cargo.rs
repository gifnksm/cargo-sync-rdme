use std::{
    borrow::Cow,
    convert::Infallible,
    env,
    ffi::{OsStr, OsString},
    fmt::Display,
    iter,
    process::{Command, ExitStatus},
    str::FromStr,
};

use cargo_metadata::{Metadata, Package};
use miette::{IntoDiagnostic as _, WrapErr as _, miette};

use crate::args::{FeatureArgs, PackageArgs, RustdocToolchainArgs, WorkspaceArgs};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Channel {
    Stable,
    Beta,
    Nightly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Toolchain {
    pub(crate) version: String,
    pub(crate) pre_release: Option<String>,
}

impl FromStr for Toolchain {
    type Err = Infallible;

    fn from_str(release: &str) -> Result<Self, Self::Err> {
        let (version, pre_release) = release
            .split_once('-')
            .map_or((release, None), |(v, c)| (v, Some(c)));
        Ok(Self {
            version: version.to_owned(),
            pre_release: pre_release.map(ToOwned::to_owned),
        })
    }
}

impl Display for Toolchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            version,
            pre_release,
        } = self;
        if let Some(pre_release) = pre_release {
            write!(f, "{version}-{pre_release}")
        } else {
            write!(f, "{version}")
        }
    }
}

impl Toolchain {
    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    #[cfg(test)]
    pub(crate) fn pre_release(&self) -> Option<&str> {
        self.pre_release.as_deref()
    }

    pub(crate) fn channel(&self) -> Option<Channel> {
        let Self { pre_release, .. } = self;
        match pre_release.as_deref() {
            Some(s) if s == "beta" || s.starts_with("beta.") => Some(Channel::Beta),
            Some("nightly") => Some(Channel::Nightly),
            None => Some(Channel::Stable),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum ToolchainError {
    #[error("failed to execute the command: {}", commandline.display())]
    CommandExecutionFailed {
        #[source]
        source: std::io::Error,
        commandline: OsString,
    },
    #[error("the command failed with status `{status}`: {}\nstderr:\n{}", commandline.display(), String::from_utf8_lossy(stderr))]
    CommandFailed {
        commandline: OsString,
        status: ExitStatus,
        stderr: Vec<u8>,
    },
    #[error("the command output was not valid UTF-8 string: {}\nstderr:\n{}", commandline.display(), String::from_utf8_lossy(stderr))]
    InvalidUtf8Output {
        #[source]
        source: std::string::FromUtf8Error,
        commandline: OsString,
        stderr: Vec<u8>,
    },
    #[error(
        "the command output did not contain a `release:` line: {}\nstderr:\n{}", commandline.display(), String::from_utf8_lossy(stderr),
    )]
    NoReleaseLineInOutput {
        commandline: OsString,
        stderr: Vec<u8>,
    },
}

fn describe_command(cmd: &Command) -> OsString {
    let mut s = OsString::new();
    s.push(cmd.get_program());
    for arg in cmd.get_args() {
        s.push(" ");
        s.push(arg);
    }
    s
}

pub(crate) fn toolchain(args: Option<&RustdocToolchainArgs>) -> Result<Toolchain, ToolchainError> {
    let mut cmd = if let Some(args) = args {
        command_for_build_doc(args)
    } else {
        command()
    };
    let output = cmd
        .args(["--version", "--verbose"])
        .output()
        .map_err(|source| ToolchainError::CommandExecutionFailed {
            source,
            commandline: describe_command(&cmd),
        })?;
    if !output.status.success() {
        return Err(ToolchainError::CommandFailed {
            commandline: describe_command(&cmd),
            status: output.status,
            stderr: output.stderr,
        });
    }
    let stdout =
        String::from_utf8(output.stdout).map_err(|source| ToolchainError::InvalidUtf8Output {
            source,
            commandline: describe_command(&cmd),
            stderr: output.stderr.clone(),
        })?;
    let release_line = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix("release:"))
        .ok_or_else(|| ToolchainError::NoReleaseLineInOutput {
            commandline: describe_command(&cmd),
            stderr: output.stderr,
        })?;
    let Ok(toolchain) = Toolchain::from_str(release_line.trim());
    Ok(toolchain)
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use super::*;

    #[test]
    fn toolchain_from_str_parses_valid_release_str() {
        let stable = Toolchain::from_str("1.97.1").unwrap();
        assert_eq!(stable.version(), "1.97.1");
        assert!(stable.pre_release().is_none());
        assert_eq!(stable.channel().unwrap(), Channel::Stable);
        assert_eq!(stable.to_string(), "1.97.1");

        let beta = Toolchain::from_str("1.98.0-beta.7").unwrap();
        assert_eq!(beta.version(), "1.98.0");
        assert_eq!(beta.pre_release().unwrap(), "beta.7");
        assert_eq!(beta.channel().unwrap(), Channel::Beta);
        assert_eq!(beta.to_string(), "1.98.0-beta.7");

        let nightly = Toolchain::from_str("1.99.0-nightly").unwrap();
        assert_eq!(nightly.version(), "1.99.0");
        assert_eq!(nightly.pre_release().unwrap(), "nightly");
        assert_eq!(nightly.channel().unwrap(), Channel::Nightly);
        assert_eq!(nightly.to_string(), "1.99.0-nightly");

        let alpha = Toolchain::from_str("1.99.0-alpha.0").unwrap();
        assert_eq!(alpha.version(), "1.99.0");
        assert_eq!(alpha.pre_release().unwrap(), "alpha.0");
        assert!(alpha.channel().is_none());
        assert_eq!(alpha.to_string(), "1.99.0-alpha.0");
    }
}
