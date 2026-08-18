use std::{fs, io, rc::Rc, sync::Arc};

use cargo_metadata::camino::Utf8PathBuf;
use miette::{NamedSource, SourceOffset, SourceSpan};

use serde::Deserialize;

use snafu::{ResultExt as _, Snafu};
use toml::Spanned;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum ReadFileError {
    #[snafu(display("failed to read {name}: {path}"))]
    Io {
        name: String,
        path: Utf8PathBuf,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to parse {name}"))]
    ParseToml {
        name: String,
        #[snafu(source(from(toml::de::Error, Box::new)))]
        source: Box<toml::de::Error>,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
        #[label]
        label: Option<SourceSpan>,
    },
    #[snafu(display("failed to parse {name}"))]
    ParseJson {
        name: String,
        #[snafu(source(from(serde_json::Error, Box::new)))]
        source: Box<serde_json::Error>,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
        #[label]
        label: SourceSpan,
    },
}

#[derive(Debug, Clone)]
struct SourceInfo {
    name: String,
    path: Utf8PathBuf,
    text: Arc<str>,
}

impl SourceInfo {
    fn open(name: impl Into<String>, path: impl Into<Utf8PathBuf>) -> Result<Self, ReadFileError> {
        let name = name.into();
        let path = path.into();
        let text = fs::read_to_string(&path)
            .with_context(|_source| IoSnafu {
                name: name.clone(),
                path: path.clone(),
            })?
            .into();
        Ok(Self { name, path, text })
    }

    pub(crate) fn to_named_source(&self) -> NamedSource<Arc<str>> {
        NamedSource::new(&self.path, Arc::clone(&self.text))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WithSource<T> {
    source_info: Rc<SourceInfo>,
    value: T,
}

impl<T> WithSource<T> {
    pub(crate) fn from_toml(
        name: impl Into<String>,
        path: impl Into<Utf8PathBuf>,
    ) -> Result<Self, ReadFileError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let source_info = SourceInfo::open(name, path)?;

        let value: T = toml::from_str(&source_info.text).with_context(|source| {
            let label = source.span().map(SourceSpan::from);
            let source_code = source_info.to_named_source();
            ParseTomlSnafu {
                name: source_info.name.clone(),
                source_code,
                label,
            }
        })?;

        let source_info = Rc::new(source_info);
        Ok(Self { source_info, value })
    }

    pub(crate) fn from_json(
        name: impl Into<String>,
        path: impl Into<Utf8PathBuf>,
    ) -> Result<Self, ReadFileError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let source_info = SourceInfo::open(name, path)?;

        let value: T = serde_json::from_str(&source_info.text).with_context(|source| {
            let offset =
                SourceOffset::from_location(&source_info.text, source.line(), source.column());
            let label = SourceSpan::new(offset, 1);
            let source_code = source_info.to_named_source();
            ParseJsonSnafu {
                name: source_info.name.clone(),
                source_code,
                label,
            }
        })?;

        let source_info = Rc::new(source_info);
        Ok(Self { source_info, value })
    }

    pub(crate) fn name(&self) -> &str {
        &self.source_info.name
    }

    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    pub(crate) fn into_value(self) -> T {
        self.value
    }

    pub(crate) fn to_named_source(&self) -> NamedSource<Arc<str>> {
        self.source_info.to_named_source()
    }

    pub(crate) fn map<U>(&self, f: impl FnOnce(&T) -> U) -> WithSource<U> {
        WithSource {
            source_info: Rc::clone(&self.source_info),
            value: f(&self.value),
        }
    }

    #[cfg(test)]
    pub(crate) fn dummy(value: T) -> Self {
        Self {
            source_info: Rc::new(SourceInfo {
                name: "dummy".to_string(),
                path: Utf8PathBuf::from("dummy"),
                text: "dummy".into(),
            }),
            value,
        }
    }
}

impl<T> WithSource<&'_ Spanned<T>> {
    pub(crate) fn span(&self) -> SourceSpan {
        SourceSpan::from(self.value().span())
    }
}
