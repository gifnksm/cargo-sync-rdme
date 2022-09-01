use indoc::indoc;

use super::*;

#[test]
fn test_license() {
    fn get_license(manifest: Manifest) -> Option<metadata::License> {
        manifest
            .package
            .unwrap()
            .into_inner()
            .metadata
            .unwrap()
            .into_inner()
            .cargo_sync_rdme
            .badges
            .license
    }

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            license = true
        "#};
    let license = get_license(toml::from_str(input).unwrap()).unwrap();
    assert!(license.link.is_none());

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            license = false
        "#};
    let license = get_license(toml::from_str(input).unwrap());
    assert!(license.is_none());

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            license = {}
        "#};
    let license = get_license(toml::from_str(input).unwrap()).unwrap();
    assert!(license.link.is_none());

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            license = { link = "foo" }
        "#};
    let license = get_license(toml::from_str(input).unwrap()).unwrap();
    assert_eq!(license.link.unwrap(), "foo");
}

#[test]
fn test_github_actions() {
    fn get_github_actions(manifest: Manifest) -> Option<metadata::GithubActions> {
        manifest
            .package
            .unwrap()
            .into_inner()
            .metadata
            .unwrap()
            .into_inner()
            .cargo_sync_rdme
            .badges
            .github_actions
    }

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = true
        "#};
    let gh = get_github_actions(toml::from_str(input).unwrap()).unwrap();
    assert!(gh.workflows.is_empty());

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = false
        "#};
    let gh = get_github_actions(toml::from_str(input).unwrap());
    assert!(gh.is_none());

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = {}
        "#};
    let gh = get_github_actions(toml::from_str(input).unwrap()).unwrap();
    assert!(gh.workflows.is_empty());

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = { workflows = "foo.yml" }
        "#};
    let gh = get_github_actions(toml::from_str(input).unwrap()).unwrap();
    assert_eq!(gh.workflows.len(), 1);
    assert_eq!(gh.workflows[0].file, "foo.yml");

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = { workflows = { file = "foo.yml" } }
        "#};
    let gh = get_github_actions(toml::from_str(input).unwrap()).unwrap();
    assert_eq!(gh.workflows.len(), 1);
    assert_eq!(gh.workflows[0].file, "foo.yml");

    let input = indoc! {r#"
            [package.metadata.cargo-sync-rdme.badges]
            github-actions = { workflows = [ "foo.yml", {file = "bar.yml"} ] }
        "#};
    let gh = get_github_actions(toml::from_str(input).unwrap()).unwrap();
    assert_eq!(gh.workflows.len(), 2);
    assert_eq!(gh.workflows[0].file, "foo.yml");
    assert_eq!(gh.workflows[1].file, "bar.yml");
}
