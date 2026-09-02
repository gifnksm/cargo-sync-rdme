use serde::Deserialize;

use crate::source::{DeserializeAsTomlError, SourceFile};

pub(crate) mod badge;
mod de;
pub(crate) mod rustdoc;
#[cfg(test)]
mod testing;

// To detect items that do not have explicit values, wrap cargo's standard
// configuration items in Options.
#[derive(Debug, Clone, Default, Deserialize)]
struct Manifest {
    #[serde(default)]
    package: Option<Package>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Package {
    #[serde(default)]
    metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Metadata {
    #[serde(default)]
    cargo_sync_rdme: Option<Config>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Config {
    #[serde(default, deserialize_with = "de::string_or_seq")]
    pub(crate) extra_targets: Vec<String>,
    #[serde(default)]
    pub(crate) badge: badge::Badge,
    #[serde(default)]
    pub(crate) rustdoc: rustdoc::Rustdoc,
}

impl Config {
    pub(crate) fn parse(manifest_file: &SourceFile) -> Result<Self, DeserializeAsTomlError> {
        let manifest = manifest_file.deserialize_as_toml::<Manifest>()?;
        let config = manifest
            .package
            .and_then(|package| package.metadata)
            .and_then(|metadata| metadata.cargo_sync_rdme)
            .unwrap_or_default();
        Ok(config)
    }
}
