use std::{
    borrow::Cow,
    fs,
    io::{self, Write as _},
    sync::Arc,
};

use cargo_metadata::{
    Metadata, Package, PackageName,
    camino::{Utf8Path, Utf8PathBuf},
};
use miette::NamedSource;
use serde::de::Deserialize;
use tempfile::NamedTempFile;

use crate::traits::PackageExt as _;

#[derive(Debug, Clone)]
pub(crate) struct PackageTextFileDisplayPath {
    pub(crate) package: PackageName,
    pub(crate) path: Utf8PathBuf,
}

impl From<&PackageTextFileLoader<'_>> for PackageTextFileDisplayPath {
    fn from(loader: &PackageTextFileLoader<'_>) -> Self {
        Self {
            package: loader.package.name.clone(),
            path: loader.workspace_relative_path.clone().into_owned(),
        }
    }
}

impl From<&PackageTextFile<'_>> for PackageTextFileDisplayPath {
    fn from(file: &PackageTextFile<'_>) -> Self {
        file.loader.into()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PackageTextFileLoader<'a> {
    package: &'a Package,
    workspace_relative_path: Cow<'a, Utf8Path>,
    path: Cow<'a, Utf8Path>,
}

impl<'a> PackageTextFileLoader<'a> {
    pub(crate) fn from_package_relative_path(
        workspace: &'a Metadata,
        package: &'a Package,
        package_relative_path: &'a Utf8Path,
    ) -> Self {
        let workspace_relative_path = package
            .workspace_relative_root_directory(workspace)
            .join(package_relative_path)
            .into();
        let path = workspace
            .workspace_root
            .join(&workspace_relative_path)
            .into();
        Self {
            package,
            workspace_relative_path,
            path,
        }
    }

    pub(crate) fn from_path(
        workspace: &'a Metadata,
        package: &'a Package,
        path: &'a Utf8Path,
    ) -> Self {
        let workspace_relative_path = path
            .strip_prefix(&workspace.workspace_root)
            .unwrap_or(path)
            .into();
        let path = path.into();
        Self {
            package,
            workspace_relative_path,
            path,
        }
    }

    pub(crate) fn load(&self) -> io::Result<PackageTextFile<'_>> {
        let text = fs::read_to_string(self.path.as_ref())?.into();
        Ok(PackageTextFile { loader: self, text })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PackageTextFile<'a> {
    loader: &'a PackageTextFileLoader<'a>,
    text: Arc<str>,
}

impl PackageTextFile<'_> {
    pub(crate) fn path(&self) -> &Utf8Path {
        self.loader.path.as_ref()
    }

    pub(crate) fn text(&self) -> &str {
        self.text.as_ref()
    }

    pub(crate) fn to_named_source(&self) -> NamedSource<Arc<str>> {
        NamedSource::new(
            self.loader.workspace_relative_path.as_ref(),
            Arc::clone(&self.text),
        )
    }

    pub(crate) fn replace_file_content(&mut self, new_text: Arc<str>) -> io::Result<()> {
        let output_dir = self.loader.path.parent().unwrap();
        let mut tempfile = NamedTempFile::new_in(output_dir)?;
        tempfile.as_file_mut().write_all(new_text.as_bytes())?;
        tempfile.as_file_mut().sync_data()?;
        let file = tempfile
            .persist(self.loader.path.as_ref())
            .map_err(|err| err.error)?;
        file.sync_all()?;
        drop(file);
        self.text = new_text;
        Ok(())
    }

    pub(crate) fn parse_as_json<'a, T>(&'a self) -> Result<T, serde_json::Error>
    where
        T: Deserialize<'a>,
    {
        serde_json::from_str(self.text())
    }
}
