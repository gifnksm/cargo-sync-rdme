use std::{collections::HashMap, fmt, str::FromStr, sync::Arc};

use serde::{
    Deserialize,
    de::{Error as _, Visitor},
};
use void::Void;

use super::de;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Metadata {
    #[serde(default)]
    pub(crate) cargo_sync_rdme: CargoSyncRdme,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct CargoSyncRdme {
    #[serde(default, deserialize_with = "de::string_or_seq")]
    pub(crate) extra_targets: Vec<String>,
    #[serde(default)]
    pub(crate) badge: Badge,
    #[serde(default)]
    pub(crate) rustdoc: Rustdoc,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Badge {
    pub(crate) style: Option<BadgeStyle>,
    pub(crate) default: Option<Arc<[BadgeItem]>>,
    pub(crate) groups: HashMap<Arc<str>, Arc<[BadgeItem]>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
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
enum BadgeKind {
    Maintenance,
    License,
    CratesIo,
    DocsRs,
    RustVersion,
    GithubActions,
    Codecov,
}

impl FromStr for BadgeKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
                    return Err(());
                }
            }
        };
        Ok(kind)
    }
}

impl BadgeKind {
    fn expecting() -> &'static [&'static str] {
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

fn deserialize_badge_list<'de, D>(deserializer: D) -> Result<Arc<[BadgeItem]>, D::Error>
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
            while let Some(key) = map.next_key::<&str>()? {
                #[derive(Deserialize)]
                #[serde(bound = "T: Default + Deserialize<'de>")]
                struct BoolOrMap<T>(#[serde(deserialize_with = "de::bool_or_map")] Option<T>);

                let kind = BadgeKind::from_str(key)
                    .map_err(|()| M::Error::unknown_variant(key, BadgeKind::expecting()))?;

                match kind {
                    BadgeKind::Maintenance => {
                        if map.next_value::<bool>()? {
                            data.push(BadgeItem::Maintenance);
                        }
                    }
                    BadgeKind::License => {
                        if let BoolOrMap(Some(license)) = map.next_value::<BoolOrMap<License>>()? {
                            data.push(BadgeItem::License(license));
                        }
                    }
                    BadgeKind::CratesIo => {
                        if map.next_value::<bool>()? {
                            data.push(BadgeItem::CratesIo);
                        }
                    }
                    BadgeKind::DocsRs => {
                        if map.next_value::<bool>()? {
                            data.push(BadgeItem::DocsRs);
                        }
                    }
                    BadgeKind::RustVersion => {
                        if map.next_value::<bool>()? {
                            data.push(BadgeItem::RustVersion);
                        }
                    }
                    BadgeKind::GithubActions => {
                        if let BoolOrMap(Some(github_actions)) =
                            map.next_value::<BoolOrMap<GithubActions>>()?
                        {
                            data.push(BadgeItem::GithubActions(github_actions));
                        }
                    }
                    BadgeKind::Codecov => {
                        if let BoolOrMap(Some(codecov)) = map.next_value::<BoolOrMap<Codecov>>()? {
                            data.push(BadgeItem::Codecov(codecov));
                        }
                    }
                }
            }
            Ok(data.into())
        }
    }

    deserializer.deserialize_any(BadgeList)
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
                    #[serde(deserialize_with = "deserialize_badge_list")] Arc<[BadgeItem]>,
                );

                let mut data = Badge::default();

                while let Some(key) = map.next_key::<String>()? {
                    let expected = &["badges", "badges-*", "style"];
                    if key.as_str() == "style" {
                        data.style = map.next_value()?;
                        continue;
                    }
                    if key == "badges" {
                        let value = map.next_value::<BadgeList>()?;
                        if data.default.replace(value.0).is_some() {
                            return Err(M::Error::duplicate_field("badges"));
                        }
                        continue;
                    }
                    if let Some(rest) = key.strip_prefix("badges-") {
                        if rest.is_empty() {
                            return Err(M::Error::custom(format_args!(
                                "invalid field name: `{key}` (expected `badges-*` where `*` is a non-empty string)"
                            )));
                        }
                        let group = rest.to_owned();
                        let value = map.next_value::<BadgeList>()?;
                        data.groups.entry(group.into()).or_insert(value.0);
                        continue;
                    }
                    return Err(M::Error::unknown_field(&key, expected));
                }

                Ok(data)
            }
        }

        deserializer.deserialize_any(Badges)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Rustdoc {
    #[serde(default)]
    pub(crate) html_root_url: Option<String>,
    #[serde(default)]
    pub(crate) mappings: HashMap<String, String>,
}
