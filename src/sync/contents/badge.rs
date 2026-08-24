use std::{borrow::Cow, cmp::Ordering, fmt, fmt::Write as _, fs, io, sync::Arc};

use cargo_metadata::{
    Metadata, Package,
    camino::{Utf8Path, Utf8PathBuf},
    semver::{Comparator, Op, VersionReq},
};
use miette::{NamedSource, SourceSpan};
use serde::Deserialize;
use snafu::{IntoError as _, OptionExt as _, ResultExt as _, Snafu, ensure};
use url::Url;

use super::Escape;
use crate::{
    config::{
        GetConfigError,
        manifest::{
            badges::MaintenanceStatus,
            package::metadata::badge::item::{
                BadgeItem, Codecov, GithubActions, GithubActionsWorkflow, License,
            },
        },
    },
    sync::ManifestFile,
};

type CreateResult<T> = Result<T, Box<CreateBadgeError>>;

pub(super) fn create_all(
    badges: &[BadgeItem],
    manifest: &ManifestFile,
    workspace: &Metadata,
    package: &Package,
) -> Result<String, CreateAllBadgesError> {
    let mut output = String::new();

    let mut errors = vec![];

    for badge in badges {
        match BadgeLinkSet::from_config(badge, manifest, workspace, package) {
            Ok(BadgeLinkSet::None) => {}
            Ok(BadgeLinkSet::One(badge)) => writeln!(&mut output, "{badge}").unwrap(),
            Ok(BadgeLinkSet::ManyResult(bs)) => {
                for b in bs {
                    match b {
                        Ok(b) => writeln!(&mut output, "{b}").unwrap(),
                        Err(e) => errors.push(*e),
                    }
                }
            }
            Err(err) => errors.push(*err),
        }
    }

    ensure!(errors.is_empty(), CreateAllBadgesSnafu { errors });

    Ok(output)
}

#[derive(Debug)]
enum BadgeLinkSet {
    None,
    One(BadgeLink),
    ManyResult(Vec<CreateResult<BadgeLink>>),
}

impl From<BadgeLink> for BadgeLinkSet {
    fn from(badge: BadgeLink) -> Self {
        Self::One(badge)
    }
}

impl From<Option<BadgeLink>> for BadgeLinkSet {
    fn from(badge: Option<BadgeLink>) -> Self {
        match badge {
            Some(badge) => Self::One(badge),
            None => Self::None,
        }
    }
}

impl From<Vec<CreateResult<BadgeLink>>> for BadgeLinkSet {
    fn from(badges: Vec<CreateResult<BadgeLink>>) -> Self {
        Self::ManyResult(badges)
    }
}

impl BadgeLinkSet {
    fn from_config(
        config: &BadgeItem,
        manifest: &ManifestFile,
        workspace: &Metadata,
        package: &Package,
    ) -> CreateResult<Self> {
        Ok(match config {
            BadgeItem::Maintenance => BadgeLink::maintenance(manifest)?.into(),
            BadgeItem::License(license) => BadgeLink::license(license, manifest, package)?.into(),
            BadgeItem::CratesIo => BadgeLink::crates_io(manifest, package).into(),
            BadgeItem::DocsRs => BadgeLink::docs_rs(manifest, package).into(),
            BadgeItem::RustVersion => BadgeLink::rust_version(manifest, package)?.into(),
            BadgeItem::GithubActions(github_actions) => {
                BadgeLink::github_actions(github_actions, manifest, workspace, package)?.into()
            }
            BadgeItem::Codecov(codecov) => BadgeLink::codecov(codecov, manifest, package)?.into(),
        })
    }
}

#[derive(Debug, Snafu, miette::Diagnostic)]
#[snafu(display("failed to create badges"))]
pub(in super::super) struct CreateAllBadgesError {
    #[related]
    errors: Vec<CreateBadgeError>,
}

