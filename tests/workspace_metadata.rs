//! Integration test for workspace-level metadata configuration.

use std::fs;

use indoc::formatdoc;
use similar_asserts::assert_eq;
use test_helper::{self as helper, Workspace};

#[test]
fn workspace_metadata_applies_to_package_and_resolves_relative_extra_targets() {
    let fixture_name = "workspace_metadata";
    let workspace = Workspace::from_fixture(fixture_name);

    workspace
        .cargo_sync_rdme_default()
        .args(["-p", "pkg-a"])
        .assert()
        .success();

    let workspace_dir = workspace.root_path();
    let fixture_dir = helper::package_fixtures_dir().join(fixture_name);

    for (relative_path, title) in [
        ("docs/workspace.md", "workspace extra"),
        ("pkg-a/docs/package.md", "pkg-a extra"),
        ("pkg-a/README.md", "pkg-a"),
    ] {
        assert_eq!(
            fs::read_to_string(workspace_dir.join(relative_path)).unwrap(),
            formatdoc! {r"
                # {title}

                <!-- cargo-sync-rdme badge [[ -->
                [![crates.io](https://img.shields.io/crates/v/pkg-a.svg?logo=rust&style=for-the-badge)](https://crates.io/crates/pkg-a)
                <!-- cargo-sync-rdme ]] -->
            "}
        );
    }

    for relative_path in ["pkg-b/README.md", "README.md"] {
        assert_eq!(
            fs::read_to_string(workspace_dir.join(relative_path)).unwrap(),
            fs::read_to_string(fixture_dir.join(relative_path)).unwrap()
        );
    }
}
