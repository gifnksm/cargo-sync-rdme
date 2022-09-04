use cargo_metadata::camino::Utf8Path;

use crate::Result;

#[cfg(feature = "vcs-git")]
mod git;

pub(crate) fn discover(path: impl AsRef<Utf8Path>) -> Result<Option<Box<dyn Vcs>>> {
    let path = path.as_ref();

    // suppress unused variable warning
    let _ = path;

    #[cfg(feature = "vcs-git")]
    if let Some(vcs) = git::Git::discover(path)? {
        return Ok(Some(Box::new(vcs)));
    }

    Ok(None)
}

pub(crate) trait Vcs {
    fn workdir(&self) -> Option<&Utf8Path>;
    fn status_file(&self, path: &Utf8Path) -> Result<Status>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum Status {
    Dirty,
    Staged,
    Clean,
}
