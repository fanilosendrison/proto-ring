//! RepositoryState explicit-scope admission and exact spelling validation.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::{StateCaptureError, path};

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

pub(crate) fn explicit_scope_path_identity(
    repository: &Path,
    declared: &str,
) -> Result<Option<PathBuf>, StateCaptureError> {
    let exact = PathBuf::from(declared);
    if validate_spelling(&exact).is_err() || reject_git_implementation_path(&exact).is_err() {
        return Ok(None);
    }
    let root = Dir::open_ambient_dir(repository, ambient_authority()).map_err(|error| {
        StateCaptureError::new(format!("cannot open exact worktree capability: {error}"))
    })?;
    let components: Vec<_> = exact.iter().collect();
    let mut prefix = PathBuf::new();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        prefix.push(component);
        match root.symlink_metadata(&prefix) {
            Ok(_) => match root.canonicalize(&prefix) {
                Ok(_) => {}
                Err(error) if is_capability_escape(&error) => return Ok(None),
                Err(error) => {
                    return Err(StateCaptureError::new(format!(
                        "cannot classify explicit scope parent traversal: {error}"
                    )));
                }
            },
            Err(error) if error.kind() == ErrorKind::NotFound => {
                path::reject_git_admin(&root, &exact)?;
                return Ok(Some(exact));
            }
            Err(error) => {
                return Err(StateCaptureError::new(format!(
                    "cannot inspect explicit scope parent traversal: {error}"
                )));
            }
        }
    }
    path::reject_git_admin(&root, &exact)?;
    Ok(Some(exact))
}

fn is_capability_escape(error: &std::io::Error) -> bool {
    error.kind() == ErrorKind::PermissionDenied
        && error.to_string() == "a path led outside of the filesystem"
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
        || matches!(path.components().next(), Some(Component::Prefix(_)))
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::explicit_scope_path_identity;

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "proto-ring-issue68-scope-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&root).expect("test root is created");
            Self(root.canonicalize().expect("test root resolves"))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn exact_scope_identity_preserves_absent_direct_spelling() {
        let root = TestRoot::new();
        assert_eq!(
            explicit_scope_path_identity(root.path(), "missing/child").expect("admission succeeds"),
            Some(PathBuf::from("missing/child"))
        );
    }

    #[test]
    fn routing_only_spellings_are_not_scope_identities() {
        let root = TestRoot::new();
        for declared in [
            "",
            "config/../target",
            "config/./target",
            "config/",
            "config//target",
            ".git/config",
        ] {
            assert_eq!(
                explicit_scope_path_identity(root.path(), declared)
                    .expect("inadmissibility is controlled"),
                None,
                "{declared:?}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn final_external_symlink_is_admitted_but_intermediate_escape_is_not() {
        use std::os::unix::fs::symlink;

        let tree = TestRoot::new();
        let repository = tree.path().join("repository");
        let outside = tree.path().join("outside");
        fs::create_dir(&repository).expect("repository exists");
        fs::create_dir(&outside).expect("outside directory exists");
        fs::write(outside.join("target"), b"outside").expect("outside target exists");
        symlink("../outside/target", repository.join("final-link")).expect("final symlink exists");
        symlink("../outside", repository.join("parent-link")).expect("intermediate symlink exists");

        assert_eq!(
            explicit_scope_path_identity(&repository, "final-link")
                .expect("final symlink admission succeeds"),
            Some(PathBuf::from("final-link"))
        );
        assert_eq!(
            explicit_scope_path_identity(&repository, "parent-link/target")
                .expect("intermediate escape is controlled"),
            None
        );
    }
}
