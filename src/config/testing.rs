use indoc::formatdoc;

use crate::{
    config::{Manifest, badge::Badge, rustdoc::Rustdoc},
    source::SourceFile,
};

pub(crate) fn badge_manifest(badge: &str) -> String {
    formatdoc! {r#"
        [package]
        name = "foo"
        version = "0.1.0"

        [package.metadata.cargo-sync-rdme.badge]
        {badge}
    "#}
}

pub(crate) fn rustdoc_manifest(rustdoc: &str) -> String {
    formatdoc! {r#"
        [package]
        name = "foo"
        version = "0.1.0"

        [package.metadata.cargo-sync-rdme.rustdoc]
        {rustdoc}
    "#}
}

pub(crate) fn parse_manifest(source: &str) -> Manifest {
    let source_file = SourceFile::new_for_test("Cargo.toml", source);
    source_file.deserialize_as_toml().unwrap()
}

#[track_caller]
pub(crate) fn parse_manifest_err(source: &str, prefix: &str, spanned: &str) {
    let source_file = SourceFile::new_for_test("Cargo.toml", source);
    let err = source_file.deserialize_as_toml::<Manifest>().unwrap_err();
    assert!(
        err.message.starts_with(prefix),
        "message: {:?}",
        err.message
    );
    let label = err.label.unwrap();
    let span = label.offset()..label.offset() + label.len();
    assert_eq!(source.get(span).unwrap(), spanned);
}

#[track_caller]
pub(crate) fn parse_badge(source: &str) -> Badge {
    parse_manifest(source)
        .package
        .unwrap()
        .metadata
        .unwrap()
        .cargo_sync_rdme
        .unwrap()
        .badge
}

#[track_caller]
pub(crate) fn parse_rustdoc(source: &str) -> Rustdoc {
    parse_manifest(source)
        .package
        .unwrap()
        .metadata
        .unwrap()
        .cargo_sync_rdme
        .unwrap()
        .rustdoc
}
