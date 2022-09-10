use std::{fmt, fs, io, sync::Arc};

use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    Metadata, Package,
};
use miette::{NamedSource, SourceSpan};
use serde::Deserialize;

use super::Escape;
use crate::{
    config::{badges, metadata, package, GetConfigError},
    sync::ManifestFile,
};

type CreateResult<T> = std::result::Result<T, CreateBadgeError>;

pub(super) fn create_all(
    badges: Arc<[metadata::BadgeItem]>,
    manifest: &ManifestFile,
    workspace: &Metadata,
    package: &Package,
) -> Result<String, CreateAllBadgesError> {
    let mut output = vec![];

    let mut errors = vec![];

    for badge in &*badges {
        match Output::from_config(badge, manifest, workspace, package) {
            Ok(Output::None) => {}
            Ok(Output::One(badge)) => output.push(badge),
            Ok(Output::ManyResult(bs)) => {
                for b in bs {
                    match b {
                        Ok(b) => output.push(b),
                        Err(e) => errors.push(e),
                    }
                }
            }
            Err(err) => errors.push(err),
        }
    }

    if !errors.is_empty() {
        return Err(CreateAllBadgesError { errors });
    }

    Ok(output.iter().map(|badge| format!("{badge}\n")).collect())
}

#[derive(Debug)]
enum Output {
    None,
    One(Badge),
    ManyResult(Vec<CreateResult<Badge>>),
}

impl From<Badge> for Output {
    fn from(badge: Badge) -> Self {
        Self::One(badge)
    }
}

impl From<Option<Badge>> for Output {
    fn from(badge: Option<Badge>) -> Self {
        match badge {
            Some(badge) => Self::One(badge),
            None => Self::None,
        }
    }
}

impl From<Vec<CreateResult<Badge>>> for Output {
    fn from(badges: Vec<CreateResult<Badge>>) -> Self {
        Self::ManyResult(badges)
    }
}

impl Output {
    fn from_config(
        config: &metadata::BadgeItem,
        manifest: &ManifestFile,
        workspace: &Metadata,
        package: &Package,
    ) -> CreateResult<Self> {
        Ok(match config {
            metadata::BadgeItem::Maintenance => Badge::maintenance(manifest)?.into(),
            metadata::BadgeItem::License(license) => {
                Badge::license(license, manifest, package)?.into()
            }
            metadata::BadgeItem::CratesIo => Badge::crates_io(package).into(),
            metadata::BadgeItem::DocsRs => Badge::docs_rs(package).into(),
            metadata::BadgeItem::RustVersion => Badge::rust_version(manifest)?.into(),
            metadata::BadgeItem::GithubActions(github_actions) => {
                Badge::github_actions(github_actions, manifest, workspace)?.into()
            }
            metadata::BadgeItem::Codecov => Badge::codecov(manifest)?.into(),
        })
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("failed to create badges of README")]
pub(in super::super) struct CreateAllBadgesError {
    #[related]
    errors: Vec<CreateBadgeError>,
}

#[derive(Debug, Clone)]
struct Badge {
    alt: String,
    link: Option<String>,
    image: String,
}

impl fmt::Display for Badge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let need_escape = &['\\', '`', '_', '[', ']', '(', ')', '!'];

