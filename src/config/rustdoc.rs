use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Rustdoc {
    #[serde(default)]
    pub(crate) html_root_url: Option<String>,
    #[serde(default)]
    pub(crate) mappings: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use similar_asserts::assert_eq;

    use crate::config::testing;

    use super::*;

    #[test]
    fn deserialize_rustdoc_parses_valid_maps() {
        let source = testing::rustdoc_manifest(indoc! {r#"
            html-root-url = "https://example.com/docs/"
            mappings = {
              SomeType = "./docs/some-type.md",
              SomeTrait = "./docs/some-trait.md",
            }
        "#});
        let rustdoc = testing::parse_rustdoc(&source);
        assert_eq!(
            rustdoc,
            Rustdoc {
                html_root_url: Some("https://example.com/docs/".into()),
                mappings: HashMap::from([
                    ("SomeType".into(), "./docs/some-type.md".into()),
                    ("SomeTrait".into(), "./docs/some-trait.md".into()),
                ]),
            }
        );
    }
}
