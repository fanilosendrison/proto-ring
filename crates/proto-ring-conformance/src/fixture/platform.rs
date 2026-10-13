use std::fs;
use std::path::Path;

use super::{FixtureError, failure};

#[cfg(unix)]
pub(super) fn apply_chmod(path: &Path, mode: u32) -> Result<(), FixtureError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|error| failure(format!("chmod failed: {error}")))
}

#[cfg(windows)]
pub(super) fn apply_chmod(path: &Path, mode: u32) -> Result<(), FixtureError> {
    let mut permissions = fs::metadata(path)
        .map_err(|error| failure(format!("chmod metadata failed: {error}")))?
        .permissions();
    permissions.set_readonly(mode & 0o222 == 0);
    fs::set_permissions(path, permissions)
        .map_err(|error| failure(format!("chmod failed: {error}")))
}

#[cfg(unix)]
pub(super) fn create_symlink(
    target: &str,
    path: &Path,
    _directory: bool,
) -> Result<(), FixtureError> {
    std::os::unix::fs::symlink(target, path)
        .map_err(|error| failure(format!("symlink failed: {error}")))
}

#[cfg(windows)]
pub(super) fn create_symlink(
    target: &str,
    path: &Path,
    directory: bool,
) -> Result<(), FixtureError> {
    let native_target = target.replace('/', std::path::MAIN_SEPARATOR_STR);
    let target_path = Path::new(&native_target);

    let result = if directory {
        std::os::windows::fs::symlink_dir(target_path, path)
    } else {
        std::os::windows::fs::symlink_file(target_path, path)
    };

    result.map_err(|error| failure(format!("symlink failed: {error}")))
}
