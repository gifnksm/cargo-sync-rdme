use std::fmt;

use serde::{
    Deserialize,
    de::{self, DeserializeSeed},
};

use super::GetConfigError;
use crate::{config::TomlTable, source::Spanned};

#[derive(Debug, Clone, Default)]
pub(crate) struct Badges {
    pub(crate) toml_table: TomlTable,
    pub(crate) maintenance: Option<Maintenance>,
}

const KEY_MAINTENANCE: &str = "maintenance";

impl Badges {
    pub(crate) fn try_maintenance(&self) -> Result<&Maintenance, Box<GetConfigError>> {
        let maintenance = self
            .maintenance
            .as_ref()
            .ok_or_else(|| self.toml_table.missing_key_error(KEY_MAINTENANCE))?;
        Ok(maintenance)
    }
}

#[derive(Debug, Clone)]
pub(super) struct BadgesSeed(pub(super) TomlTable);

impl<'de> DeserializeSeed<'de> for BadgesSeed {
    type Value = Badges;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BadgesVisitor(TomlTable);

        impl<'de> de::Visitor<'de> for BadgesVisitor {
            type Value = Badges;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("table")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let toml_table = self.0;
                let mut maintenance = None;

                while let Some(key) = map.next_key::<Spanned<String>>()? {
                    match key.value.as_str() {
                        KEY_MAINTENANCE => {
                            maintenance = Some(map.next_value_seed(MaintenanceSeed(
                                toml_table.child(key.as_deref()),
                            ))?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(Badges {
                    toml_table,
                    maintenance,
                })
            }
        }

        deserializer.deserialize_map(BadgesVisitor(self.0))
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Maintenance {
    pub(crate) toml_table: TomlTable,
    pub(crate) status: Option<MaintenanceStatus>,
}

const KEY_STATUS: &str = "status";

impl Maintenance {
    pub(crate) fn try_status(&self) -> Result<MaintenanceStatus, Box<GetConfigError>> {
        let status = self
            .status
            .as_ref()
            .ok_or_else(|| self.toml_table.missing_key_error(KEY_STATUS))?;
        Ok(*status)
    }
}

#[derive(Debug, Clone)]
pub(super) struct MaintenanceSeed(pub(super) TomlTable);

impl<'de> DeserializeSeed<'de> for MaintenanceSeed {
    type Value = Maintenance;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MaintenanceVisitor(TomlTable);

        impl<'de> de::Visitor<'de> for MaintenanceVisitor {
            type Value = Maintenance;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("table")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let toml_table = self.0;
                let mut status = None;

                while let Some(key) = map.next_key::<Spanned<String>>()? {
                    match key.value.as_str() {
                        KEY_STATUS => status = Some(map.next_value()?),
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(Maintenance { toml_table, status })
            }
        }

        deserializer.deserialize_map(MaintenanceVisitor(self.0))
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MaintenanceStatus {
    ActivelyDeveloped,
    PassivelyMaintained,
    AsIs,
    Experimental,
    LookingForMaintainer,
    Deprecated,
    #[default]
    None,
}

impl MaintenanceStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ActivelyDeveloped => "actively-developed",
            Self::PassivelyMaintained => "passively-maintained",
            Self::AsIs => "as-is",
            Self::Experimental => "experimental",
            Self::LookingForMaintainer => "looking-for-maintainer",
            Self::Deprecated => "deprecated",
            Self::None => "done",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::testing;

    #[test]
    fn try_maintenance_returns_error_when_not_set() {
        let source = indoc::indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"

            [badges]
        "#};
        let manifest = testing::parse_manifest(source);
        let (key, table, span, source_code) = manifest
            .try_badges()
            .unwrap()
            .try_maintenance()
            .unwrap_err()
            .into_missing_key_in_table();
        assert_eq!(key, "maintenance");
        assert_eq!(table, "badges");
        assert_eq!(&source[span.offset()..][..span.len()], "badges");
        assert_eq!(source_code.name(), "Cargo.toml");
    }

    #[test]
    fn try_status_returns_error_when_not_set() {
        let source = indoc::indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"

            [badges]
            maintenance = {}
        "#};
        let manifest = testing::parse_manifest(source);
        let (key, table, span, source_code) = manifest
            .try_badges()
            .unwrap()
            .try_maintenance()
            .unwrap()
            .try_status()
            .unwrap_err()
            .into_missing_key_in_table();
        assert_eq!(key, "status");
        assert_eq!(table, "badges.maintenance");
        assert_eq!(&source[span.offset()..][..span.len()], "maintenance");
        assert_eq!(source_code.name(), "Cargo.toml");
    }
}
