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

# Build docs across exhaustive feature patterns.
doc-all *args:
    cargo hack doc --workspace --feature-powerset {{ args }}

# Build docs.rs-compatible docs for all packages.
docs-rs-all *args:
    rustup run nightly cargo hack docs-rs {{ args }}

# Synchronize README snippets for all packages.
sync-rdme-all *args:
    rustup run nightly cargo run -- --workspace {{ args }}

# Detect unused dependencies.
machete *args:
    cargo machete {{ args }}

# Run all CI-equivalent checks.
ci: ci-rustfmt ci-check ci-clippy ci-rustdoc ci-docs-rs ci-sync-rdme ci-machete ci-test ci-coverage

# CI: formatting must be clean.
ci-rustfmt:
    just fmt --check

# CI: compile checks.
ci-check:
    just check-all

# CI: clippy warnings are treated as errors.
ci-clippy:
    just clippy-all -- -D warnings

# CI: rustdoc warnings are treated as errors.
[env("RUSTDOCFLAGS", x'${RUSTDOCFLAGS:-} -D warnings')]
ci-rustdoc:
    just doc-all --no-deps

# CI: docs.rs warnings are treated as errors.
[env("RUSTDOCFLAGS", x'${RUSTDOCFLAGS:-} -D warnings')]
ci-docs-rs:
    just docs-rs-all

# CI: README sync must produce no diff.
ci-sync-rdme:
    just sync-rdme-all --check

# CI: dependency hygiene.
ci-machete:
    just machete

# CI: test suite.
ci-test:
    just test-all

# CI: uploadable coverage artifact.
ci-coverage:
    just llvm-cov-all --codecov --output-path target/codecov.json

# Pre-release gate is equivalent to full CI.
pre-release:
    if [ -n "${GITHUB_ACTIONS:-}" ]; then \
        echo "Skip pre-release CI on GitHub Actions (GITHUB_ACTIONS=${GITHUB_ACTIONS})"; \
    else \
        just ci; \
    fi
