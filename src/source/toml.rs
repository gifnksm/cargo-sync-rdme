use std::{borrow::Borrow, fmt, sync::Arc};

use miette::{Diagnostic, NamedSource, SourceSpan};
use self_cell::self_cell;
use snafu::{OptionExt as _, Snafu};
use toml::de;

use crate::source::{SourceFileRef, Spanned};

#[derive(Debug)]
pub(crate) struct TomlDocument {
    inner: Inner,
}

type Owner = SourceFileRef;
type Dependent<'a> = toml::Spanned<de::DeTable<'a>>;

self_cell! {
    struct Inner {
        owner: Owner,
        #[covariant]
        dependent: Dependent,
    }

    impl { Debug }
}

#[derive(Debug, Snafu, Diagnostic)]
#[snafu(display("TOML parse error: {message}"))]
pub(crate) struct ParseTomlError {
    pub(crate) message: String,
    #[source_code]
    pub(crate) source_code: NamedSource<Arc<str>>,
    #[label]
    pub(crate) label: Option<SourceSpan>,
}

impl Borrow<dyn Diagnostic> for Box<ParseTomlError> {
    fn borrow(&self) -> &(dyn Diagnostic + 'static) {
        &**self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::Datetime => "datetime",
            Self::Array => "array",
            Self::Table => "table",
        }
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

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum FindEntryError {
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

impl FindEntryError {
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

fn build_toml_path(path: &[&str]) -> Vec<String> {
    path.iter().copied().map(ToOwned::to_owned).collect()
}

impl TomlDocument {
    pub(crate) fn parse(source: SourceFileRef) -> Result<Self, Box<ParseTomlError>> {
        Ok(Self {
            inner: Inner::try_new(source, |source| {
                de::DeTable::parse(&source.text).map_err(|err| {
                    let message = err.message();
                    let source_code = source.to_named_source().with_language("toml");
                    let label = err.span().map(SourceSpan::from);
                    ParseTomlSnafu {
                        message,
                        source_code,
                        label,
                    }
                    .build()
                })
            })?,
        })
    }

    pub(crate) fn named_source(&self) -> NamedSource<Arc<str>> {
        self.inner.borrow_owner().to_named_source()
    }

    fn document(&self) -> Spanned<&de::DeTable<'_>> {
        self.inner.borrow_dependent().into()
    }

    pub(crate) fn find_entry<'a>(
        &'a self,
        path: &[&str],
    ) -> Result<Spanned<&'a de::DeValue<'a>>, Box<FindEntryError>> {
        assert!(!path.is_empty());
        let (&head, tail) = path.split_first().unwrap();
        let document = self.document();
        let kv = document
            .value
            .get_key_value(head)
            .with_context(|| MissingTopLevelKeySnafu {
                key: head,
                span: document.source_span(),
                source_code: self.named_source(),
            })?;
        let mut value_key: Spanned<&de::DeString<'_>> = kv.0.into();
        let mut value: Spanned<&de::DeValue<'_>> = kv.1.into();
        for (i, key) in tail.iter().copied().enumerate() {
            let table_path = &path[..=i];
            let kv = value
                .value
                .as_table()
                .with_context(|| UnexpectedValueTypeSnafu {
                    path: build_toml_path(table_path),
                    expected: ValueType::Table,
                    actual: ValueType::of(value.value),
                    span: value.source_span(),
                    source_code: self.named_source(),
                })?
                .get_key_value(key)
                .with_context(|| MissingKeyInTableSnafu {
                    key,
                    table: build_toml_path(table_path),
                    span: value_key.source_span(),
                    source_code: self.named_source(),
                })?;
            value_key = kv.0.into();
            value = kv.1.into();
        }
        Ok(value)
    }

    pub(crate) fn find_entry_as_str<'a>(
        &'a self,
        path: &[&str],
    ) -> Result<Spanned<&'a str>, Box<FindEntryError>> {
        let value = self.find_entry(path)?;
        let s = value
            .value
            .as_str()
            .with_context(|| UnexpectedValueTypeSnafu {
                path: build_toml_path(path),
                expected: ValueType::String,
                actual: ValueType::of(value.value),
                span: value.source_span(),
                source_code: self.named_source(),
            })?;
        Ok(Spanned {
            span: value.span,
            value: s,
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::source::SourceFile;

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
        let doc = TomlDocument::parse(source.to_source_file_ref()).unwrap();

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
        let doc = TomlDocument::parse(source.to_source_file_ref()).unwrap();

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
        let doc = TomlDocument::parse(source.to_source_file_ref()).unwrap();

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
        let doc = TomlDocument::parse(source.to_source_file_ref()).unwrap();

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
        let doc = TomlDocument::parse(source.to_source_file_ref()).unwrap();

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
        let doc = TomlDocument::parse(source.to_source_file_ref()).unwrap();

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
