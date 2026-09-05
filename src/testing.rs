use indexmap::IndexMap;
use similar_asserts::assert_eq;

#[track_caller]
pub(crate) fn assert_indexmap_eq<K, V, I>(actual: &IndexMap<K, V>, expected: I)
where
    I: IntoIterator<Item = (K, V)>,
    K: Clone + PartialEq,
    V: Clone + PartialEq,
{
    let actual = actual
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();
    let expected = expected.into_iter().collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
