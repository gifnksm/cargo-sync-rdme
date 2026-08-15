//! Integration test helpers.

#![allow(missing_docs, clippy::missing_panics_doc)]

use std::{
    collections::BTreeMap,
    env,
    ffi::OsStr,
    fs, iter,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Once},
};

use assert_cmd::{Command, assert::Assert};
use assert_fs::{TempDir, fixture::ChildPath, prelude::*};
use cargo_metadata::{Metadata, MetadataCommand};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, TextMergeStream};
use scraper::{Html, Selector};

pub const SPAN_START_MARKER: &str = "<!-- SYNC_RDME_INTEGRATION_TEST::SPAN_START -->";
pub const SPAN_END_MARKER: &str = "<!-- SYNC_RDME_INTEGRATION_TEST::SPAN_END -->";
pub const HTML_ROOT_URL: &str = "https://example.com/html_root/";

pub fn assert_nightly_toolchain_installed() {
    assert_toolchain_installed("nightly");
}

pub fn assert_toolchain_installed(toolchain: &'static str) {
    static CHECKED_TOOLCHAINS: Mutex<BTreeMap<&'static str, Arc<Once>>> =
        Mutex::new(BTreeMap::new());
    static RUSTUP_GUARD: Mutex<()> = Mutex::new(());

    let mut map = CHECKED_TOOLCHAINS.lock().unwrap();
    let once = Arc::clone(
        map.entry(toolchain)
            .or_insert_with(|| Arc::new(Once::new())),
    );
    drop(map);
    once.call_once(|| {
        let _guard = RUSTUP_GUARD.lock().unwrap();
        let result = Command::new("rustup")
            .args(["run", toolchain, "cargo", "--version", "--verbose"])
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
    let exe = env::var_os("CARGO_BIN_EXE_cargo-sync-rdme").unwrap();
    let mut cmd = Command::new(exe);
    cmd.current_dir(workspace.root_path())
        .args(["--toolchain", "nightly", "--allow-no-vcs"]);
    cmd
}

#[must_use]
pub fn sync_rdme_command_with_toolchain(workspace: &Workspace, toolchain: &str) -> Command {
    let exe = env::var_os("CARGO_BIN_EXE_cargo-sync-rdme").unwrap();
    let exe_dir = Path::new(&exe).parent().unwrap().to_path_buf();

    let path_env = env::var_os("PATH").unwrap_or_default();
    let path_env = env::split_paths(&path_env);
    let path_env = env::join_paths(iter::once(exe_dir).chain(path_env)).unwrap();

    let mut cmd = Command::new("rustup");
    cmd.current_dir(workspace.root_path())
        .args([
            "run",
            toolchain,
            "cargo",
            "sync-rdme",
            "--toolchain",
            "nightly",
            "--allow-no-vcs",
        ])
        .env("PATH", path_env);
    cmd
}

#[expect(clippy::must_use_candidate)]
pub fn sync_readme(workspace: &Workspace) -> Assert {
    sync_readme_with_args(workspace, <[&str; 0]>::default())
}

#[expect(clippy::must_use_candidate)]
pub fn sync_readme_with_toolchain(workspace: &Workspace, toolchain: &str) -> Assert {
    let result = sync_rdme_command_with_toolchain(workspace, toolchain)
        .assert()
        .success();
    eprintln!("{result}");
    result
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
        .args(["doc", "--workspace"])
        .assert()
        .success();
    eprintln!("{result}");
}

pub fn run_rustdoc_with_toolchain(workspace: &Workspace, toolchain: &str) {
    let mut cmd = Command::new("rustup");
    let result = cmd
        .current_dir(workspace)
        .args(["run", toolchain, "cargo", "doc", "--workspace"])
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

fn absolute_url_to_relative_url(url: &str, crate_name: &str) -> String {
    debug_assert!(HTML_ROOT_URL.ends_with('/'));
    let Some(path) = url.strip_prefix(HTML_ROOT_URL) else {
        return url.to_owned();
    };
    match path.split_once('/') {
        Some((link_crate_name, relative_url)) if link_crate_name == crate_name => {
            relative_url.to_owned()
        }
        _ => format!("../{path}"),
    }
}

#[must_use]
pub fn collect_links_from_markdown<P>(md: P, crate_name: &str) -> Vec<(String, String)>
where
    P: AsRef<Path>,
{
    let mut links = vec![];
    for event in events_from_markdown(md) {
        let Event::Start(Tag::Link {
            dest_url, title, ..
        }) = event
        else {
            continue;
        };
        let relative_url = absolute_url_to_relative_url(&dest_url, crate_name);
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
