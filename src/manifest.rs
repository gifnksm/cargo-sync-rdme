use std::{borrow::Borrow, io, str::FromStr as _, sync::Arc};

use miette::{Diagnostic, NamedSource, SourceSpan};
use snafu::Snafu;
use strum::{EnumString, IntoStaticStr};

use crate::{
    config::Config,
    source::{ParseTomlResultExt as _, SourceFileLoader, TomlDocument, TomlError},
};

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
    pub(crate) fn new_for_test(
        source: &crate::source::SourceFile,
    ) -> Result<Self, Box<ManifestError>> {
        let document = source.parse_as_toml()?;
        Ok(Self { document })
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

    use crate::source::{self, SourceFile};

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
