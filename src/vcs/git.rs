use cargo_metadata::camino::Utf8Path;

use super::{Status, Vcs};
use crate::Result;

pub(super) struct Git {
    repo: git2::Repository,
}

impl Git {
    pub(super) fn discover(path: impl AsRef<Utf8Path>) -> Result<Option<Self>> {
        let path = path.as_ref();

        let repo = match git2::Repository::discover(path) {
            Ok(repo) => repo,
            Err(err) if err.code() == git2::ErrorCode::NotFound => return Ok(None),
            Err(err) => bail!(err),
        };
        Ok(Some(Self { repo }))
    }
}

impl Vcs for Git {
    fn workdir(&self) -> Option<&Utf8Path> {
        self.repo
            .workdir()
            .map(|path| <&Utf8Path>::try_from(path).unwrap())
    }

    fn status_file(&self, path: &Utf8Path) -> Result<Status> {
        let status = match self.repo.status_file(path.as_ref()) {
            Ok(status) => status,
            Err(err) if err.code() == git2::ErrorCode::NotFound => {
                // treat untracked files as dirty
                return Ok(Status::Dirty);
            }
            Err(err) => bail!(err),
        };

        if status.is_wt_new()
            || status.is_wt_modified()
            || status.is_wt_deleted()
            || status.is_wt_renamed()
            || status.is_wt_typechange()
        {
            return Ok(Status::Dirty);
        }

        if status.is_index_new()
            || status.is_index_modified()
            || status.is_index_deleted()
            || status.is_index_renamed()
            || status.is_index_typechange()
        {
            return Ok(Status::Staged);
        }

        Ok(Status::Clean)
    }
}
