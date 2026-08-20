use std::range::Range;

use miette::SourceSpan;

use crate::traits::StrExt as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Spanned<T> {
    pub(crate) value: T,
    pub(crate) span: Range<usize>,
}

impl<T> Spanned<T> {
    pub(crate) fn new(value: T, span: Range<usize>) -> Self {
        Self { value, span }
    }

    pub(crate) fn source_span(&self) -> SourceSpan {
        SourceSpan::from(std::ops::Range::from(self.span))
    }

    pub(crate) fn as_deref(&self) -> Spanned<&T::Target>
    where
        T: std::ops::Deref,
    {
        Spanned {
            value: &self.value,
            span: self.span,
        }
    }
}

impl<'a> Spanned<&'a str> {
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

    fn substr(&self, substr: &'a str) -> Self {
        let range = self.value.substr_range_shim(substr).unwrap();
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
