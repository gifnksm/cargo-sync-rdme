use std::env;

use cli_xtask::{
    camino::Utf8PathBuf,
    clap::CommandFactory,
    color_eyre::eyre::{ensure, eyre},
    config::{ConfigBuilder, DistConfigBuilder},
    workspace, Result, Xtask,
};

fn main() -> Result<()> {
    let bin_path = build_sync_rdme()?;
    let path = env::var("PATH").unwrap_or_default();
    let sep = if cfg!(windows) { ";" } else { ":" };
    env::set_var(
        "PATH",
        format!("{}{}{}", bin_path.parent().unwrap(), sep, path),
    );

    <Xtask>::main_with_config(|| {
        let workspace = workspace::current();
        let (dist, package) = DistConfigBuilder::from_root_package(workspace)?;
        let command = cargo_sync_rdme::App::command();
        let target = package
            .binary_by_name(&command.get_name().replace(' ', "-"))?
            .command(command)
            .build()?;
        let dist = dist.package(package.target(target).build()?).build()?;
        let config = ConfigBuilder::new().dist(dist).build()?;
        Ok(config)
    })
}

fn build_sync_rdme() -> Result<Utf8PathBuf> {
    let metadata = workspace::current();
    let package = metadata
        .root_package()
        .ok_or_else(|| eyre!("cargo-sync-rdme package not found"))?;
    let target = &package
        .targets
        .iter()
        .find(|t| t.kind.iter().any(|k| k == "bin"))
        .ok_or_else(|| eyre!("binary target not found in cargo-sync-rdme package"))?;
    ensure!(
        target.name == "cargo-sync-rdme",
        "invalid binary target name"
    );
    let bin_path =
        cli_xtask::cargo::build(metadata, Some(package), Some(target), None, false, None)?
            .next()
            .ok_or_else(|| eyre!("no output file found"))??;
    Ok(bin_path)
}
