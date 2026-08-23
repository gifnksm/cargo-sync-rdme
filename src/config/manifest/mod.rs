use std::sync::LazyLock;

use serde::Deserialize;
use toml::Spanned;

use crate::{
    config::{GetConfigError, KeyNotSet},
    with_source::WithSource,
};

pub(crate) mod badges;
pub(crate) mod package;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Manifest {
    #[serde(default)]
    pub(crate) package: Option<Spanned<package::Package>>,
    #[serde(default)]
    pub(crate) badges: Option<Spanned<badges::Badges>>,
}

impl WithSource<Manifest> {
    pub(crate) fn try_badges(
        &self,
    ) -> Result<WithSource<&Spanned<badges::Badges>>, GetConfigError> {
        let badges = self.value().badges.as_ref().ok_or_else(|| KeyNotSet {
            name: self.name().to_owned(),
            key: "badges".to_owned(),
            span: (0..0).into(),
            source_code: self.to_named_source(),
        })?;
        Ok(self.map(|_| badges))
    }
}

impl Manifest {
    pub(crate) fn config(&self) -> &package::metadata::CargoSyncRdme {
        static DEFAULT: LazyLock<package::metadata::CargoSyncRdme> =
            LazyLock::new(Default::default);
        (|| {
            Some(
                &self
                    .package
                    .as_ref()?
                    .get_ref()
                    .metadata
                    .as_ref()?
                    .get_ref()
                    .cargo_sync_rdme,
            )
        })()
        .unwrap_or(&DEFAULT)
    }
}
