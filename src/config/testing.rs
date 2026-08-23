use indoc::formatdoc;

use crate::config::manifest::{
    Manifest,
    package::metadata::{badge::Badge, rustdoc::Rustdoc},
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
pub(crate) fn parse_badge(source: &str) -> Badge {
    let manifest = toml::from_str::<Manifest>(source).unwrap();
    manifest
        .package
        .unwrap()
        .into_inner()
        .metadata
        .unwrap()
        .into_inner()
        .cargo_sync_rdme
        .badge
}

#[track_caller]
pub(crate) fn parse_rustdoc(source: &str) -> Rustdoc {
    let manifest = toml::from_str::<Manifest>(source).unwrap();
    manifest
        .package
        .unwrap()
        .into_inner()
        .metadata
        .unwrap()
        .into_inner()
        .cargo_sync_rdme
        .rustdoc
}

#[track_caller]
pub(crate) fn parse_err(source: &str, prefix: &str, spanned: &str) {
    let err = toml::from_str::<Manifest>(source).unwrap_err();
    assert!(
        err.message().starts_with(prefix),
        "message: {:?}",
        err.message()
    );
    assert_eq!(source.get(err.span().unwrap()).unwrap(), spanned);
}
