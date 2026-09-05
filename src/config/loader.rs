use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use cargo_metadata::{Package, PackageId, PackageName};
use miette::Diagnostic;

use snafu::{ResultExt as _, Snafu};

use crate::{
    args::Args,
    config::{ApplyLayer as _, Config},
    manifest::{ManifestError, ManifestLoader, ManifestLoaderError},
    source::SourceFilePath,
};

#[derive(Debug, Default)]
pub(crate) struct ConfigLoader {
    #[expect(clippy::option_option)]
    workspace_config: Option<Option<Arc<Config>>>,
    package_config: HashMap<PackageId, Option<Arc<Config>>>,
}

impl ConfigLoader {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn load_workspace_config(
        &mut self,
        manifest_loader: &mut ManifestLoader<'_>,
    ) -> Result<Option<Arc<Config>>, ConfigLoaderError> {
        if let Some(workspace_config) = &self.workspace_config {
            return Ok(workspace_config.clone());
        }

        let manifest = manifest_loader.load_workspace_manifest()?;
        let config =
            manifest
                .workspace_config()
                .with_context(|_source| LoadWorkspaceManifestSnafu {
                    manifest: manifest.source_file(),
                })?;
        let config = config.map(Arc::new);
        self.workspace_config = Some(config.clone());
        Ok(config)
    }

    pub(crate) fn load_package_config(
        &mut self,
        manifest_loader: &mut ManifestLoader<'_>,
        package: &Package,
    ) -> Result<Option<Arc<Config>>, ConfigLoaderError> {
        let entry = match self.package_config.entry(package.id.clone()) {
            Entry::Occupied(entry) => return Ok(entry.get().clone()),
            Entry::Vacant(entry) => entry,
        };

        let manifest = manifest_loader.load_package_manifest(package)?;
        let config =
            manifest
                .package_config()
                .with_context(|_source| LoadPackageManifestFileSnafu {
                    package: package.name.clone(),
                    manifest: manifest.source_file(),
                })?;
        let config = config.map(Arc::new);
        entry.insert(config.clone());
        Ok(config)
    }
}

impl Config {
    pub(crate) fn from_args(args: &Args) -> Config {
        let mut config = Config::default();

        if let Some(toolchain) = &args.toolchain.toolchain {
            config.rustdoc.toolchain = Some(toolchain.clone());
        }

        config
    }

    pub(crate) fn load(
        manifest_loader: &mut ManifestLoader<'_>,
        config_loader: &mut ConfigLoader,
        args: &Args,
        package: &Package,
    ) -> Result<Self, ConfigLoaderError> {
        let mut config = Self::default();

        if let Some(workspace_config) = &config_loader.load_workspace_config(manifest_loader)? {
            config.apply_layer(workspace_config);
        }

        if let Some(package_config) =
            &config_loader.load_package_config(manifest_loader, package)?
        {
            config.apply_layer(package_config);
        }

        let args_config = Self::from_args(args);
        config.apply_layer(&args_config);

        Ok(config)
    }
}

#[derive(Debug, Snafu, Diagnostic)]
pub(crate) enum ConfigLoaderError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    ManifestLoader {
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<ManifestLoaderError>,
    },
    #[snafu(display("failed to load workspace manifest file: {path}", path = manifest.workspace_path))]
    LoadWorkspaceManifest {
        manifest: SourceFilePath,
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<ManifestError>,
    },
    #[snafu(display("failed to load package manifest file for package `{package}`: {path}", path = manifest.workspace_path))]
    LoadPackageManifestFile {
        package: PackageName,
        manifest: SourceFilePath,
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<ManifestError>,
    },
}

#[cfg(test)]
mod tests {
    use std::{slice, sync::LazyLock};

    use cargo_metadata::{
        Metadata, MetadataBuilder, PackageBuilder, WorkspaceDefaultMembers, camino::Utf8Path,
        semver::Version,
    };
    use indoc::indoc;
    use similar_asserts::assert_eq;

    use crate::{
        config::{
            Inheritable,
            badge::{
                BadgeStyle,
                item::{BadgeItem, BadgeItemKey},
            },
            rustdoc::Rustdoc,
            testing,
        },
        manifest::Manifest,
        source::SourceFile,
    };

    use super::*;

    static WORKSPACE_ROOT: LazyLock<&Utf8Path> =
        LazyLock::new(|| Utf8Path::new("/path/to/workspace"));

    fn package<N>(name: N) -> Package
    where
        N: Into<String>,
    {
        let name = name.into();
        let id = PackageId {
            repr: format!("{name}@0.1.0"),
        };
        let manifest_path = WORKSPACE_ROOT.join(&name).join("Cargo.toml");
        let name = PackageName::new(name);
        let version = Version::new(0, 1, 0);
        PackageBuilder::new(name, version, id, manifest_path)
            .build()
            .unwrap()
    }

