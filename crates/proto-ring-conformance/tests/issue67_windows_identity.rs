#![cfg(windows)]
#![forbid(unsafe_code)]

use proto_ring_engine::repository_state::{self, conformance_internal};
use std::ffi::OsString;
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

fn initialize_repository(repository: &Path) {
    fs::create_dir_all(repository).expect("repository directory is created");
    git(repository, &["init", "-q", "-b", "main"]);
    git(repository, &["config", "user.name", "Windows Author"]);
    git(
        repository,
        &["config", "user.email", "windows@example.invalid"],
    );
    fs::write(repository.join("tracked.txt"), b"tracked\n").expect("tracked file is written");
    git(repository, &["add", "tracked.txt"]);
    git(repository, &["commit", "-q", "-m", "baseline"]);
}

fn initialized_repository() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    initialize_repository(temporary.path());
    temporary
}

fn assert_protected_admin_rejection(error: repository_state::StateCaptureError) {
    assert!(
        error.to_string().contains("protected Git administrative"),
        "unexpected rejection boundary: {error}",
    );
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
    assert!(
        fs::symlink_metadata(linked.join(".git"))
            .expect("linked .git exists")
            .file_type()
            .is_file()
    );
}

#[test]
fn issue67_windows_git_admin_alias_uses_physical_handle_identity() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    repository_state::capture(repository, &[PathBuf::from(".GIT/config")])
        .expect_err("case-insensitive Git-admin directory alias is rejected");
    repository_state::capture(repository, &[PathBuf::from(".gitignore")])
        .expect("Git-admin prefix lookalike remains admissible");
    eprintln!("ISSUE67_WINDOWS_PHYSICAL_DOT_GIT_ALIAS_REJECTED=PASS");
}

#[test]
fn issue67_windows_nested_git_admin_case_alias_is_rejected() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    let nested = repository.join("nested");
    fs::write(repository.join(".gitignore"), b"nested/\n")
        .expect("outer nested-repository ignore policy is written");
    git(repository, &["add", ".gitignore"]);
    git(
        repository,
        &["commit", "-q", "-m", "ignore nested repository"],
    );
    initialize_repository(&nested);
    assert!(nested.join(".git").is_dir());

    repository_state::capture(repository, &[PathBuf::from("nested/tracked.txt")])
        .expect("nested repository content remains governed when explicit");
    let error = repository_state::capture(repository, &[PathBuf::from("nested/.GIT/config")])
        .expect_err("case-insensitive nested Git-admin alias is rejected");
    assert_protected_admin_rejection(error);
    repository_state::capture(repository, &[PathBuf::from("nested/.gitignore")])
        .expect("nested Git-admin prefix lookalike remains admissible");
    eprintln!("ISSUE67_WINDOWS_NESTED_DOT_GIT_ALIAS_REJECTED=PASS");
}

#[test]
fn issue67_windows_nested_git_file_alias_is_rejected() {
    let temporary = tempfile::tempdir().expect("temporary root exists");
    let outer = temporary.path().join("outer");
    let source = temporary.path().join("source");
    let linked = outer.join("nested-linked");
    initialize_repository(&outer);
    initialize_repository(&source);
    fs::write(outer.join(".gitignore"), b"nested-linked/\n")
        .expect("outer ignore policy is written");
    git(&outer, &["add", ".gitignore"]);
    git(&outer, &["commit", "-q", "-m", "ignore linked worktree"]);
    add_linked_worktree(&source, &linked);

    repository_state::capture(&outer, &[]).expect("ordinary outer capture remains valid");
    let error = repository_state::capture(&outer, &[PathBuf::from("nested-linked/.GIT")])
        .expect_err("case-insensitive nested Git-file namespace alias is rejected");
    assert_protected_admin_rejection(error);
    eprintln!("ISSUE67_WINDOWS_NESTED_DOT_GIT_FILE_ALIAS_REJECTED=PASS");
}

#[test]
fn issue67_windows_git_admin_identity_is_id128() {
    let temporary = initialized_repository();
    repository_state::capture(temporary.path(), &[PathBuf::from(".gitignore")])
        .expect("qualification filesystem provides exact Id128 identity");
    eprintln!("ISSUE67_WINDOWS_FILE_ID_INFO_128=PASS");
}

