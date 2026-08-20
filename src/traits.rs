use std::{ffi::OsString, process::Command, range::Range};

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

pub(crate) trait StrExt {
    fn substr_range_shim(&self, substr: &str) -> Option<Range<usize>>;
}

mod imp {
    use std::{ops, range::Range};

    use miette::SourceSpan;

    use crate::traits::{RangeExt, StrExt};

    impl RangeExt for Range<usize> {
        fn to_span(self) -> SourceSpan {
            SourceSpan::from(ops::Range::from(self))
        }
    }

    impl StrExt for str {
        fn substr_range_shim(&self, substr: &str) -> Option<Range<usize>> {
            let range = self.as_bytes().as_ptr_range();
            let substr_range = substr.as_bytes().as_ptr_range();
            let start = range.start.addr();
            let substr_start = substr_range.start.addr();
            if substr_start < start {
                return None;
            }
            let end = range.end.addr();
            let substr_end = substr_range.end.addr();
            if substr_end > end {
                return None;
            }
            Some(((substr_start - start)..(substr_end - start)).into())
        }
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
