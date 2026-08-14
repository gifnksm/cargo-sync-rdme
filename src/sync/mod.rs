use std::{
    fs,
    io::{self, Write as _},
    sync::Arc,
};

use cargo_metadata::{
    Metadata, Package,
    camino::{Utf8Path, Utf8PathBuf},
};
use miette::{IntoDiagnostic as _, NamedSource, WrapErr as _, bail};
use pulldown_cmark::{Options, Parser};
use tempfile::NamedTempFile;
use vcs_modify_guard::{AllowOptions, ModificationSafety, UnsafeModificationReason};

use crate::{
    Result,
    cli::{Args, FixArgs},
    config::Manifest,
    diff,
    traits::PackageExt as _,
    with_source::WithSource,
};

mod contents;
mod marker;

#[derive(Debug, Clone)]
struct MarkdownFile {
    path: Utf8PathBuf,
    text: Arc<str>,
}

impl MarkdownFile {
    fn new(package: &Package, path: &Utf8Path) -> Result<Self> {
        let path = package.root_directory().join(path);
        let text = fs::read_to_string(&path)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!(
                    "failed to read README of {package}: {path}",
                    package = package.name
                )
            })?
            .into();
        Ok(Self { path, text })
    }

    fn to_named_source(&self) -> NamedSource<Arc<str>> {
        NamedSource::new(self.path.clone(), Arc::clone(&self.text))
    }
}

type ManifestFile = WithSource<Manifest>;

pub(crate) fn sync_all(args: &Args, workspace: &Metadata, package: &Package) -> Result<()> {
    let manifest = ManifestFile::from_toml("package manifest", &package.manifest_path)?;
    let _span = tracing::info_span!("sync", "{}", package.name).entered();

    let paths = package
        .readme
        .as_deref()
        .into_iter()
        .chain(
            manifest
                .value()
                .config()
                .extra_targets
                .iter()
                .map(Utf8Path::new),
        )
        .collect::<Vec<_>>();

    if paths.is_empty() {
        bail!(
            "no target files found. Please specify `package.readme` or `package.metadata.cargo-sync-rdme.extra-targets`"
        );
    }

    for path in paths {
        tracing::info!("syncing {path}...");

        let markdown = MarkdownFile::new(package, path)?;

        // Setup markdown parser
        let parser = Parser::new_ext(&markdown.text, Options::all()).into_offset_iter();

        // Find replace markers from markdown file
        let all_markers = marker::find_all(&markdown, &manifest, parser)?;

        // Create contents for each marker
        let replaces = all_markers.iter().map(|x| x.0.clone());
        let all_contents = contents::create_all(replaces, args, &manifest, workspace, package)?;

        // Replace markers with content
        let new_text = marker::replace_all(&markdown.text, &all_markers, &all_contents);

        // Compare new markdown file with old one
        let changed = new_text.as_str() != &*markdown.text;
        if !changed {
            tracing::info!("already up-to-date {path}");
            continue;
        }

        // Update README if allowed
        check_update_allowed(&args.fix, &markdown.path, &markdown.text, &new_text)?;
        write_readme(&markdown.path, &new_text)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to write markdown file: {path}"))?;

        tracing::info!("updated {path}");
    }

    Ok(())
}

pub(crate) fn write_readme(path: &Utf8Path, text: &str) -> io::Result<()> {
    let output_dir = path.parent().unwrap();
    let mut tempfile = NamedTempFile::new_in(output_dir)?;
    tempfile.as_file_mut().write_all(text.as_bytes())?;
    tempfile.as_file_mut().sync_data()?;
    let file = tempfile.persist(path).map_err(|err| err.error)?;
    file.sync_all()?;
    drop(file);
    Ok(())
}

fn check_update_allowed<P>(
    args: &FixArgs,
    markdown: P,
    old_text: &str,
    new_text: &str,
) -> Result<()>
where
    P: AsRef<Utf8Path>,
{
    let FixArgs {
        check,
        allow_no_vcs,
        allow_dirty,
        allow_staged,
    } = args;
    let markdown = markdown.as_ref();

    if *check {
        bail!(
            "the file is not up-to-date: {markdown}\n{}",
            diff::diff(old_text, new_text)
        );
    }

    let safety = AllowOptions::new()
        .allow_no_vcs(*allow_no_vcs)
        .allow_dirty(*allow_dirty)
        .allow_staged(*allow_staged)
        .check_safe_to_modify(markdown)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to check if the file can be modified: {markdown}"))?;

    match safety {
        ModificationSafety::Safe => {}
        ModificationSafety::Unsafe(reason) => match reason {
            UnsafeModificationReason::NoVcs => {
                bail!(
                    "the file is not under version control: {markdown}\nUse --allow-no-vcs to override this check."
                );
            }
            UnsafeModificationReason::Dirty { .. } => {
                bail!(
                    "the file has uncommitted changes: {markdown}\nUse --allow-dirty to override this check."
                );
            }
            UnsafeModificationReason::Staged { .. } => {
                bail!(
                    "the file has staged changes: {markdown}\nUse --allow-staged to override this check."
                );
            }
            reason => bail!(
                "the file is not safe to modify for some reason: {markdown}\nreason: {reason:?}"
            ),
        },
    }
    Ok(())
}
