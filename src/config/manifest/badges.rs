use serde::Deserialize;

use super::{GetConfigError, KeyNotSet};
use crate::source::SourceFileSpanned;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Badges {
    #[serde(default)]
    pub(crate) maintenance: Option<SourceFileSpanned<Maintenance>>,
}

impl SourceFileSpanned<&Badges> {
    pub(crate) fn try_maintenance(
        &self,
    ) -> Result<SourceFileSpanned<&Maintenance>, GetConfigError> {
        let maintenance = self.value.maintenance.as_ref().ok_or_else(|| KeyNotSet {
            key: "badges.maintenance".to_owned(),
            span: self.source_span(),
            source_code: self.source.to_named_source(),
        })?;
        Ok(maintenance.as_ref())
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Maintenance {
    #[serde(default)]
    pub(crate) status: Option<MaintenanceStatus>,
}

impl SourceFileSpanned<&Maintenance> {
    pub(crate) fn try_status(&self) -> Result<MaintenanceStatus, GetConfigError> {
        let status = self.value.status.as_ref().ok_or_else(|| KeyNotSet {
            key: "badges.maintenance.status".to_owned(),
            span: self.source_span(),
            source_code: self.source.to_named_source(),
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
    use crate::config::testing;

    use super::*;

    #[test]
    fn try_maintenance_returns_error_when_not_set() {
        let source = indoc::indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"

            [badges]
        "#};
        let manifest = testing::parse_manifest(source);
        let GetConfigError::KeyNotSet { source: err } = manifest
            .try_badges()
            .unwrap()
            .try_maintenance()
            .unwrap_err();
        assert_eq!(err.key, "badges.maintenance");
        assert_eq!(&source[err.span.offset()..][..err.span.len()], "[badges]");
        assert_eq!(err.source_code.name(), "Cargo.toml");
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
        let GetConfigError::KeyNotSet { source: err } = manifest
            .try_badges()
            .unwrap()
            .try_maintenance()
            .unwrap()
            .try_status()
            .unwrap_err();
        assert_eq!(err.key, "badges.maintenance.status");
        assert_eq!(&source[err.span.offset()..][..err.span.len()], "{}");
        assert_eq!(err.source_code.name(), "Cargo.toml");
    }
}
