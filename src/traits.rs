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
    use super::{SourceSpan, StrSpanExt};

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
    }
}
