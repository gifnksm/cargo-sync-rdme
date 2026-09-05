use std::{
    borrow::Borrow,
    collections::{HashMap, hash_map::Entry},
    io,
    str::FromStr as _,
    sync::Arc,
};

use cargo_metadata::{Metadata, Package, PackageId, PackageName, camino::Utf8PathBuf};
use miette::{Diagnostic, NamedSource, SourceSpan};
use snafu::{ResultExt as _, Snafu};
use strum::{EnumString, IntoStaticStr};

use crate::{
    config::Config,
    source::{
        ParseTomlResultExt as _, SourceFile, SourceFileLoader, SourceFilePath, TomlDocument,
        TomlError,
    },
};

#[derive(Debug)]
pub(crate) struct ManifestLoader<'a> {
    workspace: &'a Metadata,
    root_package: Option<&'a Package>,
    workspace_manifest_path: Utf8PathBuf,
    package_manifest: HashMap<PackageId, Arc<Manifest>>,
    workspace_manifest: Option<Arc<Manifest>>,
}

impl<'a> ManifestLoader<'a> {
    pub(crate) fn new(workspace: &'a Metadata) -> Self {
        // We don't use `Metadata::root_package()` here because it may return non-root package in some cases.
        // <https://github.com/oli-obk/cargo_metadata/issues/321>
        let workspace_manifest_path = workspace.workspace_root.join("Cargo.toml");
        let root_package = workspace
            .packages
            .iter()
            .find(|package| package.manifest_path == workspace_manifest_path);
        Self {
            workspace,
            root_package,
            workspace_manifest_path,
            package_manifest: HashMap::new(),
            workspace_manifest: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn add_package_manifest(&mut self, package_id: PackageId, manifest: Arc<Manifest>) {
        self.package_manifest.insert(package_id, manifest);
    }

    #[cfg(test)]
    pub(crate) fn set_workspace_manifest(&mut self, manifest: Arc<Manifest>) {
        self.workspace_manifest = Some(manifest);
    }

    pub(crate) fn is_root_package(&self, package: &Package) -> bool {
        self.root_package.is_some_and(|root| root.id == package.id)
    }

    pub(crate) fn load_package_manifest(
        &mut self,
        package: &Package,
    ) -> Result<Arc<Manifest>, Box<ManifestLoaderError>> {
        let entry = match self.package_manifest.entry(package.id.clone()) {
            Entry::Occupied(entry) => return Ok(Arc::clone(entry.get())),
            Entry::Vacant(entry) => entry,
        };

        let loader = SourceFileLoader::from_path(self.workspace, &package.manifest_path);
        let manifest =
            Manifest::load(&loader).with_context(|_source| LoadPackageManifestFileSnafu {
                package: package.name.clone(),
                manifest: &loader,
            })?;
        let manifest = Arc::new(manifest);
        entry.insert(Arc::clone(&manifest));
        if self.is_root_package(package) {
            self.workspace_manifest = Some(Arc::clone(&manifest));
        }
        Ok(manifest)
    }

    pub(crate) fn load_workspace_manifest(
        &mut self,
    ) -> Result<Arc<Manifest>, Box<ManifestLoaderError>> {
        if let Some(manifest) = &self.workspace_manifest {
            return Ok(Arc::clone(manifest));
        }

        if let Some(package) = self.root_package {
            return self.load_package_manifest(package);
        }

        let loader = SourceFileLoader::from_path(self.workspace, &self.workspace_manifest_path);
        let manifest = Manifest::load(&loader)
            .with_context(|_source| LoadWorkspaceManifestFileSnafu { manifest: &loader })?;
        let manifest = Arc::new(manifest);
        self.workspace_manifest = Some(Arc::clone(&manifest));
        Ok(manifest)
    }
}

#[derive(Debug, Snafu, Diagnostic)]
pub(crate) enum ManifestLoaderError {
    #[snafu(display("failed to load package manifest file for package `{package}`: {path}", path = manifest.workspace_path))]
    LoadPackageManifestFile {
        package: PackageName,
        manifest: SourceFilePath,
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<ManifestError>,
    },
    #[snafu(display("failed to load workspace manifest file: {path}", path = manifest.workspace_path))]
    LoadWorkspaceManifestFile {
        manifest: SourceFilePath,
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<ManifestError>,
    },
}

#[derive(Debug)]
pub(crate) struct Manifest {
    document: TomlDocument,
}

impl Manifest {
    pub(crate) fn load(loader: &SourceFileLoader) -> Result<Self, Box<ManifestError>> {
        let file = loader.load()?;
        let document = file.parse_as_toml()?;
        Ok(Self { document })
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(source: &SourceFile) -> Result<Self, Box<ManifestError>> {
        let document = source.parse_as_toml()?;
        Ok(Self { document })
    }

    pub(crate) fn source_file(&self) -> &SourceFile {
        self.document.source_file()
    }

    pub(crate) fn workspace_config(&self) -> Result<Option<Config>, Box<ManifestError>> {
        let config = self
            .document
            .deserialize_entry(&["workspace", "metadata", "cargo-sync-rdme"])
            .ignore_missing_key_error()?;
        Ok(config)
    }

    pub(crate) fn package_config(&self) -> Result<Option<Config>, Box<ManifestError>> {
        let config = self
            .document
            .deserialize_entry(&["package", "metadata", "cargo-sync-rdme"])
            .ignore_missing_key_error()?;
        Ok(config)
    }

    pub(crate) fn maintenance_status(&self) -> Result<MaintenanceStatus, Box<ManifestError>> {
        let value = self
            .document
            .find_entry_as_str(&["badges", "maintenance", "status"])?;
        MaintenanceStatus::from_str(value.value).map_err(|strum::ParseError::VariantNotFound| {
            InvalidValueSnafu {
                kind: "maintenance status",
                value: value.value,
                span: value.source_span(),
                source_code: self.document.named_source(),
            }
            .build()
            .into()
        })
    }
}

#[derive(Debug, Snafu, Diagnostic)]
pub(crate) enum ManifestError {
    #[snafu(transparent)]
    ReadManifest {
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    Toml {
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<TomlError>,
    },
    #[snafu(display("invalid value for {kind}: {value}"))]
    InvalidValue {
        kind: String,
        value: String,
        #[label]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
    },
}

impl Borrow<dyn Diagnostic> for Box<ManifestError> {
    fn borrow(&self) -> &(dyn Diagnostic + 'static) {
        &**self
    }
}

impl From<io::Error> for Box<ManifestError> {
    fn from(source: io::Error) -> Self {
        Box::new(source.into())
    }
}

impl From<Box<TomlError>> for Box<ManifestError> {
    fn from(source: Box<TomlError>) -> Self {
        Box::new(source.into())
    }
}

impl ManifestError {
    #[cfg(test)]
    #[track_caller]
    pub(crate) fn into_toml(self) -> TomlError {
        let ManifestError::Toml { source } = self else {
            panic!("unexpected error: {self:?}");
        };
        *source
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn into_invalid_value(self) -> (String, String, SourceSpan, NamedSource<Arc<str>>) {
        let ManifestError::InvalidValue {
            kind,
            value,
            span,
            source_code,
        } = self
        else {
            panic!("unexpected error: {self:?}");
        };
        (kind, value, span, source_code)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, IntoStaticStr, EnumString)]
#[strum(serialize_all = "kebab-case")]
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
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use similar_asserts::assert_eq;

    use crate::source;

    use super::*;

    #[test]
    fn maintenance_status_returns_parsed_value() {
        let source = indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"

            [badges]
            maintenance = { status = "actively-developed" }
        "#};
        let source = SourceFile::new_for_test("Cargo.toml", source);
        let manifest = Manifest::new_for_test(&source).unwrap();
        let status = manifest.maintenance_status().unwrap();
        assert_eq!(status, MaintenanceStatus::ActivelyDeveloped);
    }

    #[test]
    fn maintenance_status_returns_error_when_key_missing() {
        let source = indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"
        "#};
        let source = SourceFile::new_for_test("Cargo.toml", source);
        let manifest = Manifest::new_for_test(&source).unwrap();
        let (key, span, source_code) = manifest
            .maintenance_status()
            .unwrap_err()
            .into_toml()
            .into_missing_top_level_key();
        assert_eq!(key, "badges");
        source.assert_source_span(span, "");
        assert_eq!(source_code.name(), "Cargo.toml");

        let source = indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"

            [badges]
        "#};
        let source = SourceFile::new_for_test("Cargo.toml", source);
        let manifest = Manifest::new_for_test(&source).unwrap();
        let (key, table, span, source_code) = manifest
            .maintenance_status()
            .unwrap_err()
            .into_toml()
            .into_missing_key_in_table();
        assert_eq!(key, "maintenance");
        assert_eq!(source::render_toml_path(&table), "badges");
        source.assert_source_span(span, "badges");
        assert_eq!(source_code.name(), "Cargo.toml");

        let source = indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"

            [badges]
            maintenance = {}
        "#};
        let source = SourceFile::new_for_test("Cargo.toml", source);
        let manifest = Manifest::new_for_test(&source).unwrap();
        let (key, table, span, source_code) = manifest
            .maintenance_status()
            .unwrap_err()
            .into_toml()
            .into_missing_key_in_table();
        assert_eq!(key, "status");
        assert_eq!(source::render_toml_path(&table), "badges.maintenance");
        source.assert_source_span(span, "maintenance");
        assert_eq!(source_code.name(), "Cargo.toml");
    }

    #[test]
    fn maintenance_status_returns_error_on_invalid_value() {
        let source = indoc! {r#"
            [package]
            name = "test"
            version = "0.1.0"

            [badges]
            maintenance = { status = "invalid" }
        "#};
        let source = SourceFile::new_for_test("Cargo.toml", source);
        let manifest = Manifest::new_for_test(&source).unwrap();
        let (kind, value, span, source_code) = manifest
            .maintenance_status()
            .unwrap_err()
            .into_invalid_value();
        assert_eq!(kind, "maintenance status");
        assert_eq!(value, "invalid");
        source.assert_source_span(span, r#""invalid""#);
        assert_eq!(source_code.name(), "Cargo.toml");
    }
}
