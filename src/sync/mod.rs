use std::{io, sync::Arc};

use cargo_metadata::{Metadata, Package, PackageName};
use snafu::{ResultExt as _, Snafu, ensure};
use supports_color::Stream;
use tracing::Level;
use vcs_modify_guard::{AllowOptions, ModificationSafety, UnsafeModificationReason};

use crate::{
    args::{Args, FeatureSelection, FixArgs, Mode},
    config::Config,
    diff,
    manifest::Manifest,
    source::{SourceFile, SourceFileLoader, SourceFilePath},
    sync::{contents::CreateAllContentsError, marker::ParseMarkersError},
};

mod contents;
mod marker;
mod replace;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum SyncError {
    #[snafu(display("failed to read markdown file for package `{package}`: {markdown}", markdown = markdown.workspace_path))]
    ReadMarkdownFile {
        package: PackageName,
        markdown: SourceFilePath,
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("failed to write markdown file for package `{package}`: {markdown}", markdown = markdown.workspace_path))]
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
        source: Box<ParseMarkersError>,
    },
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    CreateContents {
        #[snafu(source)]
        #[diagnostic_source]
        source: CreateAllContentsError,
    },
    #[snafu(display("failed to write diff output"))]
    WriteDiff {
        #[snafu(source)]
        source: io::Error,
    },
    #[snafu(display("markdown file for package `{package}` is not up to date: {markdown}", markdown = markdown.workspace_path))]
    CheckFailed {
        package: PackageName,
        markdown: SourceFilePath,
    },
    #[snafu(display(
        "failed to check whether the markdown file can be modified for package `{package}`: {markdown}", markdown = markdown.workspace_path
    ))]
    CheckFileModificationSafety {
        package: PackageName,
        markdown: SourceFilePath,
        #[snafu(source)]
        source: vcs_modify_guard::ModifyGuardError,
    },
    #[snafu(display(
        "markdown file for package `{package}` is not under version control: {markdown}\nUse --allow-no-vcs to override this check.", markdown = markdown.workspace_path
    ))]
    NoVcs {
        package: PackageName,
        markdown: SourceFilePath,
    },
    #[snafu(display(
        "markdown file for package `{package}` has uncommitted changes: {markdown}\nUse --allow-dirty to override this check.", markdown = markdown.workspace_path
    ))]
    DirtyFile {
        package: PackageName,
        markdown: SourceFilePath,
    },
    #[snafu(display(
        "markdown file for package `{package}` has staged changes: {markdown}\nUse --allow-staged to override this check.", markdown = markdown.workspace_path
    ))]
    StagedFile {
        package: PackageName,
        markdown: SourceFilePath,
    },
    #[snafu(display(
        "markdown file for package `{package}` is not safe to modify for some reason: {markdown}\nreason: {reason:?}", markdown = markdown.workspace_path
    ))]
    UnsafeToModifyForSomeReason {
        package: PackageName,
        markdown: SourceFilePath,
        reason: UnsafeModificationReason,
    },
}

impl From<Box<ParseMarkersError>> for Box<SyncError> {
    fn from(value: Box<ParseMarkersError>) -> Self {
        Box::new(value.into())
    }
}

impl From<CreateAllContentsError> for Box<SyncError> {
    fn from(value: CreateAllContentsError) -> Self {
        Box::new(value.into())
    }
}

#[derive(Debug)]
pub(crate) struct PackageSyncContext<'a> {
    mode: Mode,
    verbosity: Option<Level>,
    diff_stream: Stream,
    fix: &'a FixArgs,
    install_toolchain: bool,
    feature: &'a FeatureSelection,
    workspace: &'a Metadata,
    package: &'a Package,
    manifest: Arc<Manifest>,
    config: Config,
}

impl<'a> PackageSyncContext<'a> {
    pub(crate) fn new(
        diff_stream: Stream,
        args: &'a Args,
        workspace: &'a Metadata,
        package: &'a Package,
        manifest: Arc<Manifest>,
        config: Config,
    ) -> Self {
        Self {
            diff_stream,
            mode: args.mode.mode(),
            verbosity: args.verbosity.into(),
            fix: &args.fix,
            install_toolchain: args.toolchain.install_toolchain,
            feature: &args.feature,
            workspace,
            package,
            manifest,
            config,
        }
    }
}

impl From<&PackageSyncContext<'_>> for PackageName {
    fn from(cx: &PackageSyncContext<'_>) -> Self {
        cx.package.name.clone()
    }
}

pub(crate) fn sync_all(cx: &PackageSyncContext<'_>) -> Result<(), Box<SyncError>> {
    let _span = tracing::info_span!("sync", "{}", cx.package.name).entered();

    let loaders = package_target_files(cx);

    ensure!(!loaders.is_empty(), NoTargetFilesFoundSnafu { package: cx });

    for loader in loaders {
        tracing::info!("syncing markdown file: {}", loader.workspace_path());

        let mut markdown = loader
            .load()
            .with_context(|_source| ReadMarkdownFileSnafu {
                package: cx,
                markdown: &loader,
            })?;

        let all_markers = marker::parse_markers(cx, &markdown)?;

        tracing::info!(
            "creating replacement contents for markdown file: {}",
            loader.workspace_path()
        );
        let all_contents = contents::create_all(cx, all_markers)?;

        let new_text = replace::replace_all(markdown.text(), &all_contents);

        let changed = new_text.as_str() != markdown.text();
        if !changed {
            tracing::info!(
                "markdown file is already up to date: {}",
                loader.workspace_path()
            );
            continue;
        }

        match cx.mode {
            Mode::Check => {
                tracing::warn!(
                    "markdown file is not up to date: {}",
                    loader.workspace_path()
                );
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

        tracing::info!("updated markdown file: {}", loader.workspace_path());
    }

    Ok(())
}

fn package_target_files(cx: &PackageSyncContext<'_>) -> Vec<SourceFileLoader> {
    let mut paths = vec![];
    paths.extend(cx.package.readme.as_ref().map(|readme| {
        SourceFileLoader::from_package_relative_path(cx.workspace, cx.package, readme)
    }));
    paths.extend(
        cx.config
            .extra_targets
            .iter()
            .map(|path| SourceFileLoader::from_path(cx.workspace, path)),
    );
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
