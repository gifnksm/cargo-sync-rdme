# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

* Add `--install-toolchain` to install the Rust toolchain specified by `--toolchain` when it is not already installed.
* Add `--color` to control colored output.

### Fixed

* Pass `--features` (not the invalid `--feature`) when forwarding feature selection to Cargo for rustdoc builds.
* Resolve intra-doc links to workspace packages even when rustdoc does not provide an `html_root_url` for that package, instead of leaving those references unresolved.
* When using `--toolchain`, intra-doc links to Rust standard-library items now generate links to the documentation for the toolchain running Cargo instead of the version selected by `--toolchain`.

  If an item lives at a different documentation path in those two versions, the generated link may be invalid.
  To override a specific target, add a mapping in the package's `Cargo.toml` under `[package.metadata.cargo-sync-rdme.rustdoc]`, for example:

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  mappings = { "std::io::Result" = "https://doc.rust-lang.org/stable/std/io/error/type.Result.html" }
  ```

### Changed

* When `--color` is not specified, colored output now follows whether stderr is connected to a terminal and respects the `NO_COLOR` and `FORCE_COLOR` environment variables.
* Respect the `CARGO` environment variable when invoking Cargo to build rustdoc output, except when `--toolchain` is specified.
* When neither `--package` nor `--workspace` is specified, select Cargo's default workspace packages instead of always syncing only the workspace root package.

## [0.7.0] - 2026-08-11

### Fixed

* Fix a bug that could cause `cargo sync-rdme` to panic when converting documentation containing valid intra-doc links that `cargo-sync-rdme` cannot resolve.

### Changed

* Emit resolved intra-doc links as reference-style Markdown links instead of expanding them to inline links with resolved URLs.

  Inline intra-doc links are converted to generated reference-style links. Existing intra-doc reference links keep their labels and reference form when possible, and `cargo-sync-rdme` adds or adjusts reference definitions as needed to keep the output valid. Non-intra-doc links are left unchanged.
* Add `title` attributes to resolved intra-doc links after expansion so generated Markdown matches `rustdoc` more closely.

  Resolved intra-doc links now carry titles such as `"struct crate::Type"` in generated reference definitions. Existing explicit Markdown titles are preserved.
* Match `rustdoc` more closely for namespace-qualified intra-doc links such as [`struct@Struct`] by omitting the `namespace@` prefix from the rendered link text while still resolving the link to the namespace-qualified target.

### Removed

* Stop publishing prebuilt Linux release artifacts for `i686-unknown-linux-gnu`.

## [0.6.0] - 2026-08-07

### Fixed

* Generate correct URLs for many more intra-doc link targets, including enum variant fields, trait items, implementation items, and primitive associated items.

  Known limitations remain:

  * rustdoc JSON does not always provide enough information to distinguish required vs provided trait methods ([rust-lang/rust#160662]).
  * rustdoc may report inconsistent ancestor crate IDs in resolved paths, which can still affect some links ([rust-lang/rust#160665]).
  * Links whose resolved paths collide across namespaces are not fully supported yet, such as `std::i32` (primitive type vs module) and `std::clone::Clone` (trait vs derive).

### Changed

* Bump MSRV from 1.88.0 to 1.96.0
* Remove the workaround for missing `paths` entries on intra-doc link targets in rustdoc JSON and rely on the upstream nightly rustdoc fix instead.

  This workaround was originally for [rust-lang/rust#101687], and the upstream fix landed in [rust-lang/rust#156474] (fixing [rust-lang/rust#152511]) and is available in `nightly-2026-07-23` and later. Older nightly toolchains may no longer resolve some intra-doc links correctly.
* Replace the internal `git2`-based VCS safety check implementation with the dedicated [`vcs-modify-guard`] crate using the `gix` backend.
* (Breaking) Remove the now-unnecessary `vendored-libgit2` feature after switching away from the internal `git2`-based implementation.

[`vcs-modify-guard`]: https://crates.io/crates/vcs-modify-guard
[rust-lang/rust#101687]: https://github.com/rust-lang/rust/issues/101687
[rust-lang/rust#152511]: https://github.com/rust-lang/rust/issues/152511
[rust-lang/rust#156474]: https://github.com/rust-lang/rust/pull/156474
[rust-lang/rust#160662]: https://github.com/rust-lang/rust/issues/160662
[rust-lang/rust#160665]: https://github.com/rust-lang/rust/issues/160665

## [0.5.2] - 2026-07-31

### Fixed

* Use `zip` release assets for `cargo binstall` on Windows
* Statically link the C runtime for Windows MSVC builds so the `souko` executable does not depend on Visual C++ runtime DLLs

## [0.5.1] - 2026-07-31

### Changed

* Bump supported rustdoc JSON format version from 57 to 61. (bump `rustdoc-types` from 0.57.3 to 0.61.0)
* (Breaking) cargo-sync-rdme is now a binary-only crate and no longer exposes a library target, so it can no longer be used as a dependency by other Rust crates.
* Distribute Windows release archives as `.zip` instead of `.tar.gz`

## [0.5.0] - 2026-04-30

### Added

* Support `component` and `flag` options for Codecov badges.

### Changed

* Bump supported rustdoc JSON format version from 55 to 57. (bump `rustdoc-types` from 0.55.0 to 0.57.3)
* Bump MSRV from 1.86.0 to 1.88.0

## [0.4.3] - 2025-08-30

### Added

* Support mapping overrides for rustdoc links.

### Changed

* Bump supported rustdoc JSON format version from 36 to 55. (bump `rustdoc-types` from 0.35.0 to 0.55.0)
* Bump MSRV from 1.78.0 to 1.86.0

## [0.4.2] - 2025-02-28

### Fixed

<!-- markdownlint-disable-next-line MD038 -->
* Match code block `#` hiding behavior with rustdoc: hide lines beginning with any number of whitespace plus `# ` (or a plain `#`), and turn `##` at the beginning of lines into `#`.

