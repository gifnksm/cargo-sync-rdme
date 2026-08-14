//! Integration test to ensure that feature selection arguments work as expected.

use rstest::rstest;
use similar_asserts::assert_eq;
use test_helper::{self as helper, Workspace};

#[rstest]
#[case(&[], &["DEFAULT", "FEAT_A"])]
#[case(&["--features", "default"], &["DEFAULT", "FEAT_A"])]
#[case(&["--features", "feat-b"], &["DEFAULT", "FEAT_A", "FEAT_B"])]
#[case(&["--all-features"], &["DEFAULT", "FEAT_A", "FEAT_B", "FEAT_C"])]
#[case(&["--no-default-features"], &[])]
#[case(&["--no-default-features", "--features", "feat-a"], &["FEAT_A"])]
#[case(&["--no-default-features", "--features", "feat-b"], &["FEAT_B"])]
#[case(&["--no-default-features", "--all-features"], &["DEFAULT", "FEAT_A", "FEAT_B", "FEAT_C"])]
fn select_features_by_flags(#[case] flags: &[&str], #[case] expected: &[&str]) {
    helper::ensure_nightly_toolchain_installed();

    let crate_name = "features";
    let workspace = Workspace::from_fixture(crate_name);
    let readme_path = workspace
        .metadata()
        .root_package()
        .unwrap()
        .readme()
        .unwrap();

    helper::sync_readme_with_args(&workspace, flags);

    let list_items = helper::collect_list_item_from_markdown(readme_path);
    assert_eq!(list_items, expected);
}
