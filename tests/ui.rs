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
#[case("root")]
#[case("pkg-a")]
fn marker_parse_errors_matches_snapshot(#[case] package: &str) {
    let workspace = Workspace::from_fixture("workspace");
    let readme_path = workspace
        .metadata()
        .workspace_packages()
        .into_iter()
        .find(|p| p.name == package)
        .unwrap()
        .readme()
        .unwrap();

    let mut file = File::create(readme_path).unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme unknown-specifier -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme title -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme rustdoc -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme badge -->").unwrap();
    writeln!(&mut file, "<!-- cargo-sync-rdme badge:invalid-group -->").unwrap();
    file.flush().unwrap();
    drop(file);

    workspace
        .cargo_sync_rdme_default()
        .force_color()
        .args(["-p", package])
        .assert()
        .failure()
        .stdout_eq("")
        .stderr_eq(expected(&format!("marker_parse_error_{package}.stderr")));
}
