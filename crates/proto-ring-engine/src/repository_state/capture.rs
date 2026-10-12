use std::ffi::OsString;
use std::path::{Path, PathBuf};

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::git::{self, RepositoryStructure};
use super::path::{self, ObservedObject, ObservedPath};
use super::{RepositoryState, RepositoryStateIdentity, StateCaptureError};

pub fn capture(
    repository: &Path,
    explicit_scope: &[PathBuf],
) -> Result<RepositoryState, StateCaptureError> {
    capture_internal(repository, explicit_scope, &[], |_| Ok(()))
}

#[cfg(feature = "conformance-internal")]
pub(in crate::repository_state) fn capture_with_validation_hook<F>(
    repository: &Path,
    explicit_scope: &[PathBuf],
    hook: F,
) -> Result<RepositoryState, StateCaptureError>
where
    F: FnOnce(&Path) -> Result<(), StateCaptureError>,
{
    capture_internal(repository, explicit_scope, &[], hook)
}

#[cfg(feature = "conformance-internal")]
pub(in crate::repository_state) fn capture_with_ambient_environment(
    repository: &Path,
    explicit_scope: &[PathBuf],
    ambient: &[(OsString, OsString)],
) -> Result<RepositoryState, StateCaptureError> {
    capture_internal(repository, explicit_scope, ambient, |_| Ok(()))
}

fn capture_internal<F>(
    repository: &Path,
    explicit_scope: &[PathBuf],
    ambient: &[(OsString, OsString)],
    hook: F,
) -> Result<RepositoryState, StateCaptureError>
where
    F: FnOnce(&Path) -> Result<(), StateCaptureError>,
{
    let root = git::resolve_worktree_root_with_environment(repository, ambient)?;
    let scope = path::validate_explicit_scope(explicit_scope)?;
    let root_dir = Dir::open_ambient_dir(&root, ambient_authority()).map_err(|error| {
        StateCaptureError::new(format!("cannot open exact worktree capability: {error}"))
    })?;
    let structure_before = git::capture_structure(&root, &scope, ambient)?;
    let records_first = path::observe_paths(&root_dir, &structure_before.paths, &scope)?;
    hook(&root)?;
    let records_second = path::observe_paths(&root_dir, &structure_before.paths, &scope)?;
    if records_second != records_first {
        return Err(StateCaptureError::new(
            "governed path state drifted during capture validation",
        ));
    }
    let structure_after = git::capture_structure(&root, &scope, ambient)?;
    if structure_after != structure_before {
        return Err(StateCaptureError::new(
            "Git structure or governed path membership drifted during capture",
        ));
    }
    let identity = encode_identity(&structure_before, &records_first)?;
    Ok(RepositoryState {
        repository: root,
        identity: RepositoryStateIdentity(identity),
        scope_paths: scope.into_iter().collect(),
    })
}

fn encode_identity(
    structure: &RepositoryStructure,
    records: &[ObservedPath],
) -> Result<Vec<u8>, StateCaptureError> {
    let mut output = Vec::new();
    frame(&mut output, b"proto-ring repository-state canonical bytes")?;
    optional_frame(&mut output, structure.symbolic_head.as_deref())?;
    optional_frame(&mut output, structure.head_oid.as_deref())?;
    frame(&mut output, &structure.index_raw)?;
    length(&mut output, records.len())?;
    for record in records {
        frame(&mut output, &path::os_units(record.path.as_os_str()))?;
        match &record.object {
            ObservedObject::Absent => output.push(0),
            ObservedObject::Regular { mode, bytes } => {
                output.push(1);
                output.extend_from_slice(&mode.to_be_bytes());
                frame(&mut output, bytes)?;
            }
            ObservedObject::Symlink { mode, target } => {
                output.push(2);
                output.extend_from_slice(&mode.to_be_bytes());
                frame(&mut output, &path::os_units(target.as_os_str()))?;
            }
            ObservedObject::Directory { mode } => {
                output.push(3);
                output.extend_from_slice(&mode.to_be_bytes());
            }
        }
    }
    Ok(output)
}

fn optional_frame(output: &mut Vec<u8>, value: Option<&[u8]>) -> Result<(), StateCaptureError> {
    match value {
        Some(value) => {
            output.push(1);
            frame(output, value)
        }
        None => {
            output.push(0);
            Ok(())
        }
    }
}

fn frame(output: &mut Vec<u8>, value: &[u8]) -> Result<(), StateCaptureError> {
    length(output, value.len())?;
    output.extend_from_slice(value);
    Ok(())
}

fn length(output: &mut Vec<u8>, value: usize) -> Result<(), StateCaptureError> {
    let value = u64::try_from(value)
        .map_err(|_| StateCaptureError::new("identity component exceeds u64 framing"))?;
    output.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use cap_std::ambient_authority;
    use cap_std::fs::Dir;

    use super::path;

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestTree(PathBuf);

    impl TestTree {
        fn new() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "proto-ring-issue67-capability-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&root).expect("create test tree");
            Self(root.canonicalize().expect("resolve test tree"))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn escaping_tree() -> (TestTree, PathBuf, PathBuf) {
        let tree = TestTree::new();
        let repository = tree.path().join("repo");
        let outside = tree.path().join("outside");
        fs::create_dir_all(repository.join("parent")).expect("create contained parent");
        fs::create_dir(&outside).expect("create outside directory");
        fs::write(repository.join("parent/file.txt"), b"contained").expect("write contained file");
        fs::write(outside.join("file.txt"), b"MUST_NOT_BE_OBSERVED").expect("write sentinel");
        (tree, repository, PathBuf::from("parent/file.txt"))
    }

    #[test]
    fn observation_rejects_explicit_parent_escape_after_scope_admission() {
        let (_tree, repository, relative) = escaping_tree();
        let explicit = path::validate_explicit_scope(std::slice::from_ref(&relative))
            .expect("contained path is admitted before mutation");
        fs::remove_dir_all(repository.join("parent")).expect("remove contained parent");
        symlink("../outside", repository.join("parent")).expect("install escaping parent");
        let root_dir = Dir::open_ambient_dir(&repository, ambient_authority()).expect("open root");
        assert!(
            path::observe_paths(&root_dir, std::slice::from_ref(&relative), &explicit).is_err()
        );
    }

    #[test]
    fn observation_rejects_discovered_intermediate_symlink_escape() {
        let (_tree, repository, relative) = escaping_tree();
        fs::remove_dir_all(repository.join("parent")).expect("remove contained parent");
        symlink("../outside", repository.join("parent")).expect("install escaping parent");
        let root_dir = Dir::open_ambient_dir(&repository, ambient_authority()).expect("open root");
        assert!(
            path::observe_paths(&root_dir, std::slice::from_ref(&relative), &BTreeSet::new(),)
                .is_err()
        );
    }

    #[test]
    fn capability_regular_open_cannot_follow_final_symlink_outside_root() {
        let (tree, repository, _) = escaping_tree();
        fs::write(tree.path().join("outside/secret"), b"secret").expect("write secret");
        symlink("../outside/secret", repository.join("probe")).expect("install final symlink");
        let root_dir = Dir::open_ambient_dir(&repository, ambient_authority()).expect("open root");
        assert!(root_dir.open("probe").is_err());
    }
}
