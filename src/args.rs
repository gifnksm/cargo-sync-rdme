use std::path::PathBuf;

use clap::ArgAction;
use tracing::Level;

/// Command line interface definition for `cargo-sync-rdme` command.
#[derive(Debug, Clone, Default, clap::Parser)]
#[clap(
    name = "cargo-sync-rdme",
    bin_name = "cargo sync-rdme",
    version,
    about = "Cargo subcommand to synchronize README with crate documentation."
)]
pub(crate) struct Args {
    #[clap(flatten)]
    pub(crate) verbosity: Verbosity,
    #[clap(flatten)]
    pub(crate) workspace: WorkspaceArgs,
    #[clap(flatten)]
    pub(crate) package: PackageArgs,
    #[clap(flatten)]
    pub(crate) feature: FeatureArgs,
    #[clap(flatten)]
    pub(crate) toolchain: RustdocToolchainArgs,
    #[clap(flatten)]
    pub(crate) fix: FixArgs,
}

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
    pub(crate) manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct PackageArgs {
    /// Sync READMEs for all packages in the workspace.
    #[clap(long)]
    pub(crate) workspace: bool,

    /// Package to sync README.
    #[clap(long = "package", short, value_name = "SPEC")]
    pub(crate) packages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct FeatureArgs {
    /// Space or comma separated list of features to activate.
    #[clap(long, short = 'F', value_name = "FEATURES")]
    pub(crate) features: Vec<String>,

    /// Activate all available features.
    #[clap(long)]
    pub(crate) all_features: bool,

    /// Do not activate the `default` feature.
    #[clap(long)]
    pub(crate) no_default_features: bool,
}

#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct RustdocToolchainArgs {
    /// Toolchain name to run `cargo rustdoc` with.
    #[clap(long)]
    pub(crate) toolchain: Option<String>,
    /// Install the Rust toolchain specified by `--toolchain` if it is not already installed.
    #[clap(long)]
    pub(crate) install_toolchain: bool,
}

#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default, clap::Args)]
pub(crate) struct FixArgs {
    /// Check if READMEs are synced.
    #[clap(long)]
    pub(crate) check: bool,
    /// Sync README even if a VCS was not detected.
    #[clap(long)]
    pub(crate) allow_no_vcs: bool,
    /// Sync README even if the target file is dirty.
    #[clap(long)]
    pub(crate) allow_dirty: bool,
    /// Sync README even if the target file has staged changes.
    #[clap(long)]
    pub(crate) allow_staged: bool,
}
