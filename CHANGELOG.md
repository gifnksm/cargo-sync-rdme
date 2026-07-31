# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

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
[Unreleased]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.5.2...HEAD
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
