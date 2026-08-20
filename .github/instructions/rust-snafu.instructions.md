---
applyTo: "**/*.rs"
---

# Rust Snafu review guidance

When reviewing Rust code in this repository, be aware that many errors
are constructed through Snafu-generated selectors written as
`...Snafu { ... }`.

These selector values are then consumed by generated selector APIs or
related helper APIs, for example through `.build()`,
`.into_error(source)`, `ensure!(..., ...)`, `.context(...)`, or
`.with_context(...)`.

Treat these as uses of generated selector APIs, not as direct
constructions of the final error value.

In the common ways this repository uses Snafu selectors, the generated
APIs accept inputs under `Into<FinalFieldType>` bounds and convert
those inputs with `Into::into(...)`.

Examples from this repository include:

- `NoSuchBadgeGroupSnafu { group: group.value, ... }`, where
  `group.value` is `&str` and the final stored field type is `String`
- `MissingRepositoryMetadataSnafu { path: &package.manifest_path }`,
  where the argument is borrowed and the final stored field type is an
  owned path buffer such as `Utf8PathBuf`
- `MissingRustVersionMetadataSnafu { path: &package.manifest_path }`
- `MissingLicenseMetadataSnafu { path: &package.manifest_path }`
- `OpenWorkflowsDirSnafu { path: workflows_dir_path }.into_error(source)`
- `ReadWorkflowsDirSnafu { path: workflows_dir_path.clone() }.into_error(source)`

Because ordinary type-checking failures are already caught by the Rust
compiler and CI, do not prioritize speculative review comments that
merely compare selector argument types against the final stored error
field types.

In particular, do not leave review comments that merely restate that
`&str` is not `String` or that a borrowed path is not an owned path
buffer when the code is using a Snafu selector.

Prefer comments about concrete semantic or behavioral issues that would
still matter after successful compilation. If you suspect a real issue
at a Snafu selector call site, evaluate it using the generated
selector semantics and report it only when there is concrete evidence
that the code would behave incorrectly or that the selector usage is
invalid despite the surrounding pattern.
