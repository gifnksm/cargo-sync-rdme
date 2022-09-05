use std::{fmt::Write, process::ExitStatus};

use cargo_metadata::{Metadata, Package};
use pulldown_cmark::{BrokenLink, Event, Options, Parser, Tag};
use rustdoc_types::{Crate, Id, Item, ItemKind};

use crate::{
    with_source::{ReadFileError, WithSource},
    App,
};

type CreateResult<T> = Result<T, CreateRustdocError>;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(in super::super) enum CreateRustdocError {
    #[error("failed to create rustdoc process")]
    Spawn(#[source] std::io::Error),
    #[error("rustdoc exited with non-zero status code: {0}")]
    Exit(ExitStatus),
    #[error(transparent)]
    #[diagnostic(transparent)]
    ReadFileError(#[from] ReadFileError),
    #[error("crate {crate_name} does not have a crate-level documentation")]
    RootDocNotFound { crate_name: String },
}

pub(super) fn create(app: &App, workspace: &Metadata, package: &Package) -> CreateResult<String> {
    run_rustdoc(app, package)?;

    let output_file = workspace
        .target_directory
        .join("doc")
        .join(format!("{}.json", package.name.replace('-', "_")));

    let doc_with_source: WithSource<Crate> = WithSource::from_json("rustdoc output", &output_file)?;
    let doc = doc_with_source.value();

    let root = doc.index.get(&doc.root).unwrap();
    let root_links = &root.links;
    let root_doc = extract_root_doc(root)?;

    // TODO: make this configurable
    let local_html_root_url = format!(
        "https://docs.rs/{}/{}",
        package.name,
        doc.crate_version.as_deref().unwrap_or("latest")
    );

    let to_url = |text: &str| {
        let id = root_links.get(text)?;
        create_doc_url(doc, &local_html_root_url, id)
    };

    let mut callback = |link: BrokenLink| {
        let url = to_url(link.reference.as_ref())?;
        Some((url.into(), "".into()))
    };

    let convert_link = |mut event| {
        match &mut event {
            Event::Start(Tag::Link(_link_type, url, _title))
            | Event::End(Tag::Link(_link_type, url, _title)) => {
                if let Some(full_url) = to_url(url.as_ref()) {
                    *url = full_url.into();
                }
            }
            _ => {}
        }
        event
    };

    let events =
        Parser::new_with_broken_link_callback(&root_doc, Options::all(), Some(&mut callback))
            .map(convert_link);

    let mut buf = String::with_capacity(root_doc.len());
    pulldown_cmark_to_cmark::cmark(events, &mut buf).unwrap();
    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }

    Ok(buf)
}

fn run_rustdoc(app: &App, package: &Package) -> CreateResult<()> {
    let mut command = app.toolchain.cargo_command();
    command
        .args(["rustdoc", "--package", &package.name])
        .args(app.feature.cargo_args())
        .args([
            "-Zrustdoc-map",
            "--",
            "-Zunstable-options",
            "--output-format=json",
        ]);

    tracing::info!(
        "executing {}{}",
        command.get_program().to_string_lossy(),
        command.get_args().fold(String::new(), |mut s, a| {
            s.push(' ');
            s.push_str(a.to_string_lossy().as_ref());
            s
        })
    );

    let status = command.status().map_err(CreateRustdocError::Spawn)?;
    if !status.success() {
        return Err(CreateRustdocError::Exit(status));
    }
    Ok(())
}

fn extract_root_doc(root: &Item) -> CreateResult<String> {
    let mut root_doc = root
        .docs
        .clone()
        .ok_or_else(|| CreateRustdocError::RootDocNotFound {
            crate_name: root.name.clone().unwrap(),
        })?;

    if !root_doc.is_empty() && !root_doc.ends_with('\n') {
        root_doc.push('\n');
    }
    Ok(root_doc)
}

fn create_doc_url(doc: &Crate, local_html_root_url: &str, id: &Id) -> Option<String> {
    let item = doc.paths.get(id).unwrap();
    let html_root_url = if item.crate_id == 0 {
        // local item
        local_html_root_url
    } else {
        // external item
        let external_crate = doc.external_crates.get(&item.crate_id).unwrap();
        external_crate.html_root_url.as_ref()?
    };

    let mut url = html_root_url.trim_end_matches('/').to_owned();
    let mut join = |paths: &[String], args| {
        for path in paths {
            write!(&mut url, "/{}", path).unwrap();
        }
        write!(&mut url, "/{}", args).unwrap();
    };
    match (&item.kind, item.path.as_slice()) {
        (ItemKind::Module, [ps @ ..]) => join(ps, format_args!("index.html")),
        // (ItemKind::ExternCrate, [..]) => todo!(),
        // (ItemKind::Import, [..]) => todo!(),
        (ItemKind::Struct, [ps @ .., name]) => join(ps, format_args!("struct.{name}.html")),
        // (ItemKind::StructField, [..]) => todo!(),
        (ItemKind::Union, [ps @ .., name]) => join(ps, format_args!("union.{name}.html")),
        (ItemKind::Enum, [ps @ .., name]) => join(ps, format_args!("enum.{name}.html")),
        // (ItemKind::Variant, [..]) => todo!(),
        (ItemKind::Function, [ps @ .., name]) => join(ps, format_args!("fn.{name}.html")),
        (ItemKind::Typedef, [ps @ .., name]) => join(ps, format_args!("type.{name}.html")),
        // (ItemKind::OpaqueTy, [..]) => todo!(),
        (ItemKind::Constant, [ps @ .., name]) => join(ps, format_args!("constant.{name}.html")),
        (ItemKind::Trait, [ps @ .., name]) => join(ps, format_args!("trait.{name}.html")),
        // (ItemKind::TraitAlias, [..]) => todo!(),
        // (ItemKind::Method, [..]) => todo!(),
        // (ItemKind::Impl, [..]) => todo!(),
        (ItemKind::Static, [ps @ .., name]) => join(ps, format_args!("static.{name}.html")),
        // (ItemKind::ForeignType, [..]) => todo!(),
        (ItemKind::Macro, [ps @ .., name]) => join(ps, format_args!("macro.{name}.html")),
        (ItemKind::ProcAttribute, [ps @ .., name]) => join(ps, format_args!("attr.{name}.html")),
        (ItemKind::ProcDerive, [ps @ .., name]) => join(ps, format_args!("derive.{name}.html")),
        // (ItemKind::AssocConst, [..]) => todo!(),
        // (ItemKind::AssocType, [..]) => todo!(),
        (ItemKind::Primitive, [ps @ .., name]) => join(ps, format_args!("primitive.{name}.html")),
        // (ItemKind::Keyword, [..]) => todo!(),
        (item, path) => tracing::warn!(?item, ?path, "unexpected intra-doc link item & path found"),
    }
    Some(url)
}
