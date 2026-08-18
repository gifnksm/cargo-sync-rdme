//! Integration test for CLI output snapshots.

use snapbox::{Data, data::DataFormat};
use test_helper::{self as helper, CargoSyncRdme};

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
