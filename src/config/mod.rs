use std::sync::Arc;

use miette::{NamedSource, SourceSpan};
use snafu::Snafu;

// To detect items that do not have explicit values, wrap cargo's standard
// configuration items in Options.

mod de;
pub(crate) mod manifest;
#[cfg(test)]
mod testing;

#[derive(Debug, Snafu, miette::Diagnostic)]
pub(crate) enum GetConfigError {
    #[snafu(transparent)]
    #[diagnostic(transparent)]
    KeyNotSet {
        #[snafu(source(from(KeyNotSet, Box::new)))]
        #[diagnostic_source]
        source: Box<KeyNotSet>,
    },
}

#[derive(Debug, Snafu, miette::Diagnostic)]
#[snafu(display("key `{key}` is not set in `{path}`", path = source_code.name()))]
pub(crate) struct KeyNotSet {
    key: String,
    #[label]
    span: SourceSpan,
    #[source_code]
    source_code: NamedSource<Arc<str>>,
}

impl GetConfigError {
    pub(crate) fn with_key(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        match &mut self {
            Self::KeyNotSet { source } => source.key = key,
        }
        self
    }
}
