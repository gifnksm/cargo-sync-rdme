use std::{
    fs,
    io::{self, Write as _},
    sync::Arc,
};

use cargo_metadata::{
    Metadata, Package,
    camino::{Utf8Path, Utf8PathBuf},
};
use miette::NamedSource;
use pulldown_cmark::{Options, Parser};
use snafu::{ResultExt as _, Snafu, ensure};
use supports_color::Stream;
use tempfile::NamedTempFile;
use vcs_modify_guard::{AllowOptions, ModificationSafety, UnsafeModificationReason};

use crate::{
    args::{FeatureSelection, FixArgs, Mode, RustdocToolchainArgs},
    config::Manifest,
    diff,
    traits::PackageExt as _,
    with_source::{self, WithSource},
};

mod contents;
mod marker;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum SyncError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    ReadPackageManifest {
        #[snafu(source)]
        #[diagnostic_source]
        source: with_source::ReadFileError,
    },
    #[snafu(display("failed to read markdown file: {path}"))]
    ReadMarkdownFile {
        path: Utf8PathBuf,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to write markdown file: {path}"))]
    WriteMarkdownFile {
        path: Utf8PathBuf,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display(
        "no target files found. Please specify `package.readme` or `package.metadata.cargo-sync-rdme.extra-targets`"
    ))]
    NoTargetFilesFound,
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    FindMarkers {
        #[snafu(source)]
        #[diagnostic_source]
        source: marker::FindAllError,
    },
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    CreateContents {
        #[snafu(source)]
        #[diagnostic_source]
        source: contents::CreateAllContentsError,
    },
    #[snafu(display("the file is not up-to-date: {markdown}\n{diff}"))]
    FileIsNotUpToDate { markdown: Utf8PathBuf, diff: String },
    #[snafu(display("failed to check whether the file can be modified: {markdown}"))]
    CheckFileModificationSafety {
        markdown: Utf8PathBuf,
        #[snafu(source)]
        source: vcs_modify_guard::ModifyGuardError,
    },
    #[snafu(display(
        "the file is not under version control: {markdown}\nUse --allow-no-vcs to override this check."
    ))]
    NoVcs { markdown: Utf8PathBuf },
    #[snafu(display(
        "the file has uncommitted changes: {markdown}\nUse --allow-dirty to override this check."
    ))]
    DirtyFile { markdown: Utf8PathBuf },
    #[snafu(display(
        "the file has staged changes: {markdown}\nUse --allow-staged to override this check."
    ))]
    StagedFile { markdown: Utf8PathBuf },
    #[snafu(display(
        "the file is not safe to modify for some reason: {markdown}\nreason: {reason:?}"
    ))]
    UnsafeToModifyForSomeReason {
        markdown: Utf8PathBuf,
        reason: UnsafeModificationReason,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct SyncOptions<'a> {
    pub(crate) mode: Mode,
    pub(crate) diagnostic_stream: Stream,
    pub(crate) fix: &'a FixArgs,
    pub(crate) toolchain: &'a RustdocToolchainArgs,
    pub(crate) feature: &'a FeatureSelection,
}

#[derive(Debug, Clone)]
struct MarkdownFile {
    path: Utf8PathBuf,
    text: Arc<str>,
}

impl MarkdownFile {
    fn new(package: &Package, path: &Utf8Path) -> Result<Self, SyncError> {
        let path = package.root_directory().join(path);
        let text = fs::read_to_string(&path)
            .context(ReadMarkdownFileSnafu { path: &path })?
            .into();
        Ok(Self { path, text })
    }

    fn to_named_source(&self) -> NamedSource<Arc<str>> {
        NamedSource::new(self.path.clone(), Arc::clone(&self.text))
    }
}

type ManifestFile = WithSource<Manifest>;

pub(crate) fn sync_all(
    workspace: &Metadata,
    package: &Package,
    options: &SyncOptions<'_>,
) -> Result<(), SyncError> {
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

    ensure!(!paths.is_empty(), NoTargetFilesFoundSnafu);

    for path in paths {
        tracing::info!("syncing markdown file: {path}");

        let markdown = MarkdownFile::new(package, path)?;

        // Setup markdown parser
        let parser = Parser::new_ext(&markdown.text, Options::all()).into_offset_iter();

        // Find replace markers from markdown file
        let all_markers = marker::find_all(&markdown, &manifest, parser)?;

        // Create contents for each marker
        let replaces = all_markers.iter().map(|x| x.0.clone());
        let all_contents = contents::create_all(replaces, &manifest, workspace, package, options)?;

        // Replace markers with content
        let new_text = marker::replace_all(&markdown.text, &all_markers, &all_contents);

        // Compare new markdown file with old one
        let changed = new_text.as_str() != &*markdown.text;
        if !changed {
            tracing::info!("markdown file is already up to date: {path}");
            continue;
        }

        match options.mode {
            Mode::Check => {
                return Err(FileIsNotUpToDateSnafu {
                    markdown: &markdown.path,
                    diff: diff::diff(&markdown.text, &new_text, options.diagnostic_stream),
                }
                .build());
            }
            Mode::Fix => {}
        }

        // Update README if allowed
        check_update_allowed(&markdown.path, options.fix)?;
        write_markdown(&markdown.path, &new_text).context(WriteMarkdownFileSnafu {
            path: markdown.path,
        })?;

        tracing::info!("updated markdown file: {path}");
    }

    Ok(())
}

pub(crate) fn write_markdown(path: &Utf8Path, text: &str) -> io::Result<()> {
    let output_dir = path.parent().unwrap();
    let mut tempfile = NamedTempFile::new_in(output_dir)?;
    tempfile.as_file_mut().write_all(text.as_bytes())?;
    tempfile.as_file_mut().sync_data()?;
    let file = tempfile.persist(path).map_err(|err| err.error)?;
    file.sync_all()?;
    drop(file);
    Ok(())
}

fn check_update_allowed<P>(markdown: P, options: &FixArgs) -> Result<(), SyncError>
where
    P: AsRef<Utf8Path>,
{
    let FixArgs {
        allow_no_vcs,
        allow_dirty,
        allow_staged,
    } = options;
    let markdown = markdown.as_ref();

    let safety = AllowOptions::new()
        .allow_no_vcs(*allow_no_vcs)
        .allow_dirty(*allow_dirty)
        .allow_staged(*allow_staged)
        .check_safe_to_modify(markdown)
        .context(CheckFileModificationSafetySnafu { markdown })?;

    match safety {
        ModificationSafety::Safe => Ok(()),
        ModificationSafety::Unsafe(reason) => match reason {
            UnsafeModificationReason::NoVcs => Err(NoVcsSnafu { markdown }.build()),
            UnsafeModificationReason::Dirty { .. } => Err(DirtyFileSnafu { markdown }.build()),
            UnsafeModificationReason::Staged { .. } => Err(StagedFileSnafu { markdown }.build()),
            reason => Err(UnsafeToModifyForSomeReasonSnafu { markdown, reason }.build()),
        },
    }
}
