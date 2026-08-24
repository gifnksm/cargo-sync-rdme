use serde::Deserialize;
use toml::Spanned;

use super::{GetConfigError, KeyNotSet};
use crate::with_source::WithSource;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Badges {
    #[serde(default)]
    pub(crate) maintenance: Option<Spanned<Maintenance>>,
}

impl<'a> WithSource<&'a Spanned<Badges>> {
    pub(crate) fn try_maintenance(
        &self,
    ) -> Result<WithSource<&'a Spanned<Maintenance>>, GetConfigError> {
        let maintenance = self
            .value()
            .get_ref()
            .maintenance
            .as_ref()
            .ok_or_else(|| KeyNotSet {
                name: self.name().to_owned(),
                key: "badges.maintenance".to_owned(),
                span: self.span(),
                source_code: self.to_named_source(),
            })?;
        Ok(self.map(|_| maintenance))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Maintenance {
    #[serde(default)]
    pub(crate) status: Option<MaintenanceStatus>,
}

impl WithSource<&Spanned<Maintenance>> {
    pub(crate) fn try_status(&self) -> Result<MaintenanceStatus, GetConfigError> {
        let status = self
            .value()
            .get_ref()
            .status
            .as_ref()
            .ok_or_else(|| KeyNotSet {
                name: self.name().to_owned(),
                key: "badges.maintenance.status".to_owned(),
                span: self.span(),
                source_code: self.to_named_source(),
            })?;
        Ok(*status)
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
    use crate::config::manifest::Manifest;

    use super::*;

    #[test]
    fn try_maintenance_returns_error_when_not_set() {
        let source = indoc::indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"

            [badges]
        "#};
        let manifest =
            WithSource::<Manifest>::dummy_with_source(source, toml::from_str(source).unwrap());
        let GetConfigError::KeyNotSet { source: err } = manifest
            .try_badges()
            .unwrap()
            .try_maintenance()
            .unwrap_err();
        assert_eq!(err.name, "dummy-name");
        assert_eq!(err.key, "badges.maintenance");
        assert_eq!(&source[err.span.offset()..][..err.span.len()], "[badges]");
        assert_eq!(err.source_code.name(), "dummy-path");
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
        let manifest =
            WithSource::<Manifest>::dummy_with_source(source, toml::from_str(source).unwrap());
        let GetConfigError::KeyNotSet { source: err } = manifest
            .try_badges()
            .unwrap()
            .try_maintenance()
            .unwrap()
            .try_status()
            .unwrap_err();
        assert_eq!(err.name, "dummy-name");
        assert_eq!(err.key, "badges.maintenance.status");
        assert_eq!(&source[err.span.offset()..][..err.span.len()], "{}");
        assert_eq!(err.source_code.name(), "dummy-path");
    }
}
