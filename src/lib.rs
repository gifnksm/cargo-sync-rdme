//! Cargo subcommand to synchronize README with crate documentation
//!
//! # Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cargo-sync-rdme = "0.0.0"
//! ```
#![doc(html_root_url = "https://docs.rs/cargo-sync-rdme/0.0.0")]

use std::{env, io};

use clap::Parser;
use cli::Subcommand;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[macro_use]
mod macros;

mod cli;
mod config;
mod sync;
mod toml;
mod traits;
mod vcs;
mod with_source;

pub use self::cli::App;
pub type Error = miette::Error;
pub type Result<T> = miette::Result<T>;

pub fn main() -> Result<()> {
    let app = App::parse();
    install_logger(app.verbosity.into())?;

    match app.subcommand {
        Subcommand::SyncRdme(sync_rdme) => {
            let workspace = sync_rdme.workspace.metadata()?;
            for package in sync_rdme.package.packages(&workspace)? {
                sync::sync_readme(&sync_rdme, &workspace, package)?;
            }
        }
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
