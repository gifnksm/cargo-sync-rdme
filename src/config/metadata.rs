use std::{fmt, str::FromStr};

use serde::{
    de::{Error, Visitor},
    Deserialize,
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
    #[serde(default, deserialize_with = "deserialize_badges")]
    pub(crate) badges: Vec<Badge>,
    #[serde(default)]
    pub(crate) rustdoc: Rustdoc,
}

#[derive(Debug, Clone)]
pub(crate) enum Badge {
    Maintenance,
    License(License),
    CratesIo,
    DocsRs,
    RustVersion,
    GithubActions(GithubActions),
    Codecov,
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

fn deserialize_badges<'de, D>(deserializer: D) -> Result<Vec<Badge>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Data;

    impl<'de> Visitor<'de> for Data {
        type Value = Vec<Badge>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("map")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let mut data = vec![];
            while let Some(key) = map.next_key::<&str>()? {
                let kind = BadgeKind::from_str(key)
                    .map_err(|_| M::Error::unknown_variant(key, BadgeKind::expecting()))?;
                #[derive(Deserialize)]
                #[serde(bound = "T: Default + Deserialize<'de>")]
                struct Wrap<T>(#[serde(deserialize_with = "de::bool_or_map")] Option<T>);

                match kind {
                    BadgeKind::Maintenance => {
                        if map.next_value::<bool>()? {
                            data.push(Badge::Maintenance);
                        }
                    }
                    BadgeKind::License => {
                        if let Wrap(Some(license)) = map.next_value::<Wrap<License>>()? {
                            data.push(Badge::License(license));
                        }
                    }
                    BadgeKind::CratesIo => {
                        if map.next_value::<bool>()? {
                            data.push(Badge::CratesIo);
                        }
                    }
                    BadgeKind::DocsRs => {
                        if map.next_value::<bool>()? {
                            data.push(Badge::DocsRs);
                        }
                    }
                    BadgeKind::RustVersion => {
                        if map.next_value::<bool>()? {
                            data.push(Badge::RustVersion);
                        }
                    }
                    BadgeKind::GithubActions => {
                        if let Wrap(Some(github_actions)) =
                            map.next_value::<Wrap<GithubActions>>()?
                        {
                            data.push(Badge::GithubActions(github_actions));
                        }
                    }
                    BadgeKind::Codecov => {
                        if map.next_value::<bool>()? {
                            data.push(Badge::Codecov);
                        }
                    }
                }
            }
            Ok(data)
        }
    }

    deserializer.deserialize_any(Data)
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct License {
    #[serde(default)]
    pub(crate) link: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct GithubActions {
    #[serde(default, deserialize_with = "de::string_or_map_or_seq")]
    pub(crate) workflows: Vec<GithubActionsWorkflow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Rustdoc {
    #[serde(default)]
    pub(crate) html_root_url: Option<String>,
}
