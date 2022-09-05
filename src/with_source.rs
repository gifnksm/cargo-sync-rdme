use std::{fs, io, rc::Rc};

use cargo_metadata::camino::Utf8PathBuf;
use miette::{NamedSource, SourceOffset, SourceSpan};
use serde::Deserialize;

use ::toml::Spanned;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum ReadFileError {
    #[error("failed to read {name}: {path}")]
    Io {
        name: String,
        path: Utf8PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse {name}")]
    ParseToml {
        name: String,
        #[source]
        source: toml::de::Error,
        #[source_code]
        source_code: NamedSource,
        #[label]
        label: Option<SourceSpan>,
    },
    #[error("failed to parse {name}")]
    ParseJson {
        name: String,
        #[source]
        source: serde_json::Error,
        #[source_code]
        source_code: NamedSource,
        #[label]
        label: SourceSpan,
    },
}

#[derive(Debug, Clone)]
struct SourceInfo {
    name: String,
    path: Utf8PathBuf,
    text: String,
}

impl SourceInfo {
    fn open(name: impl Into<String>, path: impl Into<Utf8PathBuf>) -> Result<Self, ReadFileError> {
        let name = name.into();
        let path = path.into();
        let text = fs::read_to_string(&path).map_err(|err| ReadFileError::Io {
            name: name.clone(),
            path: path.clone(),
            source: err,
        })?;

        Ok(Self { name, path, text })
    }

    pub(crate) fn to_named_source(&self) -> NamedSource {
        NamedSource::new(&self.path, self.text.clone())
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

        let value: T = toml::from_str(&source_info.text).map_err(|err| {
            let label = err.line_col().map(|(line, col)| {
                let offset = SourceOffset::from_location(&source_info.text, line + 1, col + 1);
                SourceSpan::new(offset, SourceOffset::from(1))
            });
            let source_code = source_info.to_named_source();
            ReadFileError::ParseToml {
                name: source_info.name.clone(),
                source: err,
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

        let value: T = serde_json::from_str(&source_info.text).map_err(|err| {
            let offset = SourceOffset::from_location(&source_info.text, err.line(), err.column());
            let label = SourceSpan::new(offset, SourceOffset::from(1));
            let source_code = source_info.to_named_source();
            ReadFileError::ParseJson {
                name: source_info.name.clone(),
                source: err,
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

    pub(crate) fn to_named_source(&self) -> NamedSource {
        self.source_info.to_named_source()
    }

    pub(crate) fn map<U>(&self, f: impl FnOnce(&T) -> U) -> WithSource<U> {
        WithSource {
            source_info: Rc::clone(&self.source_info),
            value: f(&self.value),
        }
    }
}

impl<T> WithSource<&'_ Spanned<T>> {
    pub(crate) fn span(&self) -> SourceSpan {
        SourceSpan::from(self.value().start()..self.value().end())
    }
}
