use serde::Deserialize;
use toml::Spanned;

use super::metadata;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Package {
    #[serde(default)]
    pub(crate) metadata: Option<Spanned<metadata::Metadata>>,
}
