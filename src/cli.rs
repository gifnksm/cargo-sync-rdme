mod args;

/// Cargo subcommand to synchronize README with crate documentation.
///
/// This binary should typically be invoked as `cargo sync-rdme` (in which case
/// this message will not be seen), not `cargo-sync-rdme`.
#[derive(Debug, Clone, Default, clap::Parser)]
#[clap(bin_name = "cargo", version)]
pub struct App {
    #[clap(flatten)]
    pub(crate) verbosity: args::Verbosity,
    #[clap(subcommand)]
    pub(crate) subcommand: Subcommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Subcommand {
    SyncRdme(SyncRdme),
}

impl Default for Subcommand {
    fn default() -> Self {
        Self::SyncRdme(SyncRdme::default())
    }
}

/// Cargo subcommand to synchronize README with crate documentation.
#[derive(Debug, Clone, Default, clap::Args)]
#[clap(version)]
pub struct SyncRdme {
    #[clap(flatten)]
    pub(crate) workspace: args::WorkspaceArgs,
    #[clap(flatten)]
    pub(crate) package: args::PackageArgs,
    #[clap(flatten)]
    pub(crate) feature: args::FeatureArgs,
    #[clap(flatten)]
    pub(crate) fix: args::FixArgs,
}
