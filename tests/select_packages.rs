//! Integration test to ensure that package selection arguments work as expected.

use assert_fs::prelude::*;
use rstest::rstest;
use similar_asserts::assert_eq;
use test_helper::{self as helper, Workspace};

#[rstest]
#[case("workspace", "", &[], &["root"])]
#[case("workspace", "", &["--workspace"], &["pkg-a", "pkg-b", "root"])]
#[case("workspace", "", &["-p", "pkg-a", "-p", "pkg-b"], &["pkg-a", "pkg-b"])]
#[case("workspace", "", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace", "pkg-a", &[], &["pkg-a"])]
#[case("workspace", "pkg-a", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace", "pkg-b", &[], &["pkg-b"])]
#[case("workspace", "pkg-b/src", &[], &["pkg-b"])]
#[case("workspace_default", "", &[], &["pkg-a"])]
#[case("workspace_default", "", &["--workspace"], &["pkg-a", "pkg-b", "root"])]
#[case("workspace_default", "", &["-p", "pkg-a", "-p", "pkg-b"], &["pkg-a", "pkg-b"])]
#[case("workspace_default", "", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace_default", "pkg-a", &[], &["pkg-a"])]
#[case("workspace_default", "pkg-a", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace_default", "pkg-b", &[], &["pkg-b"])]
#[case("workspace_default", "pkg-b/src", &[], &["pkg-b"])]
#[case("workspace_virtual", "", &[], &["pkg-a", "pkg-b"])]
#[case("workspace_virtual", "", &["--workspace"], &["pkg-a", "pkg-b"])]
#[case("workspace_virtual", "", &["-p", "pkg-a", "-p", "pkg-b"], &["pkg-a", "pkg-b"])]
#[case("workspace_virtual", "", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace_virtual", "pkg-a", &[], &["pkg-a"])]
#[case("workspace_virtual", "pkg-a", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace_virtual", "pkg-b", &[], &["pkg-b"])]
#[case("workspace_virtual", "pkg-b/src", &[], &["pkg-b"])]
fn select_target_packages_by_flags(
    #[case] fixture_name: &str,
    #[case] cwd: &str,
    #[case] flags: &[&str],
    #[case] expected: &[&str],
) {
    helper::ensure_nightly_toolchain_installed();

    let workspace = Workspace::from_fixture(fixture_name);

    let mut cmd = helper::sync_rdme_command(&workspace);
    cmd.args(flags).current_dir(workspace.child(cwd));
    let result = cmd.assert().success();
    eprintln!("{result}");

    let mut updated = workspace
        .metadata()
        .workspace_packages()
        .into_iter()
        .filter(|pkg| {
            let readme = pkg.readme().unwrap();
            eprintln!("{}", pkg.name);
            eprintln!("{}", std::fs::read_to_string(&readme).unwrap());
            match helper::collect_list_item_from_markdown(&readme).as_slice() {
                [s] if s.trim() == "UPDATED" => true,
                [s] if s.trim() == "NOT_UPDATED" => false,
                items => panic!("Unexpected content `{items:?}` in README: {readme:?}"),
            }
        })
        .map(|pkg| pkg.name.as_ref())
        .collect::<Vec<_>>();

    updated.sort_unstable();

    assert_eq!(updated, expected);
}
