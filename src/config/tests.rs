use std::sync::Arc;

use indoc::indoc;

use crate::config::metadata::{BadgeItem, GithubActions, GithubActionsWorkflow, License};

use super::*;

fn get_badges(manifest: Manifest) -> Arc<[BadgeItem]> {
    let badges = &manifest
        .package
        .unwrap()
        .into_inner()
        .metadata
        .unwrap()
        .into_inner()
        .cargo_sync_rdme
        .badge
        .badges[""];
    Arc::clone(badges)
}

#[test]
fn test_badges_order() {
    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            license = true
            maintenance = true
            github-actions = false
            crates-io = true
            codecov = true
            docs-rs = false
            rust-version = true
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        *badges,
        [
            BadgeItem::License(_),
            BadgeItem::Maintenance,
            BadgeItem::CratesIo,
            BadgeItem::Codecov,
            BadgeItem::RustVersion
        ]
    ));
}

#[test]
fn test_duplicated_badges() {
    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            license = true
            license-x = true
            maintenance = true
            license-z = true
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        *badges,
        [
            BadgeItem::License(_),
            BadgeItem::License(_),
            BadgeItem::Maintenance,
            BadgeItem::License(_),
        ]
    ));
}

#[test]
fn test_license() {
    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            license = true
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        &*badges,
        [BadgeItem::License(License { link: None })]
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            license = false
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(&*badges, []));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            license = {}
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        &*badges,
        [BadgeItem::License(License { link: None })]
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            license = { link = "foo" }
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        &*badges,
        [BadgeItem::License(License { link: Some(link) })] if link == "foo"
    ));
}

#[test]
fn test_github_actions() {
    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            github-actions = true
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })] if matches!(workflows.as_slice(), &[])
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            github-actions = false
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(*badges, []));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            github-actions = {}
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })] if matches!(workflows.as_slice(), &[])
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            github-actions = { workflows = "foo.yml" }
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })]
        if matches!(
            workflows.as_slice(),
            [
                GithubActionsWorkflow { name: None, file }
            ] if file == "foo.yml"
        )
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            github-actions = { workflows = { file = "foo.yml" } }
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })]
        if matches!(
            workflows.as_slice(),
            [
                GithubActionsWorkflow { name: None, file }
            ] if file == "foo.yml"
        )
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badge.badges]
            github-actions = { workflows = [ "foo.yml", {file = "bar.yml"} ] }
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })]
        if matches!(
            &workflows.as_slice(), &[
                GithubActionsWorkflow { name: None, file: file1 },
                GithubActionsWorkflow { name: None, file: file2 }
            ] if file1 == "foo.yml" && file2 == "bar.yml")
    ));
}
