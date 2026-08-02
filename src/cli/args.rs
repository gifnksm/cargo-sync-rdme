use std::{path::PathBuf, process::Command};

use cargo_metadata::{Metadata, Package, camino::Utf8Path};
use clap::ArgAction;
use miette::{IntoDiagnostic as _, WrapErr as _};
use tracing::Level;
use vcs_modify_guard::{AllowOptions, ModificationSafety, UnsafeModificationReason};

use crate::{Result, diff};

#[derive(Debug, Clone, Copy, Default, clap::Args)]
pub(crate) struct Verbosity {
    /// More output per occurrence.
    #[clap(long, short = 'v', action = ArgAction::Count, global = true)]
    verbose: u8,
    /// Less output per occurrence.
    #[clap(
        long,
        short = 'q',
        action = ArgAction::Count,
        global = true,
        conflicts_with = "verbose"
    )]
    quiet: u8,
}

impl From<Verbosity> for Option<Level> {
    fn from(verb: Verbosity) -> Self {
        let level = i8::try_from(verb.verbose).unwrap_or(i8::MAX)
            - i8::try_from(verb.quiet).unwrap_or(i8::MAX);
        match level {
            i8::MIN..=-3 => None,
            -2 => Some(Level::ERROR),
            -1 => Some(Level::WARN),
            0 => Some(Level::INFO),
            1 => Some(Level::DEBUG),
            2..=i8::MAX => Some(Level::TRACE),
        }
    }
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct WorkspaceArgs {
    /// Path to Cargo.toml.
    #[clap(long, value_name = "PATH")]
    manifest_path: Option<PathBuf>,
}

impl WorkspaceArgs {
    pub(crate) fn metadata(&self) -> Result<Metadata> {
        let mut cmd = cargo_metadata::MetadataCommand::new();
        if let Some(path) = &self.manifest_path {
            cmd.manifest_path(path);
        }
        let workspace = cmd
            .exec()
            .into_diagnostic()
            .wrap_err("failed to get package metadata")?;
        Ok(workspace)
    }
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct PackageArgs {
    /// Sync READMEs for all packages in the workspace.
    #[clap(long)]
    workspace: bool,

    /// Package to sync README.
    #[clap(long, short, value_name = "SPEC")]
    package: Option<Vec<String>>,
}

impl PackageArgs {
    pub(crate) fn packages<'a>(&self, workspace: &'a Metadata) -> Result<Vec<&'a Package>> {
        if self.workspace {
            return Ok(workspace.workspace_packages());
        }

        if let Some(names) = &self.package {
            let packages = names
                .iter()
                .map(|name| {
                    workspace
                        .packages
                        .iter()
                        .find(|pkg| *pkg.name == *name)
                        .ok_or_else(|| miette!("package not found: {name}"))
                })
                .collect();
            return packages;
        }

        let package = workspace
            .root_package()
            .ok_or_else(|| miette!("no root package found"))?;
        Ok(vec![package])
    }
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct FeatureArgs {
    /// Space or comma separated list of features to activate.
    #[clap(long, short = 'F', value_name = "FEATURES")]
    features: Vec<String>,

    /// Activate all available features.
    #[clap(long)]
    all_features: bool,

    /// Do not activate the `default` feature.
    #[clap(long)]
    no_default_features: bool,
}

impl FeatureArgs {
    pub(crate) fn cargo_args(&self) -> impl Iterator<Item = &str> + '_ {
        self.all_features
            .then_some("--all-features")
            .into_iter()
            .chain(self.features.iter().flat_map(|f| ["--feature", f]))
            .chain(self.no_default_features.then_some("--no-default-features"))
    }
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct ToolchainArgs {
    /// Toolchain name to run `cargo rustdoc` with.
    #[clap(long)]
    toolchain: Option<String>,
}

impl ToolchainArgs {
    pub(crate) fn cargo_command(&self) -> Command {
        if let Some(toolchain) = &self.toolchain {
            // rustup run toolchain cargo ...
            // cargo +nightly ...` fails on windows, so use rustup instead
            // https://github.com/rust-lang/rustup/issues/3036
            let mut command = Command::new("rustup");
            command.args(["run", toolchain, "cargo"]);
            command
        } else {
            Command::new("cargo")
        }
    }
}

#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct FixArgs {
    /// Check if READMEs are synced.
    #[clap(long)]
    check: bool,
    /// Sync README even if a VCS was not detected.
    #[clap(long)]
    allow_no_vcs: bool,
    /// Sync README even if the target file is dirty.
    #[clap(long)]
    allow_dirty: bool,
    /// Sync README even if the target file has staged changes.
    #[clap(long)]
    allow_staged: bool,
}

impl FixArgs {
    pub(crate) fn check_update_allowed(
        &self,
        readme_path: impl AsRef<Utf8Path>,
        old_text: &str,
        new_text: &str,
    ) -> Result<()> {
        let readme_path = readme_path.as_ref();

        if self.check {
            bail!(
                "README is not synced: {readme_path}\n{}",
                diff::diff(old_text, new_text)
            );
        }

        let safety = AllowOptions::new()
            .allow_no_vcs(self.allow_no_vcs)
            .allow_dirty(self.allow_dirty)
            .allow_staged(self.allow_staged)
            .check_safe_to_modify(readme_path)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!("failed to check if README can be modified: {readme_path}")
            })?;

        match safety {
            ModificationSafety::Safe => {}
            ModificationSafety::Unsafe(reason) => match reason {
                UnsafeModificationReason::NoVcs => {
                    bail!(
                        "README is not under version control: {readme_path}\nUse --allow-no-vcs to override this check."
                    );
                }
                UnsafeModificationReason::Dirty { .. } => {
                    bail!(
                        "README has uncommitted changes: {readme_path}\nUse --allow-dirty to override this check."
                    );
                }
                UnsafeModificationReason::Staged { .. } => {
                    bail!(
                        "README has staged changes: {readme_path}\nUse --allow-staged to override this check."
                    );
                }
                reason => bail!(
                    "README is not safe to modify for some reason: {readme_path}\nreason: {reason:?}"
                ),
            },
        }

        Ok(())
    }
}
