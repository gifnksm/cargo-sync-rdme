use std::{
    fs,
    io::{self, Write},
};

use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    Metadata, Package,
};
use miette::{IntoDiagnostic, NamedSource, WrapErr};
use pulldown_cmark::{Options, Parser};
use tempfile::NamedTempFile;

use crate::{cli::App, config::Manifest, traits::PackageExt, with_source::WithSource, Result};

mod contents;
mod marker;

#[derive(Debug, Clone)]
struct ReadmeFile {
    path: Utf8PathBuf,
    text: String,
}

impl ReadmeFile {
    fn new(package: &Package) -> Result<Self> {
        let readme = &package
            .readme
            .as_deref()
            .ok_or_else(|| miette!("no README found. Please specify package.readme"))?;
        let path = package.root_directory().join(readme);
        let text = fs::read_to_string(&path)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!(
                    "failed to read README of {package}: {path}",
                    package = package.name
                )
            })?;
        Ok(Self { path, text })
    }

    fn to_named_source(&self) -> NamedSource {
        NamedSource::new(self.path.clone(), self.text.clone())
    }
}

type ManifestFile = WithSource<Manifest>;

pub(crate) fn sync_readme(app: &App, workspace: &Metadata, package: &Package) -> Result<()> {
    let manifest = ManifestFile::from_toml("package manifest", &package.manifest_path)?;

    let readme = ReadmeFile::new(package)?;

    // Setup README parser
    let parser = Parser::new_ext(&readme.text, Options::all()).into_offset_iter();

    // Find replace markers from README
    let all_markers = marker::find_all(&readme, &manifest, parser)?;

    // Create contents for each marker
    let replaces = all_markers.iter().map(|x| x.0.clone());
    let all_contents = contents::create_all(replaces, app, &manifest, workspace, package)?;

    // Replace markers with content
    let new_text = marker::replace_all(&readme.text, &all_markers, &all_contents);

    // Compare new README with old README
    let changed = new_text != readme.text;
    if !changed {
        return Ok(());
    }

    // Update README if allowed
    app.fix.check_update_allowed(&readme.path)?;
    write_readme(&readme.path, &new_text)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to write README: {}", readme.path))?;

    Ok(())
}

pub(crate) fn write_readme(path: &Utf8Path, text: &str) -> io::Result<()> {
    let output_dir = path.parent().unwrap();
    let mut tempfile = NamedTempFile::new_in(output_dir)?;
    tempfile.as_file_mut().write_all(text.as_bytes())?;
    tempfile.as_file_mut().sync_data()?;
    let file = tempfile.persist(path).map_err(|err| err.error)?;
    file.sync_all()?;
    drop(file);
    Ok(())
}
