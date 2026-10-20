#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

pub fn invoke(request: &Value) -> (Output, Value) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_proto-ring-compat"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("compat binary starts");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let response = serde_json::from_slice(&output.stdout).unwrap();
    (output, response)
}

pub fn compose(root: &Path, projection: Option<&str>) -> (Output, Value) {
    let mut input = json!({"repository":root,"binding_ids":["provider"]});
    if let Some(projection) = projection {
        input["projection"] = json!(projection);
    }
    invoke(
        &json!({"transport_version":1,"operation":"repository-governance-state.compose","input":input}),
    )
}

pub fn complete_fixture() -> tempfile::TempDir {
    fixture("rgs-complete")
}

pub fn optional_fixture() -> tempfile::TempDir {
    fixture("rgs-optional")
}

fn fixture(name: &str) -> tempfile::TempDir {
    let temporary = tempfile::tempdir().unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    copy_directory(&source, temporary.path());
    git(temporary.path(), &["init", "-q", "-b", "main"]);
    git(temporary.path(), &["config", "user.name", "Compat Test"]);
    git(
        temporary.path(),
        &["config", "user.email", "compat@example.invalid"],
    );
    git(temporary.path(), &["add", "-A"]);
    git(temporary.path(), &["commit", "-q", "-m", "fixture"]);
    temporary
}

fn copy_directory(source: &Path, target: &Path) {
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir(&destination).unwrap();
            copy_directory(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn git(root: &Path, arguments: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(arguments)
            .status()
            .unwrap()
            .success()
    );
}

#[derive(Debug, Eq, PartialEq)]
pub struct RepositorySnapshot {
    files: Vec<(String, Vec<u8>)>,
    head: Vec<u8>,
    index: Vec<u8>,
    status: Vec<u8>,
    config: Vec<u8>,
}

pub fn snapshot(root: &Path) -> RepositorySnapshot {
    let mut files = Vec::new();
    collect(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    RepositorySnapshot {
        files,
        head: git_output(root, &["rev-parse", "HEAD"]),
        index: git_output(root, &["ls-files", "--stage", "-z"]),
        status: git_output(root, &["status", "--porcelain=v1", "-z"]),
        config: git_output(root, &["config", "--local", "--list", "-z"]),
    }
}

fn collect(root: &Path, directory: &Path, output: &mut Vec<(String, Vec<u8>)>) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        if directory == root && entry.file_name() == ".git" {
            continue;
        }
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            collect(root, &path, output);
        } else {
            output.push((
                path.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                fs::read(path).unwrap(),
            ));
        }
    }
}

fn git_output(root: &Path, arguments: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .unwrap();
    assert!(output.status.success());
    output.stdout
}
