use serde::Deserialize;
use toml::Spanned;

use super::GetConfigError;
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
        let maintenance = self.value().get_ref().maintenance.as_ref().ok_or_else(|| {
            GetConfigError::KeyNotSet {
                name: self.name().to_owned(),
                key: "badges.maintenance".to_owned(),
                span: self.span(),
                source_code: self.to_named_source(),
            }
        })?;
        Ok(self.map(|_| maintenance))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Maintenance {
    #[serde(default)]
    pub(crate) status: Option<Spanned<MaintenanceStatus>>,
}

impl<'a> WithSource<&'a Spanned<Maintenance>> {
    pub(crate) fn try_status(
        &self,
    ) -> Result<WithSource<&'a Spanned<MaintenanceStatus>>, GetConfigError> {
        let status =
            self.value()
                .get_ref()
                .status
                .as_ref()
                .ok_or_else(|| GetConfigError::KeyNotSet {
                    name: self.name().to_owned(),
                    key: "badges.maintenance.status".to_owned(),
                    span: self.span(),
                    source_code: self.to_named_source(),
                })?;
        Ok(self.map(|_| status))
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
    pub(crate) fn as_str(&self) -> &'static str {
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
