use std::{
    cmp,
    fmt::{self, Display},
    hash::{self, Hash},
    range::{Range, legacy},
};

use miette::SourceSpan;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Spanned<T> {
    pub(crate) value: T,
    pub(crate) span: Range<usize>,
}

impl<T> PartialEq for Spanned<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Spanned<T> where T: Eq {}

impl<T> PartialOrd for Spanned<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T> Ord for Spanned<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> Hash for Spanned<T>
where
    T: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.value.hash(state);
    }
}

impl<T> From<toml::Spanned<T>> for Spanned<T> {
    fn from(value: toml::Spanned<T>) -> Self {
        Self {
            span: value.span().into(),
            value: value.into_inner(),
        }
    }
}

impl<'a, T> From<&'a toml::Spanned<T>> for Spanned<&'a T> {
    fn from(value: &'a toml::Spanned<T>) -> Self {
        Self {
            span: value.span().into(),
            value: value.get_ref(),
        }
    }
}

impl<T> Spanned<T> {
    pub(crate) fn new<R>(value: T, span: R) -> Self
    where
        R: Into<Range<usize>>,
    {
        let span = span.into();
        Self { value, span }
    }

    pub(crate) fn source_span(&self) -> SourceSpan {
        SourceSpan::from(legacy::Range::from(self.span))
    }
}

impl<'a> Spanned<&'a str> {
    #[cfg(test)]
    pub(crate) fn from_str(value: &'a str) -> Self {
        Self {
            value,
            span: (0..value.len()).into(),
        }
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn assert_span(&self, span: Range<usize>, expected: &str) {
        let start = span.start - self.span.start;
        let end = span.end - self.span.start;
        let actual = &self.value[start..end];
        similar_asserts::assert_eq!(actual, expected);
    }

    #[cfg(test)]
    #[track_caller]
    #[expect(clippy::needless_pass_by_value)]
    pub(crate) fn assert_spanned<T>(&self, target: Spanned<T>, expected: &str) {
        self.assert_span(target.span, expected);
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn assert_source_span(&self, source_span: SourceSpan, expected: &str) {
        let start = source_span.offset() - self.span.start;
        let actual = &self.value[start..][..source_span.len()];
        similar_asserts::assert_eq!(actual, expected);
    }

    #[cfg(test)]
    #[track_caller]
    pub(crate) fn assert_spanned_str(&self, target: Spanned<&str>, expected: &str) {
        similar_asserts::assert_eq!(target.value, expected);
        self.assert_spanned(target, expected);
        // ensure that the target is substring of self by pointer address comparison.
        self.value.substr_range(target.value).unwrap();
    }

    pub(crate) fn prefix_of(&self, other: Self) -> Self {
        let end = other.span.start - self.span.start;
        let prefix = &self.value[..end];
        self.substr(prefix)
    }

    pub(crate) fn end(&self) -> Self {
        let end = &self.value[self.value.len()..];
        self.substr(end)
    }

    pub(crate) fn trim(&self) -> Self {
        let trimmed = self.value.trim();
        self.substr(trimmed)
    }

    pub(crate) fn trim_start(&self) -> Self {
        let trimmed = self.value.trim_start();
        self.substr(trimmed)
    }

    pub(crate) fn trim_end(&self) -> Self {
        let trimmed = self.value.trim_end();
        self.substr(trimmed)
    }

    pub(crate) fn strip_prefix_str(&self, prefix: &str) -> Option<Self> {
        let stripped = self.value.strip_prefix(prefix)?;
        Some(self.substr(stripped))
    }

    pub(crate) fn strip_suffix_str(&self, suffix: &str) -> Option<Self> {
        let stripped = self.value.strip_suffix(suffix)?;
        Some(self.substr(stripped))
    }

    pub(crate) fn split_once_fn(&self, f: impl Fn(char) -> bool) -> Option<(Self, Self)> {
        let (head, tail) = self.value.split_once(f)?;
        Some((self.substr(head), self.substr(tail)))
    }

    pub(crate) fn substr(&self, substr: &'a str) -> Self {
        let range = self.value.substr_range(substr).unwrap();
        Spanned {
            value: substr,
            span: Range {
                start: self.span.start + range.start,
                end: self.span.start + range.end,
            },
        }
    }
}

impl<T> PartialEq<str> for Spanned<T>
where
    T: PartialEq<str>,
{
    fn eq(&self, other: &str) -> bool {
        self.value == *other
    }
}

impl<T> PartialEq<&str> for Spanned<T>
where
    T: for<'a> PartialEq<&'a str>,
{
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

impl<T> Display for Spanned<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'de, T> Deserialize<'de> for Spanned<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let spanned = <toml::Spanned<T>>::deserialize(deserializer)?;
        Ok(Self {
            span: spanned.span().into(),
            value: spanned.into_inner(),
        })
    }
}
