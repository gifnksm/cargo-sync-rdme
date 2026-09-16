# Configuration

You can customize the behavior of `cargo-sync-rdme` by adding the following section to `Cargo.toml`.

```toml
[package.metadata.cargo-sync-rdme]
extra-targets = "./docs/configuration.md"

[package.metadata.cargo-sync-rdme.badge]
style = "for-the-badge"
badges = {
  maintenance = true,
  license = true,
}

[package.metadata.cargo-sync-rdme.rustdoc]
html-root-url = "<url>"
```

You can also define common configuration for all packages in a workspace by adding the following section to the workspace `Cargo.toml`.

```toml
[workspace.metadata.cargo-sync-rdme]
...
```

## Top-level table (`cargo-sync-rdme`)

You can configure `cargo-sync-rdme` in `Cargo.toml` under either `package.metadata` or `workspace.metadata`.

Examples below use `package.metadata`.

```toml
[package.metadata.cargo-sync-rdme]
extra-targets = "./docs/configuration.md"
```

### `extra-targets`

Additional Markdown files to synchronize.

By default, `cargo sync-rdme` updates the package README specified by `package.readme`. The `extra-targets` option specifies additional Markdown files to synchronize.

Relative paths are resolved from the `Cargo.toml` file that declares `extra-targets`.

You can specify either a string or an array of strings.

* **Value type:** `string` or `[string]`
* **Default:** `[]` (no additional Markdown files are synchronized)
* **Examples:**

  Specifying a single Markdown file:

  ```toml
  [package.metadata.cargo-sync-rdme]
  extra-targets = "./docs/configuration.md"
  ```

  Specifying multiple Markdown files:

  ```toml
  [package.metadata.cargo-sync-rdme]
  extra-targets = ["./docs/configuration.md", "./docs/usage.md"]
  ```

## `badge` table

The `badge` table configures the badges synchronized by `cargo-sync-rdme`.
It can be defined under either `package.metadata.cargo-sync-rdme` or `workspace.metadata.cargo-sync-rdme`:

