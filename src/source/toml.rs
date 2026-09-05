use std::{borrow::Borrow, fmt, sync::Arc};

use miette::{Diagnostic, NamedSource, SourceSpan};
use self_cell::self_cell;
use serde::Deserialize;
use snafu::Snafu;
use strum::IntoStaticStr;
use toml::de;

use crate::source::{SourceFile, Spanned, file};

#[derive(Debug)]
pub(crate) struct TomlDocument {
    inner: Inner,
}

type Owner = SourceFile;
type Dependent<'a> = toml::Spanned<de::DeTable<'a>>;

self_cell! {
    struct Inner {
        owner: Owner,
        #[covariant]
        dependent: Dependent,
    }

    impl { Debug }
}

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum TomlError {
    #[snafu(display("{message}"))]
    ParseToml {
        message: String,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
        #[label]
        label: Option<SourceSpan>,
    },
    #[snafu(display("{message}"))]
    DeserializeToml {
        message: String,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
        #[label]
        label: Option<SourceSpan>,
    },
    #[snafu(display("missing top-level key `{key}`"))]
    MissingTopLevelKey {
        key: String,
        #[label]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
    },
    #[snafu(display("missing key `{key}` in table `{table}`", table = render_toml_path(table)))]
    MissingKeyInTable {
        key: String,
        table: Vec<String>,
        #[label]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
    },
    #[snafu(display("unexpected value type, expected {expected}, got {actual}: {path}", path = render_toml_path(path)))]
    UnexpectedValueType {
        path: Vec<String>,
        expected: ValueType,
        actual: ValueType,
        #[label]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
    },
}

