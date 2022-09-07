use std::str::FromStr;

use serde::Deserialize;
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
    #[serde(default)]
    pub(crate) badges: Badges,
    #[serde(default)]
    pub(crate) rustdoc: Rustdoc,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Badges {
    #[serde(default)]
    pub(crate) maintenance: bool,
    #[serde(default, deserialize_with = "de::bool_or_map")]
    pub(crate) license: Option<License>,
    #[serde(default)]
    pub(crate) crates_io: bool,
    #[serde(default)]
    pub(crate) docs_rs: bool,
    #[serde(default)]
    pub(crate) rust_version: bool,
    #[serde(default, deserialize_with = "de::bool_or_map")]
    pub(crate) github_actions: Option<GithubActions>,
    #[serde(default)]
    pub(crate) codecov: bool,
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
