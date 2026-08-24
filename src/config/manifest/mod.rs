use std::sync::LazyLock;

use serde::Deserialize;

use crate::{
    config::{GetConfigError, KeyNotSet},
    source::SourceFileSpanned,
};

pub(crate) mod badges;
pub(crate) mod package;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Manifest {
    #[serde(default)]
    pub(crate) package: Option<package::Package>,
    #[serde(default)]
    pub(crate) badges: Option<SourceFileSpanned<badges::Badges>>,
}

impl SourceFileSpanned<Manifest> {
    pub(crate) fn try_badges(&self) -> Result<SourceFileSpanned<&badges::Badges>, GetConfigError> {
        let badges = self.value.badges.as_ref().ok_or_else(|| KeyNotSet {
            key: "badges".to_owned(),
            span: self.source_span(),
            source_code: self.source.to_named_source(),
        })?;
        Ok(badges.as_ref())
    }
}

impl Manifest {
    pub(crate) fn config(&self) -> &package::metadata::CargoSyncRdme {
        static DEFAULT: LazyLock<package::metadata::CargoSyncRdme> =
            LazyLock::new(Default::default);
        (|| Some(&self.package.as_ref()?.metadata.as_ref()?.cargo_sync_rdme))().unwrap_or(&DEFAULT)
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
        let manifest = source_file
            .parse_as_toml::<SourceFileSpanned<Manifest>>()
            .unwrap();
        let GetConfigError::KeyNotSet { source: err } = manifest.try_badges().unwrap_err();
        assert_eq!(err.key, "badges");
        assert_eq!(err.span, (0..0).into());
        assert_eq!(err.source_code.name(), "Cargo.toml");
    }
}
