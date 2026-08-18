//! Integration test for CLI output snapshots.

use snapbox::Data;
use test_helper::{self as helper, CargoSyncRdme};

#[test]
fn help_matches_snapshot() {
    CargoSyncRdme::new()
        .force_color()
        .args(["--help"])
        .assert()
        .success()
        .stdout_eq(Data::read_from(
            &helper::snapshot_path("help.stdout.term.svg"),
            None,
        ))
        .stderr_eq("");
}
