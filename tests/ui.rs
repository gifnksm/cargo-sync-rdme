//! Integration test for CLI output snapshots.

use std::{fs::File, io::Write as _};

use rstest::rstest;
use snapbox::{Data, data::DataFormat};
use test_helper::{self as helper, CargoSyncRdme, Workspace};

fn expected(test_name: &str, snapshot_name: &str) -> Data {
    Data::read_from(
        &helper::snapshot_fixtures_dir()
            .join(test_name)
            .join(format!("{snapshot_name}.term.svg")),
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
        .stdout_eq(expected("help", "stdout"))
        .stderr_eq("");
}

#[rstest]
fn marker_parse_errors_matches_snapshot(
    #[values("root", "pkg-a")] package_name: &str,
    #[values("readme", "extra")] target_name: &str,
) {
    let workspace = Workspace::from_fixture("workspace");
    let package = workspace.package(package_name).unwrap();
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

    let mut file = File::options().append(true).open(target_path).unwrap();
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
        .stderr_eq(expected(
            "marker_parse_error",
            &format!("{package_name}.{target_name}.stderr"),
        ));
}

#[rstest]
#[case("root")]
#[case("pkg-a")]
fn check_output_matches_snapshot(#[case] package_name: &str) {
    let workspace = Workspace::from_fixture("workspace");
    workspace
        .cargo_sync_rdme_snapshot_default()
        .args(["-p", package_name, "--check"])
        .assert()
        .failure()
        .stdout_eq("")
        .stderr_eq(expected("check_output", &format!("{package_name}.stderr")));
}

#[rstest]
#[case("root")]
#[case("pkg-a")]
fn sync_output_matches_snapshot(#[case] package_name: &str) {
    let workspace = Workspace::from_fixture("workspace");
    workspace
        .cargo_sync_rdme_snapshot_default()
        .args(["-p", package_name])
        .assert()
        .success()
        .stdout_eq("")
        .stderr_eq(expected("sync_output", &format!("{package_name}.stderr")));
}

#[rstest]
fn basic_config_error_matches_snapshot(
    #[values("root", "pkg-a")] package_name: &str,
    #[values("unknown-table", "unknown-field", "invalid-value")] error_kind: &str,
) {
    let workspace = Workspace::from_fixture("no_config");
    let package = workspace.package(package_name).unwrap();

    let mut manifest = File::options()
        .append(true)
        .open(&package.manifest_path)
        .unwrap();
    match error_kind {
        "unknown-table" => {
            writeln!(&mut manifest, "[package.metadata.cargo-sync-rdme.unknown]").unwrap();
            writeln!(&mut manifest, "foo = true").unwrap();
        }
        "unknown-field" => {
            writeln!(&mut manifest, "[package.metadata.cargo-sync-rdme]").unwrap();
            writeln!(&mut manifest, "unknown = true").unwrap();
        }
        "invalid-value" => {
            writeln!(&mut manifest, "[package.metadata.cargo-sync-rdme]").unwrap();
            writeln!(&mut manifest, "extra-targets = false").unwrap();
        }
        _ => panic!("unexpected error kind: {error_kind}"),
    }
    manifest.flush().unwrap();
    drop(manifest);

    workspace
        .cargo_sync_rdme_snapshot_default()
        .args(["--workspace"])
        .assert()
        .failure()
        .stdout_eq("")
        .stderr_eq(expected(
            "basic_config_error",
            &format!("{package_name}.{error_kind}.stderr"),
        ));
}
