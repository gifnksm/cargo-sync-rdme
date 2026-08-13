use std::assert_matches;

use indoc::{formatdoc, indoc};
use similar_asserts::assert_eq;

use crate::config::metadata::{
    BadgeItem, Codecov, GithubActions, GithubActionsWorkflow, License, Rustdoc,
};

use super::*;

fn badge_manifest(config: &str) -> Manifest {
    toml::from_str(&formatdoc! {r"
        [package.metadata.cargo-sync-rdme.badge]
        {config}
    "})
    .unwrap()
}

fn rustdoc_manifest(config: &str) -> Manifest {
    toml::from_str(&formatdoc! {r"
        [package.metadata.cargo-sync-rdme.rustdoc]
        {config}
    "})
    .unwrap()
}

fn get_badge_group(manifest: Manifest, group: &str) -> Arc<[BadgeItem]> {
    let badges = &manifest
        .package
        .unwrap()
        .into_inner()
        .metadata
        .unwrap()
        .into_inner()
        .cargo_sync_rdme
        .badge
        .badges[group];
    Arc::clone(badges)
}

fn get_badges(manifest: Manifest) -> Arc<[BadgeItem]> {
    get_badge_group(manifest, "")
}

fn get_rustdoc(manifest: Manifest) -> Rustdoc {
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

#[test]
fn test_badges_order() {
    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          license = true,
          maintenance = true,
          github-actions = false,
          crates-io = true,
          codecov = true,
          docs-rs = false,
          rust-version = true,
        }
    "}));
    assert_matches!(
        *badges,
        [
            BadgeItem::License(_),
            BadgeItem::Maintenance,
            BadgeItem::CratesIo,
            BadgeItem::Codecov(_),
            BadgeItem::RustVersion
        ]
    );
}

#[test]
fn test_duplicated_badges() {
    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          license = true,
          license-x = true,
          maintenance = true,
          license-z = true,
        }
    "}));
    assert_matches!(
        *badges,
        [
            BadgeItem::License(_),
            BadgeItem::License(_),
            BadgeItem::Maintenance,
            BadgeItem::License(_),
        ]
    );
}

#[test]
fn test_badge_groups() {
    let badges = get_badge_group(
        badge_manifest(indoc! {r"
            badges-foo = {
              license = true,
              maintenance = true,
            }
        "}),
        "foo",
    );
    assert_matches!(*badges, [BadgeItem::License(_), BadgeItem::Maintenance]);
}

#[test]
fn test_old_badge_table_syntax_still_parses() {
    let badges = get_badges(
        toml::from_str(indoc! {r"
            [package.metadata.cargo-sync-rdme.badge.badges]
            license = true
        "})
        .unwrap(),
    );
    assert_matches!(&*badges, [BadgeItem::License(License { link: None })]);
}

#[test]
fn test_license() {
    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          license = true,
        }
    "}));
    assert_matches!(&*badges, [BadgeItem::License(License { link: None })]);

    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          license = false,
        }
    "}));
    assert_matches!(&*badges, []);

    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          license = {},
        }
    "}));
    assert_matches!(&*badges, [BadgeItem::License(License { link: None })]);

    let badges = get_badges(badge_manifest(indoc! {r#"
        badges = {
          license = { link = "foo" },
        }
    "#}));
    assert_matches!(
        &*badges,
        [BadgeItem::License(License { link: Some(link) })] if link == "foo"
    );
}

#[test]
fn test_github_actions() {
    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          github-actions = true,
        }
    "}));
    assert_matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })] if matches!(workflows.as_slice(), &[])
    );

    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          github-actions = false,
        }
    "}));
    assert_matches!(*badges, []);

    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          github-actions = {},
        }
    "}));
    assert_matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })] if matches!(workflows.as_slice(), &[])
    );

    let badges = get_badges(badge_manifest(indoc! {r#"
        badges = {
          github-actions = { workflows = "foo.yml" },
        }
    "#}));
    assert_matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })]
        if matches!(
            workflows.as_slice(),
            [
                GithubActionsWorkflow { name: None, file }
            ] if file == "foo.yml"
        )
    );

    let badges = get_badges(badge_manifest(indoc! {r#"
        badges = {
          github-actions = { workflows = { file = "foo.yml" } },
        }
    "#}));
    assert_matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })]
        if matches!(
            workflows.as_slice(),
            [
                GithubActionsWorkflow { name: None, file }
            ] if file == "foo.yml"
        )
    );

    let badges = get_badges(badge_manifest(indoc! {r#"
        badges = {
          github-actions = { workflows = [ "foo.yml", { file = "bar.yml" } ] },
        }
    "#}));
    assert_matches!(
        &*badges,
        [BadgeItem::GithubActions(GithubActions { workflows })]
        if matches!(
            &workflows.as_slice(), &[
                GithubActionsWorkflow { name: None, file: file1 },
                GithubActionsWorkflow { name: None, file: file2 }
            ] if file1 == "foo.yml" && file2 == "bar.yml")
    );
}

#[test]
fn test_codecov() {
    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          codecov = true,
        }
    "}));
    assert_matches!(
        &*badges,
        [BadgeItem::Codecov(Codecov {
            flag: None,
            component: None,
        })]
    );

    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          codecov = false,
        }
    "}));
    assert_matches!(*badges, []);

    let badges = get_badges(badge_manifest(indoc! {r"
        badges = {
          codecov = {},
        }
    "}));
    assert_matches!(
        &*badges,
        [BadgeItem::Codecov(Codecov {
            flag: None,
            component: None,
        })]
    );

    let badges = get_badges(badge_manifest(indoc! {r#"
        badges = {
          codecov = { component = "core" },
        }
    "#}));
    assert_matches!(
        &*badges,
        [BadgeItem::Codecov(Codecov {
            flag: None,
            component: Some(component),
        })] if component == "core"
    );

    let badges = get_badges(badge_manifest(indoc! {r#"
        badges = {
          codecov = { flag = "unit" },
        }
    "#}));
    assert_matches!(
        &*badges,
        [BadgeItem::Codecov(Codecov {
            flag: Some(flag),
            component: None,
        })] if flag == "unit"
    );

    let badges = get_badges(badge_manifest(indoc! {r#"
        badges = {
          codecov = { component = "core", flag = "unit" },
        }
    "#}));
    assert_matches!(
        &*badges,
        [BadgeItem::Codecov(Codecov {
            flag: Some(flag),
            component: Some(component),
        })] if flag == "unit" && component == "core"
    );
}

#[test]
fn test_rustdoc() {
    let rustdoc = get_rustdoc(rustdoc_manifest(indoc! {r#"
        html-root-url = "https://example.com/docs/"
        mappings = {
          SomeType = "./docs/some-type.md",
          SomeTrait = "./docs/some-trait.md",
        }
    "#}));

    assert_eq!(
        rustdoc.html_root_url.as_deref(),
        Some("https://example.com/docs/")
    );
    assert_eq!(
        rustdoc.mappings.get("SomeType").map(String::as_str),
        Some("./docs/some-type.md")
    );
    assert_eq!(
        rustdoc.mappings.get("SomeTrait").map(String::as_str),
        Some("./docs/some-trait.md")
    );
}
