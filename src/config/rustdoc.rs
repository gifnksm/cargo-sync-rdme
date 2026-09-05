use std::collections::HashMap;

use serde::Deserialize;

use crate::config::ApplyLayer;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Rustdoc {
    #[serde(default)]
    pub(crate) html_root_url: Option<String>,
    #[serde(default)]
    pub(crate) mappings: HashMap<String, String>,
}

impl ApplyLayer for Rustdoc {
    fn apply_layer(&mut self, layer: &Self) {
        let Self {
            html_root_url,
            mappings,
        } = self;
        html_root_url.apply_layer(&layer.html_root_url);
        mappings.apply_layer(&layer.mappings);
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
        };
        let layer = Rustdoc {
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
        };

        target.apply_layer(&layer);

        let Rustdoc {
            html_root_url: target_html_root_url,
            mappings: target_mappings,
        } = target;

        assert_eq!(
            target_html_root_url,
            Some("https://docs.example.com/layer/".to_owned())
        );
        assert_eq!(
            target_mappings,
            HashMap::from([
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
            ])
        );
    }

    #[test]
    fn deserialize_rustdoc_parses_valid_maps() {
        let source = testing::rustdoc_manifest(indoc! {r#"
            html-root-url = "https://docs.example.com/my-crate/"
            mappings = {
              "std::io::Result" = "https://doc.rust-lang.org/stable/std/io/error/type.Result.html",
              "crate::SomeTrait" = "https://reference.example.com/items/some-trait",
            }
        "#});
        let rustdoc = testing::parse_rustdoc(&source);
        assert_eq!(
            rustdoc,
            Rustdoc {
                html_root_url: Some("https://docs.example.com/my-crate/".into()),
                mappings: HashMap::from([
                    (
                        "std::io::Result".into(),
                        "https://doc.rust-lang.org/stable/std/io/error/type.Result.html".into(),
                    ),
                    (
                        "crate::SomeTrait".into(),
                        "https://reference.example.com/items/some-trait".into(),
                    ),
                ]),
            }
        );
    }
}
