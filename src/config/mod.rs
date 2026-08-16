use std::sync::{Arc, LazyLock};

use miette::{NamedSource, SourceSpan};
use serde::Deserialize;
use snafu::Snafu;
use toml::Spanned;

use crate::with_source::WithSource;

// To detect items that do not have explicit values, wrap cargo's standard
// configuration items in Options.

pub(crate) mod badges;
mod de;
pub(crate) mod metadata;
pub(crate) mod package;
#[cfg(test)]
mod tests;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum GetConfigError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    KeyNotSet {
        #[snafu(source(from(KeyNotSet, Box::new)))]
        #[diagnostic_source]
        source: Box<KeyNotSet>,
    },
}

#[derive(Debug, Snafu, miette::Diagnostic)]
#[snafu(display("key `{key}` is not set in {name}"))]
pub(crate) struct KeyNotSet {
    name: String,
    key: String,
    #[label]
    span: SourceSpan,
    #[source_code]
    source_code: NamedSource<Arc<str>>,
}

impl GetConfigError {
    pub(crate) fn with_key(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        match &mut self {
            Self::KeyNotSet { source } => source.key = key,
        }
        self
    }
}

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
    pub(crate) fn config(&self) -> &metadata::CargoSyncRdme {
        static DEFAULT: LazyLock<metadata::CargoSyncRdme> = LazyLock::new(Default::default);
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
