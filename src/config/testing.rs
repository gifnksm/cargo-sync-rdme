use indoc::formatdoc;

use crate::{
    config::{badge::Badge, rustdoc::Rustdoc},
    manifest::Manifest,
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

#[track_caller]
fn manifest(source: &str) -> Manifest {
    let source = SourceFile::new_for_test("Cargo.toml", source);
    Manifest::new_for_test(&source).unwrap()
}

#[track_caller]
pub(crate) fn parse_config_err(source: &str, prefix: &str, spanned: &str) {
    let source = SourceFile::new_for_test("Cargo.toml", source);
    let (message, label, source_code) = Manifest::new_for_test(&source)
        .unwrap_err()
        .into_toml()
        .into_parse_toml();
    assert!(message.starts_with(prefix), "message: {message:?}");
    let label = label.unwrap();
    let span = label.offset()..label.offset() + label.len();
    assert_eq!(source.text().get(span).unwrap(), spanned);
    assert_eq!(source_code.name(), "Cargo.toml");
}

#[track_caller]
pub(crate) fn deserialize_config_err(source: &str, prefix: &str, spanned: &str) {
    let (message, label, source_code) = manifest(source)
        .package_config()
        .unwrap_err()
        .into_toml()
        .into_deserialize_toml();
    assert!(message.starts_with(prefix), "message: {message:?}");
    let label = label.unwrap();
    let span = label.offset()..label.offset() + label.len();
    assert_eq!(source.get(span).unwrap(), spanned);
    assert_eq!(source_code.name(), "Cargo.toml");
}

#[track_caller]
pub(crate) fn parse_badge(source: &str) -> Badge {
    manifest(source).package_config().unwrap().unwrap().badge
}

#[track_caller]
pub(crate) fn parse_rustdoc(source: &str) -> Rustdoc {
    manifest(source).package_config().unwrap().unwrap().rustdoc
}
