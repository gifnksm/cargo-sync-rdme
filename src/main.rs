//! Cargo subcommand to synchronize a package README and additional configured
//! Markdown files with package metadata and crate documentation.
//!
//! See [repository's README] for `cargo-sync-rdme` command usage.
//!
//! [repository's README]: https://github.com/gifnksm/cargo-sync-rdme/blob/main/README.md

// Keep this lint local to the binary crate as a workaround for rust-lang/rust#159078,
// where enabling it workspace-wide produces false positives for library test targets.
// <https://github.com/rust-lang/rust/issues/159078>
#![warn(dead_code_pub_in_binary)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::{env, io, process};

use clap::{ColorChoice, CommandFactory as _, Parser as _};
use clap_complete::{Generator, Shell};
use miette::MietteHandlerOpts;
use supports_color::Stream;
use tracing::Level;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

use crate::{args::Args, sync::SyncOptions};

mod args;
mod cargo;
mod config;
mod diff;
mod sync;
mod traits;
mod with_source;

/// Entry point of `cargo-sync-rdme` command.
fn main() -> miette::Result<()> {
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

    let args = Args::parse_from(args);
    let use_color = should_use_color(args.color);
    set_console_color(use_color);
    set_miette_hook(use_color);
    install_logger(args.verbosity.into(), use_color);

    let sync_options = SyncOptions {
        mode: args.mode.mode(),
        diagnostic_stream: Stream::Stderr,
        fix: &args.fix,
        toolchain: &args.toolchain,
        feature: &args.feature,
    };

    let workspace = cargo::metadata(&args.workspace)?;
    for package in cargo::select_packages(&workspace, &args.package)? {
        sync::sync_all(&workspace, package, &sync_options)?;
    }

    Ok(())
}

fn should_use_color(choice: ColorChoice) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Auto => supports_color::on(Stream::Stderr).is_some(),
        ColorChoice::Never => false,
    }
}

fn set_console_color(use_color: bool) {
    console::set_colors_enabled_stderr(use_color);
}

fn set_miette_hook(use_color: bool) {
    miette::set_hook(Box::new(move |_| {
        Box::new(MietteHandlerOpts::new().color(use_color).build())
    }))
    .unwrap();
}

fn install_logger(verbosity: Option<Level>, use_color: bool) {
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
        .with_ansi(use_color)
        .with_writer(io::stderr)
        .with_target(false)
        .init();
}

fn print_completion(bin_name: &str, shell: &str) {
    fn print<G>(bin_name: &str, g: G)
    where
        G: Generator,
    {
        clap_complete::generate(g, &mut Args::command(), bin_name, &mut io::stdout());
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
    clap_mangen::generate_to(Args::command(), output_dir).unwrap();
}
