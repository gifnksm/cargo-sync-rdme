//! Integration test for CLI output snapshots.

use std::{fs::File, io::Write as _};

use rstest::rstest;
use snapbox::{Data, data::DataFormat};
use test_helper::{self as helper, CargoSyncRdme, Workspace};

fn expected(fixture_name: &str) -> Data {
    Data::read_from(
        &helper::snapshot_path(&format!("{fixture_name}.term.svg")),
        Some(DataFormat::TermSvg),
    )
}

#[test]
fn help_matches_snapshot() {
    CargoSyncRdme::new()
        .force_color()
        .args(["--help"])
        .assert()
        .success()
        .stdout_eq(expected("help.stdout"))
        .stderr_eq("");
}

#[rstest]
fn marker_parse_errors_matches_snapshot(
    #[values("root", "pkg-a")] package_name: &str,
    #[values("readme", "extra")] target_name: &str,
) {
    let workspace = Workspace::from_fixture("workspace");
    let package = workspace
        .metadata()
        .workspace_packages()
        .into_iter()
        .find(|p| p.name == package_name)
        .unwrap();
    let target_path = match target_name {
        "readme" => package.readme().unwrap(),
        "extra" => package
            .manifest_path
            .parent()
            .unwrap()
            .join("doc")
            .join("extra.md"),
        _ => panic!("unexpected target_name: {target_name}"),
    };

    let mut file = File::create(target_path).unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme unknown-specifier -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme title -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme rustdoc -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme badge -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme badge:invalid-group -->").unwrap();
    file.flush().unwrap();
    drop(file);

    workspace
        .cargo_sync_rdme_snapshot_default()
        .args(["-p", package_name])
        .assert()
        .failure()
        .stdout_eq("")
        .stderr_eq(expected(&format!(
            "marker_parse_error.{package_name}.{target_name}.stderr"
        )));
}
