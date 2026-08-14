pub(crate) use args::*;

mod args;

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
