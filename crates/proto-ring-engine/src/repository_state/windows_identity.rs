//! Windows physical identity for an exact protected Git-administrative entry.

use super::StateCaptureError;
use super::git_admin::ClassificationRole;
use cap_std::fs::Dir;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WindowsObjectIdentity(prikk_ffi::FileIdentity);

pub(super) enum ProtectedGitAdmin {
    Directory(WindowsObjectIdentity),
    File(WindowsObjectIdentity),
}

pub(super) fn open_protected_git_admin(
    root_dir: &Dir,
    protected_relative_path: &Path,
) -> Result<ProtectedGitAdmin, StateCaptureError> {
    let metadata = root_dir
        .symlink_metadata(protected_relative_path)
        .map_err(identity_error)?;
    if metadata.file_type().is_dir() {
        Ok(ProtectedGitAdmin::Directory(directory_identity(
            root_dir,
            protected_relative_path,
        )?))
    } else if metadata.file_type().is_file() {
        Ok(ProtectedGitAdmin::File(file_identity(
            root_dir,
            protected_relative_path,
        )?))
    } else {
        Err(StateCaptureError::new(
            "Git administrative entry has unsupported kind",
        ))
    }
}

fn file_identity(
    root_dir: &Dir,
    relative: &Path,
) -> Result<WindowsObjectIdentity, StateCaptureError> {
    let file = root_dir.open(relative).map_err(identity_error)?;
    exact_identity(&file.into_std())
}

fn directory_identity(
    root_dir: &Dir,
    relative: &Path,
) -> Result<WindowsObjectIdentity, StateCaptureError> {
    let directory = root_dir.open_dir(relative).map_err(identity_error)?;
    exact_identity(&directory.into_std_file())
}

fn exact_identity(file: &std::fs::File) -> Result<WindowsObjectIdentity, StateCaptureError> {
    match prikk_ffi::identity_of(file).map_err(identity_error)? {
        identity @ prikk_ffi::FileIdentity::Id128 { .. } => Ok(WindowsObjectIdentity(identity)),
        prikk_ffi::FileIdentity::Id64 { .. } => Err(StateCaptureError::new(
            "unambiguous Windows protected filesystem identity unavailable",
        )),
    }
}

fn resolved_ancestor_is_protected(
    root_dir: &Dir,
    resolved: &Path,
    protected: &WindowsObjectIdentity,
) -> Result<bool, StateCaptureError> {
    for ancestor in resolved
        .ancestors()
        .filter(|path| !path.as_os_str().is_empty())
    {
        if root_dir
            .metadata(ancestor)
            .map_err(identity_error)?
            .is_dir()
            && directory_identity(root_dir, ancestor)? == *protected
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn reject_if_protected(
    root_dir: &Dir,
    exact_final_candidate: &Path,
    resolved: &Path,
    protected_relative_path: &Path,
    protected: &ProtectedGitAdmin,
    role: ClassificationRole,
) -> Result<(), StateCaptureError> {
    match protected {
        ProtectedGitAdmin::Directory(identity) => {
            let inside = resolved_ancestor_is_protected(root_dir, resolved, identity)?;
            if *identity != current_directory_identity(root_dir, protected_relative_path)? {
                return Err(protected_changed());
            }
            if inside {
                return Err(protected_traversal());
            }
        }
        ProtectedGitAdmin::File(identity) => {
            let aliases_protected_file = root_dir
                .metadata(resolved)
                .map_err(identity_error)?
                .is_file()
                && file_identity(root_dir, resolved)? == *identity;
            if *identity != current_file_identity(root_dir, protected_relative_path)? {
                return Err(protected_changed());
            }
            if aliases_protected_file
                && (matches!(role, ClassificationRole::IntermediateTraversal)
                    || !exact_final_entry_exists(root_dir, exact_final_candidate)?)
            {
                return Err(protected_traversal());
            }
        }
    }
    Ok(())
}

fn current_directory_identity(
    root_dir: &Dir,
    protected_relative_path: &Path,
) -> Result<WindowsObjectIdentity, StateCaptureError> {
    let metadata = root_dir
        .symlink_metadata(protected_relative_path)
        .map_err(identity_error)?;
    if !metadata.file_type().is_dir() {
        return Err(protected_changed());
    }
    directory_identity(root_dir, protected_relative_path)
}

fn current_file_identity(
    root_dir: &Dir,
    protected_relative_path: &Path,
) -> Result<WindowsObjectIdentity, StateCaptureError> {
    let metadata = root_dir
        .symlink_metadata(protected_relative_path)
        .map_err(identity_error)?;
    if !metadata.file_type().is_file() {
        return Err(protected_changed());
    }
    file_identity(root_dir, protected_relative_path)
}

fn exact_final_entry_exists(root_dir: &Dir, candidate: &Path) -> Result<bool, StateCaptureError> {
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

fn protected_changed() -> StateCaptureError {
    StateCaptureError::new("protected Git administrative entry changed during classification")
}

fn protected_traversal() -> StateCaptureError {
    StateCaptureError::new("governed path enters protected Git administrative subtree")
}

fn identity_error(_: std::io::Error) -> StateCaptureError {
    StateCaptureError::new("cannot classify Windows Git administrative traversal")
}
