# Contribution Guidelines

First off, thank you for considering contributing to `cargo-sync-rdme`.

If your contribution is not straightforward, please first discuss the change you
wish to make by creating a new issue before making the change.

## Reporting Issues

Before reporting an issue on the
[issue tracker](https://github.com/gifnksm/cargo-sync-rdme/issues),
please check that it has not already been reported by searching for some related
keywords.

## Pull Requests

Try to do one pull request per change.

For Rust code changes, run the following checks before opening or updating a
pull request:

```console
cargo fmt --all --check
cargo clippy --all-features --all-targets --workspace
cargo test
```

For changes limited to documentation or other non-Rust files, check the files
relevant to your changes; you do not need to run the Rust checks above.

After opening or updating a pull request, review the CI results and fix any
failures caused by your changes.

When a pull request resolves an issue, reference it in the PR description with
`Closes #<number>`. When it is related to an issue but does not resolve it,
reference it with `Refs #<number>`.

### Updating the Changelog

Document notable changes that affect users of this crate in the **Unreleased**
section of
[CHANGELOG](https://github.com/gifnksm/cargo-sync-rdme/blob/main/CHANGELOG.md).

Include changes to public APIs, behavior, performance, security, and
compatibility requirements such as the minimum supported Rust version (MSRV).

Changes limited to internal refactoring, tests, CI, development tooling, or
minor documentation edits do not require a changelog entry unless they have a
notable impact on users.

Add the pull request number to changelog entries when available. If the pull
request number is not known yet, update the changelog after creating the pull
request.

Place each changelog entry in one of the following subsections, depending on the
types of changes defined by
[Keep a changelog](https://keepachangelog.com/en/1.0.0/):

* `Added` for new features.
* `Changed` for changes in existing functionality.
* `Deprecated` for soon-to-be removed features.
* `Removed` for now removed features.
* `Fixed` for any bug fixes.
* `Security` in case of vulnerabilities.

If the required subsection does not exist yet under **Unreleased**, create it!

## Developing

If you want to contribute to the development of `cargo-sync-rdme`, you can
follow the steps below.

### Setup

This is no different than other Rust projects.

```console
git clone https://github.com/gifnksm/cargo-sync-rdme
cd cargo-sync-rdme
```

### Useful Commands

* Optionally, run the full CI-equivalent suite locally, including docs and
  tests, using mise. Installing mise is not required to contribute:

  ```console
  mise run ci
  ```

* Build and run release version:

  ```console
  cargo build --release && cargo run --release
  ```

If you use mise, see `mise tasks ls` for more commands.
