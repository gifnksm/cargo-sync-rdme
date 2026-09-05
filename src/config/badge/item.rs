use std::{debug_assert_matches, fmt, str::FromStr};

use indexmap::IndexMap;
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeSeed, Error as _, Visitor},
};
use void::Void;

use crate::config::{ApplyLayer, Inheritable, badge::BadgeMap, de};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BadgeItem {
    Maintenance,
    License(License),
    CratesIo,
    DocsRs,
    RustVersion,
    GithubActions(GithubActions),
    Codecov(Codecov),
}

impl ApplyLayer for BadgeItem {
    fn apply_layer(&mut self, layer: &Self) {
        match (&mut *self, layer) {
            (Self::License(target), Self::License(layer)) => target.apply_layer(layer),
            (Self::GithubActions(target), Self::GithubActions(layer)) => target.apply_layer(layer),
            (Self::Codecov(target), Self::Codecov(layer)) => target.apply_layer(layer),
            (
                target @ (Self::Maintenance
                | Self::License(_)
                | Self::CratesIo
                | Self::DocsRs
                | Self::RustVersion
                | Self::GithubActions(_)
                | Self::Codecov(_)),
                _,
            ) => {
                debug_assert_matches!(
                    (&target, layer),
                    (Self::Maintenance, Self::Maintenance)
                        | (Self::License(_), Self::License(_))
                        | (Self::CratesIo, Self::CratesIo)
                        | (Self::DocsRs, Self::DocsRs)
                        | (Self::RustVersion, Self::RustVersion)
                        | (Self::GithubActions(_), Self::GithubActions(_))
                        | (Self::Codecov(_), Self::Codecov(_))
                );
                target.clone_from(layer);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum BadgeItemKey {
    Maintenance(Option<String>),
    License(Option<String>),
    CratesIo(Option<String>),
    DocsRs(Option<String>),
    RustVersion(Option<String>),
    GithubActions(Option<String>),
    Codecov(Option<String>),
}

impl BadgeItemKey {
    fn expected() -> &'static [&'static str] {
        &[
            "maintenance",
            "license",
            "crates-io",
            "docs-rs",
            "rust-version",
            "github-actions",
            "codecov",
            "maintenance-*",
            "license-*",
            "crates-io-*",
            "docs-rs-*",
            "rust-version-*",
            "github-actions-*",
            "codecov-*",
        ]
    }
}

pub(super) fn deserialize_badge_map<'de, D>(deserializer: D) -> Result<BadgeMap, D::Error>
where
    D: Deserializer<'de>,
{
    struct BadgeList;

    impl<'de> Visitor<'de> for BadgeList {
        type Value = BadgeMap;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("map")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let mut data = IndexMap::new();
            while let Some(kind) = map.next_key::<BadgeItemKey>()? {
                let (key, item) = map.next_value_seed(kind)?;
                data.insert(key, item);
            }
            Ok(data)
        }
    }

    deserializer.deserialize_any(BadgeList)
}

impl<'de> Deserialize<'de> for BadgeItemKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        let kind = match s {
            "maintenance" => Self::Maintenance(None),
            "license" => Self::License(None),
            "crates-io" => Self::CratesIo(None),
            "docs-rs" => Self::DocsRs(None),
            "rust-version" => Self::RustVersion(None),
            "github-actions" => Self::GithubActions(None),
            "codecov" => Self::Codecov(None),
            _ => {
                if let Some(suffix) = s.strip_prefix("maintenance-") {
                    Self::Maintenance(Some(suffix.to_owned()))
                } else if let Some(suffix) = s.strip_prefix("license-") {
                    Self::License(Some(suffix.to_owned()))
                } else if let Some(suffix) = s.strip_prefix("crates-io-") {
                    Self::CratesIo(Some(suffix.to_owned()))
                } else if let Some(suffix) = s.strip_prefix("docs-rs-") {
                    Self::DocsRs(Some(suffix.to_owned()))
                } else if let Some(suffix) = s.strip_prefix("rust-version-") {
                    Self::RustVersion(Some(suffix.to_owned()))
                } else if let Some(suffix) = s.strip_prefix("github-actions-") {
                    Self::GithubActions(Some(suffix.to_owned()))
                } else if let Some(suffix) = s.strip_prefix("codecov-") {
                    Self::Codecov(Some(suffix.to_owned()))
                } else {
                    return Err(D::Error::unknown_field(s, BadgeItemKey::expected()));
                }
            }
        };
        Ok(kind)
    }
}

