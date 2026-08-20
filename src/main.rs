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

use std::{assert_matches, env, io, process};

use clap::{ColorChoice, CommandFactory as _};
use clap_complete::{Generator, Shell};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use miette::MietteHandlerOpts;
use supports_color::Stream;
use tracing::Level;
use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt::writer::BoxMakeWriter};

use crate::{args::Args, sync::SyncOptions};

mod args;
mod cargo;
mod config;
mod diff;
mod parse;
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

    let args = args::parse();
    let output_stream = Stream::Stderr;
    let use_color = should_use_color(args.color, output_stream);
    set_console_color(use_color, output_stream);
    set_miette_hook(use_color, output_stream);
    install_logger(args.verbosity, use_color, output_stream);

    let sync_options = SyncOptions {
        mode: args.mode.mode(),
        verbosity: args.verbosity.into(),
        diff_stream: output_stream,
        fix: &args.fix,
        toolchain: &args.toolchain,
        feature: &args.feature,
    };

    let workspace = cargo::metadata(&args.manifest)?;
    for package in cargo::select_packages(&workspace, &args.package)? {
        sync::sync_all(&workspace, package, &sync_options)
            .map_err(|source| miette::Report::new_boxed(source))?;
    }

    Ok(())
}

fn should_use_color(choice: ColorChoice, stream: Stream) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Auto => supports_color::on(stream).is_some(),
        ColorChoice::Never => false,
    }
}

fn set_console_color(use_color: bool, stream: Stream) {
    match stream {
        Stream::Stdout => console::set_colors_enabled(use_color),
        Stream::Stderr => console::set_colors_enabled_stderr(use_color),
    }
}

fn set_miette_hook(use_color: bool, stream: Stream) {
    // Keep the same `Stream`-based interface as the other setup functions, but
    // errors returned from `main` are always printed to stderr, so only
    // `Stream::Stderr` is valid here.
    assert_matches!(stream, Stream::Stderr);

    miette::set_hook(Box::new(move |_| {
        Box::new(MietteHandlerOpts::new().color(use_color).build())
    }))
    .unwrap();
}

fn install_logger(verbosity: Verbosity<InfoLevel>, use_color: bool, stream: Stream) {
    let env_filter = if !verbosity.is_present() && env::var_os("RUST_LOG").is_some() {
        EnvFilter::from_default_env()
    } else {
        let default_level = match verbosity.into() {
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
    let writer = match stream {
        Stream::Stdout => BoxMakeWriter::new(io::stdout),
        Stream::Stderr => BoxMakeWriter::new(io::stderr),
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_ansi(use_color)
        .with_writer(writer)
        .with_target(false)
        .without_time()
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
