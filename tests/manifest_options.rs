//! Integration test to ensure that `cargo-sync-rdme` accepts manifest options.

use rstest::rstest;
use similar_asserts::assert_eq;
use test_helper::{self as helper, Workspace};

#[rstest]
#[case(&[])]
#[case(&["--manifest-path", "Cargo.toml"])]
#[case(&["--ignore-rust-version"])]
#[case(&["--locked"])]
#[case(&["--offline"])]
#[case(&["--frozen"])]
#[case(&["--manifest-path", "Cargo.toml","--ignore-rust-version", "--locked", "--offline", "--frozen"])]
fn cargo_sync_rdme_accepts_manifest_options(
    #[case] flags: &[&str],
    #[values(false, true)] generate_lockfile: bool,
    #[values(false, true)] invalid_rust_version_package: bool,
) {
    let ignore_rust_version = flags.contains(&"--ignore-rust-version");
    let require_lockfile = flags.contains(&"--locked") || flags.contains(&"--frozen");

    let fail_by_rust_version = invalid_rust_version_package && !ignore_rust_version;
    let fail_by_lockfile = require_lockfile && !generate_lockfile;
    let expect_success = !fail_by_rust_version && !fail_by_lockfile;

    let workspace = Workspace::from_fixture("manifest_options");

    if generate_lockfile {
        workspace.generate_lockfile();
    }

    let mut cmd = workspace.cargo_sync_rdme_default();
    if invalid_rust_version_package {
        cmd.args(["-p", "invalid-rust-version"]);
    }
    let assert = cmd.args(flags).assert();

    if fail_by_lockfile {
        let assert = assert.failure();
        let stderr = std::str::from_utf8(&assert.get_output().stderr).unwrap();
        assert!(
            stderr.contains("cannot create the lock file"),
            "stderr did not contain expected error message: {stderr:?}"
        );
    } else if fail_by_rust_version {
        let assert = assert.failure();
        let stderr = std::str::from_utf8(&assert.get_output().stderr).unwrap();
        assert!(
            stderr.contains("is not supported by the following package:"),
            "stderr did not contain expected error message: {stderr:?}"
        );
    } else {
        assert.success();
    }

    let expect_updated = [
        ("root", expect_success && !invalid_rust_version_package),
        (
            "invalid-rust-version",
            expect_success && invalid_rust_version_package,
        ),
    ];

    for (pkg_name, expect_updated) in expect_updated {
        let readme = workspace.package(pkg_name).unwrap().readme().unwrap();
        let list_items = helper::collect_list_item_from_markdown_file(&readme);
        if expect_updated {
            assert_eq!(list_items, ["UPDATED"]);
        } else {
            assert_eq!(list_items, ["NOT_UPDATED"]);
        }
    }
}
