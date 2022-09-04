use cli_xtask::{
    clap::CommandFactory,
    config::{ConfigBuilder, DistConfigBuilder},
    workspace, Result, Xtask,
};

fn main() -> Result<()> {
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