#[derive(Debug, Snafu, miette::Diagnostic)]
enum CreateBadgeError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    GetConfig {
        #[snafu(source)]
        #[diagnostic_source]
        source: GetConfigError,
    },
    #[snafu(display("neither `package.license` nor `package.license-file` is set: {path}"))]
    MissingLicenseMetadata { path: Utf8PathBuf },
    #[snafu(display("`package.rust-version` is not set: {path}"))]
    MissingRustVersionMetadata { path: Utf8PathBuf },
    #[snafu(display("`package.repository` is not set: {path}"))]
    MissingRepositoryMetadata { path: Utf8PathBuf },
    #[snafu(display("failed to open GitHub Actions workflows directory: {path}"))]
    OpenWorkflowsDir {
        #[snafu(source)]
        source: io::Error,
        path: Utf8PathBuf,
    },
    #[snafu(display("failed to read GitHub Actions workflows directory: {path}"))]
    ReadWorkflowsDir {
        #[snafu(source)]
        source: io::Error,
        path: Utf8PathBuf,
    },
    #[snafu(display("failed to read GitHub Actions workflow file: {path}"))]
    ReadWorkflowFile {
        #[snafu(source)]
        source: io::Error,
        path: Utf8PathBuf,
    },
    #[snafu(display("failed to parse GitHub Actions workflow file: {path}"))]
    ParseWorkflowFile {
        #[snafu(source)]
        source: serde_yaml::Error,
        path: Utf8PathBuf,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
        #[label]
        span: Option<SourceSpan>,
    },
    #[snafu(display("`package.repository` must start with `https://github.com/`"))]
    InvalidGithubRepository,
}

#[derive(Debug, Clone)]
struct ShieldsIo<'a> {
    path: Cow<'a, str>,
    label: Option<Cow<'a, str>>,
    logo: Option<Cow<'a, str>>,
    extra_queries: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> ShieldsIo<'a> {
    fn with_path(path: impl Into<Cow<'a, str>>) -> Self {
        Self {
            path: path.into(),
            label: None,
            logo: None,
            extra_queries: vec![],
        }
    }

    fn new_static(label: &str, message: &str, color: &str) -> Self {
        let message = message
            .replace('-', "--")
            .replace('_', "__")
            .replace(' ', "_");
        Self::with_path(format!("badge/{label}-{message}-{color}.svg"))
    }

    fn new_maintenance(status: MaintenanceStatus) -> Option<Self> {
        use MaintenanceStatus as Ms;
        // image url borrowed from https://gist.github.com/taiki-e/ad73eaea17e2e0372efb76ef6b38f17b
        let color = match status {
            Ms::ActivelyDeveloped => "brightgreen",
            Ms::PassivelyMaintained => "yellowgreen",
            Ms::AsIs => "yellow",
            Ms::Experimental => "blue",
            Ms::LookingForMaintainer => "orange",
            Ms::Deprecated => "red",
            Ms::None => return None,
        };
        Some(Self::new_static("maintenance", status.as_str(), color))
    }

    fn new_license(package_name: &str) -> Self {
        Self::with_path(format!("crates/l/{package_name}.svg"))
    }

    fn new_version(package_name: &str) -> Self {
        Self::with_path(format!("crates/v/{package_name}.svg"))
    }

    fn new_docs_rs(package_name: &str) -> Self {
        Self::with_path(format!("docsrs/{package_name}.svg"))
    }

    fn new_rust_version(version: &VersionReq) -> Self {
        Self::new_static("rust", &format!("{version}"), "93450a")
    }

    fn new_github_actions(repo_path: &str, name: &str) -> Self {
        Self::with_path(format!(
            "github/actions/workflow/status/{repo_path}/{name}.svg"
        ))
    }

    fn new_codecov(repo_path: &'a str, component: Option<&'a str>, flag: Option<&'a str>) -> Self {
        let mut this = Self::with_path(format!("codecov/c/github/{repo_path}.svg"));
        if let Some(component) = component {
            this.extra_queries
                .push(("component".into(), component.into()));
        }
        if let Some(flag) = flag {
            this.extra_queries.push(("flag".into(), flag.into()));
        }
        this
    }

    fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn logo(mut self, logo: impl Into<Cow<'a, str>>) -> Self {
        self.logo = Some(logo.into());
        self
    }

    fn build(self, manifest: &ManifestFile) -> Url {
        let mut url = Url::parse("https://img.shields.io/").unwrap();
        url.set_path(&self.path);
        {
            let mut query = url.query_pairs_mut();
            if let Some(label) = self.label {
                query.append_pair("label", &label);
            }
            if let Some(logo) = self.logo {
                query.append_pair("logo", &logo);
            }
            if let Some(style) = &manifest.value.config().badge.style {
                query.append_pair("style", style.as_str());
            }
            for (key, value) in self.extra_queries {
                query.append_pair(&key, &value);
            }
            query.finish();
        }
        url
    }
}

