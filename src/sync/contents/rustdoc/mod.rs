use std::borrow::Cow;

use cargo_metadata::PackageName;
use miette::Diagnostic;
use pulldown_cmark::{Event, Options};
use snafu::{OptionExt as _, ResultExt as _, Snafu};

use crate::{
    cargo,
    sync::{
        PackageSyncContext,
        contents::rustdoc::{document::UrlOptions, intra_link::LinkMappingConfig},
    },
};

mod build;
mod code_block;
mod document;
mod heading;
mod intra_link;

#[derive(Debug, Snafu, Diagnostic)]
pub(in crate::sync) enum CreateRustdocError {
    #[snafu(display("failed to build rustdoc JSON output for package `{package}`"))]
    BuildRustdoc {
        package: PackageName,
        #[snafu(source)]
        #[diagnostic_source]
        source: Box<build::BuildRustdocError>,
    },
    #[snafu(display("package `{package}` does not have a root item"))]
    RootNotFound { package: PackageName },
    #[snafu(display("package `{package}` does not have crate-level documentation"))]
    RootDocNotFound { package: PackageName },
    #[snafu(display("failed to determine the Rust toolchain version"))]
    DetermineToolchain {
        #[snafu(source)]
        #[diagnostic_source]
        source: cargo::ToolchainError,
    },
    #[snafu(display("failed to construct markdown from rustdoc output"))]
    Render {
        #[snafu(source)]
        source: pulldown_cmark_to_cmark::Error,
    },
}

pub(super) fn create(cx: &PackageSyncContext<'_>) -> Result<String, CreateRustdocError> {
    let doc = build::build_rustdoc(cx).with_context(|_source| BuildRustdocSnafu { package: cx })?;
    let root = doc
        .root_item()
        .with_context(|| RootNotFoundSnafu { package: cx })?;

    let build_url_options = build_url_options(cx)?;
    let mapping_config = build_mapping_config(cx);
    let resolver = doc.intra_link_resolver(&build_url_options);
    let mapper = mapping_config
        .build_mapper(&resolver, root)
        .with_context(|| RootDocNotFoundSnafu { package: cx })?;

    let events = mapper.build_parser(main_body_opts());
    let events = heading::convert(events);
    let events = code_block::convert(events);

    let output = render(events)?;
    Ok(output)
}

fn build_url_options<'a>(
    cx: &'a PackageSyncContext<'_>,
) -> Result<UrlOptions<'a>, CreateRustdocError> {
    let config = &cx.config;
    let local_html_root_url = config.rustdoc.html_root_url.as_deref().map_or_else(
        || format!("https://docs.rs/{}/{}", cx.package.name, cx.package.version).into(),
        Cow::Borrowed,
    );
    let expected_toolchain = cargo::toolchain(None).context(DetermineToolchainSnafu)?;
    let rustdoc_toolchain =
        cargo::toolchain(Some(cx.toolchain)).context(DetermineToolchainSnafu)?;
    Ok(UrlOptions {
        local_html_root_url,
        expected_toolchain,
        rustdoc_toolchain,
    })
}

fn build_mapping_config<'a>(cx: &'a PackageSyncContext<'_>) -> LinkMappingConfig<'a> {
    LinkMappingConfig {
        mappings: &cx.config.rustdoc.mappings,
    }
}

// Same options as rustdoc uses for the main body of the crate-level documentation.
// <https://github.com/rust-lang/rust/blob/153ecc4f74035b709bb3e1eb9546f1d934865042/compiler/rustc_resolve/src/rustdoc.rs#L250-L257>
// These extensions are also explicitly documented in the rustdoc book:
// <https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html#markdown>.
fn main_body_opts() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        // Keep smart punctuation enabled so synced Markdown matches rustdoc's
        // rendered output more closely, including on renderers like GitHub that
        // do not apply those substitutions themselves.
        | Options::ENABLE_SMART_PUNCTUATION
}

fn render<'a, I>(events: I) -> Result<String, CreateRustdocError>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut buf = String::new();
    pulldown_cmark_to_cmark::cmark(events, &mut buf).context(RenderSnafu)?;
    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }
    Ok(buf)
}
