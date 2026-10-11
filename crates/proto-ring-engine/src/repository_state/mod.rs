//! Implementation-private exact local repository observation.

mod capture;
mod git;
mod path;

use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

pub use capture::capture;
pub use git::resolve_worktree_root;

#[cfg(feature = "conformance-internal")]
#[doc(hidden)]
pub mod conformance_internal {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::{RepositoryState, StateCaptureError};

    pub fn capture_with_validation_hook<F>(
        repository: &Path,
        explicit_scope: &[PathBuf],
        hook: F,
    ) -> Result<RepositoryState, StateCaptureError>
    where
        F: FnOnce(&Path) -> Result<(), StateCaptureError>,
    {
        super::capture::capture_with_validation_hook(repository, explicit_scope, hook)
    }

    pub fn capture_with_ambient_environment(
        repository: &Path,
        explicit_scope: &[PathBuf],
        ambient: &[(OsString, OsString)],
    ) -> Result<RepositoryState, StateCaptureError> {
        super::capture::capture_with_ambient_environment(repository, explicit_scope, ambient)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryState {
    pub repository: PathBuf,
    pub identity: RepositoryStateIdentity,
    pub scope_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryStateIdentity(pub Vec<u8>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateCaptureError(String);

impl StateCaptureError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for StateCaptureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for StateCaptureError {}

pub fn discover_repository_paths(
    repository: &Path,
    explicit_scope: &[PathBuf],
) -> Result<Vec<PathBuf>, StateCaptureError> {
    let root = resolve_worktree_root(repository)?;
    let scope = path::validate_explicit_scope(&root, explicit_scope)?;
    git::capture_structure(&root, &scope, &[]).map(|structure| structure.paths)
}

pub(crate) type AmbientEnvironment = [(OsString, OsString)];