#[derive(Debug, Clone)]
struct BadgeLink {
    alt: String,
    link: Option<String>,
    image: String,
}

impl fmt::Display for BadgeLink {
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
            write!(f, "![{}]({})", Escape(&self.alt, need_escape), self.image)
        }
    }
}

impl BadgeLink {
    fn maintenance(manifest: &ManifestFile) -> CreateResult<Option<Self>> {
        let status = (|| manifest.try_badges()?.try_maintenance()?.try_status())()
            .map_err(|err| CreateBadgeError::from(err.with_key("badges.maintenance.status")))?;

        let image = match ShieldsIo::new_maintenance(status) {
            Some(shields_io) => shields_io.build(manifest).to_string(),
            None => return Ok(None),
        };

        let alt = format!("Maintenance: {}", status.as_str());
        let link = Some(
            "https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section".to_owned(),
        );

        let badge = Self { alt, link, image };
        Ok(Some(badge))
    }

    fn license(
        license: &License,
        manifest: &ManifestFile,
        package: &Package,
    ) -> CreateResult<Self> {
        let (license_str, license_path) = if let Some(name) = &package.license {
            (name.as_str(), package.license_file.as_deref())
        } else if let Some(file) = &package.license_file {
            ("non-standard", Some(file.as_ref()))
        } else {
            return Err(MissingLicenseMetadataSnafu {
                path: &package.manifest_path,
            }
            .build()
            .into());
        };

        let alt = format!("License: {license_str}");
        let link = license
            .link
            .clone()
            .or_else(|| license_path.map(ToString::to_string));
        let image = ShieldsIo::new_license(&package.name)
            .build(manifest)
            .to_string();
        Ok(Self { alt, link, image })
    }

    fn crates_io(manifest: &ManifestFile, package: &Package) -> Self {
        let alt = "crates.io".to_owned();
        let link = Some(format!("https://crates.io/crates/{}", package.name));
        let image = ShieldsIo::new_version(&package.name)
            .logo("rust")
            .build(manifest)
            .to_string();
        Self { alt, link, image }
    }

    fn docs_rs(manifest: &ManifestFile, package: &Package) -> Self {
        let alt = "docs.rs".to_owned();
        let link = Some(format!("https://docs.rs/{}", package.name));
        let image = ShieldsIo::new_docs_rs(&package.name)
            .logo("docs.rs")
            .build(manifest)
            .to_string();
        Self { alt, link, image }
    }

    fn rust_version(manifest: &ManifestFile, package: &Package) -> CreateResult<Self> {
        let rust_version =
            package
                .rust_version
                .as_ref()
                .context(MissingRustVersionMetadataSnafu {
                    path: &package.manifest_path,
                })?;

        let rust_version = VersionReq {
            comparators: vec![Comparator {
                op: Op::Caret,
                major: rust_version.major,
                minor: Some(rust_version.minor),
                patch: Some(rust_version.patch),
                pre: rust_version.pre.clone(),
            }],
        };

        let alt = format!("Rust: {rust_version}");
        let link = Some(
            "https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field"
                .to_owned(),
        );
        let image = ShieldsIo::new_rust_version(&rust_version)
            .logo("rust")
            .build(manifest)
            .to_string();
        Ok(Self { alt, link, image })
    }

