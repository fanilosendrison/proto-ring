#![forbid(unsafe_code)]

use proto_ring_engine::repository_state;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn git(repository: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .output()
        .expect("Git executes");
    assert!(
        output.status.success(),
        "Git command failed: {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn initialize_committed_repository(repository: &Path, file_name: &str) {
    fs::create_dir_all(repository).expect("repository directory is created");
    git(repository, &["init", "-q", "-b", "main"]);
    git(repository, &["config", "user.name", "Git File Root Author"]);
    git(
        repository,
        &["config", "user.email", "git-file-root@example.invalid"],
    );
    fs::write(repository.join(file_name), b"baseline\n").expect("tracked file is written");
    git(repository, &["add", file_name]);
    git(repository, &["commit", "-q", "-m", "baseline"]);
}

fn add_linked_worktree(main: &Path, linked: &Path) {
    let output = Command::new("git")
        .arg("-C")
        .arg(main)
        .args(["worktree", "add", "--detach"])
        .arg(linked)
        .arg("HEAD")
        .output()
        .expect("Git worktree command executes");
    assert!(
        output.status.success(),
        "Git worktree add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn add_local_submodule(superproject: &Path, source: &Path) {
    let output = Command::new("git")
        .arg("-C")
        .arg(superproject)
        .args(["-c", "protocol.file.allow=always", "submodule", "add"])
        .arg(source)
        .arg("sub")
        .output()
        .expect("Git submodule command executes");
    assert!(
        output.status.success(),
        "Git submodule add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    git(superproject, &["commit", "-q", "-am", "add submodule"]);
}

fn assert_git_file_root_is_observable(repository: &Path) {
    let git_metadata = fs::symlink_metadata(repository.join(".git"))
        .expect("worktree root Git administrative entry exists");
    assert!(git_metadata.file_type().is_file());
    let canonical_repository = repository
        .canonicalize()
        .expect("requested Git worktree root canonicalizes");
    assert_eq!(
        repository_state::resolve_worktree_root(repository).expect("exact Git root resolves"),
        canonical_repository
    );
    let state = repository_state::capture(repository, &[]).expect("Git-file-root capture succeeds");
    assert_eq!(state.repository, canonical_repository);
    repository_state::discover_repository_paths(repository, &[])
        .expect("Git-file-root discovery succeeds");
}

#[test]
fn issue67_linked_worktree_git_file_root_is_supported() {
    let temporary = tempfile::tempdir().expect("temporary root exists");
    let root = temporary.path();
    let main = root.join("main");
    let linked = root.join("linked");
    initialize_committed_repository(&main, "tracked.txt");
    add_linked_worktree(&main, &linked);

    assert_git_file_root_is_observable(&linked);
    let before = repository_state::capture(&linked, &[]).expect("first linked capture succeeds");
    fs::write(linked.join("tracked.txt"), b"changed\n").expect("tracked content is changed");
    let after = repository_state::capture(&linked, &[]).expect("second linked capture succeeds");
    assert_ne!(before.identity, after.identity);

    eprintln!("ISSUE67_LINKED_WORKTREE_GIT_FILE_ROOT=PASS");
    eprintln!("ISSUE67_LINKED_WORKTREE_CONTENT_SENSITIVE=PASS");
}

#[test]
fn issue67_submodule_git_file_root_is_supported() {
    let temporary = tempfile::tempdir().expect("temporary root exists");
    let root = temporary.path();
    let source = root.join("source");
    let superproject = root.join("super");
    initialize_committed_repository(&source, "source.txt");
    initialize_committed_repository(&superproject, "super.txt");
    add_local_submodule(&superproject, &source);

    let submodule = superproject.join("sub");
    assert_git_file_root_is_observable(&submodule);
    let discovered = repository_state::discover_repository_paths(&submodule, &[])
        .expect("submodule path discovery succeeds");
    assert!(discovered.contains(&PathBuf::from("source.txt")));

    eprintln!("ISSUE67_SUBMODULE_GIT_FILE_ROOT=PASS");
}

#[cfg(unix)]
fn same_unix_object(left: &Path, right: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    let left = fs::metadata(left).expect("left metadata exists");
    let right = fs::metadata(right).expect("right metadata exists");
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(unix)]
fn exact_stored_names(directory: &Path) -> Vec<std::ffi::OsString> {
    fs::read_dir(directory)
        .expect("directory enumeration succeeds")
        .map(|entry| entry.expect("directory entry is readable").file_name())
        .collect()
}

#[cfg(unix)]
fn linked_worktree_fixture() -> (tempfile::TempDir, PathBuf) {
    let temporary = tempfile::tempdir().expect("temporary root exists");
    let root = temporary
        .path()
        .canonicalize()
        .expect("temporary root is exact");
    let main = root.join("main");
    let linked = root.join("linked");
    initialize_committed_repository(&main, "tracked.txt");
    add_linked_worktree(&main, &linked);
    assert!(
        fs::symlink_metadata(linked.join(".git"))
            .expect("linked .git exists")
            .file_type()
            .is_file()
    );
    (temporary, linked)
}

#[cfg(unix)]
#[test]
fn issue67_unix_linked_worktree_dot_git_file_hardlink_is_distinct() {
    let (_temporary, linked) = linked_worktree_fixture();
    let before = repository_state::capture(&linked, &[]).expect("initial capture succeeds");
    let git_file = linked.join(".git");
    let hardlink = linked.join("gitfile-hardlink");
    fs::hard_link(&git_file, &hardlink).expect("distinct hard-link entry is created");
    assert!(same_unix_object(&git_file, &hardlink));

    let names = exact_stored_names(&linked);
    assert!(names.iter().any(|name| name == ".git"));
    assert!(names.iter().any(|name| name == "gitfile-hardlink"));
    let untracked = git(&linked, &["ls-files", "--others", "-z"]);
    assert!(
        untracked
            .stdout
            .split(|byte| *byte == 0)
            .any(|path| path == b"gitfile-hardlink")
    );
    let discovered = repository_state::discover_repository_paths(&linked, &[])
        .expect("hard-link discovery succeeds");
    assert!(discovered.contains(&PathBuf::from("gitfile-hardlink")));
    let after = repository_state::capture(&linked, &[])
        .expect("distinct Git-file hard-link entry is governed");
    assert_ne!(before.identity, after.identity);

    eprintln!("ISSUE67_UNIX_DOT_GIT_FILE_HARDLINK_DISTINCT_ENTRY=PASS");
    eprintln!("ISSUE67_UNIX_DOT_GIT_FILE_HARDLINK_GIT_VISIBLE=PASS");
    eprintln!("ISSUE67_UNIX_DOT_GIT_FILE_HARDLINK_GOVERNED=PASS");
}

#[cfg(unix)]
#[test]
fn issue67_unix_git_file_namespace_alias_respects_stored_entry_identity() {
    let (_temporary, linked) = linked_worktree_fixture();
    let git_file = linked.join(".git");
    let case_variant = linked.join(".GIT");
    let aliases = fs::metadata(&case_variant)
        .map(|_| same_unix_object(&git_file, &case_variant))
        .unwrap_or(false);
    eprintln!(
        "HOST_FILEFORM_DOT_GIT_CASE_ALIAS={}",
        if aliases { "yes" } else { "no" }
    );

    let names = exact_stored_names(&linked);
    assert!(names.iter().any(|name| name == ".git"));
    if aliases {
        assert!(!names.iter().any(|name| name == ".GIT"));
        repository_state::capture(&linked, &[PathBuf::from(".GIT")])
            .expect_err("namespace alias without exact stored entry is rejected");
        eprintln!("ISSUE67_UNIX_FILEFORM_NAMESPACE_ALIAS_REJECTED=PASS");
    } else {
        eprintln!("ISSUE67_UNIX_FILEFORM_NAMESPACE_ALIAS=NOT_APPLICABLE");
        fs::hard_link(&git_file, &case_variant)
            .expect("case-variant distinct hard-link entry is created");
        let names = exact_stored_names(&linked);
        assert!(names.iter().any(|name| name == ".git"));
        assert!(names.iter().any(|name| name == ".GIT"));
        repository_state::capture(&linked, &[PathBuf::from(".GIT")])
            .expect("exact stored case-variant hard link is governed");
        eprintln!("ISSUE67_UNIX_DISTINCT_CASE_VARIANT_HARDLINK_GOVERNED=PASS");
    }
}

#[cfg(unix)]
#[test]
fn issue67_unix_git_file_hardlink_cannot_be_traversed_as_parent() {
    let (_temporary, linked) = linked_worktree_fixture();
    fs::hard_link(linked.join(".git"), linked.join("gitfile-hardlink"))
        .expect("distinct hard-link entry is created");
    repository_state::capture(&linked, &[PathBuf::from("gitfile-hardlink/child")])
        .expect_err("protected Git-file object cannot be traversed as a parent");
    eprintln!("ISSUE67_UNIX_DOT_GIT_FILE_HARDLINK_INTERMEDIATE_REJECTED=PASS");
}
