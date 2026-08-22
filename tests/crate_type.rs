//! Integration test to ensure that the library documentation is preferred over the binary documentation when both are present in a crate.

use test_helper::{self as helper, Workspace};

#[test]
fn prefers_lib_docs_over_bin_docs() {
    let fixture_name = "crate_type";
    let workspace = Workspace::from_fixture(fixture_name);

    workspace
        .cargo_sync_rdme_default()
        .args(["--workspace"])
        .assert()
        .success();

    let bin_pkg = workspace.package("bin").unwrap();
    let lib_pkg = workspace.package("lib").unwrap();
    let bin_lib_pkg = workspace.package("bin-lib").unwrap();

    let list_item = helper::collect_list_item_from_markdown_file(bin_pkg.readme().unwrap());
    assert_eq!(list_item, ["bin/src/main.rs"]);

    let list_item = helper::collect_list_item_from_markdown_file(lib_pkg.readme().unwrap());
    assert_eq!(list_item, ["lib/src/lib.rs"]);

    let list_item = helper::collect_list_item_from_markdown_file(bin_lib_pkg.readme().unwrap());
    assert_eq!(list_item, ["bin-lib/src/lib.rs"]);
}
