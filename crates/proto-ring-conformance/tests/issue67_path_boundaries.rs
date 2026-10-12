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

#[cfg(unix)]
fn ignore_nested_repository(repository: &Path) {
    fs::write(repository.join(".gitignore"), b"nested/\n")
        .expect("outer nested-repository ignore policy is written");
    git(repository, &["add", ".gitignore"]);
    git(
        repository,
        &["commit", "-q", "-m", "ignore nested repository"],
    );
}

#[cfg(unix)]
fn initialize_nested_repository(repository: &Path) -> PathBuf {
    let nested = repository.join("nested");
    git(repository, &["init", "-q", "-b", "main", "nested"]);
    git(&nested, &["config", "user.name", "Nested Author"]);
    git(&nested, &["config", "user.email", "nested@example.invalid"]);
    fs::write(nested.join("tracked.txt"), b"nested\n").expect("nested file is written");
    git(&nested, &["add", "tracked.txt"]);
    git(&nested, &["commit", "-q", "-m", "nested baseline"]);
    assert!(nested.join(".git").is_dir());
    nested
}

#[cfg(unix)]
fn assert_protected_admin_rejection(error: repository_state::StateCaptureError) {
    assert!(
        error.to_string().contains("protected Git administrative"),
        "unexpected rejection boundary: {error}",
    );
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
    match (fs::metadata(left), fs::metadata(right)) {
        (Ok(left), Ok(right)) => left.dev() == right.dev() && left.ino() == right.ino(),
        _ => false,
    }
}

#[cfg(unix)]
#[test]
fn issue67_nested_git_admin_case_alias_respects_physical_identity() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    ignore_nested_repository(repository);
    let nested = initialize_nested_repository(repository);
    let aliases = same_filesystem_object(&nested.join(".git"), &nested.join(".GIT"));
    eprintln!(
        "HOST_NESTED_DOT_GIT_CASE_ALIAS={}",
        if aliases { "yes" } else { "no" }
    );

    if aliases {
        repository_state::capture(repository, &[PathBuf::from("nested/tracked.txt")])
            .expect("nested repository content remains governed when explicit");
        for relative in ["nested/.GIT/config", "nested/.GiT/config"] {
            let error = repository_state::capture(repository, &[PathBuf::from(relative)])
                .expect_err("physical nested Git-admin alias must reject");
            assert_protected_admin_rejection(error);
        }
        eprintln!("ISSUE67_NESTED_PHYSICAL_DOT_GIT_ALIAS_REJECTED=PASS");
    } else {
        fs::create_dir(nested.join(".GIT")).expect("distinct case directory is created");
        let ordinary = nested.join(".GIT/config");
        fs::write(&ordinary, b"alpha").expect("ordinary case-variant file is written");
        let scope = [PathBuf::from("nested/.GIT/config")];
        let before = repository_state::capture(repository, &scope)
            .expect("distinct stored case variant remains governed");
        fs::write(&ordinary, b"beta").expect("ordinary case-variant file is changed");
        let after = repository_state::capture(repository, &scope)
            .expect("changed distinct case variant remains governed");
        assert_ne!(before.identity, after.identity);
        eprintln!("ISSUE67_NESTED_DISTINCT_CASE_VARIANT_OBSERVABLE=PASS");
    }
}

#[cfg(unix)]
#[test]
fn issue67_unix_stored_case_variant_git_admin_is_host_resolved() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    ignore_nested_repository(repository);
    let nested = initialize_nested_repository(repository);
    let intermediate = nested.join(".proto-ring-git-admin-rename");
    fs::rename(nested.join(".git"), &intermediate)
        .expect("nested Git directory is moved through an intermediate name");
    fs::rename(&intermediate, nested.join(".GIT"))
        .expect("nested Git directory receives stored case-variant spelling");
    let names: Vec<_> = fs::read_dir(&nested)
        .expect("nested repository is enumerable")
        .map(|entry| entry.expect("directory entry is readable").file_name())
        .collect();
    assert!(names.iter().any(|name| name == ".GIT"));
    assert!(!names.iter().any(|name| name == ".git"));

    if fs::symlink_metadata(nested.join(".git")).is_ok() {
        git(&nested, &["rev-parse", "--show-toplevel"]);
        let error = repository_state::capture(repository, &[PathBuf::from("nested/.GIT/config")])
            .expect_err("host-resolved stored Git-admin directory is rejected");
        assert_protected_admin_rejection(error);
        eprintln!("ISSUE67_UNIX_STORED_CASE_VARIANT_ADMIN=PASS");
    } else {
        let scope = [PathBuf::from("nested/.GIT/config")];
        let before = repository_state::capture(repository, &scope)
            .expect("case-sensitive distinct stored variant remains governed");
        let config = nested.join(".GIT/config");
        let mut bytes = fs::read(&config).expect("ordinary stored variant is readable");
        bytes.extend_from_slice(b"\n# changed\n");
        fs::write(&config, bytes).expect("ordinary stored variant is changed");
        let after = repository_state::capture(repository, &scope)
            .expect("changed distinct stored variant remains governed");
        assert_ne!(before.identity, after.identity);
        eprintln!("ISSUE67_UNIX_STORED_CASE_VARIANT_ADMIN=NOT_APPLICABLE");
        eprintln!("ISSUE67_UNIX_DISTINCT_STORED_CASE_VARIANT_GOVERNED=PASS");
    }
}

#[cfg(unix)]
#[test]
fn issue67_symlink_into_sibling_nested_git_admin_is_rejected() {
    use std::os::unix::fs::symlink;

    let temporary = initialized_repository();
    let repository = temporary.path();
    ignore_nested_repository(repository);
    initialize_nested_repository(repository);
    fs::create_dir(repository.join("other")).expect("sibling directory is created");
    symlink("../nested/.git/objects", repository.join("other/link"))
        .expect("sibling Git-admin symlink is created");
    git(repository, &["add", "other/link"]);
    git(repository, &["commit", "-q", "-m", "track sibling symlink"]);

    let error =
        repository_state::capture(repository, &[PathBuf::from("other/link/definitely-absent")])
            .expect_err("resolved traversal into sibling nested Git admin must reject");
    assert_protected_admin_rejection(error);
    eprintln!("ISSUE67_SIBLING_NESTED_GIT_ADMIN_SYMLINK_REJECTED=PASS");
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
