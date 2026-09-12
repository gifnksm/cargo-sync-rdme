use std::fmt::Debug;

use indexmap::IndexMap;
use similar_asserts::assert_eq;

#[track_caller]
pub(crate) fn assert_indexmap_eq<K, V, I>(actual: &IndexMap<K, V>, expected: I)
where
    I: IntoIterator<Item = (K, V)>,
    K: Clone + PartialEq + Debug,
    V: Clone + PartialEq + Debug,
{
    let actual = actual
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();
    let expected = expected.into_iter().collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
