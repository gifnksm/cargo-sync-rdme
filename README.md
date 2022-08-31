# cargo-sync-rdme

[![maintenance status: actively-developed](https://img.shields.io/badge/maintenance-actively--developed-yellowgreen.svg)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
[![license: MIT OR APACHE-2.0](https://img.shields.io/crates/l/cargo-sync-rdme.svg)](#license)
[![crates.io](https://img.shields.io/crates/v/cargo-sync-rdme.svg)](https://crates.io/crates/cargo-sync-rdme)
[![docs.rs](https://docs.rs/cargo-sync-rdme/badge.svg)](https://docs.rs/cargo-sync-rdme/)
[![rust 1.60.0+ badge](https://img.shields.io/badge/rust-1.60.0+-93450a.svg)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![Rust CI](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/ci.yml/badge.svg)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/gifnksm/cargo-sync-rdme/graph/badge.svg)](https://codecov.io/gh/gifnksm/cargo-sync-rdme)

Cargo subcommand to synchronize README with crate documentation

## Installation

There are multiple ways to install cargo-sync-rdme.
Choose any one of the methods below that best suits your needs.

### Pre-built binaries

Executable binaries are available for download on the [GitHub Release page].

[GitHub Release page]: https://github.com/gifnksm/cargo-sync-rdme/releases/

### Build from source using Rust

To build cargo-sync-rdme executable from the source, you must have the Rust toolchain installed.
To install the rust toolchain, follow [this guide](https://www.rust-lang.org/tools/install).

Once you have installed Rust, the following command can be used to build and install cargo-sync-rdme:

```console
# Install released version
$ cargo install cargo-sync-rdme

# Install latest version
$ cargo install --git https://github.com/gifnksm/cargo-sync-rdme.git cargo-sync-rdme
```

## Minimum supported Rust version (MSRV)

The minimum supported Rust version is **Rust 1.60.0**.
At least the last 3 versions of stable Rust are supported at any given time.

While a crate is a pre-release status (0.x.x) it may have its MSRV bumped in a patch release.
Once a crate has reached 1.x, any MSRV bump will be accompanied by a new minor version.

## License

This project is licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
