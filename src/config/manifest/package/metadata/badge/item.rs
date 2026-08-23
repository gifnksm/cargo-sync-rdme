use std::{fmt, str::FromStr, sync::Arc};

use serde::{
    Deserialize,
    de::{DeserializeSeed, Error as _, Visitor},
};
use void::Void;

use crate::config::de;

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

#[derive(Debug, Clone)]
enum BadgeItemKind {
    Maintenance,
    License,
    CratesIo,
    DocsRs,
    RustVersion,
    GithubActions,
    Codecov,
}

impl BadgeItemKind {
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

pub(super) fn deserialize_badge_list<'de, D>(deserializer: D) -> Result<Arc<[BadgeItem]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct BadgeList;

    impl<'de> Visitor<'de> for BadgeList {
        type Value = Arc<[BadgeItem]>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("map")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let mut data = vec![];
            while let Some(kind) = map.next_key::<BadgeItemKind>()? {
                if let Some(item) = map.next_value_seed(kind)? {
                    data.push(item);
                }
            }
            Ok(data.into())
        }
    }

    deserializer.deserialize_any(BadgeList)
}

impl<'de> Deserialize<'de> for BadgeItemKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        let kind = match s {
            "maintenance" => Self::Maintenance,
            "license" => Self::License,
            "crates-io" => Self::CratesIo,
            "docs-rs" => Self::DocsRs,
            "rust-version" => Self::RustVersion,
            "github-actions" => Self::GithubActions,
            "codecov" => Self::Codecov,
            _ => {
                if s.starts_with("maintenance-") {
                    Self::Maintenance
                } else if s.starts_with("license-") {
                    Self::License
                } else if s.starts_with("crates-io-") {
                    Self::CratesIo
                } else if s.starts_with("docs-rs-") {
                    Self::DocsRs
                } else if s.starts_with("rust-version-") {
                    Self::RustVersion
                } else if s.starts_with("github-actions-") {
                    Self::GithubActions
                } else if s.starts_with("codecov-") {
                    Self::Codecov
                } else {
                    return Err(D::Error::unknown_field(s, BadgeItemKind::expected()));
                }
            }
        };
        Ok(kind)
    }
}

impl<'de> DeserializeSeed<'de> for BadgeItemKind {
    type Value = Option<BadgeItem>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(bound = "T: Default + Deserialize<'de>")]
        struct BoolOrMap<T>(#[serde(deserialize_with = "de::bool_or_map")] Option<T>);

        let item = match self {
            BadgeItemKind::Maintenance => {
                bool::deserialize(deserializer)?.then_some(BadgeItem::Maintenance)
            }
            BadgeItemKind::License => <BoolOrMap<License>>::deserialize(deserializer)?
                .0
                .map(BadgeItem::License),
            BadgeItemKind::CratesIo => {
                bool::deserialize(deserializer)?.then_some(BadgeItem::CratesIo)
            }
            BadgeItemKind::DocsRs => bool::deserialize(deserializer)?.then_some(BadgeItem::DocsRs),
            BadgeItemKind::RustVersion => {
                bool::deserialize(deserializer)?.then_some(BadgeItem::RustVersion)
            }
            BadgeItemKind::GithubActions => <BoolOrMap<GithubActions>>::deserialize(deserializer)?
                .0
                .map(BadgeItem::GithubActions),
            BadgeItemKind::Codecov => <BoolOrMap<Codecov>>::deserialize(deserializer)?
                .0
                .map(BadgeItem::Codecov),
        };
        Ok(item)
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
    fn deserialize_badge_list_preserves_badges_order() {
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
            badge.default.as_deref().unwrap(),
            [
                BadgeItem::License(License::default()),
                BadgeItem::Maintenance,
                BadgeItem::CratesIo,
                BadgeItem::Codecov(Codecov::default()),
                BadgeItem::RustVersion
            ]
        );
    }

    #[test]
    fn deserialize_badge_list_preserves_multiple_badges_with_same_kind() {
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
            badge.default.as_deref().unwrap(),
            [
                BadgeItem::License(License::default()),
                BadgeItem::License(License::default()),
                BadgeItem::Maintenance,
                BadgeItem::License(License::default()),
            ]
        );
    }

    #[test]
    fn deserialize_badge_item_kind_rejects_unknown_field() {
        let source = testing::badge_manifest(indoc! {r"
            badges = {
              unknown = true,
            }
        "});
        testing::parse_err(&source, "unknown field `unknown`", "unknown");
    }

    #[test]
    fn deserialize_badge_item_parses_bool_values() {
        let fields = [
            ("maintenance", BadgeItem::Maintenance),
            ("license", BadgeItem::License(License::default())),
            ("crates-io", BadgeItem::CratesIo),
            ("docs-rs", BadgeItem::DocsRs),
            ("rust-version", BadgeItem::RustVersion),
            (
                "github-actions",
                BadgeItem::GithubActions(GithubActions::default()),
            ),
            ("codecov", BadgeItem::Codecov(Codecov::default())),
        ];
        for (field, item) in fields {
            let source = testing::badge_manifest(&formatdoc! {r"
                badges = {{
                  {field} = true,
                  {field}-x = true,
                }}
            "});
            let badge = testing::parse_badge(&source);
            assert_eq!(
                badge.default.as_deref().unwrap(),
                [item.clone(), item.clone()]
            );

            let source = testing::badge_manifest(&formatdoc! {r"
                badges = {{
                  {field} = false,
                  {field}-x = false,
                }}
            "});
            let badge = testing::parse_badge(&source);
            assert_eq!(badge.default.as_deref().unwrap(), []);
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
            testing::parse_err(&source, "invalid type: integer `34`", "34");
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
            badge.default.as_deref().unwrap(),
            [
                BadgeItem::License(License { link: None }),
                BadgeItem::License(License {
                    link: Some("https://example.com".to_string()),
                }),
                BadgeItem::License(License {
                    link: Some("https://example.com/x".to_string()),
                })
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
        testing::parse_err(&source, "invalid type: integer `34`", "34");

        let source = testing::badge_manifest(indoc! {r"
            badges = {
              license = { unknown = true },
            }
        "});
        testing::parse_err(&source, "unknown field `unknown`", "unknown");
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
            badge.default.as_deref().unwrap(),
            [
                BadgeItem::GithubActions(GithubActions { workflows: vec![] }),
                BadgeItem::GithubActions(GithubActions {
                    workflows: vec![GithubActionsWorkflow {
                        name: None,
                        file: "x.yaml".into()
                    }]
                }),
                BadgeItem::GithubActions(GithubActions {
                    workflows: vec![GithubActionsWorkflow {
                        name: None,
                        file: "y.yaml".into()
                    }]
                }),
                BadgeItem::GithubActions(GithubActions {
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
                }),
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
        testing::parse_err(&source, "invalid type: integer `34`", "34");

        let source = testing::badge_manifest(indoc! {r"
            badges = {
              github-actions = { workflows = { unknown = true } },
            }
        "});
        testing::parse_err(&source, "unknown field `unknown`", "unknown");
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
            badge.default.as_deref().unwrap(),
            [
                BadgeItem::Codecov(Codecov::default()),
                BadgeItem::Codecov(Codecov {
                    component: Some("core".into()),
                    flag: None
                }),
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
        testing::parse_err(&source, "invalid type: integer `34`", "34");

        let source = testing::badge_manifest(indoc! {r"
            badges = {
              codecov = { unknown = true },
            }
        "});
        testing::parse_err(&source, "unknown field `unknown`", "unknown");
    }
}
