use std::{fmt, marker::PhantomData, str::FromStr};

use cargo_metadata::camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use serde::{
    Deserialize, Deserializer,
    de::{self, IntoDeserializer as _, Visitor},
};
use void::{ResultVoidExt as _, Void};

use crate::{config::Inheritable, source};

pub(in crate::config) fn bool_or_map<'de, T, D>(deserializer: D) -> Result<Inheritable<T>, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    struct BoolOrMap<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for BoolOrMap<T>
    where
        T: Deserialize<'de> + Default,
    {
        type Value = Inheritable<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a boolean or a map")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v {
                Ok(Inheritable::Value(T::default()))
            } else {
                Ok(Inheritable::Disabled)
            }
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let v = T::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(Inheritable::Value(v))
        }
    }

    let map = deserializer.deserialize_any(BoolOrMap(PhantomData))?;
    Ok(map)
}

pub(in crate::config) fn string_or_seq<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    struct StringOrSeq<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrSeq<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or a seq")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut values = vec![];
            while let Some(value) = seq.next_element::<T>()? {
                values.push(value);
            }
            Ok(values)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![T::deserialize(v.into_deserializer())?])
        }
    }

    let seq = deserializer.deserialize_any(StringOrSeq(PhantomData))?;
    Ok(seq)
}

pub(in crate::config) fn string_or_map_or_seq<'de, T, D>(
    deserializer: D,
) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    struct StringOrMapOrSeq<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrMapOrSeq<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or a map or a seq")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            struct StringOrMap<T>(T);
            impl<'de, T> Deserialize<'de> for StringOrMap<T>
            where
                T: Deserialize<'de> + FromStr<Err = Void>,
            {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    string_or_map(deserializer).map(Self)
                }
            }

            let mut values = vec![];
            while let Some(value) = seq.next_element::<StringOrMap<T>>()? {
                values.push(value.0);
            }
            Ok(values)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![v.parse().void_unwrap()])
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let v = T::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(vec![v])
        }
    }

    let map = deserializer.deserialize_any(StringOrMapOrSeq(PhantomData))?;
    Ok(map)
}

pub(in crate::config) fn string_or_map<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    struct StringOrMap<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrMap<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or a map")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.parse().void_unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let v = T::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(v)
        }
    }

    let map = deserializer.deserialize_any(StringOrMap(PhantomData))?;
    Ok(map)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PathFromSource(pub(crate) Utf8PathBuf);

impl FromStr for PathFromSource {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Utf8PathBuf::from(s)))
    }
}

impl<'de> Deserialize<'de> for PathFromSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = Utf8PathBuf::deserialize(deserializer)?;
        if path.is_absolute() {
            return Ok(Self(path));
        }
        let current_source = source::current_source_file()?;
        let base_dir = current_source
            .path()
            .parent()
            .ok_or_else(|| de::Error::custom("current source file has no parent directory"))?;
        let path = normalize_path(&base_dir.join(path));
        Ok(Self(path))
    }
}

fn normalize_path(path: &Utf8Path) -> Utf8PathBuf {
    let mut components = path.components().peekable();
    components.next_if_eq(&Utf8Component::CurDir);
    components.collect()
}

pub(crate) fn string_or_seq_of_path_from_source<'de, D>(
    deserializer: D,
) -> Result<Vec<Utf8PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let seq = string_or_seq::<PathFromSource, _>(deserializer)?;
    Ok(seq.into_iter().map(|p| p.0).collect())
}
