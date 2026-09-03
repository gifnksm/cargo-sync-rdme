use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use indexmap::IndexMap;
use serde::{
    Deserialize,
    de::{Error as _, Visitor},
};

use crate::{
    config::badge::item::{BadgeItem, BadgeItemKey},
    parse,
};

pub(crate) mod item;

pub(crate) type BadgeMap = IndexMap<BadgeItemKey, Option<BadgeItem>>;

#[derive(Debug, Clone, Default)]
pub(crate) struct Badge {
    pub(crate) style: Option<BadgeStyle>,
    pub(crate) default: Option<BadgeMap>,
    pub(crate) groups: HashMap<String, BadgeMap>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BadgeStyle {
    #[default]
    Plastic,
    Flat,
    FlatSquare,
    ForTheBadge,
    Social,
}

impl BadgeStyle {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Plastic => "plastic",
            Self::Flat => "flat",
            Self::FlatSquare => "flat-square",
            Self::ForTheBadge => "for-the-badge",
            Self::Social => "social",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BadgeFieldKey<'de> {
    Style,
    Badges,
    BadgesGroup(&'de str),
}

impl Display for BadgeFieldKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Style => f.write_str("style"),
            Self::Badges => f.write_str("badges"),
            Self::BadgesGroup(group) => write!(f, "badges-{group}"),
        }
    }
}

impl<'de> Deserialize<'de> for BadgeFieldKey<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let key = <&str>::deserialize(deserializer)?;
        let expected = &["badges", "badges-<group>", "style"];
        if key == "style" {
            return Ok(Self::Style);
        }
        if key == "badges" {
            return Ok(Self::Badges);
        }
        if let Some(rest) = key.strip_prefix("badges-") {
            if !parse::is_valid_ident(rest) {
                return Err(D::Error::custom(format_args!(
                    "invalid field name `{key}`, expected `badges-<group>` where `<group>` matches `[A-Za-z][-_A-Za-z0-9]*`"
                )));
            }
            return Ok(Self::BadgesGroup(rest));
        }
        Err(D::Error::unknown_field(key, expected))
    }
}

