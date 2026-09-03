use std::io;

use cargo_metadata::{Metadata, Package, PackageName, camino::Utf8Path};
use snafu::{ResultExt as _, Snafu, ensure};
use supports_color::Stream;
use tracing::Level;
use vcs_modify_guard::{AllowOptions, ModificationSafety, UnsafeModificationReason};

use crate::{
    args::{Args, FeatureSelection, FixArgs, Mode, RustdocToolchainArgs},
    config::Config,
    diff,
    manifest::{Manifest, ManifestError},
    source::{SourceFile, SourceFileLoader, SourceFilePath},
};

mod contents;
mod marker;
mod replace;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum SyncError {
    #[snafu(display("failed to load package manifest file for package `{package}`: {path}", path = manifest.path))]
    LoadPackageManifestFile {
        package: PackageName,
        manifest: SourceFilePath,
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<ManifestError>,
    },
    #[snafu(display("failed to read markdown file for package `{package}`: {markdown}", markdown = markdown.path))]
    ReadMarkdownFile {
        package: PackageName,
        markdown: SourceFilePath,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to write markdown file for package `{package}`: {markdown}", markdown = markdown.path))]
    WriteMarkdownFile {
        package: PackageName,
        markdown: SourceFilePath,
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
    #[snafu(display("markdown file for package `{package}` is not up to date: {markdown}", markdown = markdown.path))]
    CheckFailed {
        package: PackageName,
        markdown: SourceFilePath,
    },
    #[snafu(display(
        "failed to check whether the markdown file can be modified for package `{package}`: {markdown}", markdown = markdown.path
    ))]
    CheckFileModificationSafety {
        package: PackageName,
        markdown: SourceFilePath,
        #[snafu(source)]
        source: vcs_modify_guard::ModifyGuardError,
    },
    #[snafu(display(
        "markdown file for package `{package}` is not under version control: {markdown}\nUse --allow-no-vcs to override this check.", markdown = markdown.path
    ))]
    NoVcs {
        package: PackageName,
        markdown: SourceFilePath,
    },
    #[snafu(display(
        "markdown file for package `{package}` has uncommitted changes: {markdown}\nUse --allow-dirty to override this check.", markdown = markdown.path
    ))]
    DirtyFile {
        package: PackageName,
        markdown: SourceFilePath,
    },
    #[snafu(display(
        "markdown file for package `{package}` has staged changes: {markdown}\nUse --allow-staged to override this check.", markdown = markdown.path
    ))]
    StagedFile {
        package: PackageName,
        markdown: SourceFilePath,
    },
    #[snafu(display(
        "markdown file for package `{package}` is not safe to modify for some reason: {markdown}\nreason: {reason:?}", markdown = markdown.path
    ))]
    UnsafeToModifyForSomeReason {
        package: PackageName,
        markdown: SourceFilePath,
        reason: UnsafeModificationReason,
    },
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

#[derive(Debug)]
pub(crate) struct PackageSyncContext<'a> {
    mode: Mode,
    verbosity: Option<Level>,
    diff_stream: Stream,
    fix: &'a FixArgs,
    toolchain: &'a RustdocToolchainArgs,
    feature: &'a FeatureSelection,
    workspace: &'a Metadata,
    package: &'a Package,
    manifest: Manifest,
    config: Config,
}

impl<'a> PackageSyncContext<'a> {
    pub(crate) fn load(
        diff_stream: Stream,
        args: &'a Args,
        workspace: &'a Metadata,
        package: &'a Package,
    ) -> Result<Self, Box<SyncError>> {
        let manifest_loader = SourceFileLoader::from_path(workspace, &package.manifest_path);
        let manifest = Manifest::load(&manifest_loader).with_context(|_source| {
            LoadPackageManifestFileSnafu {
                package: package.name.clone(),
                manifest: &manifest_loader,
            }
        })?;
        let config = manifest
            .package_config()
            .with_context(|_source| LoadPackageManifestFileSnafu {
                package: package.name.clone(),
                manifest: &manifest_loader,
            })?
            .unwrap_or_default();
        Ok(Self {
            diff_stream,
            mode: args.mode.mode(),
            verbosity: args.verbosity.into(),
            fix: &args.fix,
            toolchain: &args.toolchain,
            feature: &args.feature,
            workspace,
            package,
            manifest,
            config,
        })
    }
}

