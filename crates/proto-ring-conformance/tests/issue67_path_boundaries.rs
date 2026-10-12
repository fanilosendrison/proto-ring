#![forbid(unsafe_code)]

use proto_ring_engine::repository_state;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(repository: &Path, arguments: &[&str]) -> std::process::Output {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .output()
        .expect("Git executes");
    assert!(output.status.success(), "Git command failed: {arguments:?}");
    output
}

fn initialized_repository() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    git(temporary.path(), &["init", "-q", "-b", "main"]);
    git(
        temporary.path(),
        &["config", "user.name", "Boundary Author"],
    );
    git(
        temporary.path(),
        &["config", "user.email", "boundary@example.invalid"],
    );
    fs::write(temporary.path().join("tracked.txt"), b"tracked\n").expect("tracked file is written");
    git(temporary.path(), &["add", "tracked.txt"]);
    git(temporary.path(), &["commit", "-q", "-m", "baseline"]);
    temporary
}

#[test]
fn issue67_explicit_scope_admission_is_lexical_only() {
    let temporary = initialized_repository();
    let scope = [PathBuf::from("missing/child")];
    let first = repository_state::capture(temporary.path(), &scope)
        .expect("missing nested scope is admitted and observed as absent");
    let second = repository_state::capture(temporary.path(), &scope)
        .expect("missing nested scope remains observable");
    assert_eq!(first.scope_paths, scope);
    assert_eq!(first.identity, second.identity);
    eprintln!("ISSUE67_EXPLICIT_SCOPE_ADMISSION_LEXICAL_ONLY=PASS");
}

#[cfg(unix)]
fn same_filesystem_object(left: &Path, right: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let left = fs::metadata(left).expect("left metadata exists");
    let right = fs::metadata(right).expect("right metadata exists");
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(unix)]
#[test]
fn issue67_final_hardlink_to_git_internal_file_remains_governed() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    let source = repository.join(".git/proto-ring-hardlink-source");
    let alias = repository.join("hardlink-alias");
    fs::write(&source, b"alpha").expect("administrative source is written");
    fs::hard_link(&source, &alias).expect("worktree hard link is created");
    assert!(same_filesystem_object(&source, &alias));

    let untracked = git(repository, &["ls-files", "--others", "-z"]);
    assert_eq!(untracked.stdout, b"hardlink-alias\0");
    let discovered = repository_state::discover_repository_paths(repository, &[])
        .expect("Git-visible hard-link entry is discoverable");
    assert!(discovered.contains(&PathBuf::from("hardlink-alias")));
    assert!(!discovered.contains(&PathBuf::from(".git/proto-ring-hardlink-source")));

    let before = repository_state::capture(repository, &[])
        .expect("Git-visible hard-link worktree entry is governed");
    fs::write(&alias, b"beta").expect("governed hard-link bytes are changed");
    let after = repository_state::capture(repository, &[])
        .expect("changed hard-link worktree entry remains governed");
    assert_ne!(before.identity, after.identity);

    eprintln!("ISSUE67_FINAL_HARDLINK_ALIAS_GIT_VISIBLE=PASS");
    eprintln!("ISSUE67_FINAL_HARDLINK_ALIAS_GOVERNED=PASS");
    eprintln!("ISSUE67_DOT_GIT_SOURCE_NOT_RECURSIVELY_GOVERNED=PASS");
    eprintln!("ISSUE67_FINAL_HARDLINK_ALIAS_CONTENT_SENSITIVE=PASS");
}
