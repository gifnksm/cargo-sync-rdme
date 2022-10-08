# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Changed

* Bump rustdoc-types from 0.17.0 to 0.18.0

## [0.2.0] - 2022-09-15

### Fixed

* (breaking change) fix typo in command line arguments: `--allow-no-vsc` -> `--allow-no-vcs`

## [0.1.4] - 2022-09-15

### Added

* `--check`: show diff if README is not updated

### Changed

* Resolve links to exported public items defined in private modules

## [0.1.3] - 2022-09-14

### Fixed

* Windows pre-built binaries were broken.

## [0.1.2] - 2022-09-14

### Added

* Added [`cargo-binstall`] support for installing binaries.

[`cargo-binstall`]: https://github.com/cargo-bins/cargo-binstall

## [0.1.1] - 2022-09-13

### Changed

* Resolve more links in the documentation if possible (workaround for [rust-lang/rust#101687](https://github.com/rust-lang/rust/issues/101687)]

### Fixed

* Remove un-resolved intra-doc links from the documentation

## [0.1.0] - 2022-09-11

* First release

<!-- next-url -->
[Unreleased]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/gifnksm/cargo-sync-rdme/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/gifnksm/cargo-sync-rdme/commits/v0.1.0