impl<'de> DeserializeSeed<'de> for BadgeItemKey {
    type Value = (BadgeItemKey, Inheritable<BadgeItem>);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        fn bool_or_map<'de, D, T, U, F>(deserializer: D, f: F) -> Result<Inheritable<U>, D::Error>
        where
            T: Default + Deserialize<'de>,
            F: FnOnce(T) -> U,
            D: Deserializer<'de>,
        {
            Ok(de::bool_or_map(deserializer)?.map(f))
        }

        fn bool_to_inheritable<'de, D, T, F>(
            deserializer: D,
            f: F,
        ) -> Result<Inheritable<T>, D::Error>
        where
            D: Deserializer<'de>,
            F: FnOnce() -> T,
        {
            if bool::deserialize(deserializer)? {
                Ok(Inheritable::Value(f()))
            } else {
                Ok(Inheritable::Disabled)
            }
        }

        let item = match self {
            Self::Maintenance(_) => bool_to_inheritable(deserializer, || BadgeItem::Maintenance)?,
            Self::License(_) => bool_or_map(deserializer, BadgeItem::License)?,
            Self::CratesIo(_) => bool_to_inheritable(deserializer, || BadgeItem::CratesIo)?,
            Self::DocsRs(_) => bool_to_inheritable(deserializer, || BadgeItem::DocsRs)?,
            Self::RustVersion(_) => bool_to_inheritable(deserializer, || BadgeItem::RustVersion)?,
            Self::GithubActions(_) => bool_or_map(deserializer, BadgeItem::GithubActions)?,
            Self::Codecov(_) => bool_or_map(deserializer, BadgeItem::Codecov)?,
        };
        Ok((self, item))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct License {
    #[serde(default)]
    pub(crate) link: Option<String>,
}

