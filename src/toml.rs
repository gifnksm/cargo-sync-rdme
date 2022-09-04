use std::{fs, io};

use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use miette::{NamedSource, SourceOffset, SourceSpan};
use serde::Deserialize;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum ReadTomlError {
    #[error("failed to read {name}: {path}")]
    Io {
        name: String,
        path: Utf8PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse {name}")]
    Parse {
        name: String,
        #[source]
        source: toml::de::Error,
        #[source_code]
        source_code: NamedSource,
        #[label]
        label: Option<SourceSpan>,
    },
}

pub(crate) fn parse_with_text<T>(
    name: &str,
    path: impl AsRef<Utf8Path>,
) -> Result<(String, T), ReadTomlError>
where
    T: for<'de> Deserialize<'de>,
{
    let path = path.as_ref();

    let text = fs::read_to_string(path).map_err(|err| ReadTomlError::Io {
        name: name.to_owned(),
        path: path.to_owned(),
        source: err,
    })?;

    let data: T = toml::from_str(&text).map_err(|err| {
        let label = err.line_col().map(|(line, col)| {
            let offset = SourceOffset::from_location(&text, line + 1, col + 1);
            SourceSpan::new(offset, SourceOffset::from(1))
        });
        let source_code = NamedSource::new(path, text.clone());
        ReadTomlError::Parse {
            name: name.to_owned(),
            source: err,
            source_code,
            label,
        }
    })?;

    Ok((text, data))
}
