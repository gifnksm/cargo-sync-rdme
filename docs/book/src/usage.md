# Usage

To use `cargo-sync-rdme`, you need to add marker comments `<!-- cargo-sync-rdme ... -->` in your package README.
`cargo-sync-rdme` will replace the marker comments with the generated content.

Add the following marker comments to your package README (for example, `README.md`):

```markdown
<!-- cargo-sync-rdme rustdoc -->
```

Run the following command to synchronize your package README:

```bash
cargo sync-rdme --toolchain nightly
```

`cargo-sync-rdme` uses unstable rustdoc features when synchronizing crate documentation into Markdown files.
In that case, a nightly Rust toolchain is required.
By default, `cargo-sync-rdme` uses the same toolchain as the `cargo` command you ran.
Use the `--toolchain` option to specify the toolchain to use.
You can also specify the toolchain in the `Cargo.toml` by adding the following lines:

```toml
[package.metadata.cargo-sync-rdme.rustdoc]
toolchain = "nightly"
```

See [Configuration reference](configuration.md#rustdoctoolchain) for more information about the configuration options.

If the specified toolchain is not installed, the command may fail.
To install it manually, run the following command:

```console
rustup toolchain install nightly
```

Alternatively, `cargo-sync-rdme` can install the requested toolchain automatically:

```console
cargo sync-rdme --toolchain nightly --install-toolchain
```

After running the command, your package README will be updated as follows:

```markdown
<!-- cargo-sync-rdme rustdoc [[ -->
(your crate documentation here)
<!-- cargo-sync-rdme ]] -->
```

## Marker Comments

There are three kinds of marker comments:

* `<!-- cargo-sync-rdme title -->`: generate document title (H1 element) from package name.
* `<!-- cargo-sync-rdme badge -->`: generate badges from package metadata.
* `<!-- cargo-sync-rdme rustdoc -->`: embed sections from crate documentation.

To generate badges, you need to list the badge configurations in the `Cargo.toml` file. For example:

```toml
[package.metadata.cargo-sync-rdme.badge]
badges = {
  crates-io = true,
  docs-rs = true,
  license = true,
}
```

See [Configuration reference](configuration.md#badge-items) for more information about the configuration options.