        if let Some(link) = &self.link {
            write!(
                f,
                "[![{}]({})]({})",
                Escape(&self.alt, need_escape),
                self.image,
                link
            )
        } else {
            write!(f, "![{}]({})", Escape(&self.alt, need_escape), &self.image)
        }
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
enum CreateBadgeError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    GetConfig(#[from] GetConfigError),
    #[error("failed to open GitHub Action's workflows directory: {path}")]
    OpenWorkflowsDir {
        #[source]
        source: io::Error,
        path: Utf8PathBuf,
    },
    #[error("failed to read GitHub Action's workflows directory: {path}")]
    ReadWorkflowsDir {
        source: io::Error,
        path: Utf8PathBuf,
    },
    #[error("failed to read GitHub Action's workflow file: {path}")]
    ReadWorkflowFile {
        #[source]
        source: io::Error,
        path: Utf8PathBuf,
    },
    #[error("failed to parse GitHub Action's workflow file: {path}")]
    ParseWorkflowFile {
        #[source]
        source: serde_yaml::Error,
        path: Utf8PathBuf,
        #[source_code]
        souce_code: NamedSource,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("`package.repository` must starts with `https://github.com/`")]
    InvalidGithubRepository {
        repository: String,
        #[source_code]
        source_code: NamedSource,
        #[label]
        span: SourceSpan,
    },
}

impl Badge {
    fn maintenance(manifest: &ManifestFile) -> CreateResult<Option<Self>> {
        let status_with_source = (|| manifest.try_badges()?.try_maintenance()?.try_status())()
            .map_err(|err| err.with_key("badges.maintenance.status"))?;
        let status = status_with_source.value().get_ref();

        use badges::MaintenanceStatus as Ms;
        // image url borrowed from https://gist.github.com/taiki-e/ad73eaea17e2e0372efb76ef6b38f17b
        let image_color = match status {
            Ms::ActivelyDeveloped => "brightgreen",
            Ms::PassivelyMaintained => "yellowgreen",
            Ms::AsIs => "yellow",
            Ms::Experimental => "blue",
            Ms::LookingForMaintainer => "orange",
            Ms::Deprecated => "red",
            Ms::None => return Ok(None),
        };

        let alt = format!("Maintenance: {}", status.as_str());
        let link = Some(
            "https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section".to_owned(),
        );
        let image = format!(
            "https://img.shields.io/badge/maintenance-{}-{image_color}.svg",
            status.as_str().replace('-', "--")
        )
        .parse()
        .unwrap();

        let badge = Self { alt, link, image };
        Ok(Some(badge))
    }

    fn license(
        license: &metadata::License,
        manifest: &ManifestFile,
        package: &Package,
    ) -> CreateResult<Self> {
        let manifest_license_with_source = (|| manifest.try_package()?.try_license())()
            .map_err(|err| err.with_key("package.license` or `package.license-file"))?;
        let manifest_license = manifest_license_with_source.value();

        let (license_str, license_path) = match manifest_license {
            package::License::Name { name, path } => (
                name.get_ref().as_str(),
                path.as_ref().map(|p| p.get_ref().as_str()),
            ),
            package::License::File { path } => ("non-standard", Some(path.get_ref().as_str())),
        };

        let alt = format!("License: {}", license_str);
        let link = license
            .link
            .clone()
            .or_else(|| license_path.map(|p| p.to_string()));
        let image = format!("https://img.shields.io/crates/l/{}.svg", package.name);
        Ok(Self { alt, link, image })
    }

    fn crates_io(package: &Package) -> Self {
        let alt = "crates.io".to_owned();
        let link = Some(format!("https://crates.io/crates/{}", package.name));
        let image = format!(
            "https://img.shields.io/crates/v/{}.svg?logo=rust",
            package.name
        );
        Self { alt, link, image }
    }

    fn docs_rs(package: &Package) -> Self {
        let alt = "docs.rs".to_owned();
        let link = Some(format!("https://docs.rs/{}", package.name));
        let image = format!("https://docs.rs/{}/badge.svg?logo=docs.rs", package.name);
        Self { alt, link, image }
    }

    fn rust_version(manifest: &ManifestFile) -> CreateResult<Self> {
        let rust_version_with_source = (|| manifest.try_package()?.try_rust_version())()
            .map_err(|err| err.with_key("package.rust-version"))?;
        let rust_version = rust_version_with_source.value().get_ref();

        let alt = format!("Rust: {rust_version}");
        let link = Some(
            "https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field"
                .to_owned(),
        );
        let image =
            format!("https://img.shields.io/badge/rust-{rust_version}-93450a.svg?logo=rust");
        Ok(Self { alt, link, image })
    }

    fn github_actions(
        github_actions: &metadata::GithubActions,
        manifest: &ManifestFile,
        workspace: &Metadata,
    ) -> CreateResult<Vec<CreateResult<Self>>> {
        let repository_with_source = (|| manifest.try_package()?.try_repository())()
            .map_err(|err| err.with_key("package.repository"))?;
        let repository = repository_with_source.value().get_ref();
        if !repository.starts_with("https://github.com/") {
            return Err(CreateBadgeError::InvalidGithubRepository {
                repository: repository.to_owned(),
                source_code: repository_with_source.to_named_source(),
                span: repository_with_source.span(),
            });
        }

        let results = if github_actions.workflows.is_empty() {
            Self::github_actions_from_directory(workspace)?
        } else {
            Self::github_actions_from_config(&github_actions.workflows, workspace)?
        };

        let results = results
            .into_iter()
            .map(|res| {
                res.map(|(name, file)| {
                    let alt = format!("GitHub Actions: {}", name);
                    let link = format!(
                        "{}/actions/workflows/{}",
                        repository.trim_end_matches('/'),
                        file
                    );
                    let image = format!("{link}/badge.svg");
                    Self {
                        alt,
                        link: Some(link),
                        image,
                    }
                })
            })
            .collect();

        Ok(results)
    }

