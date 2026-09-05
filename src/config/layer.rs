use std::{collections::HashMap, hash::Hash};

use indexmap::IndexMap;

pub(in crate::config) trait ApplyLayer {
    fn apply_layer(&mut self, layer: &Self);
}

impl<T> ApplyLayer for Option<T>
where
    T: ApplyLayer + Clone,
{
    fn apply_layer(&mut self, layer: &Self) {
        match (self, layer) {
            (Some(target), Some(layer)) => target.apply_layer(layer),
            (Some(_target), None) => {}
            (target @ None, layer) => target.clone_from(layer),
        }
    }
}

impl<T> ApplyLayer for Vec<T>
where
    T: Clone,
{
    fn apply_layer(&mut self, layer: &Self) {
        self.extend(layer.iter().cloned());
    }
}

impl<K, V> ApplyLayer for IndexMap<K, V>
where
    K: Hash + Eq + Clone,
    V: ApplyLayer + Clone,
{
    fn apply_layer(&mut self, layer: &Self) {
        for (key, value) in layer {
            self.entry(key.clone())
                .and_modify(|target| target.apply_layer(value))
                .or_insert_with(|| value.clone());
        }
    }
}

impl<K, V> ApplyLayer for HashMap<K, V>
where
    K: Hash + Eq + Clone,
    V: ApplyLayer + Clone,
{
    fn apply_layer(&mut self, layer: &Self) {
        for (key, value) in layer {
            self.entry(key.clone())
                .and_modify(|target| target.apply_layer(value))
                .or_insert_with(|| value.clone());
        }
    }
}

impl ApplyLayer for String {
    fn apply_layer(&mut self, layer: &Self) {
        self.clone_from(layer);
    }
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use crate::config::testing;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ScalarAndSequence {
        scalar: String,
        sequence: Vec<&'static str>,
    }

    impl ScalarAndSequence {
        fn new<Scalar, Sequence>(scalar: Scalar, sequence: Sequence) -> Self
        where
            Scalar: Into<String>,
            Sequence: Into<Vec<&'static str>>,
        {
            Self {
                scalar: scalar.into(),
                sequence: sequence.into(),
            }
        }
    }

    impl ApplyLayer for ScalarAndSequence {
        fn apply_layer(&mut self, layer: &Self) {
            self.scalar.apply_layer(&layer.scalar);
            self.sequence.apply_layer(&layer.sequence);
        }
    }

    #[test]
    fn string_apply_layer_replaces_target_value() {
        let mut scalar = "from target".to_owned();
        let layer_scalar = "from layer".to_owned();

        scalar.apply_layer(&layer_scalar);

        assert_eq!(scalar, "from layer".to_owned());
    }

    #[test]
    fn vec_apply_layer_appends_items_in_order() {
        let mut sequence = vec!["target-1", "target-2"];
        let layer_sequence = vec!["layer-1", "layer-2"];

        sequence.apply_layer(&layer_sequence);

        assert_eq!(sequence, vec!["target-1", "target-2", "layer-1", "layer-2"]);
    }

    #[test]
    fn option_apply_layer_merges_inner_values_and_ignores_none() {
        let mut value = Some(ScalarAndSequence {
            scalar: "from target".to_owned(),
            sequence: vec!["target"],
        });
        let layer = Some(ScalarAndSequence::new("from layer", ["layer"]));
        value.apply_layer(&layer);
        assert_eq!(
            value,
            Some(ScalarAndSequence::new("from layer", ["target", "layer"]))
        );

        let mut existing = Some("from target".to_owned());
        existing.apply_layer(&None);
        assert_eq!(existing, Some("from target".to_owned()));

        let mut missing = None;
        let layer = Some("from layer".to_owned());
        missing.apply_layer(&layer);
        assert_eq!(missing, Some("from layer".to_owned()));
    }

    #[test]
    fn index_map_apply_layer_updates_values_without_reordering_keys() {
        let mut target = IndexMap::from([
            ("updated", ScalarAndSequence::new("from target", ["target"])),
            (
                "untouched",
                ScalarAndSequence::new("keep target", ["keep target"]),
            ),
        ]);
        let layer = IndexMap::from([
            ("updated", ScalarAndSequence::new("from layer", ["layer"])),
            (
                "inserted",
                ScalarAndSequence::new("only in layer", ["only in layer"]),
            ),
        ]);

        target.apply_layer(&layer);

        testing::assert_indexmap_eq(
            &target,
            [
                (
                    "updated",
                    ScalarAndSequence::new("from layer", ["target", "layer"]),
                ),
                (
                    "untouched",
                    ScalarAndSequence::new("keep target", ["keep target"]),
                ),
                (
                    "inserted",
                    ScalarAndSequence::new("only in layer", ["only in layer"]),
                ),
            ],
        );
    }

    #[test]
    fn hash_map_apply_layer_updates_values_and_inserts_missing_keys() {
        let mut target =
            HashMap::from([("updated", ScalarAndSequence::new("from target", ["target"]))]);
        let layer = HashMap::from([
            ("updated", ScalarAndSequence::new("from layer", ["layer"])),
            (
                "inserted",
                ScalarAndSequence::new("only in layer", ["only in layer"]),
            ),
        ]);

        target.apply_layer(&layer);

        assert_eq!(
            target,
            HashMap::from([
                (
                    "updated",
                    ScalarAndSequence::new("from layer", ["target", "layer"]),
                ),
                (
                    "inserted",
                    ScalarAndSequence::new("only in layer", ["only in layer"]),
                ),
            ])
        );
    }
}
