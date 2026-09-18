# Installation

There are multiple ways to install `cargo-sync-rdme`.
Choose any one of the methods below that best suits your needs.

## Pre-built binaries

Executable binaries are published on the [GitHub Release page].
Download the appropriate archive for your platform (Windows, macOS, Linux) and architecture (x86_64, aarch64) and extract the archive.
The archive contains the `cargo-sync-rdme` executable.

To run `cargo-sync-rdme` as a Cargo subcommand, place the `cargo-sync-rdme` executable in a directory on your `PATH`.

If you use [`cargo-binstall`], you can install `cargo-sync-rdme` with the following command:

```console
cargo binstall cargo-sync-rdme
```

[GitHub Release page]: https://github.com/gifnksm/cargo-sync-rdme/releases/
[`cargo-binstall`]: https://github.com/cargo-bins/cargo-binstall

## Install from source

To install `cargo-sync-rdme` from source, the Rust toolchain must be installed on your system.
See [the Rust installation guide] if you do not have Rust installed yet.

Then install either the latest released version or the current development version from the Git repository.

```console
cargo install cargo-sync-rdme
```

```console
cargo install --git https://github.com/gifnksm/cargo-sync-rdme.git cargo-sync-rdme
```

[the Rust installation guide]: https://www.rust-lang.org/tools/install

## Arch User Repository (AUR)

If you are using an Arch Linux-based distribution, you can install `cargo-sync-rdme` from the AUR.
The packages are maintained by the developer of `cargo-sync-rdme` and are updated with each release.

There are two packages available in the AUR:

* [`cargo-sync-rdme-bin`]: package that installs pre-built binaries
* [`cargo-sync-rdme`]: package that builds from source

[`cargo-sync-rdme-bin`]: https://aur.archlinux.org/packages/cargo-sync-rdme-bin
[`cargo-sync-rdme`]: https://aur.archlinux.org/packages/cargo-sync-rdme

## Verify the installation

Run:

```console
$ cargo sync-rdme --version
cargo sync-rdme <version>
```

If the command succeeds, `cargo-sync-rdme` is installed and available on your `PATH`.
