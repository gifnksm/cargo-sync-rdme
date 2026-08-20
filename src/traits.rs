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

pub(crate) trait StrSpanExt: Sized {
    fn trim(&self) -> Self {
        self.trim_start().trim_end()
    }
    fn trim_start(&self) -> Self;
    fn trim_end(&self) -> Self;
    fn strip_prefix_str(&self, prefix: &str) -> Option<Self>;
    fn strip_suffix_str(&self, suffix: &str) -> Option<Self>;
    fn split_once_fn(&self, f: impl Fn(char) -> bool) -> Option<(Self, Self)>;
}

mod imp {
    use std::{ops, range::Range};

    use miette::SourceSpan;

    use crate::traits::{RangeExt, StrExt, StrSpanExt};

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

    fn adjust_range(offset: usize, substr_range: Range<usize>) -> Range<usize> {
        ((offset + substr_range.start)..(offset + substr_range.end)).into()
    }

    impl StrSpanExt for (&str, Range<usize>) {
        fn trim_start(&self) -> Self {
            let substr = self.0.trim_start();
            let range = adjust_range(self.1.start, self.0.substr_range_shim(substr).unwrap());
            (substr, range)
        }

        fn trim_end(&self) -> Self {
            let substr = self.0.trim_end();
            let range = adjust_range(self.1.start, self.0.substr_range_shim(substr).unwrap());
            (substr, range)
        }

        fn strip_prefix_str(&self, prefix: &str) -> Option<Self> {
            let substr = self.0.strip_prefix(prefix)?;
            let range = adjust_range(self.1.start, self.0.substr_range_shim(substr).unwrap());
            Some((substr, range))
        }

        fn strip_suffix_str(&self, suffix: &str) -> Option<Self> {
            let substr = self.0.strip_suffix(suffix)?;
            let range = adjust_range(self.1.start, self.0.substr_range_shim(substr).unwrap());
            Some((substr, range))
        }

        fn split_once_fn(&self, f: impl Fn(char) -> bool) -> Option<(Self, Self)> {
            let (head, tail) = self.0.split_once(f)?;
            let head_range = adjust_range(self.1.start, self.0.substr_range_shim(head).unwrap());
            let tail_range = adjust_range(self.1.start, self.0.substr_range_shim(tail).unwrap());
            Some(((head, head_range), (tail, tail_range)))
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
