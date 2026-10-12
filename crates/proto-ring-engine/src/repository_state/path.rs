use super::StateCaptureError;
#[cfg(windows)]
use super::windows_identity::{ProtectedGitAdmin, open_protected_git_admin, reject_if_protected};
use cap_std::fs::{Dir, Metadata};
use std::collections::BTreeSet;
use std::io::{ErrorKind, Read};
use std::path::{Component, Path, PathBuf};
#[cfg(windows)]
use {cap_std::time::SystemTime, std::time::UNIX_EPOCH};
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
    entries: &[PathBuf],
) -> Result<BTreeSet<PathBuf>, StateCaptureError> {
    let mut validated = BTreeSet::new();
    for entry in entries {
        validate_spelling(entry)?;
        reject_git_implementation_path(entry)?;
        validated.insert(entry.clone());
    }
    Ok(validated)
}
pub(crate) fn reject_git_implementation_path(relative: &Path) -> Result<(), StateCaptureError> {
    if relative
        .components()
        .any(|component| matches!(component, Component::Normal(value) if value == ".git"))
    {
        return Err(StateCaptureError::new(
            "Git implementation path cannot enter RepositoryState observation",
        ));
    }
    Ok(())
}
#[cfg(unix)]
struct ProtectedGitAdmin(PathBuf, u64, u64);
#[cfg(unix)]
fn open_protected_git_admin(root_dir: &Dir) -> Result<ProtectedGitAdmin, StateCaptureError> {
    use cap_std::fs::MetadataExt;
    root_dir.symlink_metadata(".git").map_err(git_guard_error)?;
    let location = root_dir.canonicalize(".git").map_err(git_guard_error)?;
    let metadata = root_dir.metadata(".git").map_err(git_guard_error)?;
    Ok(ProtectedGitAdmin(location, metadata.dev(), metadata.ino()))
}
enum Traversal {
    Safe,
    Missing,
}
pub(crate) fn reject_git_admin(root: &Dir, path: &Path) -> Result<(), StateCaptureError> {
    classify_git_path(root, path).map(|_| ())
}
fn classify_git_path(root_dir: &Dir, relative: &Path) -> Result<Traversal, StateCaptureError> {
    let protected = open_protected_git_admin(root_dir)?;
    let components: Vec<_> = relative.iter().collect();
    let mut prefix = PathBuf::new();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        prefix.push(component);
        match root_dir.symlink_metadata(&prefix) {
            Ok(_) => classify_git_admin_traversal(root_dir, &prefix, &protected)?,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Traversal::Missing),
            Err(error) => return Err(git_guard_error(error)),
        }
    }
    if let Some(component) = components.last() {
        prefix.push(component);
        match root_dir.symlink_metadata(&prefix) {
            Ok(metadata) if metadata.file_type().is_symlink() => {}
            Ok(_) => classify_git_admin_traversal(root_dir, &prefix, &protected)?,
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(git_guard_error(error)),
        }
    }
    Ok(Traversal::Safe)
}
fn classify_git_admin_traversal(
    root_dir: &Dir,
    candidate: &Path,
    protected: &ProtectedGitAdmin,
) -> Result<(), StateCaptureError> {
    let resolved = root_dir.canonicalize(candidate).map_err(git_guard_error)?;
    #[cfg(windows)]
    return reject_if_protected(root_dir, &resolved, protected);
    #[cfg(unix)]
    {
        let inside = resolved == protected.0
            || resolved.strip_prefix(&protected.0).is_ok()
            || physical_ancestor_is_protected(root_dir, &resolved, protected)?;
        if inside {
            return Err(StateCaptureError::new(
                "governed path enters protected Git administrative subtree",
            ));
        }
        Ok(())
    }
}
#[cfg(unix)]
fn physical_ancestor_is_protected(
    root_dir: &Dir,
    resolved: &Path,
    protected: &ProtectedGitAdmin,
) -> Result<bool, StateCaptureError> {
    use cap_std::fs::MetadataExt;
    for ancestor in resolved
        .ancestors()
        .filter(|path| !path.as_os_str().is_empty())
    {
        let metadata = root_dir.metadata(ancestor).map_err(git_guard_error)?;
        if metadata.dev() == protected.1 && metadata.ino() == protected.2 {
            return Ok(true);
        }
    }
    Ok(false)
}
fn git_guard_error(_: std::io::Error) -> StateCaptureError {
    StateCaptureError::new("cannot classify Git administrative traversal")
}
pub(crate) fn observe_paths(
    root_dir: &Dir,
    paths: &[PathBuf],
    explicit: &BTreeSet<PathBuf>,
) -> Result<Vec<ObservedPath>, StateCaptureError> {
    paths
        .iter()
        .map(|relative| {
            Ok(ObservedPath {
                path: relative.clone(),
                object: observe_path(root_dir, relative, explicit.contains(relative))?,
            })
        })
        .collect()
}
fn observe_path(
    root_dir: &Dir,
    relative: &Path,
    explicitly_observed: bool,
) -> Result<ObservedObject, StateCaptureError> {
    if let Traversal::Missing = classify_git_path(root_dir, relative)? {
        return Ok(ObservedObject::Absent);
    }
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
        reject_git_admin(root_dir, relative)?;
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
        reject_git_admin(root_dir, relative)?;
        return Ok(ObservedObject::Symlink {
            mode: relevant_mode(&before),
            target,
        });
    }
    if file_type.is_dir() {
        return if explicitly_observed {
            reject_git_admin(root_dir, relative)?;
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
pub(crate) fn os_units(value: &std::ffi::OsStr) -> Vec<u8> {
    os_units_impl(value)
}
#[cfg(unix)]
fn os_units_impl(value: &std::ffi::OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    value.as_bytes().to_vec()
}
#[cfg(windows)]
fn os_units_impl(value: &std::ffi::OsStr) -> Vec<u8> {
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
struct MetadataGuard(u32, u64, i64, i64, i64, i64, u64, u64);
#[cfg(unix)]
impl From<&Metadata> for MetadataGuard {
    fn from(metadata: &Metadata) -> Self {
        use cap_std::fs::MetadataExt;
        Self(
            metadata.mode(),
            metadata.size(),
            metadata.mtime(),
            metadata.mtime_nsec(),
            metadata.ctime(),
            metadata.ctime_nsec(),
            metadata.dev(),
            metadata.ino(),
        )
    }
}
#[cfg(windows)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct MetadataGuard(
    u64,
    Option<std::time::Duration>,
    Option<std::time::Duration>,
    bool,
    (bool, bool, bool),
);
#[cfg(windows)]
impl From<&Metadata> for MetadataGuard {
    fn from(metadata: &Metadata) -> Self {
        Self(
            metadata.len(),
            system_time(metadata.modified()),
            system_time(metadata.created()),
            metadata.permissions().readonly(),
            (
                metadata.file_type().is_file(),
                metadata.file_type().is_dir(),
                metadata.file_type().is_symlink(),
            ),
        )
    }
}
#[cfg(windows)]
fn system_time(value: std::io::Result<SystemTime>) -> Option<std::time::Duration> {
    value.ok()?.into_std().duration_since(UNIX_EPOCH).ok()
}
