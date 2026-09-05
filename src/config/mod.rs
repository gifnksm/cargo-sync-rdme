use cargo_metadata::camino::Utf8PathBuf;
use serde::Deserialize;

use crate::config::{badge::Badge, layer::ApplyLayer, rustdoc::Rustdoc};

pub(crate) use self::loader::*;

pub(crate) mod badge;
mod de;
mod layer;
mod loader;
pub(crate) mod rustdoc;
#[cfg(test)]
mod testing;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Config {
    #[serde(default, deserialize_with = "de::string_or_seq_of_path_from_source")]
    pub(crate) extra_targets: Vec<Utf8PathBuf>,
    #[serde(default)]
    pub(crate) badge: Badge,
    #[serde(default)]
    pub(crate) rustdoc: Rustdoc,
}

impl ApplyLayer for Config {
    fn apply_layer(&mut self, layer: &Self) {
        let Self {
            extra_targets,
            badge,
            rustdoc,
        } = self;
        extra_targets.apply_layer(&layer.extra_targets);
        badge.apply_layer(&layer.badge);
        rustdoc.apply_layer(&layer.rustdoc);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Inheritable<T> {
    #[default]
    Inherit,
    Disabled,
    Value(T),
}

impl<T> ApplyLayer for Inheritable<T>
where
    T: Clone + ApplyLayer,
{
    fn apply_layer(&mut self, layer: &Self) {
        match (&mut *self, layer) {
            (_, Self::Inherit) => {}
            (target, Self::Disabled) => *target = Self::Disabled,
            (target @ (Self::Disabled | Self::Inherit), Self::Value(layer)) => {
                *target = Self::Value(layer.clone());
            }
            (Self::Value(target), Self::Value(layer)) => target.apply_layer(layer),
        }
    }
}

impl<T> Inheritable<T> {
    pub(crate) fn as_option(&self) -> Option<&T> {
        match self {
            Self::Inherit | Self::Disabled => None,
            Self::Value(value) => Some(value),
        }
    }

    pub(crate) fn map<U, F>(self, f: F) -> Inheritable<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Inherit => Inheritable::Inherit,
            Self::Disabled => Inheritable::Disabled,
            Self::Value(value) => Inheritable::Value(f(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use indoc::indoc;
    use similar_asserts::assert_eq;

    use super::*;

    #[test]
    fn config_deserialize_deserializes_extra_targets_from_str() {
        let manifest = testing::manifest(indoc! {r#"
            [package]
            name = "foo"
            version = "0.1.0"

            [package.metadata.cargo-sync-rdme]
            extra-targets = "./docs/target.md"
        "#});

        let config = manifest.package_config().unwrap().unwrap();
        assert_eq!(config.extra_targets, ["/path/to/workspace/docs/target.md"]);
    }

    #[test]
    fn config_apply_layer_updates_extra_targets_badge_and_rustdoc() {
        let mut target = Config {
            extra_targets: vec!["./docs/target.md".into()],
            badge: Badge {
                style: Some(badge::BadgeStyle::Flat),
                ..Badge::default()
            },
            rustdoc: Rustdoc {
                html_root_url: Some("https://docs.example.com/target/".to_owned()),
                mappings: HashMap::from([(
                    "target::TargetType".to_owned(),
                    "https://reference.example.com/items/target-type".into(),
                )]),
            },
        };
        let layer = Config {
            extra_targets: vec!["./docs/layer.md".into()],
            badge: Badge {
                style: Some(badge::BadgeStyle::FlatSquare),
                ..Badge::default()
            },
            rustdoc: Rustdoc {
                html_root_url: Some("https://docs.example.com/layer/".to_owned()),
                mappings: HashMap::from([(
                    "target::LayerType".to_owned(),
                    "https://reference.example.com/items/layer-type".to_owned(),
                )]),
            },
        };

        target.apply_layer(&layer);

        let Config {
            extra_targets: target_extra_targets,
            badge: target_badge,
            rustdoc: target_rustdoc,
        } = target;

        assert_eq!(
            target_extra_targets,
            ["./docs/target.md", "./docs/layer.md"]
        );
        assert_eq!(target_badge.style, Some(badge::BadgeStyle::FlatSquare));
        assert_eq!(
            target_rustdoc,
            Rustdoc {
                html_root_url: Some("https://docs.example.com/layer/".to_owned()),
                mappings: HashMap::from([
                    (
                        "target::TargetType".to_owned(),
                        "https://reference.example.com/items/target-type".to_owned(),
                    ),
                    (
                        "target::LayerType".to_owned(),
                        "https://reference.example.com/items/layer-type".to_owned(),
                    ),
                ]),
            }
        );
    }

    #[test]
    fn inheritable_apply_layer_handles_all_state_transitions() {
        let mut target = Inheritable::Value("from target".to_owned());
        target.apply_layer(&Inheritable::Inherit);
        assert_eq!(target, Inheritable::Value("from target".to_owned()));

        let mut target = Inheritable::Value("from target".to_owned());
        target.apply_layer(&Inheritable::Disabled);
        assert_eq!(target, Inheritable::Disabled);

        let mut inherited = Inheritable::Inherit;
        inherited.apply_layer(&Inheritable::Value("from layer".to_owned()));
        assert_eq!(inherited, Inheritable::Value("from layer".to_owned()));

        let mut disabled = Inheritable::Disabled;
        disabled.apply_layer(&Inheritable::Value("from layer".to_owned()));
        assert_eq!(disabled, Inheritable::Value("from layer".to_owned()));

        let mut target = Inheritable::Value("from target".to_owned());
        target.apply_layer(&Inheritable::Value("from layer".to_owned()));
        assert_eq!(target, Inheritable::Value("from layer".to_owned()));
    }

    #[test]
    fn inheritable_helpers_preserve_state() {
        assert_eq!(Inheritable::<String>::Inherit.as_option(), None);
        assert_eq!(Inheritable::<String>::Disabled.as_option(), None);
        assert_eq!(
            Inheritable::Value("value".to_owned()).as_option(),
            Some(&"value".to_owned())
        );

        assert_eq!(
            Inheritable::Value("value".to_owned()).map(|value| value.len()),
            Inheritable::Value(5)
        );
        assert_eq!(
            Inheritable::<String>::Disabled.map(|value| value.len()),
            Inheritable::Disabled
        );
        assert_eq!(
            Inheritable::<String>::Inherit.map(|value| value.len()),
            Inheritable::Inherit
        );
    }
}
