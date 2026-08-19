//! Integration test helpers.

#![allow(missing_docs, clippy::missing_panics_doc)]

use std::{
    env,
    ffi::{OsStr, OsString},
    fs, iter,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use cargo_metadata::{Metadata, MetadataCommand};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, TextMergeStream};
use scraper::{Html, Selector};
use snapbox::{
    Assert, Redactions,
    assert::DEFAULT_ACTION_ENV,
    cmd::{self, Command, OutputAssert},
    dir::DirRoot,
};

pub const SPAN_START_MARKER: &str = "<!-- SYNC_RDME_INTEGRATION_TEST::SPAN_START -->";
pub const SPAN_END_MARKER: &str = "<!-- SYNC_RDME_INTEGRATION_TEST::SPAN_END -->";
pub const HTML_ROOT_URL: &str = "https://example.com/html_root/";

#[derive(Debug)]
pub struct Workspace {
    root: DirRoot,
    metadata: Metadata,
}

impl AsRef<Path> for Workspace {
    fn as_ref(&self) -> &Path {
        self.root_path()
    }
}

impl Workspace {
    #[must_use]
    pub fn from_fixture(fixture_name: &str) -> Self {
        let root = DirRoot::mutable_temp()
            .unwrap()
            .with_template(&package_fixture_path(fixture_name))
            .unwrap();
        let metadata = get_workspace_metadata(root.path().unwrap());
        Self { root, metadata }
    }

    #[must_use]
    pub fn root_path(&self) -> &Path {
        self.root.path().unwrap()
    }

    #[must_use]
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    #[must_use]
    pub fn redactions(&self) -> Redactions {
        let mut redactions = Redactions::new();
        redactions
            .insert("[WORKSPACE]", self.root_path().to_path_buf())
            .unwrap();
        redactions
    }

    #[must_use]
    pub fn assert(&self) -> Assert {
        Assert::new()
            .action_env(DEFAULT_ACTION_ENV)
            .redact_with(self.redactions())
    }

    pub fn insert_crate_doc_comment<P>(&self, path: P, doc_comment: &str)
    where
        P: AsRef<Path>,
    {
        assert!(!doc_comment.is_empty());
        assert!(doc_comment.ends_with('\n'));

        let librs_path = self.root_path().join(path);
        let content = fs::read_to_string(&librs_path).unwrap();
        let new_content = format!("{doc_comment}{content}");
        fs::write(&librs_path, &new_content).unwrap();
    }

    #[must_use]
    pub fn cargo_sync_rdme(&self) -> CargoSyncRdme<'_> {
        CargoSyncRdme::new_in_workspace(self)
    }

    #[must_use]
    pub fn cargo_sync_rdme_default(&self) -> CargoSyncRdme<'_> {
        let mut cmd = self.cargo_sync_rdme();
        cmd.rustdoc_toolchain("nightly").allow_no_vcs();
        cmd
    }

    #[must_use]
    pub fn cargo_sync_rdme_snapshot_default(&self) -> CargoSyncRdme<'_> {
        let mut cmd = self.cargo_sync_rdme_default();
        cmd.force_color()
            .envs([("CARGO_TERM_QUIET", "true")])
            .with_assert(self.assert());
        cmd
    }

    #[must_use]
    pub fn cargo_doc(&self) -> CargoDoc<'_> {
        CargoDoc::new_in_workspace(self)
    }

    #[must_use]
    pub fn cargo_doc_default(&self) -> CargoDoc<'_> {
        let mut cmd = CargoDoc::new_in_workspace(self);
        cmd.workspace();
        cmd
    }
}

static CARGO: LazyLock<PathBuf> = LazyLock::new(|| {
    let exe = env::var_os("CARGO").unwrap_or("cargo".into());
    PathBuf::from(exe)
});

static SYNC_RDME_EXE: LazyLock<PathBuf> = LazyLock::new(|| {
    let exe = cmd::cargo_bin("cargo-sync-rdme");
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

#[must_use]
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("fixtures")
}

#[must_use]
pub fn snapshot_path(fixture_name: &str) -> PathBuf {
    fixtures_dir().join("snapshots").join(fixture_name)
}

#[must_use]
pub fn package_fixture_path(fixture_name: &str) -> PathBuf {
    fixtures_dir().join("packages").join(fixture_name)
}

fn get_workspace_metadata(path: &Path) -> Metadata {
    MetadataCommand::new()
        .current_dir(path)
        .no_deps()
        .exec()
        .unwrap()
}

fn cargo_command(toolchain: Option<&'static str>) -> Command {
    if let Some(toolchain) = toolchain {
        Command::new("rustup").args(["run", toolchain, "cargo"])
    } else {
        Command::new(&*CARGO)
    }
    .env("PATH", &*PATH_ENV)
}

#[derive(Debug, Default)]
pub struct CargoSyncRdme<'a> {
    workspace_root: Option<&'a Path>,
    current_dir: Option<PathBuf>,
    cargo_toolchain: Option<&'static str>,
    args: Vec<OsString>,
    envs: Vec<(OsString, OsString)>,
    assert: Option<Assert>,
}

impl<'a> CargoSyncRdme<'a> {
    #[must_use]
    pub fn new_in_workspace(workspace: &'a Workspace) -> Self {
        Self {
            workspace_root: Some(workspace.root_path()),
            current_dir: Some(workspace.root_path().to_path_buf()),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_dir<P>(&mut self, subpath: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        let workspace = self.workspace_root.unwrap();
        let path = subpath.as_ref();
        assert!(path.is_relative());
        self.current_dir = Some(workspace.join(path));
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

    pub fn force_color(&mut self) -> &mut Self {
        self.envs([("CLICOLOR_FORCE", "1")]);
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

    pub fn envs<I, K, V>(&mut self, envs: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.envs.extend(
            envs.into_iter()
                .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned())),
        );
        self
    }

    pub fn with_assert(&mut self, assert: Assert) -> &mut Self {
        self.assert = Some(assert);
        self
    }

    #[must_use]
    pub fn assert(&self) -> OutputAssert {
        let mut cmd = cargo_command(self.cargo_toolchain)
            .arg("sync-rdme")
            .args(&self.args)
            .envs(self.envs.iter().map(|(k, v)| (k, v)));
        if let Some(current_dir) = &self.current_dir {
            cmd = cmd.current_dir(current_dir);
        }
        if let Some(assert) = self.assert.clone() {
            cmd = cmd.with_assert(assert);
        }
        cmd.assert()
    }
}

#[derive(Debug, Default)]
pub struct CargoDoc<'a> {
    workspace_root: Option<&'a Path>,
    current_dir: Option<PathBuf>,
    cargo_toolchain: Option<&'static str>,
    args: Vec<OsString>,
}

impl<'a> CargoDoc<'a> {
    #[must_use]
    pub fn new_in_workspace(workspace: &'a Workspace) -> Self {
        Self {
            workspace_root: Some(workspace.root_path()),
            current_dir: Some(workspace.root_path().to_path_buf()),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_dir<P>(&mut self, subpath: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        let workspace = self.workspace_root.unwrap();
        let path = subpath.as_ref();
        assert!(path.is_relative());
        self.current_dir = Some(workspace.join(path));
        self
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
    pub fn assert(&self) -> OutputAssert {
        let mut cmd = cargo_command(self.cargo_toolchain);
        if let Some(current_dir) = &self.current_dir {
            cmd = cmd.current_dir(current_dir);
        }
        cmd.arg("doc").args(&self.args).assert()
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
