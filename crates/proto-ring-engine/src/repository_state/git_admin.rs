//! Capability-relative Git-administrative protection for one resolved path.

use super::StateCaptureError;
#[cfg(unix)]
use super::unix_identity::{
    open_protected_git_admin, reject_if_ambiguous, reject_if_protected, same_protected_object,
};
#[cfg(windows)]
use super::windows_identity::{
    open_protected_git_admin, reject_if_ambiguous, reject_if_protected, same_protected_object,
};
use cap_std::fs::Dir;
use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(super) enum ClassificationRole {
    IntermediateTraversal,
    FinalNonSymlinkEntry,
}

enum ProtectedEntryResolution {
    None,
    Exact(PathBuf),
    NativeUnique(PathBuf),
    NativeAmbiguous,
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
        let resolution = protected_dot_git_entry(root_dir, &resolved_parent)?;
        if matches!(resolution, ProtectedEntryResolution::None) {
            continue;
        }

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
        match resolution {
            ProtectedEntryResolution::None => {}
            ProtectedEntryResolution::Exact(protected_path)
            | ProtectedEntryResolution::NativeUnique(protected_path) => {
                let protected = open_protected_git_admin(root_dir, &protected_path)?;
                reject_if_protected(
                    root_dir,
                    &exact_final_candidate,
                    &resolved_prefix,
                    &protected_path,
                    &protected,
                    role,
                )?;
            }
            ProtectedEntryResolution::NativeAmbiguous => {
                let literal_lookup = resolved_parent.join(".git");
                let protected = open_protected_git_admin(root_dir, &literal_lookup)?;
                if reject_if_ambiguous(root_dir, &resolved_prefix, &protected)? {
                    return Err(StateCaptureError::new(
                        "cannot unambiguously classify host-selected Git administrative entry",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn protected_dot_git_entry(
    root_dir: &Dir,
    resolved_parent: &Path,
) -> Result<ProtectedEntryResolution, StateCaptureError> {
    let entries = exact_parent_entry_names(root_dir, resolved_parent)?;
    if entries.iter().any(|name| name == OsStr::new(".git")) {
        return Ok(ProtectedEntryResolution::Exact(
            resolved_parent.join(".git"),
        ));
    }

    let literal_lookup = resolved_parent.join(".git");
    let lookup_metadata = match root_dir.symlink_metadata(&literal_lookup) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(ProtectedEntryResolution::None);
        }
        Err(error) => return Err(admin_error(error)),
    };
    if !lookup_metadata.file_type().is_dir() && !lookup_metadata.file_type().is_file() {
        return Err(StateCaptureError::new(
            "Git administrative entry has unsupported kind",
        ));
    }

    let mut matches = Vec::new();
    for name in entries {
        let exact_path = resolved_parent.join(name);
        if same_protected_object(root_dir, &literal_lookup, &exact_path)? {
            matches.push(exact_path);
        }
    }
    match matches.len() {
        0 => Err(StateCaptureError::new(
            "native Git administrative lookup has no exact stored entry",
        )),
        1 => Ok(ProtectedEntryResolution::NativeUnique(
            matches.pop().ok_or_else(|| {
                StateCaptureError::new("native Git administrative entry disappeared")
            })?,
        )),
        _ => Ok(ProtectedEntryResolution::NativeAmbiguous),
    }
}

fn exact_parent_entry_names(
    root_dir: &Dir,
    resolved_parent: &Path,
) -> Result<Vec<OsString>, StateCaptureError> {
    let entries = if resolved_parent.as_os_str().is_empty() {
        root_dir.entries().map_err(admin_error)?
    } else {
        root_dir.read_dir(resolved_parent).map_err(admin_error)?
    };
    entries
        .map(|entry| entry.map(|entry| entry.file_name()).map_err(admin_error))
        .collect()
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
