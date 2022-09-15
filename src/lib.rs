//! Cargo subcommand to synchronize README with the cargo manifest and crate documentation.
//!
//! See [repository's README] for `cargo-sync-rdme` command usage.
//!
//! # Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cargo-sync-rdme = "0.2.0"
//! ```
//!
//! [repository's README]: https://github.com/gifnksm/cargo-sync-rdme/blob/main/README.md
#![doc(html_root_url = "https://docs.rs/cargo-sync-rdme/0.2.0")]
#![warn(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    single_use_lifetimes,
    unreachable_pub,
    unused
)]

use std::{env, io};

use clap::Parser;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[macro_use]
mod macros;

mod cli;
mod config;
mod diff;
mod sync;
mod traits;
mod vcs;
mod with_source;

pub use self::cli::App;

/// Error type for `cargo-sync-rdme` command.
pub type Error = miette::Error;

/// Result type for `cargo-sync-rdme` command.
pub type Result<T> = miette::Result<T>;

/// Entry point of `cargo-sync-rdme` command.
pub fn main() -> Result<()> {
    // If this command is run by cargo, the first argument is the subcommand name `sync-rdme`.
    // We need to remove it to avoid parsing error.
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
    if env::var_os("RUST_LOG").is_none() {
        match verbosity {
            Some(Level::ERROR) => env::set_var("RUST_LOG", "error"),
            Some(Level::WARN) => env::set_var("RUST_LOG", "warn"),
            Some(Level::INFO) => env::set_var("RUST_LOG", "info"),
            Some(Level::DEBUG) => env::set_var("RUST_LOG", "debug"),
            Some(Level::TRACE) => env::set_var("RUST_LOG", "trace"),
            None => env::set_var("RUST_LOG", "off"),
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(io::stderr)
        .with_target(false)
        .try_init()
        .map_err(|e| miette!(e))?;

    Ok(())
}
