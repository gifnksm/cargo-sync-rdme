//! Integration test helpers.

#![allow(missing_docs, clippy::missing_panics_doc)]

use std::{
    env,
    ffi::{OsStr, OsString},
    fs, iter,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use assert_cmd::{Command, assert::Assert};
use assert_fs::{TempDir, fixture::ChildPath, prelude::*};
use cargo_metadata::{Metadata, MetadataCommand};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, TextMergeStream};
use scraper::{Html, Selector};

pub const SPAN_START_MARKER: &str = "<!-- SYNC_RDME_INTEGRATION_TEST::SPAN_START -->";
pub const SPAN_END_MARKER: &str = "<!-- SYNC_RDME_INTEGRATION_TEST::SPAN_END -->";
pub const HTML_ROOT_URL: &str = "https://example.com/html_root/";

#[derive(Debug)]
pub struct Workspace {
    temp_dir: TempDir,
    metadata: Metadata,
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

    pub fn insert_crate_doc_comment<P>(&self, path: P, doc_comment: &str)
    where
        P: AsRef<Path>,
    {
        assert!(!doc_comment.is_empty());
        assert!(doc_comment.ends_with('\n'));

        let librs_path = self.child(path);
        let content = fs::read_to_string(&librs_path).unwrap();
        let new_content = format!("{doc_comment}{content}");
        fs::write(&librs_path, &new_content).unwrap();
    }

    #[must_use]
    pub fn cargo_sync_rdme(&self) -> CargoSyncRdme<'_> {
        CargoSyncRdme::new(self)
    }

    #[must_use]
    pub fn cargo_sync_rdme_default(&self) -> CargoSyncRdme<'_> {
        let mut cmd = self.cargo_sync_rdme();
        cmd.rustdoc_toolchain("nightly").allow_no_vcs();
        cmd
    }

    #[must_use]
    pub fn cargo_doc(&self) -> CargoDoc<'_> {
        CargoDoc::new(self)
    }

    #[must_use]
    pub fn cargo_doc_default(&self) -> CargoDoc<'_> {
        let mut cmd = CargoDoc::new(self);
        cmd.workspace();
        cmd
    }
}

static CARGO: LazyLock<PathBuf> = LazyLock::new(|| {
    let exe = env::var_os("CARGO").unwrap_or("cargo".into());
    PathBuf::from(exe)
});

static SYNC_RDME_EXE: LazyLock<PathBuf> = LazyLock::new(|| {
    let exe = PathBuf::from(env::var_os("CARGO_BIN_EXE_cargo-sync-rdme").unwrap());
    assert_eq!(exe.file_prefix().unwrap(), "cargo-sync-rdme");
    exe
});

static SYNC_RDME_EXE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| SYNC_RDME_EXE.parent().unwrap().to_path_buf());

static PATH_ENV: LazyLock<OsString> = LazyLock::new(|| {
    let path_env = env::var_os("PATH").unwrap_or_default();
    let path_env = env::split_paths(&path_env);
    env::join_paths(iter::once(SYNC_RDME_EXE_DIR.to_path_buf()).chain(path_env)).unwrap()
});

fn copy_package_fixtures(tempdir: &TempDir, fixture_name: &str) {
    let project_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src = PathBuf::from(format!(
        "{project_manifest_dir}/tests/fixtures/packages/{fixture_name}"
    ));

    assert!(
        src.is_dir(),
        "package fixture directory does not exist: {}",
        src.display()
    );

    tempdir.copy_from(src, &["**/*"]).unwrap();
}

fn get_workspace_metadata(path: &Path) -> Metadata {
    MetadataCommand::new()
        .current_dir(path)
        .no_deps()
        .exec()
        .unwrap()
}

fn cargo_command(toolchain: Option<&'static str>) -> Command {
    let mut cmd;
    if let Some(toolchain) = toolchain {
        cmd = Command::new("rustup");
        cmd.args(["run", toolchain, "cargo"]);
    } else {
        cmd = Command::new(&*CARGO);
    }
    cmd.env("PATH", &*PATH_ENV);
    cmd
}

#[derive(Debug)]
pub struct CargoSyncRdme<'a> {
    workspace: &'a Workspace,
    current_dir: Option<PathBuf>,
    cargo_toolchain: Option<&'static str>,
    args: Vec<OsString>,
}

impl<'a> CargoSyncRdme<'a> {
    #[must_use]
    pub fn new(workspace: &'a Workspace) -> Self {
        Self {
            workspace,
            current_dir: None,
            cargo_toolchain: None,
            args: vec![],
        }
    }

    pub fn current_dir<P>(&mut self, path: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        assert!(path.is_relative());
        self.current_dir = Some(path.to_owned());
        self
    }

    pub fn cargo_toolchain(&mut self, toolchain: &'static str) -> &mut Self {
        self.cargo_toolchain = Some(toolchain);
        self
    }

    pub fn rustdoc_toolchain(&mut self, toolchain: &'static str) -> &mut Self {
        self.args(["--toolchain", toolchain]);
        self
    }

    pub fn allow_no_vcs(&mut self) -> &mut Self {
        self.args(["--allow-no-vcs"]);
        self
    }

    pub fn args<I>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_owned()));
        self
    }

    #[must_use]
    pub fn assert(&self) -> Assert {
        let mut cmd = cargo_command(self.cargo_toolchain);
        cmd.arg("sync-rdme").args(&self.args);
        if let Some(current_dir) = &self.current_dir {
            cmd.current_dir(self.workspace.child(current_dir));
        } else {
            cmd.current_dir(self.workspace.root_path());
        }
        cmd.assert()
    }
}

#[derive(Debug)]
pub struct CargoDoc<'a> {
    workspace: &'a Workspace,
    cargo_toolchain: Option<&'static str>,
    args: Vec<OsString>,
}

impl<'a> CargoDoc<'a> {
    #[must_use]
    pub fn new(workspace: &'a Workspace) -> Self {
        Self {
            workspace,
            cargo_toolchain: None,
            args: vec![],
        }
    }

    pub fn cargo_toolchain(&mut self, toolchain: &'static str) -> &mut Self {
        self.cargo_toolchain = Some(toolchain);
        self
    }

    pub fn workspace(&mut self) -> &mut Self {
        self.args(["--workspace"]);
        self
    }

    pub fn args<I>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_owned()));
        self
    }

    #[must_use]
    pub fn assert(&self) -> Assert {
        let result = cargo_command(self.cargo_toolchain)
            .current_dir(self.workspace.root_path())
            .arg("doc")
            .args(&self.args)
            .assert();
        eprintln!("{result}");
        result
    }
}

#[must_use]
pub fn parse_markdown_file<P>(md: P) -> Vec<Event<'static>>
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
pub fn collect_links_from_markdown_file<P>(md: P, crate_name: &str) -> Vec<(String, String)>
where
    P: AsRef<Path>,
{
    let mut links = vec![];
    for event in parse_markdown_file(md) {
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
pub fn collect_list_item_from_markdown_file<P>(md: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let mut items = vec![];
    let mut in_list_item = false;
    for event in parse_markdown_file(md) {
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
pub fn collect_links_from_html_file<P>(html: P) -> Vec<(String, String)>
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
