//! Integration test helpers.

#![allow(missing_docs, clippy::missing_panics_doc)]

use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::Once,
};

use assert_cmd::{Command, assert::Assert};
use assert_fs::{TempDir, fixture::ChildPath, prelude::*};
use cargo_metadata::{Metadata, MetadataCommand};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, TextMergeStream};
use scraper::{Html, Selector};

pub const SPAN_START_MARKER: &str = "<!-- SYNC_RDME_INTEGRATION_TEST::SPAN_START -->";
pub const SPAN_END_MARKER: &str = "<!-- SYNC_RDME_INTEGRATION_TEST::SPAN_END -->";
pub const HTML_ROOT_URL: &str = "https://example.com/html_root/";

pub fn ensure_nightly_toolchain_installed() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let result = Command::new("rustup")
            .args(["run", "nightly", "cargo", "--version", "--verbose"])
            .assert()
            .success();
        eprintln!("{result}");
    });
}

#[derive(Debug)]
pub struct Workspace {
    temp_dir: TempDir,
    metadata: Metadata,
}

impl Workspace {
    #[must_use]
    pub fn from_fixture(fixture_name: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        copy_package_fixtures(&temp_dir, fixture_name);
        let metadata = get_workspace_metadata(temp_dir.path());
        Self { temp_dir, metadata }
    }

    #[must_use]
    pub fn root_path(&self) -> &Path {
        self.temp_dir.path()
    }

    #[must_use]
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

impl AsRef<Path> for Workspace {
    fn as_ref(&self) -> &Path {
        self.root_path()
    }
}

impl PathChild for Workspace {
    fn child<P>(&self, path: P) -> ChildPath
    where
        P: AsRef<Path>,
    {
        self.temp_dir.child(path)
    }
}

fn copy_package_fixtures(workspace: &TempDir, fixture_name: &str) {
    let project_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src = PathBuf::from(format!(
        "{project_manifest_dir}/tests/fixtures/packages/{fixture_name}"
    ));

    assert!(
        src.is_dir(),
        "package fixture directory does not exist: {}",
        src.display()
    );

    workspace
        .copy_from(src, &["Cargo.toml", "**/*.rs", "**/*.md"])
        .unwrap();
}

fn get_workspace_metadata(path: &Path) -> Metadata {
    MetadataCommand::new()
        .current_dir(path)
        .no_deps()
        .exec()
        .unwrap()
}

pub fn insert_crate_doc_comment<P>(workspace: &Workspace, path: P, doc_comment: &str)
where
    P: AsRef<Path>,
{
    assert!(!doc_comment.is_empty());
    assert!(doc_comment.ends_with('\n'));

    let librs_path = workspace.child(path);
    let content = fs::read_to_string(&librs_path).unwrap();
    let new_content = format!("{doc_comment}{content}");
    fs::write(&librs_path, &new_content).unwrap();
}

#[must_use]
pub fn sync_rdme_command(workspace: &Workspace) -> Command {
    let exe = env::var("CARGO_BIN_EXE_cargo-sync-rdme").unwrap();
    let mut cmd = Command::new(exe);
    cmd.current_dir(workspace.root_path())
        .args(["--toolchain", "nightly", "--allow-no-vcs"]);
    cmd
}

#[expect(clippy::must_use_candidate)]
pub fn sync_readme(workspace: &Workspace) -> Assert {
    sync_readme_with_args(workspace, <[&str; 0]>::default())
}

pub fn sync_readme_with_args<I, S>(workspace: &Workspace, args: I) -> Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let result = sync_rdme_command(workspace).args(args).assert().success();
    eprintln!("{result}");
    result
}

pub fn run_rustdoc(workspace: &Workspace) {
    let mut cmd = Command::new("cargo");
    let result = cmd
        .current_dir(workspace)
        .args(["+nightly", "doc", "--no-deps"])
        .assert()
        .success();
    eprintln!("{result}");
}

#[must_use]
pub fn events_from_markdown<P>(md: P) -> Vec<Event<'static>>
where
    P: AsRef<Path>,
{
    let md = md.as_ref();
    assert!(
        md.is_file(),
        "Markdown file does not exist: {}",
        md.display()
    );

    let content = fs::read_to_string(md).unwrap();
    let parser = TextMergeStream::new(Parser::new(&content));
    let mut in_span = false;
    let mut events = vec![];
    for event in parser {
        match event {
            Event::Html(html) if html.as_ref().trim() == SPAN_START_MARKER => in_span = true,
            Event::Html(html) if html.as_ref().trim() == SPAN_END_MARKER => in_span = false,
            event if in_span => events.push(event.into_static()),
            _ => {}
        }
    }
    events
}

#[must_use]
pub fn collect_links_from_markdown<P>(md: P, crate_name: &str) -> Vec<(String, String)>
where
    P: AsRef<Path>,
{
    let mut links = vec![];
    let mut url_prefix = HTML_ROOT_URL.to_owned();
    if !url_prefix.ends_with('/') {
        url_prefix.push('/');
    }
    url_prefix.push_str(crate_name);
    if !url_prefix.ends_with('/') {
        url_prefix.push('/');
    }
    for event in events_from_markdown(md) {
        let Event::Start(Tag::Link {
            dest_url, title, ..
        }) = event
        else {
            continue;
        };
        let relative_url = dest_url
            .strip_prefix(&url_prefix)
            .unwrap_or(&dest_url)
            .to_owned();
        links.push((relative_url, title.into_string()));
    }
    links
}

#[must_use]
pub fn collect_list_item_from_markdown<P>(md: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let mut items = vec![];
    let mut in_list_item = false;
    for event in events_from_markdown(md) {
        match event {
            Event::Start(Tag::Item) => in_list_item = true,
            Event::End(TagEnd::Item) if in_list_item => in_list_item = false,
            Event::Text(text) if in_list_item => items.push(text.into_string()),
            _ => {}
        }
    }
    items
}

#[must_use]
pub fn collect_links_from_html<P>(html: P) -> Vec<(String, String)>
where
    P: AsRef<Path>,
{
    let html = html.as_ref();
    assert!(
        html.is_file(),
        "HTML file does not exist: {}",
        html.display()
    );

    let content = fs::read_to_string(html).unwrap();
    let start = content.find(SPAN_START_MARKER).unwrap();
    let content = &content[start + SPAN_START_MARKER.len()..];
    let end = content.find(SPAN_END_MARKER).unwrap();
    let fragment = &content[..end];
    let fragment = Html::parse_fragment(fragment);
    let selector = Selector::parse("a").unwrap();
    fragment
        .select(&selector)
        .map(|element| {
            let href = element.value().attr("href").unwrap_or("").to_owned();
            let title = element.value().attr("title").unwrap_or("").to_owned();
            (href, title)
        })
        .collect()
}