impl ApplyLayer for License {
    fn apply_layer(&mut self, layer: &Self) {
        let Self { link } = self;
        link.apply_layer(&layer.link);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct GithubActions {
    #[serde(default, deserialize_with = "de::string_or_map_or_seq")]
    pub(crate) workflows: Vec<GithubActionsWorkflow>,
}

impl ApplyLayer for GithubActions {
    fn apply_layer(&mut self, layer: &Self) {
        let Self { workflows } = self;
        workflows.apply_layer(&layer.workflows);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct GithubActionsWorkflow {
    #[serde(default)]
    pub(crate) name: Option<String>,
    pub(crate) file: String,
}

impl ApplyLayer for GithubActionsWorkflow {
    fn apply_layer(&mut self, layer: &Self) {
        let Self { name, file } = self;
        name.apply_layer(&layer.name);
        file.apply_layer(&layer.file);
    }
}

impl FromStr for GithubActionsWorkflow {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: None,
            file: s.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Codecov {
    #[serde(default)]
    pub(crate) flag: Option<String>,
    #[serde(default)]
    pub(crate) component: Option<String>,
}

impl ApplyLayer for Codecov {
    fn apply_layer(&mut self, layer: &Self) {
        let Self { flag, component } = self;
        flag.apply_layer(&layer.flag);
        component.apply_layer(&layer.component);
    }
}

#[cfg(test)]
mod tests {
    use indoc::{formatdoc, indoc};
    use similar_asserts::assert_eq;

    use crate::config::testing;

    use super::*;

    fn license(link: Option<&str>) -> License {
        License {
            link: link.map(ToOwned::to_owned),
        }
    }

    fn github_actions<S>(workflows: S) -> GithubActions
    where
        S: Into<Vec<GithubActionsWorkflow>>,
    {
        GithubActions {
            workflows: workflows.into(),
        }
    }

    fn workflow(name: Option<&str>, file: &str) -> GithubActionsWorkflow {
        GithubActionsWorkflow {
            name: name.map(ToOwned::to_owned),
            file: file.to_owned(),
        }
    }

    fn codecov(flag: Option<&str>, component: Option<&str>) -> Codecov {
        Codecov {
            flag: flag.map(ToOwned::to_owned),
            component: component.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn license_apply_layer_updates_specified_link() {
        let mut target = license(Some("from target"));
        let layer = license(Some("from layer"));

        target.apply_layer(&layer);

        assert_eq!(target, license(Some("from layer")));

        let mut target = license(Some("from target"));
        let layer = license(None);

        target.apply_layer(&layer);

        assert_eq!(target, license(Some("from target")));
    }

    #[test]
    fn github_actions_apply_layer_appends_workflows_in_order() {
        let mut target = github_actions([
            workflow(Some("target 1"), "target-1.yaml"),
            workflow(None, "target-2.yaml"),
        ]);
        let layer = github_actions([
            workflow(None, "layer-1.yaml"),
            workflow(Some("layer 2"), "layer-2.yaml"),
        ]);

        target.apply_layer(&layer);

        assert_eq!(
            target,
            github_actions([
                workflow(Some("target 1"), "target-1.yaml"),
                workflow(None, "target-2.yaml"),
                workflow(None, "layer-1.yaml"),
                workflow(Some("layer 2"), "layer-2.yaml"),
            ]),
        );
    }

    #[test]
    fn github_actions_workflow_apply_layer_replaces_file_and_name() {
        let mut target = workflow(Some("from target"), "target.yaml");
        let layer = workflow(Some("from layer"), "layer.yaml");

        target.apply_layer(&layer);

        assert_eq!(target, workflow(Some("from layer"), "layer.yaml"));
    }

    #[test]
    fn codecov_apply_layer_updates_only_specified_fields() {
        let mut target = codecov(Some("from target flag"), Some("from target component"));
        let layer = codecov(None, Some("from layer component"));

        target.apply_layer(&layer);

        assert_eq!(
            target,
            codecov(Some("from target flag"), Some("from layer component")),
        );
    }

    #[test]
    fn badge_item_apply_layer_merges_same_variant_values() {
        let mut target = BadgeItem::GithubActions(github_actions([workflow(None, "target.yaml")]));
        let layer = BadgeItem::GithubActions(github_actions([workflow(None, "layer.yaml")]));

        target.apply_layer(&layer);

        assert_eq!(
            target,
            BadgeItem::GithubActions(github_actions([
                workflow(None, "target.yaml"),
                workflow(None, "layer.yaml")
            ])),
        );
    }

    #[test]
    fn deserialize_badge_map_preserves_badges_order() {
        let source = testing::badge_manifest(indoc! {"
            badges = {
              license = true,
              maintenance = true,
              github-actions = false,
              crates-io = true,
              codecov = true,
              docs-rs = false,
              rust-version = true,
            }
        "});
        let badge = testing::parse_badge(&source);
        testing::assert_indexmap_eq(
            &badge.default.unwrap(),
            [
                (
                    BadgeItemKey::License(None),
                    Inheritable::Value(BadgeItem::License(License::default())),
                ),
                (
                    BadgeItemKey::Maintenance(None),
                    Inheritable::Value(BadgeItem::Maintenance),
                ),
                (BadgeItemKey::GithubActions(None), Inheritable::Disabled),
                (
                    BadgeItemKey::CratesIo(None),
                    Inheritable::Value(BadgeItem::CratesIo),
                ),
                (
                    BadgeItemKey::Codecov(None),
                    Inheritable::Value(BadgeItem::Codecov(Codecov::default())),
                ),
                (BadgeItemKey::DocsRs(None), Inheritable::Disabled),
                (
                    BadgeItemKey::RustVersion(None),
                    Inheritable::Value(BadgeItem::RustVersion),
                ),
            ],
        );
    }

    #[test]
    fn deserialize_badge_map_preserves_multiple_badges_with_same_kind() {
        let source = testing::badge_manifest(indoc! {"
            badges = {
              license = true,
              license-x = true,
              maintenance = true,
              license-z = true,
            }
        "});
        let badge = testing::parse_badge(&source);
        testing::assert_indexmap_eq(
            &badge.default.unwrap(),
            [
                (
                    BadgeItemKey::License(None),
                    Inheritable::Value(BadgeItem::License(License::default())),
                ),
                (
                    BadgeItemKey::License(Some("x".to_owned())),
                    Inheritable::Value(BadgeItem::License(License::default())),
                ),
                (
                    BadgeItemKey::Maintenance(None),
                    Inheritable::Value(BadgeItem::Maintenance),
                ),
                (
                    BadgeItemKey::License(Some("z".to_owned())),
                    Inheritable::Value(BadgeItem::License(License::default())),
                ),
            ],
        );
    }

    #[test]
    fn deserialize_badge_item_key_rejects_unknown_field() {
        let source = testing::badge_manifest(indoc! {r"
            badges = {
              unknown = true,
            }
        "});
        testing::deserialize_config_err(&source, "unknown field `unknown`", "unknown");
    }

    #[test]
    fn deserialize_badge_item_parses_bool_values() {
        let fields: [(_, fn(_) -> _, _); _] = [
            (
                "maintenance",
                BadgeItemKey::Maintenance,
                BadgeItem::Maintenance,
            ),
            (
                "license",
                BadgeItemKey::License,
                BadgeItem::License(License::default()),
            ),
            ("crates-io", BadgeItemKey::CratesIo, BadgeItem::CratesIo),
            ("docs-rs", BadgeItemKey::DocsRs, BadgeItem::DocsRs),
            (
                "rust-version",
                BadgeItemKey::RustVersion,
                BadgeItem::RustVersion,
            ),
            (
                "github-actions",
                BadgeItemKey::GithubActions,
                BadgeItem::GithubActions(GithubActions::default()),
            ),
            (
                "codecov",
                BadgeItemKey::Codecov,
                BadgeItem::Codecov(Codecov::default()),
            ),
        ];
        for (field, key, item) in fields {
            let source = testing::badge_manifest(&formatdoc! {r"
                badges = {{
                  {field} = true,
                  {field}-x = true,
                }}
            "});
            let badge = testing::parse_badge(&source);
            testing::assert_indexmap_eq(
                &badge.default.unwrap(),
                [
                    (key(None), Inheritable::Value(item.clone())),
                    (key(Some("x".to_owned())), Inheritable::Value(item.clone())),
                ],
            );

            let source = testing::badge_manifest(&formatdoc! {r"
                badges = {{
                  {field} = false,
                  {field}-x = false,
                }}
            "});
            let badge = testing::parse_badge(&source);
            testing::assert_indexmap_eq(
                &badge.default.unwrap(),
                [
                    (key(None), Inheritable::Disabled),
                    (key(Some("x".to_owned())), Inheritable::Disabled),
                ],
            );
        }
    }

    #[test]
    fn deserialize_badge_item_rejects_invalid_type() {
        let fields = [
            "maintenance",
            "license",
            "crates-io",
            "docs-rs",
            "rust-version",
            "github-actions",
            "codecov",
        ];
        for field in fields {
            let source = testing::badge_manifest(&formatdoc! {r"
                badges = {{
                  {field} = 34,
                }}
            "});
            testing::deserialize_config_err(&source, "invalid type: integer `34`", "34");
        }
    }

    #[test]
    fn deserialize_badge_item_parses_valid_license_maps() {
        let source = testing::badge_manifest(indoc! {r#"
            badges = {
              license = {},
              license-x = { link = "https://example.com" },
              license-y = { link = "https://example.com/x" },
            }
        "#});
        let badge = testing::parse_badge(&source);
        testing::assert_indexmap_eq(
            &badge.default.unwrap(),
            [
                (
                    BadgeItemKey::License(None),
                    Inheritable::Value(BadgeItem::License(License { link: None })),
                ),
                (
                    BadgeItemKey::License(Some("x".to_owned())),
                    Inheritable::Value(BadgeItem::License(License {
                        link: Some("https://example.com".to_string()),
                    })),
                ),
                (
                    BadgeItemKey::License(Some("y".to_owned())),
                    Inheritable::Value(BadgeItem::License(License {
                        link: Some("https://example.com/x".to_string()),
                    })),
                ),
            ],
        );
    }

    #[test]
    fn deserialize_badge_item_rejects_invalid_license_maps() {
        let source = testing::badge_manifest(indoc! {r"
            badges = {
              license = { link = 34 },
            }
        "});
        testing::deserialize_config_err(&source, "invalid type: integer `34`", "34");

        let source = testing::badge_manifest(indoc! {r"
            badges = {
              license = { unknown = true },
            }
        "});
        testing::deserialize_config_err(&source, "unknown field `unknown`", "unknown");
    }

    #[test]
    fn deserialize_badge_item_parses_valid_github_actions_maps() {
        let source = testing::badge_manifest(indoc! {r#"
            badges = {
              github-actions = {},
              github-actions-x = { workflows = "x.yaml" },
              github-actions-y = { workflows = { file = "y.yaml" } },
              github-actions-xyz = { workflows = [ "x.yaml", { file = "y.yaml" } ] },
            }
        "#});
        let badge = testing::parse_badge(&source);
        testing::assert_indexmap_eq(
            &badge.default.unwrap(),
            [
                (
                    BadgeItemKey::GithubActions(None),
                    Inheritable::Value(BadgeItem::GithubActions(github_actions([]))),
                ),
                (
                    BadgeItemKey::GithubActions(Some("x".to_owned())),
                    Inheritable::Value(BadgeItem::GithubActions(github_actions([workflow(
                        None, "x.yaml",
                    )]))),
                ),
                (
                    BadgeItemKey::GithubActions(Some("y".to_owned())),
                    Inheritable::Value(BadgeItem::GithubActions(github_actions([workflow(
                        None, "y.yaml",
                    )]))),
                ),
                (
                    BadgeItemKey::GithubActions(Some("xyz".to_owned())),
                    Inheritable::Value(BadgeItem::GithubActions(github_actions([
                        workflow(None, "x.yaml"),
                        workflow(None, "y.yaml"),
                    ]))),
                ),
            ],
        );
    }

    #[test]
    fn deserialize_badge_item_rejects_invalid_github_actions_maps() {
        let source = testing::badge_manifest(indoc! {r"
            badges = {
              github-actions = { workflows = 34 },
            }
        "});
        testing::deserialize_config_err(&source, "invalid type: integer `34`", "34");

        let source = testing::badge_manifest(indoc! {r"
            badges = {
              github-actions = { workflows = { unknown = true } },
            }
        "});
        testing::deserialize_config_err(&source, "unknown field `unknown`", "unknown");
    }

    #[test]
    fn deserialize_badge_item_parses_valid_codecov_maps() {
        let source = testing::badge_manifest(indoc! {r#"
            badges = {
              codecov = {},
              codecov-x = { component = "core" },
            }
        "#});
        let badge = testing::parse_badge(&source);
        testing::assert_indexmap_eq(
            &badge.default.unwrap(),
            [
                (
                    BadgeItemKey::Codecov(None),
                    Inheritable::Value(BadgeItem::Codecov(Codecov::default())),
                ),
                (
                    BadgeItemKey::Codecov(Some("x".to_owned())),
                    Inheritable::Value(BadgeItem::Codecov(codecov(None, Some("core")))),
                ),
            ],
        );
    }

    #[test]
    fn deserialize_badge_item_rejects_invalid_codecov_maps() {
        let source = testing::badge_manifest(indoc! {r"
            badges = {
              codecov = { component = 34 },
            }
        "});
        testing::deserialize_config_err(&source, "invalid type: integer `34`", "34");

        let source = testing::badge_manifest(indoc! {r"
            badges = {
              codecov = { unknown = true },
            }
        "});
        testing::deserialize_config_err(&source, "unknown field `unknown`", "unknown");
    }
}
