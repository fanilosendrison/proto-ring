//! Windows physical identity for protected Git-administrative directories.

use super::StateCaptureError;
use cap_std::fs::Dir;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WindowsDirectoryIdentity(prikk_ffi::FileIdentity);

pub(super) struct ProtectedGitAdmin(WindowsDirectoryIdentity);

pub(super) fn open_protected_git_admin(
    root_dir: &Dir,
) -> Result<ProtectedGitAdmin, StateCaptureError> {
    root_dir.symlink_metadata(".git").map_err(identity_error)?;
    Ok(ProtectedGitAdmin(directory_identity(
        root_dir,
        Path::new(".git"),
    )?))
}

pub(super) fn directory_identity(
    root_dir: &Dir,
    relative: &Path,
) -> Result<WindowsDirectoryIdentity, StateCaptureError> {
    let directory = root_dir.open_dir(relative).map_err(identity_error)?;
    match prikk_ffi::identity_of(&directory.into_std_file()).map_err(identity_error)? {
        identity @ prikk_ffi::FileIdentity::Id128 { .. } => Ok(WindowsDirectoryIdentity(identity)),
        prikk_ffi::FileIdentity::Id64 { .. } => Err(StateCaptureError::new(
            "unambiguous Windows protected filesystem identity unavailable",
        )),
    }
}

fn resolved_ancestor_is_protected(
    root_dir: &Dir,
    resolved: &Path,
    protected: &WindowsDirectoryIdentity,
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
    resolved: &Path,
    protected: &ProtectedGitAdmin,
) -> Result<(), StateCaptureError> {
    let inside = resolved_ancestor_is_protected(root_dir, resolved, &protected.0)?;
    if protected.0 != directory_identity(root_dir, Path::new(".git"))? {
        return Err(StateCaptureError::new(
            "protected Git administrative root changed during classification",
        ));
    }
    if inside {
        return Err(StateCaptureError::new(
            "governed path enters protected Git administrative subtree",
        ));
    }
    Ok(())
}

fn identity_error(_: std::io::Error) -> StateCaptureError {
    StateCaptureError::new("cannot classify Windows Git administrative traversal")
}