    fn github_actions(
        github_actions: &GithubActions,
        manifest: &ManifestFile,
        workspace: &Metadata,
        package: &Package,
    ) -> CreateResult<Vec<CreateResult<Self>>> {
        let repository = package
            .repository
            .as_ref()
            .context(MissingRepositoryMetadataSnafu {
                path: &package.manifest_path,
            })?;
        let repo_path = repository
            .strip_prefix("https://github.com/")
            .context(InvalidGithubRepositorySnafu)?;

        let results = if github_actions.workflows.is_empty() {
            Self::github_actions_from_directory(workspace)?
        } else {
            Self::github_actions_from_config(&github_actions.workflows, workspace)
        };

        let results = results
            .into_iter()
            .map(|res| {
                res.map(|(name, file)| {
                    let alt = format!("GitHub Actions: {name}");
                    let link = format!(
                        "{}/actions/workflows/{}",
                        repository.trim_end_matches('/'),
                        file
                    );
                    let image = ShieldsIo::new_github_actions(repo_path, &file)
                        .label(&name)
                        .logo("github")
                        .build(manifest)
                        .to_string();
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

    fn codecov(
        codecov: &Codecov,
        manifest: &ManifestFile,
        package: &Package,
    ) -> CreateResult<Self> {
        let repository = package
            .repository
            .as_ref()
            .context(MissingRepositoryMetadataSnafu {
                path: &package.manifest_path,
            })?;
        let repo_path = repository
            .strip_prefix("https://github.com/")
            .context(InvalidGithubRepositorySnafu)?;

        let alt = "Codecov".to_owned();
        let link = format!("https://codecov.io/gh/{}", repo_path.trim_end_matches('/'));
        let image = ShieldsIo::new_codecov(
            repo_path,
            codecov.component.as_deref(),
            codecov.flag.as_deref(),
        )
        .label("codecov")
        .logo("codecov")
        .build(manifest)
        .to_string();
        Ok(Self {
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
                tracing::warn!(
                    "GitHub Actions workflows directory does not exist: {workflows_dir_path}"
                );
                return Ok(vec![]);
            }
            Err(source) => {
                return Err(OpenWorkflowsDirSnafu {
                    path: workflows_dir_path,
                }
                .into_error(source)
                .into());
            }
        };

        for res in dirs {
            let entry = match res {
                Ok(entry) => entry,
                Err(source) => {
                    badges.push(Err(ReadWorkflowsDirSnafu {
                        path: workflows_dir_path.clone(),
                    }
                    .into_error(source)
                    .into()));
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

        badges.sort_by(|a, b| match (a, b) {
            (Ok((a_name, a_file)), Ok((b_name, b_file))) => {
                a_name.cmp(b_name).then_with(|| a_file.cmp(b_file))
            }
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Err(_), Err(_)) => Ordering::Equal,
        });

        Ok(badges)
    }

    fn github_actions_from_config(
        workflows: &[GithubActionsWorkflow],
        workspace: &Metadata,
    ) -> Vec<CreateResult<(String, String)>> {
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

        badges
    }
}

fn read_workflow_name(workspace: &Metadata, path: &Utf8Path) -> CreateResult<String> {
    #[derive(Debug, Deserialize)]
    struct Workflow {
        #[serde(default)]
        name: Option<String>,
    }

    let text = fs::read_to_string(path).context(ReadWorkflowFileSnafu { path })?;

    let workflow: Workflow = serde_yaml::from_str(&text).with_context(|source| {
        let span = source.location().map(|l| SourceSpan::from((l.index(), 0)));
        ParseWorkflowFileSnafu {
            path,
            source_code: NamedSource::new(path, text.into()),
            span,
        }
    })?;

    // https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions
    // > If you omit name, GitHub sets it to the workflow file path relative to the
    // > root of the repository.
    Ok(workflow.name.unwrap_or_else(|| {
        path.strip_prefix(&workspace.workspace_root)
            .unwrap()
            .to_string()
    }))
}
