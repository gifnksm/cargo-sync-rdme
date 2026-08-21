use std::{
    fs,
    io::{self, Write as _},
    sync::Arc,
};

use cargo_metadata::{
    Metadata, Package, PackageName,
    camino::{Utf8Path, Utf8PathBuf},
};

use miette::NamedSource;
use snafu::{ResultExt as _, Snafu, ensure};
use supports_color::Stream;
use tempfile::NamedTempFile;
use tracing::Level;
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
mod replace;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum SyncError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    ReadPackageManifest {
        #[snafu(source)]
        #[diagnostic_source]
        source: with_source::ReadFileError,
    },
    #[snafu(display("failed to read markdown file for package `{package}`: {markdown}", package = markdown.package, markdown = markdown.path))]
    ReadMarkdownFile {
        markdown: MarkdownPath,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to write markdown file for package `{package}`: {markdown}", package = markdown.package, markdown = markdown.path))]
    WriteMarkdownFile {
        markdown: MarkdownPath,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display(
        "no target files found for package `{package}`. Specify `package.readme` or `package.metadata.cargo-sync-rdme.extra-targets`"
    ))]
    NoTargetFilesFound { package: PackageName },
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    ParseMarkers {
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<marker::ParseMarkersError>,
    },
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    CreateContents {
        #[snafu(source)]
        #[diagnostic_source]
        source: contents::CreateAllContentsError,
    },
    #[snafu(display("failed to write diff output"))]
    WriteDiff {
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("markdown file for package `{package}` is not up to date: {markdown}", package = markdown.package, markdown = markdown.path))]
    CheckFailed { markdown: MarkdownPath },
    #[snafu(display(
        "failed to check whether the markdown file can be modified for package `{package}`: {markdown}", package = markdown.package, markdown = markdown.path
    ))]
    CheckFileModificationSafety {
        markdown: MarkdownPath,
        #[snafu(source)]
        source: vcs_modify_guard::ModifyGuardError,
    },
    #[snafu(display(
        "markdown file for package `{package}` is not under version control: {markdown}\nUse --allow-no-vcs to override this check.", package = markdown.package, markdown = markdown.path
    ))]
    NoVcs { markdown: MarkdownPath },
    #[snafu(display(
        "markdown file for package `{package}` has uncommitted changes: {markdown}\nUse --allow-dirty to override this check.", package = markdown.package, markdown = markdown.path
    ))]
    DirtyFile { markdown: MarkdownPath },
    #[snafu(display(
        "markdown file for package `{package}` has staged changes: {markdown}\nUse --allow-staged to override this check.", package = markdown.package, markdown = markdown.path
    ))]
    StagedFile { markdown: MarkdownPath },
    #[snafu(display(
        "markdown file for package `{package}` is not safe to modify for some reason: {markdown}\nreason: {reason:?}", package = markdown.package, markdown = markdown.path
    ))]
    UnsafeToModifyForSomeReason {
        markdown: MarkdownPath,
        reason: UnsafeModificationReason,
    },
}

impl From<with_source::ReadFileError> for Box<SyncError> {
    fn from(value: with_source::ReadFileError) -> Self {
        Box::new(value.into())
    }
}

impl From<Box<marker::ParseMarkersError>> for Box<SyncError> {
    fn from(value: Box<marker::ParseMarkersError>) -> Self {
        Box::new(value.into())
    }
}

