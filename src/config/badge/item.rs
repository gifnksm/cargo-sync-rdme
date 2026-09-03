use std::{fmt, str::FromStr};

use indexmap::IndexMap;
use serde::{
    Deserialize,
    de::{DeserializeSeed, Error as _, Visitor},
};
use void::Void;

use crate::config::{badge::BadgeMap, de};

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
    D: serde::Deserializer<'de>,
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
        D: serde::Deserializer<'de>,
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
    type Value = (BadgeItemKey, Option<BadgeItem>);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(bound = "T: Default + Deserialize<'de>")]
        struct BoolOrMap<T>(#[serde(deserialize_with = "de::bool_or_map")] Option<T>);

        let item = match self {
            Self::Maintenance(_) => {
                bool::deserialize(deserializer)?.then_some(BadgeItem::Maintenance)
            }
            Self::License(_) => <BoolOrMap<License>>::deserialize(deserializer)?
                .0
                .map(BadgeItem::License),
            Self::CratesIo(_) => bool::deserialize(deserializer)?.then_some(BadgeItem::CratesIo),
            Self::DocsRs(_) => bool::deserialize(deserializer)?.then_some(BadgeItem::DocsRs),
            Self::RustVersion(_) => {
                bool::deserialize(deserializer)?.then_some(BadgeItem::RustVersion)
            }
            Self::GithubActions(_) => <BoolOrMap<GithubActions>>::deserialize(deserializer)?
                .0
                .map(BadgeItem::GithubActions),
            Self::Codecov(_) => <BoolOrMap<Codecov>>::deserialize(deserializer)?
                .0
                .map(BadgeItem::Codecov),
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct GithubActions {
    #[serde(default, deserialize_with = "de::string_or_map_or_seq")]
    pub(crate) workflows: Vec<GithubActionsWorkflow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct GithubActionsWorkflow {
    #[serde(default)]
    pub(crate) name: Option<String>,
    pub(crate) file: String,
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

#[cfg(test)]
mod tests {
    use indoc::{formatdoc, indoc};
    use similar_asserts::assert_eq;

    use crate::config::testing;

    use super::*;

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
        assert_eq!(
            badge.default.unwrap().into_iter().collect::<Vec<_>>(),
            [
                (
                    BadgeItemKey::License(None),
                    Some(BadgeItem::License(License::default()))
                ),
                (
                    BadgeItemKey::Maintenance(None),
                    Some(BadgeItem::Maintenance)
                ),
                (BadgeItemKey::GithubActions(None), None),
                (BadgeItemKey::CratesIo(None), Some(BadgeItem::CratesIo)),
                (
                    BadgeItemKey::Codecov(None),
                    Some(BadgeItem::Codecov(Codecov::default()))
                ),
                (BadgeItemKey::DocsRs(None), None),
                (
                    BadgeItemKey::RustVersion(None),
                    Some(BadgeItem::RustVersion)
                ),
            ]
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
        assert_eq!(
            badge.default.unwrap().into_iter().collect::<Vec<_>>(),
            [
                (
                    BadgeItemKey::License(None),
                    Some(BadgeItem::License(License::default()))
                ),
                (
                    BadgeItemKey::License(Some("x".to_owned())),
                    Some(BadgeItem::License(License::default()))
                ),
                (
                    BadgeItemKey::Maintenance(None),
                    Some(BadgeItem::Maintenance)
                ),
                (
                    BadgeItemKey::License(Some("z".to_owned())),
                    Some(BadgeItem::License(License::default()))
                ),
            ]
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
            assert_eq!(
                badge.default.unwrap().into_iter().collect::<Vec<_>>(),
                [
                    (key(None), Some(item.clone())),
                    (key(Some("x".to_owned())), Some(item.clone())),
                ]
            );

            let source = testing::badge_manifest(&formatdoc! {r"
                badges = {{
                  {field} = false,
                  {field}-x = false,
                }}
            "});
            let badge = testing::parse_badge(&source);
            assert_eq!(
                badge.default.unwrap().into_iter().collect::<Vec<_>>(),
                [(key(None), None), (key(Some("x".to_owned())), None),]
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
        assert_eq!(
            badge.default.unwrap().into_iter().collect::<Vec<_>>(),
            [
                (
                    BadgeItemKey::License(None),
                    Some(BadgeItem::License(License { link: None }))
                ),
                (
                    BadgeItemKey::License(Some("x".to_owned())),
                    Some(BadgeItem::License(License {
                        link: Some("https://example.com".to_string()),
                    }))
                ),
                (
                    BadgeItemKey::License(Some("y".to_owned())),
                    Some(BadgeItem::License(License {
                        link: Some("https://example.com/x".to_string()),
                    }))
                ),
            ]
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
        assert_eq!(
            badge.default.unwrap().into_iter().collect::<Vec<_>>(),
            [
                (
                    BadgeItemKey::GithubActions(None),
                    Some(BadgeItem::GithubActions(GithubActions {
                        workflows: vec![]
                    }))
                ),
                (
                    BadgeItemKey::GithubActions(Some("x".to_owned())),
                    Some(BadgeItem::GithubActions(GithubActions {
                        workflows: vec![GithubActionsWorkflow {
                            name: None,
                            file: "x.yaml".into()
                        }]
                    }))
                ),
                (
                    BadgeItemKey::GithubActions(Some("y".to_owned())),
                    Some(BadgeItem::GithubActions(GithubActions {
                        workflows: vec![GithubActionsWorkflow {
                            name: None,
                            file: "y.yaml".into()
                        }]
                    }))
                ),
                (
                    BadgeItemKey::GithubActions(Some("xyz".to_owned())),
                    Some(BadgeItem::GithubActions(GithubActions {
                        workflows: vec![
                            GithubActionsWorkflow {
                                name: None,
                                file: "x.yaml".into()
                            },
                            GithubActionsWorkflow {
                                name: None,
                                file: "y.yaml".into()
                            },
                        ]
                    }))
                ),
            ]
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
        assert_eq!(
            badge.default.unwrap().into_iter().collect::<Vec<_>>(),
            [
                (
                    BadgeItemKey::Codecov(None),
                    Some(BadgeItem::Codecov(Codecov::default()))
                ),
                (
                    BadgeItemKey::Codecov(Some("x".to_owned())),
                    Some(BadgeItem::Codecov(Codecov {
                        component: Some("core".into()),
                        flag: None
                    }))
                ),
            ]
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
