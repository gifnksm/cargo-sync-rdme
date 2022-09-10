use indoc::indoc;

use crate::config::metadata::{Badge, GithubActions, GithubActionsWorkflow, License};

use super::*;

fn get_badges(manifest: Manifest) -> Vec<Badge> {
    manifest
        .package
        .unwrap()
        .into_inner()
        .metadata
        .unwrap()
        .into_inner()
        .cargo_sync_rdme
        .badges
}

#[test]
fn test_badges_order() {
    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
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
        badges.as_slice(),
        [
            Badge::License(_),
            Badge::Maintenance,
            Badge::CratesIo,
            Badge::Codecov,
            Badge::RustVersion
        ]
    ));
}

#[test]
fn test_license() {
    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            license = true
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        badges.as_slice(),
        [Badge::License(License { link: None })]
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            license = false
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(badges.as_slice(), &[]));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            license = {}
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        badges.as_slice(),
        [Badge::License(License { link: None })]
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            license = { link = "foo" }
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        badges.as_slice(),
        [Badge::License(License { link: Some(link) })] if link == "foo"
    ));
}

#[test]
fn test_github_actions() {
    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = true
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        badges.as_slice(),
        [Badge::GithubActions(GithubActions { workflows })] if matches!(workflows.as_slice(), &[])
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = false
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(badges.as_slice(), &[]));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = {}
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        badges.as_slice(),
        [Badge::GithubActions(GithubActions { workflows })] if matches!(workflows.as_slice(), &[])
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = { workflows = "foo.yml" }
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        badges.as_slice(),
        [Badge::GithubActions(GithubActions { workflows })]
        if matches!(
            workflows.as_slice(),
            [
                GithubActionsWorkflow { name: None, file }
            ] if file == "foo.yml"
        )
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = { workflows = { file = "foo.yml" } }
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        badges.as_slice(),
        [Badge::GithubActions(GithubActions { workflows })]
        if matches!(
            workflows.as_slice(),
            [
                GithubActionsWorkflow { name: None, file }
            ] if file == "foo.yml"
        )
    ));

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = { workflows = [ "foo.yml", {file = "bar.yml"} ] }
        "#};
    let badges = get_badges(toml::from_str(input).unwrap());
    assert!(matches!(
        badges.as_slice(),
        [Badge::GithubActions(GithubActions { workflows })]
        if matches!(
            &workflows.as_slice(), &[
                GithubActionsWorkflow { name: None, file: file1 },
                GithubActionsWorkflow { name: None, file: file2 }
            ] if file1 == "foo.yml" && file2 == "bar.yml")
    ));
}