    fn codecov(manifest: &ManifestFile) -> CreateResult<Badge> {
        let repository_with_source = (|| manifest.try_package()?.try_repository())()
            .map_err(|err| err.with_key("package.repository"))?;
        let repository = repository_with_source.value().get_ref();
        let repo_path = repository
            .strip_prefix("https://github.com/")
            .ok_or_else(|| CreateBadgeError::InvalidGithubRepository {
                repository: repository.to_owned(),
                source_code: repository_with_source.to_named_source(),
                span: repository_with_source.span(),
            })?;

        let alt = "Codecov".to_owned();
        let link = format!("https://codecov.io/gh/{}", repo_path.trim_end_matches('/'));
        let image = format!("{link}/graph/badge.svg");
        Ok(Badge {
            alt,
            link: Some(link),
            image,
        })
    }

    fn github_actions_from_directory(
        workspace: &Metadata,
    ) -> CreateResult<Vec<CreateResult<(String, String)>>> {
        let mut badges = vec![];

        let workflows_dir_path = workspace.workspace_root.join(".github/workflows");
        let dirs = match workflows_dir_path.read_dir_utf8() {
            Ok(dirs) => dirs,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                tracing::warn!("workflows directory does not exist: {workflows_dir_path}");
                return Ok(vec![]);
            }
            Err(err) => {
                return Err(CreateBadgeError::OpenWorkflowsDir {
                    source: err,
                    path: workflows_dir_path.clone(),
                })
            }
        };

        for res in dirs {
            let entry = match res {
                Ok(entry) => entry,
                Err(err) => {
                    badges.push(Err(CreateBadgeError::ReadWorkflowsDir {
                        source: err,
                        path: workflows_dir_path.clone(),
                    }));
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_file()
                || (path.extension() != Some("yml") && path.extension() != Some("yaml"))
            {
                continue;
            }

            let name = match read_workflow_name(workspace, path) {
                Ok(name) => name,
                Err(err) => {
                    badges.push(Err(err));
                    continue;
                }
            };
            let file = path.file_name().unwrap().to_owned();

            badges.push(Ok((name, file)));
        }

        Ok(badges)
    }

    fn github_actions_from_config(
        workflows: &[metadata::GithubActionsWorkflow],
        workspace: &Metadata,
    ) -> CreateResult<Vec<CreateResult<(String, String)>>> {
        let workflows_dir_path = workspace.workspace_root.join(".github/workflows");

        let mut badges = vec![];
        for workflow in workflows {
            let full_path = workflows_dir_path.join(&workflow.file);
            let name = match &workflow.name {
                Some(name) => name.to_owned(),
                None => match read_workflow_name(workspace, &full_path) {
                    Ok(name) => name,
                    Err(err) => {
                        badges.push(Err(err));
                        continue;
                    }
                },
            };
            badges.push(Ok((name, workflow.file.clone())));
        }

        Ok(badges)
    }
}

fn read_workflow_name(workspace: &Metadata, path: &Utf8Path) -> CreateResult<String> {
    #[derive(Debug, Deserialize)]
    struct Workflow {
        #[serde(default)]
        name: Option<String>,
    }

    let text = fs::read_to_string(path).map_err(|err| CreateBadgeError::ReadWorkflowFile {
        source: err,
        path: path.to_owned(),
    })?;

    let workflow: Workflow = serde_yaml::from_str(&text).map_err(|err| {
        let span = err.location().map(|l| SourceSpan::from((l.index(), 0)));
        CreateBadgeError::ParseWorkflowFile {
            source: err,
            path: path.to_owned(),
            souce_code: NamedSource::new(path, text),
            span,
        }
    })?;

    // https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions
    // > If you omit name, GitHub sets it to the workflow file path relative to the root of the repository.
    Ok(workflow.name.unwrap_or_else(|| {
        path.strip_prefix(&workspace.workspace_root)
            .unwrap()
            .to_string()
    }))
}
