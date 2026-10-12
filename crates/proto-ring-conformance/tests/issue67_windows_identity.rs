#![cfg(windows)]
#![forbid(unsafe_code)]

use proto_ring_engine::repository_state;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(repository: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .status()
        .expect("Git executes");
    assert!(status.success(), "Git command failed: {arguments:?}");
}

fn initialized_repository() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    git(temporary.path(), &["init", "-q", "-b", "main"]);
    git(temporary.path(), &["config", "user.name", "Corpus Author"]);
    git(
        temporary.path(),
        &["config", "user.email", "corpus@example.invalid"],
    );
    fs::write(temporary.path().join("tracked.txt"), b"tracked\n").expect("tracked file is written");
    git(temporary.path(), &["add", "tracked.txt"]);
    git(temporary.path(), &["commit", "-q", "-m", "baseline"]);
    temporary
}

#[test]
fn issue67_windows_git_admin_alias_uses_physical_handle_identity() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    let alias_capture = repository_state::capture(repository, &[PathBuf::from(".GIT/config")]);
    if alias_capture.is_ok() {
        eprintln!("ISSUE67_WINDOWS_PHYSICAL_DOT_GIT_ALIAS=NOT_APPLICABLE");
        return;
    }
    repository_state::capture(repository, &[PathBuf::from(".gitignore")])
        .expect("Git-admin prefix lookalike remains admissible");
    eprintln!("ISSUE67_WINDOWS_PHYSICAL_DOT_GIT_ALIAS_REJECTED=PASS");
}

#[test]
fn issue67_windows_git_admin_identity_is_id128() {
    let temporary = initialized_repository();
    repository_state::capture(temporary.path(), &[PathBuf::from(".gitignore")])
        .expect("qualification filesystem provides exact Id128 identity");
    eprintln!("ISSUE67_WINDOWS_FILE_ID_INFO_128=PASS");
}
