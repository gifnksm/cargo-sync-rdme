use std::{
    ffi::{OsStr, OsString},
    process::Command,
    range::{Range, legacy},
};

use cargo_metadata::{Metadata, Package, camino::Utf8Path};
use miette::SourceSpan;
use shlex::Quoter;

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
    fn flag_value<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>;
    fn commandline(&self) -> String;
}

impl CommandExt for Command {
    fn flag_value<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        let mut arg = OsString::new();
        arg.push(key);
        arg.push("=");
        arg.push(value);
        self.arg(arg)
    }

    fn commandline(&self) -> String {
        let mut cmd = String::new();
        let quoter = Quoter::new().allow_nul(true);
        // quoter.quote() returns an error only when the input contains a NUL byte and `allow_nul` is false, so we can safely unwrap here.
        // <https://docs.rs/shlex/latest/shlex/enum.QuoteError.html>
        let push_str = |cmd: &mut String, s: &str| {
            cmd.push_str(&quoter.quote(s).unwrap());
        };

        push_str(&mut cmd, &self.get_program().to_string_lossy());
        for arg in self.get_args() {
            let arg = arg.to_string_lossy();
            cmd.push(' ');
            if let Some((flag, value)) = arg.split_once('=')
                && flag.starts_with('-')
                && quoter.quote(flag).is_ok_and(|q| q == flag)
            {
                cmd.push_str(flag);
                cmd.push('=');
                push_str(&mut cmd, value);
            } else {
                push_str(&mut cmd, &arg);
            }
        }
        cmd
    }
}
