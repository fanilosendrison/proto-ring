use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::time::{SystemTime, UNIX_EPOCH};

use cap_std::fs::{Dir, Metadata};

use super::StateCaptureError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedPath {
    pub(crate) path: PathBuf,
    pub(crate) object: ObservedObject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ObservedObject {
    Absent,
    Regular { mode: u64, bytes: Vec<u8> },
    Symlink { mode: u64, target: PathBuf },
    Directory { mode: u64 },
}

pub(crate) fn validate_explicit_scope(
    root: &Path,
    entries: &[PathBuf],
) -> Result<BTreeSet<PathBuf>, StateCaptureError> {
    let mut validated = BTreeSet::new();
    for entry in entries {
        validate_spelling(entry)?;
        validate_parent_containment(root, entry)?;
        validated.insert(entry.clone());
    }
    Ok(validated)
}

pub(crate) fn observe_paths(
    root_dir: &Dir,
    paths: &[PathBuf],
    explicit: &BTreeSet<PathBuf>,
) -> Result<Vec<ObservedPath>, StateCaptureError> {
    paths
        .iter()
        .map(|relative| {
            observe_path(root_dir, relative, explicit.contains(relative)).map(|object| {
                ObservedPath {
                    path: relative.clone(),
                    object,
                }
            })
        })
        .collect()
}

fn observe_path(
    root_dir: &Dir,
    relative: &Path,
    explicitly_observed: bool,
) -> Result<ObservedObject, StateCaptureError> {
    let before = match root_dir.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(ObservedObject::Absent),
        Err(error) => {
            return Err(StateCaptureError::new(format!(
                "cannot inspect governed path {relative:?}: {error}"
            )));
        }
    };
    let file_type = before.file_type();
    if file_type.is_file() {
        let guard_before = MetadataGuard::from(&before);
        let mut file = root_dir.open(relative).map_err(|error| {
            StateCaptureError::new(format!("cannot open governed file {relative:?}: {error}"))
        })?;
        let opened = file.metadata().map_err(|error| {
            StateCaptureError::new(format!(
                "cannot inspect opened governed file {relative:?}: {error}"
            ))
        })?;
        if guard_before != MetadataGuard::from(&opened) || !opened.file_type().is_file() {
            return Err(StateCaptureError::new(
                "governed regular file changed before content observation",
            ));
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|error| {
            StateCaptureError::new(format!("cannot read governed file {relative:?}: {error}"))
        })?;
        let after_handle = file.metadata().map_err(|error| {
            StateCaptureError::new(format!(
                "cannot re-inspect opened governed file {relative:?}: {error}"
            ))
        })?;
        let after_path = root_dir.symlink_metadata(relative).map_err(|error| {
            StateCaptureError::new(format!(
                "cannot re-inspect governed file {relative:?}: {error}"
            ))
        })?;
        if [&opened, &after_handle, &after_path]
            .into_iter()
            .any(|metadata| {
                guard_before != MetadataGuard::from(metadata) || !metadata.file_type().is_file()
            })
        {
            return Err(StateCaptureError::new(
                "governed regular file changed while being observed",
            ));
        }
        return Ok(ObservedObject::Regular {
            mode: relevant_mode(&before),
            bytes,
        });
    }
    if file_type.is_symlink() {
        let guard_before = MetadataGuard::from(&before);
        let target = root_dir.read_link_contents(relative).map_err(|error| {
            StateCaptureError::new(format!(
                "cannot read governed symlink {relative:?}: {error}"
            ))
        })?;
        let after = root_dir.symlink_metadata(relative).map_err(|error| {
            StateCaptureError::new(format!(
                "cannot re-inspect governed symlink {relative:?}: {error}"
            ))
        })?;
        if guard_before != MetadataGuard::from(&after) || !after.file_type().is_symlink() {
            return Err(StateCaptureError::new(
                "governed symbolic link changed while being observed",
            ));
        }
        return Ok(ObservedObject::Symlink {
            mode: relevant_mode(&before),
            target,
        });
    }
    if file_type.is_dir() {
        return if explicitly_observed {
            Ok(ObservedObject::Directory {
                mode: relevant_mode(&before),
            })
        } else {
            Err(StateCaptureError::new(
                "implicitly discovered directory cannot be represented exactly",
            ))
        };
    }
    Err(StateCaptureError::new(
        "governed filesystem object kind is unsupported",
    ))
}

