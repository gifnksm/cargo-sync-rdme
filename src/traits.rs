use std::str::FromStr;

use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    Metadata, Package,
};
use miette::SourceSpan;

/// Extension methods for [`cargo_metadata::Metadata`].
pub(crate) trait MetadataExt {
    /// Returns the workspace package with given name.
    fn workspace_package_by_name<'a>(&'a self, name: &str) -> Option<&'a Package>;

    /// Returns the workspace package with given path.
    ///
    /// `path` must be absolute and normalized.
    fn workspace_package_by_path(&'_ self, path: impl AsRef<Utf8Path>) -> Option<&'_ Package>;
}

impl MetadataExt for Metadata {
    fn workspace_package_by_name<'a>(&'a self, name: &str) -> Option<&'a Package> {
        self.workspace_packages()
            .into_iter()
            .find(|p| p.name == name)
    }

    fn workspace_package_by_path(&'_ self, path: impl AsRef<Utf8Path>) -> Option<&'_ Package> {
        let path = path.as_ref();
        assert!(path.is_absolute());
        self.workspace_packages()
            .into_iter()
            .filter(|package| path.starts_with(package.root_directory()))
            .max_by_key(|p| p.manifest_path.as_str().len())
    }
}

/// Extension methods for [`cargo_metadata::Package`].
pub(crate) trait PackageExt {
    /// Returns the package root directory.
    fn root_directory(&self) -> &Utf8Path;

    fn readme_path(&self) -> Option<Utf8PathBuf>;
}

impl PackageExt for Package {
    fn root_directory(&self) -> &Utf8Path {
        // `manifest_path` is the path to the manifest file, so parent must exist.
        self.manifest_path.parent().unwrap()
    }

    fn readme_path(&self) -> Option<Utf8PathBuf> {
        // waiting for release: https://github.com/oli-obk/cargo_metadata/issues/201
        self.readme
            .as_ref()
            .map(|name| self.root_directory().join(name))
    }
}

pub(crate) trait StrSpanExt: Sized {
    fn trim(&self) -> Self {
        self.trim_start().trim_end()
    }
    fn trim_start(&self) -> Self;
    fn trim_end(&self) -> Self;
    fn strip_prefix_str(&self, prefix: &str) -> Option<Self>;
    fn strip_suffix_str(&self, suffix: &str) -> Option<Self>;
    fn split_once_fn(&self, f: impl Fn(char) -> bool) -> Option<(Self, Self)>;
    fn parse<F>(&self) -> Result<(F, SourceSpan), (F::Err, SourceSpan)>
    where
        F: FromStr;
}

mod imp {
    use super::*;

    fn new(s: &str, offset: usize) -> (&str, SourceSpan) {
        (s, (offset, s.len()).into())
    }

    fn same_end<'a>(original: (&str, SourceSpan), trimmed: &'a str) -> (&'a str, SourceSpan) {
        let new_offset = original.1.offset() + (original.0.len() - trimmed.len());
        new(trimmed, new_offset)
    }

    fn same_start<'a>(original: (&str, SourceSpan), trimmed: &'a str) -> (&'a str, SourceSpan) {
        let new_offset = original.1.offset();
        new(trimmed, new_offset)
    }

    impl StrSpanExt for (&str, SourceSpan) {
        fn trim_start(&self) -> Self {
            same_end(*self, self.0.trim_start())
        }

        fn trim_end(&self) -> Self {
            same_start(*self, self.0.trim_end())
        }

        fn strip_prefix_str(&self, prefix: &str) -> Option<Self> {
            Some(same_end(*self, self.0.strip_prefix(prefix)?))
        }

        fn strip_suffix_str(&self, suffix: &str) -> Option<Self> {
            Some(same_start(*self, self.0.strip_suffix(suffix)?))
        }

        fn split_once_fn(&self, f: impl Fn(char) -> bool) -> Option<(Self, Self)> {
            let (head, tail) = self.0.split_once(f)?;
            Some((same_start(*self, head), same_end(*self, tail)))
        }

        fn parse<F>(&self) -> Result<(F, SourceSpan), (F::Err, SourceSpan)>
        where
            F: FromStr,
        {
            self.0.parse().map(|x| (x, self.1)).map_err(|e| (e, self.1))
        }
    }
}