impl From<&PackageSyncContext<'_>> for PackageName {
    fn from(cx: &PackageSyncContext<'_>) -> Self {
        cx.package.name.clone()
    }
}

pub(crate) fn sync_all(cx: &PackageSyncContext<'_>) -> Result<(), Box<SyncError>> {
    let _span = tracing::info_span!("sync", "{}", cx.package.name).entered();

    let paths = package_target_files(cx);

    ensure!(!paths.is_empty(), NoTargetFilesFoundSnafu { package: cx });

    for path in paths {
        tracing::info!("syncing markdown file: {path}");

        let markdown_loader =
            SourceFileLoader::from_package_relative_path(cx.workspace, cx.package, path);
        let mut markdown =
            markdown_loader
                .load()
                .with_context(|_source| ReadMarkdownFileSnafu {
                    package: cx,
                    markdown: &markdown_loader,
                })?;

        let all_markers = marker::parse_markers(cx, &markdown)?;

        tracing::info!("creating replacement contents for markdown file: {path}");
        let all_contents = contents::create_all(cx, all_markers)?;

        let new_text = replace::replace_all(markdown.text(), &all_contents);

        let changed = new_text.as_str() != markdown.text();
        if !changed {
            tracing::info!("markdown file is already up to date: {path}");
            continue;
        }

        match cx.mode {
            Mode::Check => {
                tracing::warn!("markdown file is not up to date: {path}");
                diff::write_pretty_diff(cx.diff_stream, markdown.text(), &new_text)
                    .context(WriteDiffSnafu)?;
                return Err(CheckFailedSnafu {
                    package: cx.package.name.clone(),
                    markdown: &markdown,
                }
                .build()
                .into());
            }
            Mode::Fix => {}
        }

        // Update README if allowed
        check_update_allowed(&markdown, cx.package, cx.fix)?;
        markdown
            .replace_file_content(new_text.into())
            .with_context(|_source_| WriteMarkdownFileSnafu {
                package: cx.package.name.clone(),
                markdown: &markdown,
            })?;

        tracing::info!("updated markdown file: {path}");
    }

    Ok(())
}

fn package_target_files<'a>(cx: &'a PackageSyncContext<'_>) -> Vec<&'a Utf8Path> {
    let mut paths = vec![];
    paths.extend(cx.package.readme.as_deref());
    paths.extend(cx.config.extra_targets.iter().map(Utf8Path::new));
    paths
}

fn check_update_allowed(
    markdown: &SourceFile,
    package: &Package,
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
        .check_safe_to_modify(markdown.path())
        .with_context(|_source| CheckFileModificationSafetySnafu {
            package: package.name.clone(),
            markdown,
        })?;

    match safety {
        ModificationSafety::Safe => Ok(()),
        ModificationSafety::Unsafe(reason) => match reason {
            UnsafeModificationReason::NoVcs => Err(NoVcsSnafu {
                package: package.name.clone(),
                markdown,
            }
            .build()),
            UnsafeModificationReason::Dirty { .. } => Err(DirtyFileSnafu {
                package: package.name.clone(),
                markdown,
            }
            .build()),
            UnsafeModificationReason::Staged { .. } => Err(StagedFileSnafu {
                package: package.name.clone(),
                markdown,
            }
            .build()),
            reason => Err(UnsafeToModifyForSomeReasonSnafu {
                package: package.name.clone(),
                markdown,
                reason,
            }
            .build()),
        },
    }
    .map_err(Into::into)
}
