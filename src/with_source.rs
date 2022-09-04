use std::rc::Rc;

use ::toml::Spanned;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use miette::{NamedSource, SourceSpan};
use serde::Deserialize;

use crate::toml::{self, ReadTomlError};

#[derive(Debug, Clone)]
struct SourceInfo {
    name: String,
    path: Utf8PathBuf,
    text: String,
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
    ) -> Result<Self, ReadTomlError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let name = name.into();
        let path = path.into();
        let (text, value) = toml::parse_with_text(&name, &path)?;
        let source_info = Rc::new(SourceInfo { name, path, text });
        Ok(Self { source_info, value })
    }

    pub(crate) fn name(&self) -> &str {
        &self.source_info.name
    }

    pub(crate) fn path(&self) -> &Utf8Path {
        &self.source_info.path
    }

    pub(crate) fn text(&self) -> &str {
        &self.source_info.text
    }

    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    pub(crate) fn to_named_source(&self) -> NamedSource {
        NamedSource::new(self.path(), self.text().to_owned())
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
