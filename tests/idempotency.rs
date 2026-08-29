//! Integration test to ensure that `cargo-sync-rdme` remains idempotent when rustdoc output contains `<!-- cargo-sync-rdme ... -->`-like comments.

use rstest::rstest;
use similar_asserts::assert_eq;
use test_helper::{self as helper, SPAN_END_MARKER, SPAN_START_MARKER, Workspace};

#[rstest]
#[case::valid_marker("title")]
#[case::invalid_marker("invalid")]
fn generated_markdown_is_idempotent_when_rustdoc_contains_marker_like_comment(
    #[case] marker_body: &str,
) {
    let crate_name = "empty";
    let workspace = Workspace::from_fixture(crate_name);
    let readme_path = workspace
        .metadata()
        .root_package()
        .unwrap()
        .readme()
        .unwrap();

    let doc_comment = indoc::formatdoc! {r"
        //! {SPAN_START_MARKER}
        //! * FIRST ITEM
        //! <!-- cargo-sync-rdme {marker_body} -->
        //! * SECOND ITEM
        //! {SPAN_END_MARKER}
    "};

    workspace.insert_crate_doc_comment("src/lib.rs", &doc_comment);

    workspace.cargo_sync_rdme_default().assert().success();
    workspace.cargo_doc_default().assert().success();

    let md_items = helper::collect_list_item_from_markdown_file(&readme_path);
    assert_eq!(md_items, ["FIRST ITEM", "SECOND ITEM"]);

    let readme_content = std::fs::read_to_string(&readme_path).unwrap();

    // Run sync again to ensure the README remains unchanged.
    workspace.cargo_sync_rdme_default().assert().success();

    assert_eq!(
        std::fs::read_to_string(&readme_path).unwrap(),
        readme_content
    );
}
