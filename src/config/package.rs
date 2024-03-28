use cargo_metadata::semver::VersionReq;
use serde::Deserialize;
use toml::Spanned;

use crate::with_source::WithSource;

use super::{metadata, GetConfigError, KeyNotSet};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Package {
    #[serde(default)]
    pub(crate) license: Option<Spanned<String>>,
    #[serde(default)]
    pub(crate) license_file: Option<Spanned<String>>,
    #[serde(default)]
    pub(crate) repository: Option<Spanned<String>>,
    #[serde(default)]
    pub(crate) rust_version: Option<Spanned<VersionReq>>,
    #[serde(default)]
    pub(crate) metadata: Option<Spanned<metadata::Metadata>>,
}

#[derive(Debug, Clone)]
pub(crate) enum License<'a> {
    Name {
        name: &'a Spanned<String>,
        path: &'a Option<Spanned<String>>,
    },
    File {
        path: &'a Spanned<String>,
    },
}

impl<'a> WithSource<&'a Spanned<Package>> {
    pub(crate) fn try_license(&self) -> Result<WithSource<License<'a>>, GetConfigError> {
        if let Some(name) = &self.value().get_ref().license {
            return Ok(self.map(|_| License::Name {
                name,
                path: &self.value().get_ref().license_file,
            }));
        }
        if let Some(path) = &self.value().get_ref().license_file {
            return Ok(self.map(|_| License::File { path }));
        }
        Err(KeyNotSet {
            name: self.name().to_owned(),
            key: "package.license` or `package.license-file".to_owned(),
            span: self.span(),
            source_code: self.to_named_source(),
        }
        .into())
    }

    pub(crate) fn try_repository(&self) -> Result<WithSource<&'a Spanned<String>>, GetConfigError> {
        let repository = self
            .value()
            .get_ref()
            .repository
            .as_ref()
            .ok_or_else(|| KeyNotSet {
                name: self.name().to_owned(),
                key: "package.repository".to_owned(),
                span: self.span(),
                source_code: self.to_named_source(),
            })?;
        Ok(self.map(|_| repository))
    }

    pub(crate) fn try_rust_version(
        &self,
    ) -> Result<WithSource<&'a Spanned<VersionReq>>, GetConfigError> {
        let status = self
            .value()
            .get_ref()
            .rust_version
            .as_ref()
            .ok_or_else(|| KeyNotSet {
                name: self.name().to_owned(),
                key: "package.rust-version".to_owned(),
                span: self.span(),
                source_code: self.to_named_source(),
            })?;
        Ok(self.map(|_| status))
    }
}
