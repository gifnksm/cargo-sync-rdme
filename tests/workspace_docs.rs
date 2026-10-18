//! Integration test to ensure that the workspace documentation is up to date. This
//! test will fail if the documentation is not current, prompting developers to
//! update it accordingly.

use test_helper::Workspace;

#[test]
fn ensure_workspace_documents_are_up_to_date() {
    let workspace = Workspace::open();
    let cargo_version = workspace.cargo_version(None);
    if cargo_version.ends_with("-nightly") || cargo_version.contains("-beta.") {
        eprintln!("Skipping test on nightly or beta toolchain: {cargo_version}");
        return;
    }
    workspace
        .cargo_sync_rdme_default()
        .args(["--workspace", "--check"])
        .assert()
        .success();
}