impl From<contents::CreateAllContentsError> for Box<SyncError> {
    fn from(value: contents::CreateAllContentsError) -> Self {
        Box::new(value.into())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SyncOptions<'a> {
    pub(crate) mode: Mode,
    pub(crate) verbosity: Option<Level>,
    pub(crate) diff_stream: Stream,
    pub(crate) fix: &'a FixArgs,
    pub(crate) toolchain: &'a RustdocToolchainArgs,
    pub(crate) feature: &'a FeatureSelection,
}

pub(crate) fn sync_all(
    workspace: &Metadata,
    package: &Package,
    options: &SyncOptions<'_>,
) -> Result<(), Box<SyncError>> {
    let manifest = ManifestFile::from_toml("package manifest", &package.manifest_path)?;
    let _span = tracing::info_span!("sync", "{}", package.name).entered();

    let paths = package_target_files(package, &manifest.value().config().extra_targets);

    ensure!(
        !paths.is_empty(),
        NoTargetFilesFoundSnafu {
            package: package.name.clone(),
        }
    );

    for path in paths {
        tracing::info!("syncing markdown file: {path}");

        let mut markdown = MarkdownFile::new(workspace, package, path)?;

        let all_markers = marker::parse_markers(&markdown, &manifest)?;

        tracing::info!("creating replacement contents for markdown file: {path}");
        let all_contents =
            contents::create_all(all_markers, &manifest, workspace, package, options)?;

        let new_text = replace::replace_all(&markdown.text, &all_contents);

        let changed = new_text.as_str() != &*markdown.text;
        if !changed {
            tracing::info!("markdown file is already up to date: {path}");
            continue;
        }

        match options.mode {
            Mode::Check => {
                tracing::warn!("markdown file is not up to date: {path}");
                diff::write_pretty_diff(options.diff_stream, &markdown.text, &new_text)
                    .context(WriteDiffSnafu)?;
                return Err(CheckFailedSnafu {
                    markdown: &markdown,
                }
                .build()
                .into());
            }
            Mode::Fix => {}
        }

        // Update README if allowed
        check_update_allowed(&markdown, options.fix)?;
        markdown.replace(new_text.into())?;

        tracing::info!("updated markdown file: {path}");
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct MarkdownPath {
    package: PackageName,
    path: Utf8PathBuf,
}

impl MarkdownPath {
    fn new(package: &Package, path: Utf8PathBuf) -> Self {
        Self {
            package: package.name.clone(),
            path,
        }
    }
}

impl From<&MarkdownFile<'_>> for MarkdownPath {
    fn from(markdown: &MarkdownFile<'_>) -> Self {
        Self {
            package: markdown.package.name.clone(),
            path: markdown.relative_path.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct MarkdownFile<'a> {
    package: &'a Package,
    relative_path: Utf8PathBuf,
    path: Utf8PathBuf,
    text: Arc<str>,
}

impl<'a> MarkdownFile<'a> {
    fn new(
        workspace: &'a Metadata,
        package: &'a Package,
        package_relative_path: &'a Utf8Path,
    ) -> Result<Self, Box<SyncError>> {
        let relative_path = package
            .workspace_relative_root_directory(workspace)
            .join(package_relative_path);
        let path = workspace.workspace_root.join(&relative_path);
        let text = fs::read_to_string(&path)
            .with_context(|_source| ReadMarkdownFileSnafu {
                markdown: MarkdownPath::new(package, relative_path.clone()),
            })?
            .into();
        Ok(Self {
            package,
            relative_path,
            path,
            text,
        })
    }

    fn to_named_source(&self) -> NamedSource<Arc<str>> {
        NamedSource::new(&self.relative_path, Arc::clone(&self.text))
    }

    fn replace(&mut self, new_text: Arc<str>) -> Result<(), Box<SyncError>> {
        (|| {
            let output_dir = self.path.parent().unwrap();
            let mut tempfile = NamedTempFile::new_in(output_dir)?;
            tempfile.as_file_mut().write_all(new_text.as_bytes())?;
            tempfile.as_file_mut().sync_data()?;
            let file = tempfile.persist(&self.path).map_err(|err| err.error)?;
            file.sync_all()?;
            drop(file);
            Ok(())
        })()
        .context(WriteMarkdownFileSnafu { markdown: &*self })?;
        self.text = new_text;
        Ok(())
    }
}

type ManifestFile = WithSource<Manifest>;

fn package_target_files<'a, P>(package: &'a Package, extra_targets: &'a [P]) -> Vec<&'a Utf8Path>
where
    P: AsRef<Utf8Path>,
{
    let mut paths = vec![];
    paths.extend(package.readme.as_deref());
    paths.extend(extra_targets.iter().map(AsRef::as_ref));
    paths
}

fn check_update_allowed(
    markdown: &MarkdownFile<'_>,
    options: &FixArgs,
) -> Result<(), Box<SyncError>> {
    let FixArgs {
        allow_no_vcs,
        allow_dirty,
        allow_staged,
    } = options;

    let safety = AllowOptions::new()
        .allow_no_vcs(*allow_no_vcs)
        .allow_dirty(*allow_dirty)
        .allow_staged(*allow_staged)
        .check_safe_to_modify(&markdown.path)
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
    .map_err(Into::into)
}