impl<'de> Deserialize<'de> for Badge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Badges;
        impl<'de> Visitor<'de> for Badges {
            type Value = Badge;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("map")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                #[derive(Deserialize)]
                struct BadgeList(
                    #[serde(deserialize_with = "item::deserialize_badge_map")] BadgeMap,
                );

                let mut data = Badge::default();

                while let Some(key) = map.next_key()? {
                    match key {
                        BadgeFieldKey::Style => {
                            data.style = Some(map.next_value::<BadgeStyle>()?);
                        }
                        BadgeFieldKey::Badges => {
                            let value = map.next_value::<BadgeList>()?;
                            data.default = Some(value.0);
                        }
                        BadgeFieldKey::BadgesGroup(group) => {
                            let group = group.to_owned();
                            let value = map.next_value::<BadgeList>()?;
                            data.groups.entry(group).or_insert(value.0);
                        }
                    }
                }

                Ok(data)
            }
        }

        deserializer.deserialize_any(Badges)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use similar_asserts::assert_eq;

    use crate::config::{badge::item::License, testing};

    use super::*;

    #[test]
    fn deserialize_badge_preserves_same_kind_in_groups() {
        let source = testing::badge_manifest(indoc! {"
            badges = {
              license = true,
              maintenance = true,
            }
            badges-group1 = {
              license = true,
              maintenance = true,
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
            ]
        );
        assert_eq!(
            badge.groups["group1"]
                .clone()
                .into_iter()
                .collect::<Vec<_>>(),
            [
                (
                    BadgeItemKey::License(None),
                    Some(BadgeItem::License(License::default()))
                ),
                (
                    BadgeItemKey::Maintenance(None),
                    Some(BadgeItem::Maintenance)
                ),
            ]
        );
    }

    #[test]
    fn deserialize_badge_parses_valid_group_names() {
        let source = testing::badge_manifest(indoc! {"
            badges-group1 = {
              license = true,
            }
            badges-group_2 = {
              maintenance = true,
            }
            badges-Group3 = {
              crates-io = true,
            }
            badges-group_4-foo = {
              docs-rs = true,
            }
        "});
        let badge = testing::parse_badge(&source);
        assert_eq!(
            badge.groups["group1"]
                .clone()
                .into_iter()
                .collect::<Vec<_>>(),
            [(
                BadgeItemKey::License(None),
                Some(BadgeItem::License(License::default()))
            )]
        );
        assert_eq!(
            badge.groups["group_2"]
                .clone()
                .into_iter()
                .collect::<Vec<_>>(),
            [(
                BadgeItemKey::Maintenance(None),
                Some(BadgeItem::Maintenance),
            )]
        );
        assert_eq!(
            badge.groups["Group3"]
                .clone()
                .into_iter()
                .collect::<Vec<_>>(),
            [(BadgeItemKey::CratesIo(None), Some(BadgeItem::CratesIo)),]
        );
        assert_eq!(
            badge.groups["group_4-foo"]
                .clone()
                .into_iter()
                .collect::<Vec<_>>(),
            [(BadgeItemKey::DocsRs(None), Some(BadgeItem::DocsRs)),]
        );
    }

    #[test]
    fn deserialize_badge_parses_style() {
        let source = testing::badge_manifest(indoc! {r#"
            style = "flat"
        "#});
        let badge = testing::parse_badge(&source);
        assert_eq!(badge.style, Some(BadgeStyle::Flat));
    }
    #[test]
    fn deserialize_badge_rejects_invalid_field_types() {
        let source = testing::badge_manifest(indoc! {r#"
            style = "invalid"
        "#});
        testing::deserialize_config_err(&source, "unknown variant `invalid`", r#""invalid""#);

        let source = testing::badge_manifest(indoc! {r"
            style = false
        "});
        testing::deserialize_config_err(&source, "wanted string or table", "false");
    }

    #[test]
    fn deserialize_badge_rejects_unknown_fields() {
        let source = testing::badge_manifest(indoc! {"
            unknown = true
        "});
        testing::deserialize_config_err(&source, "unknown field `unknown`", "unknown");
    }

    #[test]
    fn deserialize_badge_rejects_invalid_group_names() {
        let source = testing::badge_manifest("badges- = {}");
        testing::deserialize_config_err(&source, "invalid field name `badges-`", "badges-");

        let source = testing::badge_manifest("badges-123 = {}");
        testing::deserialize_config_err(&source, "invalid field name `badges-123`", "badges-123");

        let source = testing::badge_manifest(r#""badges-!" = {}"#);
        testing::deserialize_config_err(&source, "invalid field name `badges-!`", r#""badges-!""#);

        let source = testing::badge_manifest(r#""badges- " = {}"#);
        testing::deserialize_config_err(&source, "invalid field name `badges- `", r#""badges- ""#);

        let source = testing::badge_manifest(indoc! {"
            badges = {}
            badges-123 = {}
        "});
        testing::deserialize_config_err(&source, "invalid field name `badges-123`", "badges-123");
    }

    #[test]
    fn toml_rejects_duplicate_fields() {
        let source = testing::badge_manifest(indoc! {"
            badges-x = {}
            badges = {}
            badges = {}
        "});
        testing::parse_config_err(&source, "duplicate key", "badges");

        let source = testing::badge_manifest(indoc! {"
            badges-group1 = {}
            badges-group2 = {}
            badges-group2 = {}
        "});
        testing::parse_config_err(&source, "duplicate key", "badges-group2");
    }

    #[test]
    fn deserialize_badge_accepts_old_badge_table_syntax() {
        let source = indoc! {r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [package.metadata.cargo-sync-rdme.badge.badges]
            license = true
            maintenance = true
        "#};
        let badge = testing::parse_badge(source);
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
            ]
        );
    }
}
