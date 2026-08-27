use std::fmt;

use serde::{Deserialize, de};

use crate::{
    config::{GetConfigError, TomlTable, manifest::badges::BadgesSeed},
    source::{self, Spanned},
};

pub(crate) mod badges;
pub(crate) mod package;

#[derive(Debug, Clone, Default)]
pub(crate) struct Manifest {
    pub(crate) toml_table: TomlTable,
    pub(crate) package: Option<package::Package>,
    pub(crate) badges: Option<badges::Badges>,
}

const KEY_PACKAGE: &str = "package";
const KEY_BADGES: &str = "badges";

impl Manifest {
    pub(crate) fn try_badges(&self) -> Result<&badges::Badges, Box<GetConfigError>> {
        let badges = self
            .badges
            .as_ref()
            .ok_or_else(|| self.toml_table.missing_key_error(KEY_BADGES))?;
        Ok(badges)
    }
}

impl<'de> Deserialize<'de> for Manifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ManifestVisitor;

        impl<'de> de::Visitor<'de> for ManifestVisitor {
            type Value = Manifest;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("table")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let toml_table = TomlTable::root(source::current_source_file()?);

                let mut package = None;
                let mut badges = None;
                while let Some(key) = map.next_key::<Spanned<String>>()? {
                    match key.value.as_str() {
                        KEY_PACKAGE => {
                            package = Some(map.next_value()?);
                        }
                        KEY_BADGES => {
                            badges = Some(
                                map.next_value_seed(BadgesSeed(toml_table.child(key.as_deref())))?,
                            );
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(Manifest {
                    toml_table,
                    package,
                    badges,
                })
            }
        }

        deserializer.deserialize_map(ManifestVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::source::SourceFile;

    use super::*;

    #[test]
    fn try_badges_returns_error_when_not_set() {
        let source = indoc::indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"
        "#};
        let source_file = SourceFile::new_for_test("Cargo.toml", source);
        let manifest = source_file.parse_as_toml::<Manifest>().unwrap();
        let (key, span, source_code) = manifest
            .try_badges()
            .unwrap_err()
            .into_missing_top_level_key();
        assert_eq!(key, "badges");
        assert_eq!(span, (0..0).into());
        assert_eq!(source_code.name(), "Cargo.toml");
    }
}
