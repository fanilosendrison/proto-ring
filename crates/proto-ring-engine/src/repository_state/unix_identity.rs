//! Unix physical identity for an exact protected Git-administrative entry.

use super::StateCaptureError;
use super::git_admin::ClassificationRole;
use cap_std::fs::{Dir, MetadataExt};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UnixObjectIdentity {
    dev: u64,
    ino: u64,
}

pub(super) enum ProtectedGitAdmin {
    Directory {
        location: PathBuf,
        identity: UnixObjectIdentity,
    },
    File {
        identity: UnixObjectIdentity,
    },
}

pub(super) fn open_protected_git_admin(
    root_dir: &Dir,
    protected_relative_path: &Path,
) -> Result<ProtectedGitAdmin, StateCaptureError> {
    let entry = root_dir
        .symlink_metadata(protected_relative_path)
        .map_err(identity_error)?;
    if entry.file_type().is_dir() {
        let location = root_dir
            .canonicalize(protected_relative_path)
            .map_err(identity_error)?;
        let metadata = root_dir
            .metadata(protected_relative_path)
            .map_err(identity_error)?;
        Ok(ProtectedGitAdmin::Directory {
            location,
            identity: object_identity(&metadata),
        })
    } else if entry.file_type().is_file() {
        let metadata = root_dir
            .metadata(protected_relative_path)
            .map_err(identity_error)?;
        Ok(ProtectedGitAdmin::File {
            identity: object_identity(&metadata),
        })
    } else {
        Err(StateCaptureError::new(
            "Git administrative entry has unsupported kind",
        ))
    }
}

pub(super) fn reject_if_protected(
    root_dir: &Dir,
    exact_final_candidate: &Path,
    resolved: &Path,
    _protected_relative_path: &Path,
    protected: &ProtectedGitAdmin,
    role: ClassificationRole,
) -> Result<(), StateCaptureError> {
    match protected {
        ProtectedGitAdmin::Directory { location, identity } => {
            let inside = resolved == location
                || resolved.strip_prefix(location).is_ok()
                || physical_ancestor_is_protected(root_dir, resolved, *identity)?;
            if inside {
                return Err(protected_directory_traversal());
            }
        }
        ProtectedGitAdmin::File { identity } => {
            let candidate = root_dir.metadata(resolved).map_err(identity_error)?;
            if object_identity(&candidate) == *identity
                && (matches!(role, ClassificationRole::IntermediateTraversal)
                    || !exact_directory_entry_exists(root_dir, exact_final_candidate)?)
            {
                return Err(protected_file_traversal());
            }
        }
    }
    Ok(())
}

fn physical_ancestor_is_protected(
    root_dir: &Dir,
    resolved: &Path,
    protected: UnixObjectIdentity,
) -> Result<bool, StateCaptureError> {
    for ancestor in resolved
        .ancestors()
        .filter(|path| !path.as_os_str().is_empty())
    {
        let metadata = root_dir.metadata(ancestor).map_err(identity_error)?;
        if object_identity(&metadata) == protected {
            return Ok(true);
        }
    }
    Ok(false)
}

fn exact_directory_entry_exists(
    root_dir: &Dir,
    candidate: &Path,
) -> Result<bool, StateCaptureError> {
    let final_name = candidate
        .file_name()
        .ok_or_else(|| StateCaptureError::new("governed path has no final component"))?;
    let parent = candidate
        .parent()
        .filter(|path| !path.as_os_str().is_empty());
    if let Some(parent) = parent {
        let directory = root_dir.open_dir(parent).map_err(identity_error)?;
        directory_contains_exact_name(&directory, final_name)
    } else {
        directory_contains_exact_name(root_dir, final_name)
    }
}

fn directory_contains_exact_name(
    directory: &Dir,
    name: &std::ffi::OsStr,
) -> Result<bool, StateCaptureError> {
    for entry in directory.entries().map_err(identity_error)? {
        if entry.map_err(identity_error)?.file_name() == name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn object_identity(metadata: &cap_std::fs::Metadata) -> UnixObjectIdentity {
    UnixObjectIdentity {
        dev: metadata.dev(),
        ino: metadata.ino(),
    }
}

fn protected_directory_traversal() -> StateCaptureError {
    StateCaptureError::new("governed path enters protected Git administrative subtree")
}

fn protected_file_traversal() -> StateCaptureError {
    StateCaptureError::new("governed path enters protected Git administrative entry")
}

fn identity_error(_: std::io::Error) -> StateCaptureError {
    StateCaptureError::new("cannot classify Unix Git administrative traversal")
}