## [0.4.1] - 2025-01-26

### Changed

* Bump supported rustdoc JSON format version from 36 to 39. (bump `rustdoc-types` from 0.32.2 to 0.35.0)

## [0.4.0] - 2024-11-30

### Changed

* Use `cargo metadata` output instead of parsing `Cargo.toml`
  * Support `package.<key>.workspace = true` in `Cargo.toml`
* Bump MSRV from 1.74.0 to 1.78.0

## [0.3.9] - 2024-10-19

### Changed

* Bump supported rustdoc JSON format version from 35 to 36. (bump `rustdoc-types` from 0.31.0 to 0.32.0)

## [0.3.8] - 2024-10-14

### Changed

* Bump supported rustdoc JSON format version from 34 to 35. (bump `rustdoc-types` from 0.30.0 to 0.31.0)

## [0.3.7] - 2024-09-22

### Changed

* Bump supported rustdoc JSON format version from 30 to 34. (bump `rustdoc-types` from 0.26.0 to 0.30.0)

## [0.3.6] - 2024-06-09

### Changed

* Bump supported rustdoc JSON format version from 29 to 30. (bump `rustdoc-types` from 0.25.0 to 0.26.0)

## [0.3.5] - 2024-05-21

### Changed

* Bump supported rustdoc JSON format version from 28 to 29. (bump `rustdoc-types` from 0.24.0 to 0.25.0)

## [0.3.4] - 2024-03-28

## [0.3.3] - 2024-03-28

### Changed

* Bump supported rustdoc JSON format version from 27 to 28. (bump `rustdoc-types` from 0.23.0 to 0.24.0)
* Bump MSRV from 1.70.0 to 1.74.0

## [0.3.2] - 2023-08-25

### Changed

* Bump supported rustdoc JSON format version from 26 to 27. (bump `rustdoc-types` from 0.22.0 to 0.23.0)
* Bump MSRV from 1.65.0 to 1.70.0

## [0.3.1] - 2023-07-22

### Changed

* Bump supported rustdoc JSON format version from 24 to 26. (bump `rustdoc-types` from 0.20.0 to 0.22.0)
* Bump MSRV from 1.64.0 to 1.65.0

## [0.3.0] - 2023-01-29

### Changed

* Adapt to shields.io's breaking change
* Bump MSRV from 1.62.1 to 1.64.0

## [0.2.1] - 2023-01-04

### Changed

* Bump supported rustdoc JSON format version from 20 to 24. (bump `rustdoc-types` from 0.17.0 to 0.20.0)

## [0.2.0] - 2022-09-15

### Fixed

* **(Breaking)** Fix a typo in command-line arguments: `--allow-no-vsc` -> `--allow-no-vcs`

## [0.1.4] - 2022-09-15

### Added

* `--check`: show diff if README is not updated

### Changed

* Resolve links to exported public items defined in private modules

## [0.1.3] - 2022-09-14

### Fixed

* Prebuilt Windows binaries were broken.

## [0.1.2] - 2022-09-14

### Added

* Add support for installing binaries via [`cargo-binstall`].

[`cargo-binstall`]: https://github.com/cargo-bins/cargo-binstall

## [0.1.1] - 2022-09-13

### Changed

* Resolve more links in the documentation if possible (workaround for [rust-lang/rust#101687](https://github.com/rust-lang/rust/issues/101687))

### Fixed

* Remove unresolved intra-doc links from the documentation

## [0.1.0] - 2022-09-11

* First release

<!-- next-url -->
[Unreleased]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.9...v0.4.0
[0.3.9]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.8...v0.3.9
[0.3.8]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/gifnksm/cargo-sync-rdme/commits/v0.1.0
