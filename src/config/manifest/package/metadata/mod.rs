use serde::Deserialize;

use crate::config::de;

pub(crate) mod badge;
pub(crate) mod rustdoc;

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
    pub(crate) badge: badge::Badge,
    #[serde(default)]
    pub(crate) rustdoc: rustdoc::Rustdoc,
}
