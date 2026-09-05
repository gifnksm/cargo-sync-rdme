use std::{env, path::PathBuf};

use clap::{ColorChoice, Parser as _};
use clap_verbosity_flag::{InfoLevel, Verbosity};

pub(crate) fn parse() -> Args {
    // We support running this command both as a cargo subcommand and as a standalone binary.
    //
    // * cargo sync-rdme [OPTIONS]
    //   => cargo executes the command as: cargo-sync-rdme sync-rdme [OPTIONS]
    // * cargo-sync-rdme [OPTIONS]
    //
    // When run as a cargo subcommand, we need to remove the argv[1] (`sync-rdme`) before parsing the arguments.
    let args = env::args().enumerate().filter_map(|(idx, arg)| {
        if idx == 1 && arg == "sync-rdme" {
            None
        } else {
            Some(arg)
        }
    });
    Args::parse_from(args)
}

/// Synchronize a package README and additional configured Markdown files with package metadata and crate documentation.
#[derive(Debug, Clone, Default, clap::Parser)]
#[command(
    name = "cargo-sync-rdme",
    bin_name = "cargo sync-rdme",
    version,
    styles = clap_cargo::style::CLAP_STYLING
)]
pub(crate) struct Args {
    #[command(flatten)]
    pub(crate) verbosity: Verbosity<InfoLevel>,
    /// Coloring.
    #[arg(long, default_value_t = ColorChoice::Auto, value_name = "WHEN")]
    pub(crate) color: ColorChoice,
    #[command(flatten)]
    pub(crate) toolchain: RustdocToolchainArgs,
    #[command(flatten)]
    pub(crate) mode: ModeArgs,
    #[command(flatten)]
    pub(crate) fix: FixArgs,
    #[command(flatten, next_help_heading = "Package Selection")]
    pub(crate) package: PackageSelection,
    #[command(flatten, next_help_heading = "Feature Selection")]
    pub(crate) feature: FeatureSelection,
    #[command(flatten, next_help_heading = "Manifest Options")]
    pub(crate) manifest: ManifestOptions,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct ManifestOptions {
    /// Path to Cargo.toml.
    #[arg(long, short = 'm', value_name = "PATH")]
    pub(crate) manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct PackageSelection {
    /// Synchronize all packages in the workspace.
    #[arg(long)]
    pub(crate) workspace: bool,

    /// Package(s) to synchronize.
    #[arg(long = "package", short = 'p', value_name = "SPEC")]
    pub(crate) packages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct FeatureSelection {
    /// Space or comma separated list of features to activate.
    #[arg(long, short = 'F', value_name = "FEATURES")]
    pub(crate) features: Vec<String>,

    /// Activate all available features.
    #[arg(long)]
    pub(crate) all_features: bool,

    /// Do not activate the `default` feature.
    #[arg(long)]
    pub(crate) no_default_features: bool,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct RustdocToolchainArgs {
    /// Toolchain name to run `cargo rustdoc` with.
    #[arg(long)]
    pub(crate) toolchain: Option<String>,
    /// Install the Rust toolchain selected for rustdoc if it is not already installed.
    #[arg(long)]
    pub(crate) install_toolchain: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Check,
    Fix,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct ModeArgs {
    /// Check whether target files are up to date.
    #[arg(long)]
    check: bool,
}

impl ModeArgs {
    pub(crate) fn mode(&self) -> Mode {
        if self.check { Mode::Check } else { Mode::Fix }
    }
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct FixArgs {
    /// Synchronize target files even if no VCS was detected.
    #[arg(long)]
    pub(crate) allow_no_vcs: bool,
    /// Synchronize target files even if one is dirty.
    #[arg(long)]
    pub(crate) allow_dirty: bool,
    /// Synchronize target files even if one has staged changes.
    #[arg(long)]
    pub(crate) allow_staged: bool,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory as _;

    use super::*;

    #[test]
    fn verify_args() {
        Args::command().debug_assert();
    }
}