impl Borrow<dyn Diagnostic> for Box<TomlError> {
    fn borrow(&self) -> &(dyn Diagnostic + 'static) {
        &**self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum ValueType {
    String,
    Integer,
    Float,
    Boolean,
    Datetime,
    Array,
    Table,
}

impl ValueType {
    pub(crate) fn of(value: &de::DeValue<'_>) -> Self {
        match value {
            de::DeValue::String(_) => Self::String,
            de::DeValue::Integer(_) => Self::Integer,
            de::DeValue::Float(_) => Self::Float,
            de::DeValue::Boolean(_) => Self::Boolean,
            de::DeValue::Datetime(_) => Self::Datetime,
            de::DeValue::Array(_) => Self::Array,
            de::DeValue::Table(_) => Self::Table,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        self.into()
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

pub(crate) fn render_toml_path(path: &[String]) -> String {
    path.join(".")
}

impl TomlError {
    #[cfg(test)]
    #[track_caller]
    pub(crate) fn into_parse_toml(self) -> (String, Option<SourceSpan>, NamedSource<Arc<str>>) {
        let Self::ParseToml {
            message,
            label,
            source_code,
        } = self
        else {
            panic!("unexpected error: {self:?}");
        };
        (message, label, source_code)
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn into_deserialize_toml(
        self,
    ) -> (String, Option<SourceSpan>, NamedSource<Arc<str>>) {
        let Self::DeserializeToml {
            message,
            label,
            source_code,
        } = self
        else {
            panic!("unexpected error: {self:?}");
        };
        (message, label, source_code)
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn into_missing_top_level_key(self) -> (String, SourceSpan, NamedSource<Arc<str>>) {
        let Self::MissingTopLevelKey {
            key,
            span,
            source_code,
        } = self
        else {
            panic!("unexpected error: {self:?}");
        };
        (key, span, source_code)
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn into_missing_key_in_table(
        self,
    ) -> (String, Vec<String>, SourceSpan, NamedSource<Arc<str>>) {
        let Self::MissingKeyInTable {
            key,
            table,
            span,
            source_code,
        } = self
        else {
            panic!("unexpected error: {self:?}");
        };
        (key, table, span, source_code)
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn into_unexpected_value_type(
        self,
    ) -> (
        Vec<String>,
        ValueType,
        ValueType,
        SourceSpan,
        NamedSource<Arc<str>>,
    ) {
        let Self::UnexpectedValueType {
            path,
            expected,
            actual,
            span,
            source_code,
        } = self
        else {
            panic!("unexpected error: {self:?}");
        };
        (path, expected, actual, span, source_code)
    }
}

pub(crate) trait ParseTomlResultExt {
    type Item;
    fn ignore_missing_key_error(self) -> Result<Option<Self::Item>, Box<TomlError>>;
}

impl<T> ParseTomlResultExt for Result<T, Box<TomlError>> {
    type Item = T;

    fn ignore_missing_key_error(self) -> Result<Option<Self::Item>, Box<TomlError>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(err) => match &*err {
                TomlError::MissingTopLevelKey { .. } | TomlError::MissingKeyInTable { .. } => {
                    Ok(None)
                }
                _ => Err(err),
            },
        }
    }
}

fn build_toml_path(path: &[&str]) -> Vec<String> {
    path.iter().copied().map(ToOwned::to_owned).collect()
}

impl TomlDocument {
    pub(crate) fn parse(source: SourceFile) -> Result<Self, Box<TomlError>> {
        Ok(Self {
            inner: Inner::try_new(source, |source| {
                de::DeTable::parse(source.text()).map_err(|err| {
                    let message = err.message();
                    let source_code = source.to_named_source().with_language("toml");
                    let label = err.span().map(SourceSpan::from);
                    Box::new(
                        ParseTomlSnafu {
                            message,
                            source_code,
                            label,
                        }
                        .build(),
                    )
                })
            })?,
        })
    }

    pub(crate) fn named_source(&self) -> NamedSource<Arc<str>> {
        self.inner
            .borrow_owner()
            .to_named_source()
            .with_language("toml")
    }

    pub(crate) fn source_file(&self) -> &SourceFile {
        self.inner.borrow_owner()
    }

    fn document(&self) -> Spanned<&de::DeTable<'_>> {
        self.inner.borrow_dependent().into()
    }

    fn deserialize_toml_error(&self, err: &de::Error) -> TomlError {
        let message = err.message();
        let source_code = self.named_source();
        let label = err.span().map(SourceSpan::from);
        DeserializeTomlSnafu {
            message,
            source_code,
            label,
        }
        .build()
    }

    fn missing_top_level_key_error(&self, key: &str) -> TomlError {
        MissingTopLevelKeySnafu {
            key,
            span: self.document().source_span(),
            source_code: self.named_source(),
        }
        .build()
    }

    fn missing_key_in_table_error(
        &self,
        key: &str,
        table_path: &[&str],
        value_key: Spanned<&de::DeString<'_>>,
    ) -> TomlError {
        MissingKeyInTableSnafu {
            key,
            table: build_toml_path(table_path),
            span: value_key.source_span(),
            source_code: self.named_source(),
        }
        .build()
    }

    fn unexpected_value_type_error(
        &self,
        path: &[&str],
        expected: ValueType,
        actual: Spanned<&de::DeValue<'_>>,
    ) -> TomlError {
        UnexpectedValueTypeSnafu {
            path: build_toml_path(path),
            expected,
            actual: ValueType::of(actual.value),
            span: actual.source_span(),
            source_code: self.named_source(),
        }
        .build()
    }

    fn value_as_table<'a>(
        &self,
        value: Spanned<&'a de::DeValue<'a>>,
        path: &[&str],
    ) -> Result<Spanned<&'a de::DeTable<'a>>, Box<TomlError>> {
        let table = value
            .value
            .as_table()
            .ok_or_else(|| self.unexpected_value_type_error(path, ValueType::Table, value))?;
        Ok(Spanned {
            span: value.span,
            value: table,
        })
    }

    fn value_as_str<'a>(
        &self,
        value: Spanned<&'a de::DeValue<'a>>,
        path: &[&str],
    ) -> Result<Spanned<&'a str>, Box<TomlError>> {
        let s = value
            .value
            .as_str()
            .ok_or_else(|| self.unexpected_value_type_error(path, ValueType::String, value))?;
        Ok(Spanned {
            span: value.span,
            value: s,
        })
    }

    pub(crate) fn find_entry<'a>(
        &'a self,
        path: &[&str],
    ) -> Result<Spanned<&'a de::DeValue<'a>>, Box<TomlError>> {
        assert!(!path.is_empty());
        let (&head, tail) = path.split_first().unwrap();
        let document = self.document();
        let kv = document
            .value
            .get_key_value(head)
            .ok_or_else(|| self.missing_top_level_key_error(head))?;
        let mut value_key: Spanned<&de::DeString<'_>> = kv.0.into();
        let mut value: Spanned<&de::DeValue<'_>> = kv.1.into();
        for (i, key) in tail.iter().copied().enumerate() {
            let table_path = &path[..=i];
            let kv = self
                .value_as_table(value, table_path)?
                .value
                .get_key_value(key)
                .ok_or_else(|| self.missing_key_in_table_error(key, table_path, value_key))?;
            value_key = kv.0.into();
            value = kv.1.into();
        }
        Ok(value)
    }

    pub(crate) fn find_entry_as_str<'a>(
        &'a self,
        path: &[&str],
    ) -> Result<Spanned<&'a str>, Box<TomlError>> {
        let value = self.find_entry(path)?;
        self.value_as_str(value, path)
    }

