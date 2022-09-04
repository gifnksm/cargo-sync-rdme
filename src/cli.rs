mod args;

/// Cargo subcommand to synchronize README with crate documentation.
#[derive(Debug, Clone, Default, clap::Parser)]
#[clap(name = "cargo sync-rdme", bin_name = "cargo sync-rdme", version)]
pub struct App {
    #[clap(flatten)]
    pub(crate) verbosity: args::Verbosity,
    #[clap(flatten)]
    pub(crate) workspace: args::WorkspaceArgs,
    #[clap(flatten)]
    pub(crate) package: args::PackageArgs,
    #[clap(flatten)]
    pub(crate) feature: args::FeatureArgs,
    #[clap(flatten)]
    pub(crate) fix: args::FixArgs,
}
