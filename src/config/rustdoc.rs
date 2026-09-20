use std::collections::HashMap;

use serde::Deserialize;

use crate::config::ApplyLayer;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Rustdoc {
    #[serde(default)]
    pub(crate) toolchain: Option<String>,
    #[serde(default)]
    pub(crate) features: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) all_features: Option<bool>,
    #[serde(default)]
    pub(crate) no_default_features: Option<bool>,
    #[serde(default)]
    pub(crate) standard_library_url_mode: Option<StandardLibraryUrlMode>,
    #[serde(default)]
    pub(crate) html_root_url: Option<String>,
    #[serde(default)]
    pub(crate) mappings: HashMap<String, String>,
    #[expect(clippy::struct_field_names)]
    #[serde(default)]
    pub(crate) rustdoc_args: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) cargo_args: Option<Vec<String>>,
}

impl ApplyLayer for Rustdoc {
    fn apply_layer(&mut self, layer: &Self) {
        let Self {
            toolchain,
            features,
            all_features,
            no_default_features,
            standard_library_url_mode,
            html_root_url,
            mappings,
            rustdoc_args,
            cargo_args,
        } = self;
        toolchain.apply_layer(&layer.toolchain);
        features.apply_layer(&layer.features);
        all_features.apply_layer(&layer.all_features);
        no_default_features.apply_layer(&layer.no_default_features);
        standard_library_url_mode.apply_layer(&layer.standard_library_url_mode);
        html_root_url.apply_layer(&layer.html_root_url);
        mappings.apply_layer(&layer.mappings);
        rustdoc_args.apply_layer(&layer.rustdoc_args);
        cargo_args.apply_layer(&layer.cargo_args);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum StandardLibraryUrlMode {
    #[default]
    Channel,
    Version,
    AsIs,
}

impl ApplyLayer for StandardLibraryUrlMode {
    fn apply_layer(&mut self, layer: &Self) {
        *self = *layer;
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use similar_asserts::assert_eq;

    use crate::config::testing;

    use super::*;

    #[test]
    fn rustdoc_apply_layer_updates_html_root_url_and_mappings() {
        let mut target = Rustdoc {
            toolchain: Some("stable".to_owned()),
            standard_library_url_mode: Some(StandardLibraryUrlMode::Version),
            html_root_url: Some("https://docs.example.com/target/".to_owned()),
            mappings: HashMap::from([
                (
                    "target::TargetType".to_owned(),
                    "https://reference.example.com/items/target-type".to_owned(),
                ),
                (
                    "target::SharedType".to_owned(),
                    "https://reference.example.com/items/shared-type-from-target".to_owned(),
                ),
            ]),
            ..Default::default()
        };
        let layer = Rustdoc {
            toolchain: Some("nightly".to_owned()),
            standard_library_url_mode: Some(StandardLibraryUrlMode::AsIs),
            html_root_url: Some("https://docs.example.com/layer/".to_owned()),
            mappings: HashMap::from([
                (
                    "target::SharedType".to_owned(),
                    "https://reference.example.com/items/shared-type-from-layer".to_owned(),
                ),
                (
                    "target::LayerType".to_owned(),
                    "https://reference.example.com/items/layer-type".to_owned(),
                ),
            ]),
            ..Default::default()
        };

        target.apply_layer(&layer);

        assert_eq!(
            target,
            Rustdoc {
                toolchain: Some("nightly".to_owned()),
                standard_library_url_mode: Some(StandardLibraryUrlMode::AsIs),
                html_root_url: Some("https://docs.example.com/layer/".to_owned()),
                mappings: HashMap::from([
                    (
                        "target::TargetType".to_owned(),
                        "https://reference.example.com/items/target-type".to_owned(),
                    ),
                    (
                        "target::SharedType".to_owned(),
                        "https://reference.example.com/items/shared-type-from-layer".to_owned(),
                    ),
                    (
                        "target::LayerType".to_owned(),
                        "https://reference.example.com/items/layer-type".to_owned(),
                    ),
                ]),
                ..Default::default()
            }
        );
    }

    #[test]
    fn deserialize_rustdoc_parses_valid_maps() {
        let source = testing::rustdoc_manifest(indoc! {r#"
            toolchain = "stable"
            features = ["feature1", "feature2"]
            all-features = true
            no-default-features = false
            standard-library-url-mode = "version"
            html-root-url = "https://docs.example.com/my-crate/"
            mappings = {
              "std::io::Result" = "https://doc.rust-lang.org/stable/std/io/error/type.Result.html",
              "crate::SomeTrait" = "https://reference.example.com/items/some-trait",
            }
            rustdoc-args = ["--extern-html-root-takes-precedence", "--cfg=docsrs"]
            cargo-args = ["-Zrustdoc-scrape-examples"]
        "#});
        let rustdoc = testing::parse_rustdoc(&source);
        assert_eq!(
            rustdoc,
            Rustdoc {
                toolchain: Some("stable".to_owned()),
                features: Some(vec!["feature1".to_owned(), "feature2".to_owned()]),
                all_features: Some(true),
                no_default_features: Some(false),
                standard_library_url_mode: Some(StandardLibraryUrlMode::Version),
                html_root_url: Some("https://docs.example.com/my-crate/".to_owned()),
                mappings: HashMap::from([
                    (
                        "std::io::Result".to_owned(),
                        "https://doc.rust-lang.org/stable/std/io/error/type.Result.html".to_owned(),
                    ),
                    (
                        "crate::SomeTrait".to_owned(),
                        "https://reference.example.com/items/some-trait".to_owned(),
                    ),
                ]),
                rustdoc_args: Some(vec![
                    "--extern-html-root-takes-precedence".to_owned(),
                    "--cfg=docsrs".to_owned(),
                ]),
                cargo_args: Some(vec!["-Zrustdoc-scrape-examples".to_owned()]),
            }
        );
    }
}
