//! Cargo subcommand to synchronize README with the cargo manifest and crate
//! documentation.
//!
//! See [repository's README] for `cargo-sync-rdme` command usage.
//!
//! [repository's README]: https://github.com/gifnksm/cargo-sync-rdme/blob/main/README.md

// Keep this lint local to the binary crate as a workaround for rust-lang/rust#159078,
// where enabling it workspace-wide produces false positives for library test targets.
// <https://github.com/rust-lang/rust/issues/159078>
#![warn(dead_code_pub_in_binary)]

use std::{env, io, process};

use clap::{CommandFactory as _, Parser as _};
use clap_complete::{Generator, Shell};
use tracing::Level;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

use crate::cli::App;

#[macro_use]
mod macros;

mod cli;
mod config;
mod diff;
mod sync;
mod traits;
mod vcs;
mod with_source;

/// Result type for `cargo-sync-rdme` command.
type Result<T> = miette::Result<T>;

/// Entry point of `cargo-sync-rdme` command.
fn main() -> Result<()> {
    let bin_name = env!("CARGO_BIN_NAME");
    let env_prefix = bin_name.to_uppercase().replace('-', "_");
    if let Ok(shell) = env::var(format!("{env_prefix}_COMPLETE")) {
        print_completion(bin_name, &shell);
        process::exit(0);
    }
    if let Ok(output_dir) = env::var(format!("{env_prefix}_GENERATE_MAN_TO")) {
        generate_man(&output_dir);
        process::exit(0);
    }

    // If this command is run by cargo, the first argument is the subcommand name
    // `sync-rdme`. We need to remove it to avoid parsing error.
    let args = env::args().enumerate().filter_map(|(idx, arg)| {
        if idx == 1 && arg == "sync-rdme" {
            None
        } else {
            Some(arg)
        }
    });
    let app = App::parse_from(args);
    install_logger(app.verbosity.into())?;

    let workspace = app.workspace.metadata()?;
    for package in app.package.packages(&workspace)? {
        sync::sync_all(&app, &workspace, package)?;
    }

    Ok(())
}

fn install_logger(verbosity: Option<Level>) -> Result<()> {
    let env_filter = if env::var_os("RUST_LOG").is_some() {
        EnvFilter::from_default_env()
    } else {
        let default_level = match verbosity {
            Some(Level::ERROR) => LevelFilter::ERROR,
            Some(Level::WARN) => LevelFilter::WARN,
            Some(Level::INFO) => LevelFilter::INFO,
            Some(Level::DEBUG) => LevelFilter::DEBUG,
            Some(Level::TRACE) => LevelFilter::TRACE,
            None => LevelFilter::OFF,
        };
        EnvFilter::builder()
            .with_default_directive(default_level.into())
            .from_env_lossy()
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(io::stderr)
        .with_target(false)
        .try_init()
        .map_err(|e| miette!(e))?;

    Ok(())
}

fn print_completion(bin_name: &str, shell: &str) {
    fn print<G>(bin_name: &str, g: G)
    where
        G: Generator,
    {
        clap_complete::generate(g, &mut App::command(), bin_name, &mut io::stdout());
    }
    match shell {
        "bash" => print(bin_name, Shell::Bash),
        "elvish" => print(bin_name, Shell::Elvish),
        "fish" => print(bin_name, Shell::Fish),
        "powershell" => print(bin_name, Shell::PowerShell),
        "zsh" => print(bin_name, Shell::Zsh),
        "nushell" => print(bin_name, clap_complete_nushell::Nushell),
        _ => panic!(
            "error: unknown shell `{shell}`, expected one of `bash`, `elvish`, `fish`, `powershell`, `zsh`, `nushell`"
        ),
    }
}

fn generate_man(output_dir: &str) {
    clap_mangen::generate_to(App::command(), output_dir).unwrap();
}