#[test]
fn issue67_windows_case_variant_git_environment_is_sanitized() {
    let temporary = tempfile::tempdir().expect("temporary root exists");
    let target = temporary.path().join("target");
    let alternate = temporary.path().join("alternate");
    let trace_target = temporary.path().join("git-trace.txt");
    initialize_repository(&target);
    initialize_repository(&alternate);

    let canonical_target = target
        .canonicalize()
        .expect("target repository canonicalizes");
    let canonical_alternate = alternate
        .canonicalize()
        .expect("alternate repository canonicalizes");
    assert_ne!(canonical_target, canonical_alternate);

    let ordinary = repository_state::capture(&target, &[]).expect("ordinary capture succeeds");
    assert_eq!(ordinary.repository, canonical_target);
    let ambient = [
        (
            OsString::from("git_dir"),
            alternate.join(".git").into_os_string(),
        ),
        (
            OsString::from("Git_Work_Tree"),
            alternate.clone().into_os_string(),
        ),
        (
            OsString::from("gIt_TrAcE"),
            trace_target.clone().into_os_string(),
        ),
    ];
    let sanitized = conformance_internal::capture_with_ambient_environment(&target, &[], &ambient)
        .expect("case-variant ambient Git environment is neutralized");
    assert_eq!(sanitized.repository, ordinary.repository);
    assert_ne!(sanitized.repository, canonical_alternate);
    assert_eq!(sanitized.identity, ordinary.identity);
    assert!(!trace_target.exists());

    eprintln!("ISSUE67_WINDOWS_CASE_VARIANT_GIT_REDIRECTION_NEUTRALIZED=PASS");
    eprintln!("ISSUE67_WINDOWS_CASE_VARIANT_GIT_TRACE_SIDE_EFFECTS=0");
}

#[test]
fn issue67_windows_linked_worktree_git_file_alias_is_rejected() {
    let temporary = tempfile::tempdir().expect("temporary root exists");
    let main = temporary.path().join("main");
    let linked = temporary.path().join("linked");
    initialize_repository(&main);
    add_linked_worktree(&main, &linked);

    repository_state::capture(&linked, &[]).expect("linked worktree capture succeeds");
    repository_state::capture(&linked, &[PathBuf::from(".GIT")])
        .expect_err("case-insensitive Git-file namespace alias is rejected");
    eprintln!("ISSUE67_WINDOWS_LINKED_WORKTREE_DOT_GIT_ALIAS_REJECTED=PASS");
}

#[test]
fn issue67_windows_dot_git_file_hardlink_remains_a_distinct_entry() {
    let temporary = tempfile::tempdir().expect("temporary root exists");
    let main = temporary.path().join("main");
    let linked = temporary.path().join("linked");
    initialize_repository(&main);
    add_linked_worktree(&main, &linked);

    let git_file = linked.join(".git");
    let hardlink = linked.join("gitfile-hardlink");
    fs::hard_link(&git_file, &hardlink).expect("distinct hard-link entry is created");
    let untracked = git(&linked, &["ls-files", "--others", "-z"]);
    assert!(
        untracked
            .stdout
            .split(|byte| *byte == 0)
            .any(|path| path == b"gitfile-hardlink")
    );

    let before = repository_state::capture(&linked, &[])
        .expect("distinct Git-file hard-link entry is governed");
    let original = fs::read(&hardlink).expect("Git file bytes are read by the test");
    let changed = if original.ends_with(b"\r\n") {
        original[..original.len() - 2].to_vec()
    } else if original.ends_with(b"\n") {
        original[..original.len() - 1].to_vec()
    } else {
        let mut changed = original;
        changed.push(b'\n');
        changed
    };
    fs::write(&hardlink, changed).expect("hard-link content is changed");
    git(&linked, &["rev-parse", "--show-toplevel"]);
    let after = repository_state::capture(&linked, &[])
        .expect("changed Git-file hard-link entry remains governed");
    assert_ne!(before.identity, after.identity);

    eprintln!("ISSUE67_WINDOWS_DOT_GIT_FILE_HARDLINK_DISTINCT_ENTRY=PASS");
}
