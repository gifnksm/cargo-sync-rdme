use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Rustdoc {
    #[serde(default)]
    pub(crate) html_root_url: Option<String>,
    #[serde(default)]
    pub(crate) mappings: HashMap<String, String>,
}
