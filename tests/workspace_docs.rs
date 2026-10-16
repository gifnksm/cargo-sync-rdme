//! Integration test to ensure that the workspace documentation is up to date. This
//! test will fail if the documentation is not current, prompting developers to
//! update it accordingly.

use test_helper::Workspace;

#[test]
fn ensure_workspace_documents_are_up_to_date() {
    let workspace = Workspace::open();
    workspace
        .cargo_sync_rdme_default()
        .args(["--workspace", "--check"])
        .assert()
        .success();
}
