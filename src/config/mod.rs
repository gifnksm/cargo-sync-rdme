use serde::Deserialize;

pub(crate) mod badge;
mod de;
pub(crate) mod rustdoc;
#[cfg(test)]
mod testing;

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
