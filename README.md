<!-- cargo-sync-rdme title [[ -->
# cargo-sync-rdme
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge [[ -->
[![Maintenance: actively-developed](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/cargo-sync-rdme.svg)](#license)
[![crates.io](https://img.shields.io/crates/v/cargo-sync-rdme.svg?logo=rust)](https://crates.io/crates/cargo-sync-rdme)
[![docs.rs](https://img.shields.io/docsrs/cargo-sync-rdme?logo=docs.rs)](https://docs.rs/cargo-sync-rdme)
[![Rust: ^1.62.1](https://img.shields.io/badge/rust-^1.62.1-93450a.svg?logo=rust)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![GitHub Actions: CI](https://img.shields.io/github/workflow/status/gifnksm/cargo-sync-rdme/CI?label=CI&logo=github)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/ci.yml)
[![Codecov](https://img.shields.io/codecov/c/github/gifnksm/cargo-sync-rdme?label=codecov&logo=codecov)](https://codecov.io/gh/gifnksm/cargo-sync-rdme)
<!-- cargo-sync-rdme ]] -->

Cargo subcommand to synchronize README with the cargo manifest and crate documentation.

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

## Usage

cargo-sync-rdme is a subcommand to synchronize the contents of README.md with the cargo manifest and crate documentation.
By embedding marker comments in README.md, you can insert the documentation generated by cargo-sync-rdme.
There are three types of marker comments as follows.

* `<!-- cargo-sync-rdme title -->` : generate document title (H1 element) from package name.
* `<!-- cargo-sync-rdme badge -->` : generate badges from package metadata.
* `<!-- cargo-sync-rdme rustdoc -->` : generate documentation for a crate from document comments.

Write the README.md as follows:

```markdown
<!-- cargo-sync-rdme title -->
<!-- cargo-sync-rdme badge -->
<!-- cargo-sync-rdme rustdoc -->
```

To update the contents of README.md, run the following:

```console
cargo sync-rdme --toolchain nightly
```

cargo-sync-rdme uses the unstable features of rustdoc, so nightly toolchain is required to generate READMEs from comments in the crate documentation.
If nightly toolchain is not installed, it can be installed with the following command

```console
rustup toolchain install nightly
```

The contents of README.md will be updated as follows:

```markdown
<!-- cargo-sync-rdme title [[ -->
# (Package name)
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge [[ -->
(Badges)
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme rustdoc [[ -->
(Crate documentation)
<!-- cargo-sync-rdme ]] -->
```

See [examples/lib](examples/lib) for actual examples.

## Configuration

You can customize the behavior of cargo-sync-rdme by adding the following section to `Cargo.toml`.

```toml
[package.metadata.cargo-sync-rdme.badges]
maintenance = true
license = true

[package.metadata.cargo-sync-rdme.rustdoc]
html-root-url = "https://gifnksm.github.io/cargo-sync-rdme/"
```

See [Configuration](./docs/configuration.md) for details.

## Minimum supported Rust version (MSRV)

The minimum supported Rust version is **Rust 1.62.1**.
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
