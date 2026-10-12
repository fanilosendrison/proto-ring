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
    assert!(output.status.success(), "Git command failed: {arguments:?}");
    output
}

fn initialized_repository() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    git(temporary.path(), &["init", "-q", "-b", "main"]);
    git(
        temporary.path(),
        &["config", "user.name", "Discovery Author"],
    );
    git(
        temporary.path(),
        &["config", "user.email", "discovery@example.invalid"],
    );
    fs::write(temporary.path().join("tracked.txt"), b"tracked\n").expect("tracked file is written");
    git(temporary.path(), &["add", "tracked.txt"]);
    git(temporary.path(), &["commit", "-q", "-m", "baseline"]);
    temporary
}

#[cfg(unix)]
fn has_terminal_separator(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes().last() == Some(&b'/')
}

#[cfg(windows)]
fn has_terminal_separator(path: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;
    matches!(path.as_os_str().encode_wide().last(), Some(unit) if unit == b'/' as u16 || unit == b'\\' as u16)
}

#[test]
fn issue67_real_git_nested_repository_transport_decodes_to_filesystem_identity() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    let nested = repository.join("nested");
    fs::create_dir(&nested).expect("nested repository directory is created");
    git(&nested, &["init", "-q", "-b", "main"]);

    let output = git(repository, &["ls-files", "-z", "--others"]);
    let records: Vec<_> = output.stdout[..output.stdout.len() - 1]
        .split(|byte| *byte == 0)
        .collect();
    let nested_record = records
        .iter()
        .copied()
        .find(|record| *record == b"nested" || *record == b"nested/")
        .expect("Git reports the nested repository");
    assert!(records.iter().all(|record| {
        !record.starts_with(b"nested") || *record == b"nested" || *record == b"nested/"
    }));
    eprintln!(
        "NESTED_REPOSITORY_GIT_DIRECTORY_MARKER_OBSERVED={}",
        if nested_record == b"nested/" {
            "YES"
        } else {
            "NO"
        }
    );

    let discovered = repository_state::discover_repository_paths(repository, &[])
        .expect("nested repository path discovery succeeds");
    let discovered_nested = discovered
        .iter()
        .find(|path| path.as_path() == Path::new("nested"))
        .expect("filesystem identity is the nested worktree entry");
    assert!(!has_terminal_separator(discovered_nested));

    let state = repository_state::capture(repository, &[PathBuf::from("nested")])
        .expect("explicit nested repository capture succeeds");
    assert_eq!(state.scope_paths, [PathBuf::from("nested")]);
    eprintln!("ISSUE67_NESTED_REPOSITORY_FILESYSTEM_IDENTITY=PASS");
    eprintln!("ISSUE67_NESTED_REPOSITORY_CAPTURE=PASS");
}
