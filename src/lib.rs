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

use clap::Parser;

#[derive(clap::Parser)]
pub struct App {}

pub fn main() {
    let _args = App::parse();
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
