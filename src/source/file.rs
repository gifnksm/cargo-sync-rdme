#[cfg(test)]
use std::range::Range;
use std::{
    cell::RefCell,
    fs,
    io::{self, Write as _},
    sync::Arc,
};

use cargo_metadata::{
    Metadata, Package,
    camino::{Utf8Path, Utf8PathBuf},
};
use miette::{Diagnostic, NamedSource, SourceOffset, SourceSpan};
use serde::de::{self, Deserialize};
use snafu::Snafu;
use tempfile::NamedTempFile;

#[cfg(test)]
use crate::source::Spanned;
use crate::{
    source::toml::{TomlDocument, TomlError},
    traits::PackageExt as _,
};

#[derive(Debug, Clone)]
pub(crate) struct SourceFilePath {
    pub(crate) workspace_path: Utf8PathBuf,
}

impl From<&SourceFileLoader> for SourceFilePath {
    fn from(loader: &SourceFileLoader) -> Self {
        Self {
            workspace_path: loader.workspace_path.clone(),
        }
    }
}

impl From<&SourceFile> for SourceFilePath {
    fn from(file: &SourceFile) -> Self {
        Self {
            workspace_path: file.workspace_path.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SourceFileLoader {
    workspace_path: Utf8PathBuf,
    path: Utf8PathBuf,
}

impl SourceFileLoader {
    pub(crate) fn from_package_relative_path(
        workspace: &Metadata,
        package: &Package,
        package_relative_path: &Utf8Path,
    ) -> Self {
        let workspace_path = package
            .workspace_relative_root_directory(workspace)
            .join(package_relative_path);
        let path = workspace.workspace_root.join(&workspace_path);
        Self {
            workspace_path,
            path,
        }
    }

    pub(crate) fn from_path(workspace: &Metadata, path: &Utf8Path) -> Self {
        let workspace_path = path
            .strip_prefix(&workspace.workspace_root)
            .unwrap_or(path)
            .into();
        let path = path.into();
        Self {
            workspace_path,
            path,
        }
    }

    pub(crate) fn workspace_path(&self) -> &Utf8Path {
        self.workspace_path.as_ref()
    }

    pub(crate) fn load(&self) -> io::Result<SourceFile> {
        let text = fs::read_to_string(&self.path)?.into();
        Ok(SourceFile {
            workspace_path: self.workspace_path.clone(),
            path: self.path.clone(),
            text,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SourceFile {
    workspace_path: Utf8PathBuf,
    path: Utf8PathBuf,
    text: Arc<str>,
}

#[derive(Debug, Snafu, Diagnostic)]
#[snafu(display("JSON deserialize error: {message}"))]
pub(crate) struct DeserializeAsJsonError {
    message: String,
    #[source_code]
    source_code: NamedSource<Arc<str>>,
    #[label]
    label: SourceSpan,
}

impl SourceFile {
    #[cfg(test)]
    pub(crate) fn new_for_test<P, T>(workspace_path: P, text: T) -> Self
    where
        P: Into<Utf8PathBuf>,
        T: Into<String>,
    {
        let workspace_path = workspace_path.into();
        let path = Utf8Path::new("/path/to/workspace").join(&workspace_path);
        let text = Arc::<str>::from(text.into());
        Self {
            workspace_path,
            path,
            text,
        }
    }

    pub(crate) fn path(&self) -> &Utf8Path {
        self.path.as_ref()
    }

    pub(crate) fn text(&self) -> &str {
        self.text.as_ref()
    }

    pub(crate) fn to_named_source(&self) -> NamedSource<Arc<str>> {
        NamedSource::new(&self.workspace_path, Arc::clone(&self.text))
    }

    pub(crate) fn replace_file_content(&mut self, new_text: Arc<str>) -> io::Result<()> {
        let output_dir = self.path.parent().unwrap();
        let mut tempfile = NamedTempFile::new_in(output_dir)?;
        tempfile.as_file_mut().write_all(new_text.as_bytes())?;
        tempfile.as_file_mut().sync_data()?;
        let file = tempfile.persist(&self.path).map_err(|err| err.error)?;
        file.sync_all()?;
        drop(file);
        self.text = new_text;
        Ok(())
    }

    pub(crate) fn deserialize_as_json<'a, T>(&'a self) -> Result<T, DeserializeAsJsonError>
    where
        T: Deserialize<'a>,
    {
        serde_json::from_str(self.text()).map_err(|err| {
            let message = err.to_string();
            let source_code = self.to_named_source().with_language("json");
            let offset = SourceOffset::from_location(&self.text, err.line(), err.column());
            let label = SourceSpan::new(offset, 1);
            DeserializeAsJsonSnafu {
                message,
                source_code,
                label,
            }
            .build()
        })
    }

    pub(crate) fn parse_as_toml(&self) -> Result<TomlDocument, Box<TomlError>> {
        let _reset = set_current_source_file(self.clone());
        TomlDocument::parse(self.clone())
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn assert_span(&self, span: Range<usize>, expected: &str) {
        let actual = &self.text[span];
        similar_asserts::assert_eq!(actual, expected);
    }

    #[cfg(test)]
    #[track_caller]
    #[expect(clippy::needless_pass_by_value)]
    pub(crate) fn assert_spanned<T>(&self, target: Spanned<T>, expected: &str) {
        self.assert_span(target.span, expected);
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn assert_source_span(&self, source_span: SourceSpan, expected: &str) {
        let actual = &self.text[source_span.offset()..][..source_span.len()];
        similar_asserts::assert_eq!(actual, expected);
    }
}

thread_local! {
    static CURRENT_SOURCE_FILE: RefCell<Option<SourceFile>> = const { RefCell::new(None) };
}

pub(crate) fn current_source_file<E>() -> Result<SourceFile, E>
where
    E: de::Error,
{
    CURRENT_SOURCE_FILE
        .with(|cell| cell.borrow().clone())
        .ok_or_else(|| {
            E::custom(
                "no active TOML deserialization context found. source file information is not available.",
            )
        })
}

pub(in crate::source) fn set_current_source_file(file: SourceFile) -> Reset {
    let old = CURRENT_SOURCE_FILE.with(|cell| cell.borrow_mut().replace(file));
    Reset { old }
}

pub(in crate::source) struct Reset {
    old: Option<SourceFile>,
}
impl Drop for Reset {
    fn drop(&mut self) {
        CURRENT_SOURCE_FILE.with(|cell| {
            *cell.borrow_mut() = self.old.take();
        });
    }
}