    pub(crate) fn find_entry_as_table<'a>(
        &'a self,
        path: &[&str],
    ) -> Result<Spanned<&'a de::DeTable<'a>>, Box<TomlError>> {
        let value = self.find_entry(path)?;
        self.value_as_table(value, path)
    }

    pub(crate) fn deserialize_entry<'a, T>(&'a self, path: &[&str]) -> Result<T, Box<TomlError>>
    where
        T: Deserialize<'a>,
    {
        let _reset = file::set_current_source_file(self.source_file().clone());
        let value = self.find_entry_as_table(path)?;
        let deserializer =
            de::Deserializer::from(toml::Spanned::new(value.span.into(), value.value.clone()));
        let value =
            T::deserialize(deserializer).map_err(|err| self.deserialize_toml_error(&err))?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use similar_asserts::assert_eq;

    use super::*;

    #[test]
    fn find_entry_returns_value_with_span() {
        let source = SourceFile::new_for_test(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "my_package"
                version = "0.1.0"

                [package.metadata.cargo-sync-rdme]
                extra-targets = "foo.md"
            "#},
        );
        let doc = TomlDocument::parse(source.clone()).unwrap();

        let entry = doc.find_entry(&["package", "name"]).unwrap();
        assert_eq!(entry.value.as_str().unwrap(), "my_package");
        source.assert_spanned(entry, r#""my_package""#);

        let entry = doc.find_entry(&["package", "version"]).unwrap();
        assert_eq!(entry.value.as_str().unwrap(), "0.1.0");
        source.assert_spanned(entry, r#""0.1.0""#);

        let entry = doc.find_entry(&["package", "metadata"]).unwrap();
        entry.value.as_table().unwrap();
        source.assert_spanned(entry, "metadata");

        let entry = doc
            .find_entry(&["package", "metadata", "cargo-sync-rdme"])
            .unwrap();
        entry.value.as_table().unwrap();
        source.assert_spanned(entry, "[package.metadata.cargo-sync-rdme]");

        let entry = doc
            .find_entry(&["package", "metadata", "cargo-sync-rdme", "extra-targets"])
            .unwrap();
        assert_eq!(entry.value.as_str().unwrap(), "foo.md");
        source.assert_spanned(entry, r#""foo.md""#);
    }

    #[test]
    fn find_entry_returns_error_for_missing_top_level_key() {
        let source = SourceFile::new_for_test(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "my_package"
            "#},
        );
        let doc = TomlDocument::parse(source.clone()).unwrap();

        let err = doc.find_entry(&["dependencies"]).unwrap_err();
        let (key, span, source_code) = err.into_missing_top_level_key();
        assert_eq!(key, "dependencies");
        source.assert_source_span(span, "");
        assert_eq!(source_code.name(), "Cargo.toml");
    }

    #[test]
    fn find_entry_returns_error_for_not_a_table() {
        let source = SourceFile::new_for_test(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "my_package"
                version = "0.1.0"

                [package.metadata.cargo-sync-rdme]
                extra-targets = "foo.md"

                [package.metadata.cargo-sync-rdme.badge]
                style = "flat-square"
                badges = {}
            "#},
        );
        let doc = TomlDocument::parse(source.clone()).unwrap();

        let err = doc.find_entry(&["package", "name", "foo"]).unwrap_err();
        let (path, expected, actual, span, source_code) = err.into_unexpected_value_type();
        assert_eq!(render_toml_path(&path), "package.name");
        assert_eq!(expected, ValueType::Table);
        assert_eq!(actual, ValueType::String);
        source.assert_source_span(span, r#""my_package""#);
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc
            .find_entry(&[
                "package",
                "metadata",
                "cargo-sync-rdme",
                "extra-targets",
                "foo",
            ])
            .unwrap_err();
        let (path, expected, actual, span, source_code) = err.into_unexpected_value_type();
        assert_eq!(
            render_toml_path(&path),
            "package.metadata.cargo-sync-rdme.extra-targets"
        );
        assert_eq!(expected, ValueType::Table);
        assert_eq!(actual, ValueType::String);
        source.assert_source_span(span, r#""foo.md""#);
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc
            .find_entry(&[
                "package",
                "metadata",
                "cargo-sync-rdme",
                "badge",
                "style",
                "foo",
            ])
            .unwrap_err();
        let (path, expected, actual, span, source_code) = err.into_unexpected_value_type();
        assert_eq!(
            render_toml_path(&path),
            "package.metadata.cargo-sync-rdme.badge.style"
        );
        assert_eq!(expected, ValueType::Table);
        assert_eq!(actual, ValueType::String);
        source.assert_source_span(span, r#""flat-square""#);
        assert_eq!(source_code.name(), "Cargo.toml");
    }

    #[test]
    fn find_entry_returns_error_for_missing_key_in_table() {
        let source = SourceFile::new_for_test(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "my_package"

                [package.metadata.cargo-sync-rdme]
                extra-targets = "foo.md"

                [package.metadata.cargo-sync-rdme.badge]
                style = "flat-square"
                badges = {}
            "#},
        );
        let doc = TomlDocument::parse(source.clone()).unwrap();

        let err = doc.find_entry(&["package", "version"]).unwrap_err();
        let (key, table, span, source_code) = err.into_missing_key_in_table();
        assert_eq!(key, "version");
        assert_eq!(render_toml_path(&table), "package");
        source.assert_source_span(span, "package");
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc.find_entry(&["package", "metadata", "foo"]).unwrap_err();
        let (key, table, span, source_code) = err.into_missing_key_in_table();
        assert_eq!(key, "foo");
        assert_eq!(render_toml_path(&table), "package.metadata");
        source.assert_source_span(span, "metadata");
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc
            .find_entry(&["package", "metadata", "cargo-sync-rdme", "foo"])
            .unwrap_err();
        let (key, table, span, source_code) = err.into_missing_key_in_table();
        assert_eq!(key, "foo");
        assert_eq!(render_toml_path(&table), "package.metadata.cargo-sync-rdme");
        source.assert_source_span(span, "cargo-sync-rdme");
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc
            .find_entry(&["package", "metadata", "cargo-sync-rdme", "badge", "foo"])
            .unwrap_err();
        let (key, table, span, source_code) = err.into_missing_key_in_table();
        assert_eq!(key, "foo");
        assert_eq!(
            render_toml_path(&table),
            "package.metadata.cargo-sync-rdme.badge"
        );
        source.assert_source_span(span, "badge");
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc
            .find_entry(&[
                "package",
                "metadata",
                "cargo-sync-rdme",
                "badge",
                "badges",
                "foo",
            ])
            .unwrap_err();
        let (key, table, span, source_code) = err.into_missing_key_in_table();
        assert_eq!(key, "foo");
        assert_eq!(
            render_toml_path(&table),
            "package.metadata.cargo-sync-rdme.badge.badges"
        );
        source.assert_source_span(span, "badges");
        assert_eq!(source_code.name(), "Cargo.toml");
    }

    #[test]
    fn find_entry_as_str_returns_value_with_span() {
        let source = SourceFile::new_for_test(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "my_package"
                version = "0.1.0"
            "#},
        );
        let doc = TomlDocument::parse(source.clone()).unwrap();

        let entry = doc.find_entry_as_str(&["package", "name"]).unwrap();
        assert_eq!(entry.value, "my_package");
        source.assert_spanned(entry, r#""my_package""#);

        let entry = doc.find_entry_as_str(&["package", "version"]).unwrap();
        assert_eq!(entry.value, "0.1.0");
        source.assert_spanned(entry, r#""0.1.0""#);
    }

    #[test]
    fn find_entry_as_str_returns_error_for_not_a_string() {
        let source = SourceFile::new_for_test(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "my_package"
                version = "0.1.0"

                [package.metadata.cargo-sync-rdme]
                extra-targets = "foo.md"

                [package.metadata.cargo-sync-rdme.badge]
                style = "flat-square"
                badges = {}
            "#},
        );
        let doc = TomlDocument::parse(source.clone()).unwrap();

        let err = doc.find_entry_as_str(&["package", "metadata"]).unwrap_err();
        let (path, expected, actual, span, source_code) = err.into_unexpected_value_type();
        assert_eq!(render_toml_path(&path), "package.metadata");
        assert_eq!(expected, ValueType::String);
        assert_eq!(actual, ValueType::Table);
        source.assert_source_span(span, "metadata");
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc
            .find_entry_as_str(&["package", "metadata", "cargo-sync-rdme"])
            .unwrap_err();
        let (path, expected, actual, span, source_code) = err.into_unexpected_value_type();
        assert_eq!(render_toml_path(&path), "package.metadata.cargo-sync-rdme");
        assert_eq!(expected, ValueType::String);
        assert_eq!(actual, ValueType::Table);
        source.assert_source_span(span, "[package.metadata.cargo-sync-rdme]");
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc
            .find_entry_as_str(&["package", "metadata", "cargo-sync-rdme", "badge"])
            .unwrap_err();
        let (path, expected, actual, span, source_code) = err.into_unexpected_value_type();
        assert_eq!(
            render_toml_path(&path),
            "package.metadata.cargo-sync-rdme.badge"
        );
        assert_eq!(expected, ValueType::String);
        assert_eq!(actual, ValueType::Table);
        source.assert_source_span(span, "[package.metadata.cargo-sync-rdme.badge]");
        assert_eq!(source_code.name(), "Cargo.toml");

        let err = doc
            .find_entry_as_str(&["package", "metadata", "cargo-sync-rdme", "badge", "badges"])
            .unwrap_err();
        let (path, expected, actual, span, source_code) = err.into_unexpected_value_type();
        assert_eq!(
            render_toml_path(&path),
            "package.metadata.cargo-sync-rdme.badge.badges"
        );
        assert_eq!(expected, ValueType::String);
        assert_eq!(actual, ValueType::Table);
        source.assert_source_span(span, "{}");
        assert_eq!(source_code.name(), "Cargo.toml");
    }
}
