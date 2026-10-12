//! Capability-relative Git-administrative protection for one resolved path.

use super::StateCaptureError;
#[cfg(unix)]
use super::unix_identity::{open_protected_git_admin, reject_if_protected};
#[cfg(windows)]
use super::windows_identity::{open_protected_git_admin, reject_if_protected};
use cap_std::fs::Dir;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(super) enum ClassificationRole {
    IntermediateTraversal,
    FinalNonSymlinkEntry,
}

pub(super) fn reject_resolved_git_admin_traversal(
    root_dir: &Dir,
    original_candidate: &Path,
    resolved_candidate: &Path,
    overall_role: ClassificationRole,
) -> Result<(), StateCaptureError> {
    let components: Vec<_> = resolved_candidate.iter().collect();
    let mut resolved_prefix = PathBuf::new();
    for (index, component) in components.iter().enumerate() {
        let resolved_parent = resolved_prefix.clone();
        resolved_prefix.push(component);
        if !exact_dot_git_entry_exists(root_dir, &resolved_parent)? {
            continue;
        }

        let protected_path = resolved_parent.join(".git");
        let protected = open_protected_git_admin(root_dir, &protected_path)?;
        let is_final = index + 1 == components.len();
        let role = if is_final {
            overall_role
        } else {
            ClassificationRole::IntermediateTraversal
        };
        let exact_final_candidate = exact_final_candidate(
            original_candidate,
            &resolved_parent,
            is_final && matches!(role, ClassificationRole::FinalNonSymlinkEntry),
        )?;
        reject_if_protected(
            root_dir,
            &exact_final_candidate,
            &resolved_prefix,
            &protected_path,
            &protected,
            role,
        )?;
    }
    Ok(())
}

fn exact_dot_git_entry_exists(
    root_dir: &Dir,
    resolved_parent: &Path,
) -> Result<bool, StateCaptureError> {
    if resolved_parent.as_os_str().is_empty() {
        return directory_contains_exact_dot_git(root_dir.entries().map_err(admin_error)?);
    }
    directory_contains_exact_dot_git(root_dir.read_dir(resolved_parent).map_err(admin_error)?)
}

fn directory_contains_exact_dot_git(
    entries: cap_std::fs::ReadDir,
) -> Result<bool, StateCaptureError> {
    for entry in entries {
        if entry.map_err(admin_error)?.file_name() == OsStr::new(".git") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn exact_final_candidate(
    original_candidate: &Path,
    resolved_parent: &Path,
    final_non_symlink: bool,
) -> Result<PathBuf, StateCaptureError> {
    if !final_non_symlink {
        return Ok(resolved_parent.to_path_buf());
    }
    let final_name = original_candidate
        .file_name()
        .ok_or_else(|| StateCaptureError::new("governed path has no final component"))?;
    Ok(resolved_parent.join(final_name))
}

fn admin_error(_: std::io::Error) -> StateCaptureError {
    StateCaptureError::new("cannot enumerate resolved Git administrative ancestry")
}
