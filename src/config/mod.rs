use std::{range::Range, sync::Arc};

use miette::{NamedSource, SourceSpan};
use snafu::Snafu;

use crate::{
    source::{SourceFileRef, Spanned},
    traits::RangeExt as _,
};

// To detect items that do not have explicit values, wrap cargo's standard
// configuration items in Options.

mod de;
pub(crate) mod manifest;
#[cfg(test)]
mod testing;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum GetConfigError {
    #[snafu(display("missing top-level key `{key}`"))]
    MissingTopLevelKey {
        key: String,
        #[label]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
    },
    #[snafu(display("missing key `{key}` in table `{table}`"))]
    MissingKeyInTable {
        key: String,
        table: String,
        #[label]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<Arc<str>>,
    },
}

impl GetConfigError {
    #[cfg(test)]
    pub(crate) fn into_missing_top_level_key(self) -> (String, SourceSpan, NamedSource<Arc<str>>) {
        let Self::MissingTopLevelKey {
            key,
            span,
            source_code,
        } = self
        else {
            panic!("unexpected error: {self:?}");
        };
        (key, span, source_code)
    }

    #[cfg(test)]
    pub(crate) fn into_missing_key_in_table(
        self,
    ) -> (String, String, SourceSpan, NamedSource<Arc<str>>) {
        let Self::MissingKeyInTable {
            key,
            table,
            span,
            source_code,
        } = self
        else {
            panic!("unexpected error: {self:?}");
        };
        (key, table, span, source_code)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TomlTable {
    pub(crate) source: SourceFileRef,
    pub(crate) path: Option<String>,
    pub(crate) key_span: Range<usize>,
}

impl TomlTable {
    pub(crate) fn root(source: SourceFileRef) -> Self {
        Self {
            source,
            path: None,
            key_span: (0..0).into(),
        }
    }

    pub(crate) fn child(&self, key: Spanned<&str>) -> Self {
        let path = if let Some(parent) = &self.path {
            format!("{parent}.{key}")
        } else {
            key.value.to_owned()
        };
        Self {
            source: self.source.clone(),
            path: Some(path),
            key_span: key.span,
        }
    }

    pub(crate) fn missing_key_error(&self, key: &str) -> GetConfigError {
        let span = self.key_span.to_span();
        let source_code = self.source.to_named_source();
        if let Some(table) = &self.path {
            MissingKeyInTableSnafu {
                key,
                table,
                span,
                source_code,
            }
            .build()
        } else {
            MissingTopLevelKeySnafu {
                key,
                span,
                source_code,
            }
            .build()
        }
    }
}