* **Examples:**

  Configuration:

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  style = "flat-square"
  badges = {
    maintenance = true,
    license = { link = "#license" },
  }
  ```

  Markdown (before sync):

  ```markdown
  <!-- cargo-sync-rdme badge -->
  ```

  Markdown (after sync):

  ```markdown
  <!-- cargo-sync-rdme badge [[ -->
  [![Maintenance: actively-developed](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg?style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
  [![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/cargo-sync-rdme.svg?style=flat-square)](#license)
  <!-- cargo-sync-rdme ]] -->
  ```

Badges are output in the order in which the configuration items are written.

### `badge.style`

The `style` option specifies the style of the badge.

* **Value type:** `string`
* **Default:** none (no style is specified)
* **Possible values:**

  | Configuration           | Output                                                                                             |
  | ----------------------- | -------------------------------------------------------------------------------------------------- |
  | Not specified (default) | ![none](https://img.shields.io/badge/style-none-green.svg)                                         |
  | `style="flat"`          | ![flat](https://img.shields.io/badge/style-flat-green.svg?style=flat)                              |
  | `style="flat-square"`   | ![flat-square](https://img.shields.io/badge/style-flat--square-green.svg?style=flat-square)        |
  | `style="for-the-badge"` | ![for-the-badge](https://img.shields.io/badge/style-for--the--badge-green.svg?style=for-the-badge) |
  | `style="plastic"`       | ![plastic](https://img.shields.io/badge/style-plastic-green.svg?style=plastic)                     |
  | `style="social"`        | ![social](https://img.shields.io/badge/style-social-green.svg?style=social)                        |

### `badge.badges` and `badge.badges-<group-name>`

Defines the badge group.

`badge.badges` defines the default badge group.
In the markdown file, `<!-- cargo-sync-rdme badge -->` is replaced with the badges in the default badge group.

`badge.badges-<group-name>` defines a badge group named `<group-name>`.
`<group-name>` must match `[A-Za-z][-_A-Za-z0-9]*`.
In the markdown file, `<!-- cargo-sync-rdme badge:<group-name> -->` is replaced with the badges in the corresponding badge group.

`<key> = <value>` entries in the badge group table define the badges to output.
`<key>` is the badge kind, and `<value>` is the badge configuration.
See the [Badge items](#badge-items) section for details about the available badge kinds and their configuration.
If you want to use the same kind of badge multiple times, add a unique hyphenated suffix (`-*`) to avoid duplicate key names, such as `maintenance-foo = true`.

Badges are output in the order in which the configuration items are written.

* **Examples:**

  Configuration:

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  style = "flat-square"

  # Default badge group
  badges = {
    maintenance = true,
  }

  # `foo` badge group
  badges-foo = {
    license = true,
  }

  # `bar` badge group
  badges-bar = {
    github-actions-ci = { workflows = "ci.yml" },
    github-actions-cd = { workflows = "cd.yml" },
  }
  ```

  Markdown (before sync):

  ```markdown
  <!-- cargo-sync-rdme badge -->
  <!-- cargo-sync-rdme badge:foo -->
  <!-- cargo-sync-rdme badge:bar -->
  ```

  Markdown (after sync):

  ```markdown
  <!-- cargo-sync-rdme badge [[ -->
  [![Maintenance: actively-developed](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg?style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
  <!-- cargo-sync-rdme ]] -->

  <!-- cargo-sync-rdme badge:foo [[ -->
  ![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/cargo-sync-rdme.svg?style=flat-square)
  <!-- cargo-sync-rdme ]] -->

  <!-- cargo-sync-rdme badge:bar [[ -->
  [![GitHub Actions: CI](https://img.shields.io/github/actions/workflow/status/gifnksm/cargo-sync-rdme/ci.yml.svg?label=CI&logo=github&style=flat-square)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/ci.yml)
  [![GitHub Actions: CD](https://img.shields.io/github/actions/workflow/status/gifnksm/cargo-sync-rdme/cd.yml.svg?label=CD&logo=github&style=flat-square)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/cd.yml)
  <!-- cargo-sync-rdme ]] -->
  ```

## Badge items

The following configuration items are available for badges:

### Maintenance status

<!-- cargo-sync-rdme badge:maintenance [[ -->
[![Maintenance: actively-developed](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg?style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
<!-- cargo-sync-rdme ]] -->

A badge indicating the maintenance status of the package.

The badge is generated from the `package.metadata.maintenance.status` field in `Cargo.toml`
(see [the cargo documentation](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section) for details).

The link target of the badge is set to <https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section>.

* **Value type:** `boolean`
* **Default:** `false` (no maintenance status badge is output)
* **Possible values:**
  * `true`: Output a maintenance status badge
  * `false` (default): Do not output a maintenance status badge
* **Examples:**

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    maintenance = true
  }
  ```

### License

<!-- cargo-sync-rdme badge:license [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/cargo-sync-rdme.svg?style=flat-square)
<!-- cargo-sync-rdme ]] -->

A badge indicating the license of the package.

The badge is generated from the `package.license` field or `package.license-file` field in `Cargo.toml`
(see [the cargo documentation](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields) for details).

The link target of the badge is determined by the badge configuration.

* **Value type:** `boolean` or `table`
* **Default:** `false` (no license badge is output)
* **Possible values:**
  * `{ link = "<link>" }`: Output a license badge. The link target of the badge is set to `<link>`
  * `true`: Output a license badge
    * If `package.license-file` is specified, the link target of the badge is set to the license file
    * If `package.license` is specified, no link is set
  * `false` (default): Do not output a license badge
* **Examples:**

  Output a license badge using the default link behavior:

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    license = true
  }
  ```

  Output a license badge with a link to the specified URL:

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    license = { link = "https://opensource.org/licenses/MIT" }
  }
  ```

### crates.io

<!-- cargo-sync-rdme badge:crates-io [[ -->
[![crates.io](https://img.shields.io/crates/v/cargo-sync-rdme.svg?logo=rust&style=flat-square)](https://crates.io/crates/cargo-sync-rdme)
<!-- cargo-sync-rdme ]] -->

A badge indicating the version of the package on crates.io.

The badge is generated from the `package.name` field in `Cargo.toml`
(see [the cargo documentation](https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field) for details).

The link target of the badge is set to `https://crates.io/crates/<package name>`.

* **Value type:** `boolean`
* **Default:** `false` (no crates.io badge is output)
* **Possible values:**
  * `true`: Output a crates.io badge
  * `false` (default): Do not output a crates.io badge
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    crates-io = true
  }
  ```

### Docs.rs

<!-- cargo-sync-rdme badge:docs-rs [[ -->
[![docs.rs](https://img.shields.io/docsrs/cargo-sync-rdme.svg?logo=docs.rs&style=flat-square)](https://docs.rs/cargo-sync-rdme)
<!-- cargo-sync-rdme ]] -->

A badge indicating the documentation build status of the package on docs.rs.

The badge is generated from the `package.name` field in `Cargo.toml`
(see [the cargo documentation](https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field) for details).

The link target of the badge is set to `https://docs.rs/<package name>`.

* **Value type:** `boolean`
* **Default:** `false` (no docs.rs badge is output)
* **Possible values:**
  * `true`: Output a docs.rs badge
  * `false` (default): Do not output a docs.rs badge
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    docs-rs = true
  }
  ```

### Rust Version (MSRV)

<!-- cargo-sync-rdme badge:rust-version [[ -->
[![Rust: ^1.98.0](https://img.shields.io/badge/rust-^1.98.0-93450a.svg?logo=rust&style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
<!-- cargo-sync-rdme ]] -->

A badge indicating the minimum supported Rust version (MSRV) of the package.

The badge is generated from the `package.rust-version` field in `Cargo.toml`
(see [the cargo documentation](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field) for details).

The link target of the badge is set to <https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field>.

* **Value type:** `boolean`
* **Default:** `false` (no rust version badge is output)
* **Possible values:**
  * `true`: Output a rust version badge
  * `false` (default): Do not output a rust version badge
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    rust-version = true
  }
  ```

### GitHub Actions

<!-- cargo-sync-rdme badge:github-actions [[ -->
[![GitHub Actions: CD](https://img.shields.io/github/actions/workflow/status/gifnksm/cargo-sync-rdme/cd.yml.svg?label=CD&logo=github&style=flat-square)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/cd.yml)
[![GitHub Actions: CI](https://img.shields.io/github/actions/workflow/status/gifnksm/cargo-sync-rdme/ci.yml.svg?label=CI&logo=github&style=flat-square)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/ci.yml)
[![GitHub Actions: Deploy Rustdoc to GitHub Pages](https://img.shields.io/github/actions/workflow/status/gifnksm/cargo-sync-rdme/pages.yml.svg?label=Deploy+Rustdoc+to+GitHub+Pages&logo=github&style=flat-square)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/pages.yml)
[![GitHub Actions: Renovate Post Update](https://img.shields.io/github/actions/workflow/status/gifnksm/cargo-sync-rdme/renovate-post-update.yml.svg?label=Renovate+Post+Update&logo=github&style=flat-square)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/renovate-post-update.yml)
[![GitHub Actions: Security Audit](https://img.shields.io/github/actions/workflow/status/gifnksm/cargo-sync-rdme/audit.yml.svg?label=Security+Audit&logo=github&style=flat-square)](https://github.com/gifnksm/cargo-sync-rdme/actions/workflows/audit.yml)
<!-- cargo-sync-rdme ]] -->

A badge indicating the status of a GitHub Actions workflow.

The badge is generated from the configured workflows.

The link target of the badge is set to `<package.repository>/actions/workflows/<file>`.
`<file>` is the name of the file in the `.github/workflows` directory.

* **Value type:** `boolean` or `table`
* **Default:** `false` (no GitHub Actions badge is output)
* **Possible values:**
  * `{ workflows = [ { file = "<file>", name = "<name>" } ] }`:
    Output GitHub Actions status badges.

    The link target of the badge is set to `<package.repository>/actions/workflows/<file>`.

    The array can contain multiple workflow objects.

    `<name>` is used as the badge name.
    If `<name>` is not specified, the name of the workflow defined in the `<file>` is used as the badge name.
  * `{ workflows = [ "<file>" ] }` and `{ workflows = "<file>" }`:
    Same as `{ workflows = [ { file = "<file>" } ] }`
  * `{ workflows = [] }`:
    Output GitHub Actions status badges for all workflows in the `.github/workflows` directory.
  * `true`: Same as `github-actions = { workflows = [] }`
  * `false` (default): Do not output a GitHub Actions status badge
* **Examples:**

  Output all GitHub Actions workflow statuses:

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    github-actions = true
  }
  ```

  Output specified GitHub Actions workflow statuses:

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    github-actions = { workflows = ["ci.yml", "cd.yml"] }
  }
  ```

### Codecov

<!-- cargo-sync-rdme badge:codecov [[ -->
[![Codecov](https://img.shields.io/codecov/c/github/gifnksm/cargo-sync-rdme.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/gifnksm/cargo-sync-rdme)
<!-- cargo-sync-rdme ]] -->

A badge indicating the coverage of the package.

The badge is generated from the `package.repository` field in `Cargo.toml`
(see [the cargo documentation](https://doc.rust-lang.org/cargo/reference/manifest.html#the-repository-field) for details).

The link target of the badge is set to `https://codecov.io/gh/<repository_path>/`.

* **Value type:** `boolean` or `table`
* **Default:** `false` (no Codecov badge is output)
* **Possible values:**
  * `{ ... }`: Output a Codecov badge with the following additional options
    * `component = "<component>"`: Add the `component=<component>` query parameter to the badge image URL
    * `flag = "<flag>"`: Add the `flag=<flag>` query parameter to the badge image URL
  * `true`: Output a Codecov badge with default configuration
  * `false` (default): Do not output a Codecov badge
* **Examples:**

  Output Codecov badge with default configuration:

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    codecov = true
  }
  ```

  Output a Codecov badge with a component and a flag:

  ```toml
  [package.metadata.cargo-sync-rdme.badge]
  badges = {
    codecov = { component = "cli", flag = "integration-test" }
  }
  ```

## `rustdoc` table

The `rustdoc` table configures the crate documentation synchronized by `cargo-sync-rdme`.
It can be defined under either `package.metadata.cargo-sync-rdme` or `workspace.metadata.cargo-sync-rdme`:

* **Examples:**

  Configuration:

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  toolchain = "nightly"
  features = ["feature1", "feature2"]
  all-features = true
  no-default-features = true
  standard-library-url-mode = "channel"
  html-root-url = "<url>"
  mappings = {
    SomeType = "https://example.com/docs/struct.SomeType.html",
  }
  ```

  Markdown (before sync):

  ```markdown
  <!-- cargo-sync-rdme rustdoc -->
  ```

  Markdown (after sync):

  ```markdown
  <!-- cargo-sync-rdme rustdoc [[ -->
  (synchronized rustdoc output)
  <!-- cargo-sync-rdme rustdoc ]] -->
  ```

The following configuration items are available for rustdoc:

### `rustdoc.toolchain`

Set the default toolchain to use for generating the crate documentation.

You can override this default with the `--toolchain` command line option.

* **Value type:** `string`
* **Default:** the toolchain used to run `cargo-sync-rdme`
* **Possible values:** any valid toolchain name accepted by `rustup run <toolchain> cargo rustdoc`, such as `stable`, `beta`, `nightly`, `1.70.0`, or `1.70.0-x86_64-unknown-linux-gnu`.
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  toolchain = "nightly"
  ```

### `rustdoc.features`

Set the default feature flags to pass to `cargo rustdoc` when generating the crate documentation.

You can override these defaults with the `--features` command line options.

* **Value type:** `[string]`
* **Default:** `[]` (no feature flags are passed to `cargo rustdoc`)
* **Possible values:** array of any valid feature name defined in the crate's `Cargo.toml`.
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  features = ["feature1", "feature2"]
  ```

### `rustdoc.all-features`

Whether to pass `--all-features` to `cargo rustdoc`.

You can also enable this with the `--all-features` command line option.

* **Value type:** `boolean`
* **Default:** `false` (do not pass `--all-features` to `cargo rustdoc`)
* **Possible values:**
  * `true`: pass `--all-features` to `cargo rustdoc`
  * `false` (default): do not pass `--all-features` to `cargo rustdoc`
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  all-features = true
  ```

### `rustdoc.no-default-features`

Whether to pass `--no-default-features` to `cargo rustdoc`.

You can also enable this with the `--no-default-features` command line option.

* **Value type:** `boolean`
* **Default:** `false` (do not pass `--no-default-features` to `cargo rustdoc`)
* **Possible values:**
  * `true`: pass `--no-default-features` to `cargo rustdoc`
  * `false` (default): do not pass `--no-default-features` to `cargo rustdoc`
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  no-default-features = true
  ```

### `rustdoc.standard-library-url-mode`

Control how links to Rust standard library items hosted on <https://doc.rust-lang.org> are rewritten.

This affects generated links to items in crates such as `std`, `core`, and `alloc`.

This option only rewrites URLs under <https://doc.rust-lang.org/>.
Other URLs are left unchanged.

This is most useful when `rustdoc.toolchain` or `--toolchain` uses a different toolchain to build the rustdoc output.

If an item lives at a different documentation path in the toolchain that generated the rustdoc output and the URL selected for the generated link, the generated link may be invalid.
To override a specific target, add a mapping with `rustdoc.mappings`.

* **Value type:** `string`
* **Default:** `channel`
* **Possible values:**
  * `channel` (default): rewrite the URL to the release channel of the toolchain running Cargo: `stable`, `beta`, or `nightly`.
  * `version`: rewrite the URL to the versioned documentation for the toolchain running Cargo.
    Versioned documentation is not published for beta or nightly, so those channels use `beta` or `nightly` URLs instead.
  * `as-is`: keep the URL emitted by rustdoc.
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  standard-library-url-mode = "version"
  ```

### `rustdoc.html-root-url`

Set the root URL of the documentation for the package.

If you host the documentation on GitHub Pages, you can set the value to `https://<user>.github.io/<repository>/`.

* **Value type:** `string`
* **Default:** `https://docs.rs/<package name>/<package version>`
* **Possible values:** any valid URL.
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  html-root-url = "https://docs.rs/my-package/1.0.0"
  ```

### `rustdoc.mappings`

Allows you to provide an override for rustdoc intra-doc link resolution.

With `{ <path> = "<url>" }` mapping, the intra-doc link to `<path>` is resolved to `<url>`.

`<path>` must match the intra-doc link target exactly as written in the rustdoc source.
If you use intra-doc link shorthands such as ``[`foo`]``, the mapping key must include the backticks (for example, ``"`foo`" = "https://example.com/"``).

This is useful when the `cargo-sync-rdme` output contains incorrect links.

* **Value type:** `{ <path> = "<url>" }`
* **Default:** `{}` (no mappings are provided)
* **Possible values:** any valid path and URL.
* **Example:**

  ```toml
  [package.metadata.cargo-sync-rdme.rustdoc]
  mappings = {
    SomeType = "https://example.com/docs/struct.SomeType.html",
  }
  ```
