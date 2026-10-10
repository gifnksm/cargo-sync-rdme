//! Integration test to ensure that package selection arguments work as expected.

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
#[case("workspace", "pkg-b/src", &["--manifest-path", "../../Cargo.toml"], &["root"])]
#[case("workspace", "pkg-b/src", &["-m", "../../Cargo.toml"], &["root"])]
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
    let workspace = Workspace::from_fixture(fixture_name);

    workspace
        .cargo_sync_rdme_default()
        .args(flags)
        .current_dir(cwd)
        .assert()
        .success();

    assert_updated_packages(&workspace, expected);
}

#[rstest]
#[case("workspace", "Cargo.toml", &[], &["root"])]
#[case("workspace", "pkg-b/Cargo.toml", &[], &["pkg-b"])]
#[case("workspace", "pkg-a/Cargo.toml", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace", "pkg-a/Cargo.toml", &["--workspace"], &["pkg-a", "pkg-b", "root"])]
#[case("workspace_default", "Cargo.toml", &[], &["pkg-a"])]
#[case("workspace_default", "pkg-b/Cargo.toml", &[], &["pkg-b"])]
#[case("workspace_default", "pkg-a/Cargo.toml", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace_default", "pkg-a/Cargo.toml", &["--workspace"], &["pkg-a", "pkg-b", "root"])]
#[case("workspace_virtual", "Cargo.toml", &[], &["pkg-a", "pkg-b"])]
#[case("workspace_virtual", "pkg-b/Cargo.toml", &[], &["pkg-b"])]
#[case("workspace_virtual", "pkg-a/Cargo.toml", &["-p", "pkg-b"], &["pkg-b"])]
#[case("workspace_virtual", "pkg-a/Cargo.toml", &["--workspace"], &["pkg-a", "pkg-b"])]
fn select_target_packages_by_manifest_path_outside_workspace(
    #[case] fixture_name: &str,
    #[case] manifest: &str,
    #[case] flags: &[&str],
    #[case] expected: &[&str],
    #[values(false, true)] absolute_path: bool,
) {
    let workspace = Workspace::from_fixture(fixture_name);
    // Run from the workspace's parent so Cargo cannot discover this workspace
    // without the forwarded --manifest-path argument.
    let manifest_path = if absolute_path {
        workspace.root_path().join(manifest)
    } else {
        std::path::Path::new(workspace.root_path().file_name().unwrap()).join(manifest)
    };

    workspace
        .cargo_sync_rdme_default()
        .current_dir("..")
        .args([
            std::ffi::OsStr::new("--manifest-path"),
            manifest_path.as_os_str(),
        ])
        .args(flags)
        .assert()
        .success();

    assert_updated_packages(&workspace, expected);
}

fn assert_updated_packages(workspace: &Workspace, expected: &[&str]) {
    let mut updated = workspace
        .metadata()
        .workspace_packages()
        .into_iter()
        .filter(|pkg| {
            let readme = pkg.readme().unwrap();
            eprintln!("{}", pkg.name);
            eprintln!("{}", std::fs::read_to_string(&readme).unwrap());
            match helper::collect_list_item_from_markdown_file(&readme).as_slice() {
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