fn validate_parent_containment(root: &Path, relative: &Path) -> Result<(), StateCaptureError> {
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let mut candidate = root.join(parent);
    loop {
        match fs::symlink_metadata(&candidate) {
            Ok(_) => {
                let resolved = candidate.canonicalize().map_err(|error| {
                    StateCaptureError::new(format!(
                        "explicit scope parent cannot be resolved: {error}"
                    ))
                })?;
                if !resolved.starts_with(root) {
                    return Err(StateCaptureError::new(
                        "explicit scope has an intermediate parent outside the worktree",
                    ));
                }
                return Ok(());
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if candidate == root {
                    return Err(StateCaptureError::new(
                        "exact worktree root disappeared during scope validation",
                    ));
                }
                candidate = candidate
                    .parent()
                    .ok_or_else(|| StateCaptureError::new("scope parent escaped worktree"))?
                    .to_path_buf();
            }
            Err(error) => {
                return Err(StateCaptureError::new(format!(
                    "explicit scope parent cannot be inspected: {error}"
                )));
            }
        }
    }
}

#[cfg(unix)]
fn validate_spelling(path: &Path) -> Result<(), StateCaptureError> {
    use std::os::unix::ffi::OsStrExt;
    let bytes = path.as_os_str().as_bytes();
    if bytes.is_empty()
        || bytes.contains(&0)
        || bytes.first() == Some(&b'/')
        || bytes.last() == Some(&b'/')
        || bytes.windows(2).any(|pair| pair == b"//")
        || bytes
            .split(|byte| *byte == b'/')
            .any(|component| component == b"." || component == b"..")
    {
        return Err(StateCaptureError::new(
            "explicit scope path has inadmissible exact spelling",
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn validate_spelling(path: &Path) -> Result<(), StateCaptureError> {
    use std::os::windows::ffi::OsStrExt;
    let units: Vec<u16> = path.as_os_str().encode_wide().collect();
    let separator = |unit| unit == b'/' as u16 || unit == b'\\' as u16;
    if units.is_empty()
        || units.contains(&0)
        || path.is_absolute()
        || matches!(
            path.components().next(),
            Some(std::path::Component::Prefix(_))
        )
        || units.first().is_some_and(|unit| separator(*unit))
        || units.last().is_some_and(|unit| separator(*unit))
        || units
            .windows(2)
            .any(|pair| separator(pair[0]) && separator(pair[1]))
        || units
            .split(|unit| separator(*unit))
            .any(|component| component == [b'.' as u16] || component == [b'.' as u16, b'.' as u16])
    {
        return Err(StateCaptureError::new(
            "explicit scope path has inadmissible exact spelling",
        ));
    }
    Ok(())
}

pub(crate) fn os_units(value: &OsStr) -> Vec<u8> {
    os_units_impl(value)
}

#[cfg(unix)]
fn os_units_impl(value: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    value.as_bytes().to_vec()
}

#[cfg(windows)]
fn os_units_impl(value: &OsStr) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    value
        .encode_wide()
        .flat_map(u16::to_be_bytes)
        .collect::<Vec<_>>()
}

#[cfg(unix)]
fn relevant_mode(metadata: &Metadata) -> u64 {
    use cap_std::fs::MetadataExt;
    u64::from(metadata.mode())
}

#[cfg(windows)]
fn relevant_mode(metadata: &Metadata) -> u64 {
    u64::from(metadata.permissions().readonly())
}

#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct MetadataGuard {
    mode: u32,
    size: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
    device: u64,
    inode: u64,
}

#[cfg(unix)]
impl From<&Metadata> for MetadataGuard {
    fn from(metadata: &Metadata) -> Self {
        use cap_std::fs::MetadataExt;
        Self {
            mode: metadata.mode(),
            size: metadata.size(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

#[cfg(windows)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct MetadataGuard {
    size: u64,
    modified: Option<std::time::Duration>,
    created: Option<std::time::Duration>,
    readonly: bool,
    file_type: (bool, bool, bool),
}

#[cfg(windows)]
impl From<&Metadata> for MetadataGuard {
    fn from(metadata: &Metadata) -> Self {
        Self {
            size: metadata.len(),
            modified: system_time(metadata.modified()),
            created: system_time(metadata.created()),
            readonly: metadata.permissions().readonly(),
            file_type: (
                metadata.file_type().is_file(),
                metadata.file_type().is_dir(),
                metadata.file_type().is_symlink(),
            ),
        }
    }
}

#[cfg(windows)]
fn system_time(value: std::io::Result<SystemTime>) -> Option<std::time::Duration> {
    value.ok()?.duration_since(UNIX_EPOCH).ok()
}
