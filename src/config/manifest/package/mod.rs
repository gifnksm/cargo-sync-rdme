use serde::Deserialize;

pub(crate) mod metadata;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Package {
    #[serde(default)]
    pub(crate) metadata: Option<metadata::Metadata>,
}
