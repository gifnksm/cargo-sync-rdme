use std::{
    ffi::OsString,
    process::Command,
    range::{Range, legacy},
};

use cargo_metadata::{Metadata, Package, camino::Utf8Path};
use miette::SourceSpan;

/// Extension methods for [`cargo_metadata::Package`].
pub(crate) trait PackageExt {
    /// Returns the package root directory.
    fn root_directory(&self) -> &Utf8Path;
    /// Returns the package root directory as a workspace-relative path.
    fn workspace_relative_root_directory<'a>(&'a self, workspace: &Metadata) -> &'a Utf8Path;
}

impl PackageExt for Package {
    fn root_directory(&self) -> &Utf8Path {
        // `manifest_path` is the path to the manifest file, so parent must exist.
        self.manifest_path.parent().unwrap()
    }

    fn workspace_relative_root_directory<'a>(&'a self, workspace: &Metadata) -> &'a Utf8Path {
        let root_dir = self.root_directory();
        root_dir
            .strip_prefix(&workspace.workspace_root)
            .unwrap_or(root_dir)
    }
}

pub(crate) trait RangeExt {
    fn to_span(self) -> SourceSpan;
}

impl RangeExt for Range<usize> {
    fn to_span(self) -> SourceSpan {
        SourceSpan::from(legacy::Range::from(self))
    }
}

pub(crate) trait CommandExt {
    fn commandline(&self) -> OsString;
}

impl CommandExt for Command {
    fn commandline(&self) -> OsString {
        let mut cmd = OsString::new();
        cmd.push(self.get_program());
        for arg in self.get_args() {
            cmd.push(" ");
            cmd.push(arg);
        }
        cmd
    }
}
