# Public tasks for this repository.

# Format all crates in the workspace.
fmt *args:
    cargo fmt --all {{ args }}

# Run all compile checks.
check-all *args:
    just check-default-target {{ args }}
    just check-tests {{ args }}
    just check-benches {{ args }}

# Check default targets across exhaustive feature patterns.
check-default-target *args:
    cargo hack check --workspace --feature-powerset {{ args }}

# Check test targets across exhaustive feature patterns.
check-tests *args:
    cargo hack check --workspace --feature-powerset --tests {{ args }}

# Check benchmark targets across exhaustive feature patterns.
check-benches *args:
    cargo hack check --workspace --feature-powerset --benches {{ args }}

# Run all clippy checks.
clippy-all *args:
    just clippy-default-target {{ args }}
    just clippy-tests {{ args }}
    just clippy-benches {{ args }}

# Run clippy for default targets across exhaustive feature patterns.
clippy-default-target *args:
    cargo hack clippy --workspace --feature-powerset {{ args }}

# Run clippy for test targets across exhaustive feature patterns.
clippy-tests *args:
    cargo hack clippy --workspace --feature-powerset --tests {{ args }}

# Run clippy for benchmark targets across exhaustive feature patterns.
clippy-benches *args:
    cargo hack clippy --workspace --feature-powerset --benches {{ args }}

# Run tests across exhaustive feature patterns.
test-all *args:
    cargo hack test --workspace --feature-powerset {{ args }}

# Build coverage report.
llvm-cov-all *args:
    # Internal note: intentionally uses --all-features (not --feature-powerset).
    # Reason: powerset-style repeated runs can overwrite codecov output.
    cargo llvm-cov --workspace --all-features {{ args }}

# Build doc for GitHub Pages
doc-gh-pages *args:
    cargo doc --workspace --all-features --no-deps {{ args }}

# Build docs across exhaustive feature patterns.
doc-all *args:
    cargo hack doc --workspace --feature-powerset {{ args }}

# Synchronize README snippets for all packages.
sync-rdme-all *args:
    cargo run -- --workspace --toolchain nightly --install-toolchain {{ args }}

# Detect unused dependencies.
machete *args:
    cargo machete {{ args }}

# Check workflow files.
actionlint *args:
    actionlint {{ args }}

# Check spelling of entire workspace.
typos *args:
    typos {{ args }}

# Lint markdown files.
markdownlint *args:
    npx --yes markdownlint-cli {{ args }} .

# Format TOML files.
tombi-format *args:
    uvx tombi format {{ args }}

# Lint TOML files.
tombi-lint *args:
    uvx tombi lint {{ args }}

# Check EditorConfig compliance.
editorconfig *args:
    editorconfig-checker {{ args }}

# Run lint and static checks used for day-to-day local verification.
ci-lint: ci-rustfmt ci-tombi-format ci-tombi-lint ci-check ci-clippy ci-machete ci-actionlint ci-typos ci-markdownlint ci-editorconfig

# Run all CI-equivalent checks.
ci: ci-lint ci-rustdoc ci-sync-rdme ci-test ci-coverage-test ci-coverage-sync-rdme

# CI: formatting must be clean.
ci-rustfmt:
    just fmt --check

# CI: TOML formatting must be clean.
ci-tombi-format:
    just tombi-format --check

# CI: TOML lint must be clean.
ci-tombi-lint:
    just tombi-lint --error-on-warnings

# CI: compile checks.
ci-check:
    just check-all

# CI: clippy warnings are treated as errors.
[env("CARGO_BUILD_WARNINGS", "deny")]
ci-clippy:
    just clippy-all

# CI: rustdoc warnings are treated as errors.
[env("CARGO_BUILD_WARNINGS", "deny")]
ci-rustdoc:
    just doc-all --no-deps

# CI: README sync must produce no diff.
ci-sync-rdme:
    just sync-rdme-all --check

# CI: dependency hygiene.
ci-machete:
    just machete

# CI: check workflow files.
ci-actionlint:
    just actionlint

# CI: check spelling.
ci-typos:
    just typos

# CI: lint markdown files.
ci-markdownlint:
    just markdownlint

# CI: check EditorConfig compliance.
ci-editorconfig *args:
    just editorconfig {{ args }}

# CI: test suite.
ci-test:
    just test-all

# CI: uploadable coverage artifact.
ci-coverage-test:
    just llvm-cov-all --codecov --output-path target/codecov-test.json

# CI: uploadable coverage artifact.
ci-coverage-sync-rdme:
    cargo llvm-cov --all-features --codecov --output-path target/codecov-sync-rdme.json run -- --workspace --toolchain nightly --install-toolchain --check

# Pre-release gate is equivalent to full CI.
pre-release:
    if [ -n "${GITHUB_ACTIONS:-}" ]; then \
        echo "Skip pre-release CI on GitHub Actions (GITHUB_ACTIONS=${GITHUB_ACTIONS})"; \
    else \
        just ci; \
    fi