    fn root_package<N>(name: N) -> Package
    where
        N: Into<String>,
    {
        let name = name.into();
        let id = PackageId {
            repr: format!("{name}@0.1.0"),
        };
        let manifest_path = WORKSPACE_ROOT.join("Cargo.toml");
        let name = PackageName::new(name);
        let version = Version::new(0, 1, 0);
        PackageBuilder::new(name, version, id, manifest_path)
            .build()
            .unwrap()
    }

    fn workspace(packages: &[Package]) -> Metadata {
        MetadataBuilder::default()
            .workspace_root(*WORKSPACE_ROOT)
            .workspace_members(
                packages
                    .iter()
                    .map(|package| package.id.clone())
                    .collect::<Vec<_>>(),
            )
            .workspace_default_members(WorkspaceDefaultMembers::default())
            .packages(packages)
            .resolve(None)
            .target_directory(WORKSPACE_ROOT.join("target"))
            .build_directory(None)
            .workspace_metadata(serde_json::Value::Null)
            .version(1_usize)
            .build()
            .unwrap()
    }

    fn manifest(workspace_path: &str, source: &str) -> Arc<Manifest> {
        let source = SourceFile::new_for_test(workspace_path, source);
        Arc::new(Manifest::new_for_test(&source).unwrap())
    }

    #[test]
    fn config_load_applies_workspace_and_package_layers_for_member_package() {
        let package = package("member");
        let workspace = workspace(slice::from_ref(&package));
        let mut manifest_loader = ManifestLoader::new(&workspace);
        let mut config_loader = ConfigLoader::new();

        manifest_loader.set_workspace_manifest(manifest(
            "Cargo.toml",
            indoc! {r#"
                [workspace.metadata.cargo-sync-rdme]
                extra-targets = ["./docs/workspace.md"]

                [workspace.metadata.cargo-sync-rdme.badge]
                style = "flat"
                badges = { license = { link = "https://workspace.example/license" } }

                [workspace.metadata.cargo-sync-rdme.rustdoc]
                html-root-url = "https://docs.example.com/workspace/"
                mappings = { "member::SharedType" = "https://reference.example.com/items/shared-type-from-workspace" }
            "#},
        ));
        manifest_loader.add_package_manifest(
            package.id.clone(),
            manifest(
                "member/Cargo.toml",
                indoc! {r#"
                    [package]
                    name = "member"
                    version = "0.1.0"

                    [package.metadata.cargo-sync-rdme]
                    extra-targets = ["./docs/package.md"]

                    [package.metadata.cargo-sync-rdme.badge]
                    style = "flat-square"
                    badges = { license = false, crates-io = true }

                    [package.metadata.cargo-sync-rdme.rustdoc]
                    mappings = {
                      "member::SharedType" = "https://reference.example.com/items/shared-type-from-package",
                      "member::PackageType" = "https://reference.example.com/items/package-type",
                    }
                "#},
            ),
        );

        let config = Config::load(
            &mut manifest_loader,
            &mut config_loader,
            &Args::default(),
            &package,
        )
        .unwrap();
        let Config {
            extra_targets,
            badge,
            rustdoc,
        } = config;

        assert_eq!(
            extra_targets,
            [
                "/path/to/workspace/docs/workspace.md",
                "/path/to/workspace/member/docs/package.md",
            ]
        );
        assert_eq!(badge.style, Some(BadgeStyle::FlatSquare));
        testing::assert_indexmap_eq(
            &badge.default.unwrap(),
            [
                (BadgeItemKey::License(None), Inheritable::Disabled),
                (
                    BadgeItemKey::CratesIo(None),
                    Inheritable::Value(BadgeItem::CratesIo),
                ),
            ],
        );
        assert_eq!(
            rustdoc,
            Rustdoc {
                toolchain: None,
                html_root_url: Some("https://docs.example.com/workspace/".to_owned()),
                mappings: HashMap::from([
                    (
                        "member::SharedType".to_owned(),
                        "https://reference.example.com/items/shared-type-from-package".to_owned(),
                    ),
                    (
                        "member::PackageType".to_owned(),
                        "https://reference.example.com/items/package-type".to_owned(),
                    ),
                ]),
            }
        );
    }

    #[test]
    fn config_load_applies_workspace_and_package_layers_for_root_package() {
        let package = root_package("root");
        let workspace = workspace(slice::from_ref(&package));
        let mut manifest_loader = ManifestLoader::new(&workspace);
        let mut config_loader = ConfigLoader::new();

        manifest_loader.add_package_manifest(
            package.id.clone(),
            manifest(
                "Cargo.toml",
                indoc! {r#"
                    [package]
                    name = "root"
                    version = "0.1.0"

                    [workspace.metadata.cargo-sync-rdme]
                    extra-targets = ["./docs/workspace.md"]

                    [package.metadata.cargo-sync-rdme]
                    extra-targets = ["./docs/package.md"]
                "#},
            ),
        );

        let config = Config::load(
            &mut manifest_loader,
            &mut config_loader,
            &Args::default(),
            &package,
        )
        .unwrap();

        assert_eq!(
            config.extra_targets,
            [
                "/path/to/workspace/docs/workspace.md",
                "/path/to/workspace/docs/package.md"
            ]
        );
    }
}
