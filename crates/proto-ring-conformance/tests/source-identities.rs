#![forbid(unsafe_code)]

//! Exact source-identity preservation for four independently inspected modules.
//! This gate deliberately makes no claim about arbitrary Rust source semantics.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct ProtectedSource {
    path: &'static str,
    expected_blob: &'static str,
}

const PROTECTED_SOURCES: &[ProtectedSource] = &[
    ProtectedSource {
        path: "crates/proto-ring-engine/src/evidence_requirements.rs",
        expected_blob: "edf175deef0cdc46c9af2f94b208c6f70f89d438",
    },
    ProtectedSource {
        path: "crates/proto-ring-engine/src/exact_evidence_binding.rs",
        expected_blob: "7e4fe66d85891e11c13c1397638c1c4d4ae479ac",
    },
    ProtectedSource {
        path: "crates/proto-ring-engine/src/projection_registry.rs",
        expected_blob: "165ae955a932ebb240beb3103c065aa33898fc44",
    },
    ProtectedSource {
        path: "crates/proto-ring-engine/src/governance_bindings.rs",
        expected_blob: "5bd1d49817230711e1236e8ba9d3167cc9d13d77",
    },
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("conformance crate belongs to the workspace")
        .to_path_buf()
}

fn verify_protected_sources(source_root: &Path, git_root: &Path) -> Result<(), String> {
    for protected in PROTECTED_SOURCES {
        let source_path = source_root.join(protected.path);
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|error| format!("{} is unavailable: {error}", protected.path))?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(format!("{} is not a regular file", protected.path));
        }
        require_tracked_path(git_root, protected.path)?;
        let actual_blob = hash_working_file(git_root, protected.path, &source_path)?;
        if actual_blob != protected.expected_blob {
            return Err(format!(
                "{} has Git blob {actual_blob}, expected {}",
                protected.path, protected.expected_blob
            ));
        }
    }
    Ok(())
}

fn require_tracked_path(git_root: &Path, relative_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .current_dir(git_root)
        .args(["ls-files", "--error-unmatch", "--", relative_path])
        .output()
        .map_err(|error| format!("could not execute Git for {relative_path}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{relative_path} is not a tracked file: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    parse_tracked_path(relative_path, &output.stdout)
}

fn parse_tracked_path(relative_path: &str, stdout: &[u8]) -> Result<(), String> {
    let mut expected = relative_path.as_bytes().to_vec();
    expected.push(b'\n');
    if stdout != expected {
        return Err(format!(
            "Git returned malformed tracked-path output for {relative_path}"
        ));
    }
    Ok(())
}

fn hash_working_file(
    git_root: &Path,
    relative_path: &str,
    source_path: &Path,
) -> Result<String, String> {
    let path_argument = format!("--path={relative_path}");
    let output = Command::new("git")
        .current_dir(git_root)
        .arg("hash-object")
        .arg(path_argument)
        .arg("--")
        .arg(source_path)
        .output()
        .map_err(|error| format!("could not execute Git for {relative_path}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Git could not hash {relative_path}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    parse_blob_id(relative_path, &output.stdout)
}

fn parse_blob_id(relative_path: &str, stdout: &[u8]) -> Result<String, String> {
    if stdout.len() != 41 || stdout[40] != b'\n' {
        return Err(format!(
            "Git returned malformed blob output for {relative_path}"
        ));
    }
    let digest = &stdout[..40];
    if !digest
        .iter()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    {
        return Err(format!(
            "Git returned malformed blob output for {relative_path}"
        ));
    }
    String::from_utf8(digest.to_vec())
        .map_err(|_| format!("Git returned malformed blob output for {relative_path}"))
}

fn copy_protected_sources() -> tempfile::TempDir {
    let root = repository_root();
    let temporary = tempfile::tempdir().expect("temporary source root is available");
    for protected in PROTECTED_SOURCES {
        let destination = temporary.path().join(protected.path);
        fs::create_dir_all(destination.parent().expect("protected source has a parent"))
            .expect("temporary source parent is created");
        fs::copy(root.join(protected.path), destination)
            .expect("protected source is copied without changing its bytes");
    }
    temporary
}

fn assert_rejected(source_root: &Path) {
    assert!(
        verify_protected_sources(source_root, &repository_root()).is_err(),
        "changed protected source unexpectedly retained its accepted identity"
    );
}

#[test]
fn accepted_production_sources_have_exact_git_blob_identities() {
    let root = repository_root();
    verify_protected_sources(&root, &root).unwrap();
}

#[test]
fn changing_one_byte_of_each_protected_source_fails() {
    for protected in PROTECTED_SOURCES {
        let temporary = copy_protected_sources();
        let path = temporary.path().join(protected.path);
        let mut content = fs::read(&path).expect("protected source is readable");
        content[0] ^= 1;
        fs::write(path, content).expect("temporary source mutation is written");
        assert_rejected(temporary.path());
    }
}

#[test]
fn import_alias_change_fails() {
    assert_appended_change_fails("\nuse unknown_external as hidden;\n");
}

#[test]
fn qualified_external_call_change_fails() {
    assert_appended_change_fails("\nfn mutation() { unknown_external::operation(); }\n");
}

#[test]
fn macro_module_and_cfg_declaration_changes_fail() {
    for addition in [
        "\nmacro_rules! changed { () => {} }\n",
        "\nmod changed {}\n",
        "\n#[cfg(test)] mod changed {}\n",
    ] {
        assert_appended_change_fails(addition);
    }
}

fn assert_appended_change_fails(addition: &str) {
    let temporary = copy_protected_sources();
    let path = temporary.path().join(PROTECTED_SOURCES[0].path);
    let mut content = fs::read(&path).expect("protected source is readable");
    content.extend_from_slice(addition.as_bytes());
    fs::write(path, content).expect("temporary source mutation is written");
    assert_rejected(temporary.path());
}

#[test]
fn missing_protected_source_fails() {
    let temporary = copy_protected_sources();
    fs::remove_file(temporary.path().join(PROTECTED_SOURCES[0].path))
        .expect("temporary protected source is removed");
    assert_rejected(temporary.path());
}

#[test]
fn nonregular_protected_source_fails() {
    let temporary = copy_protected_sources();
    let path = temporary.path().join(PROTECTED_SOURCES[0].path);
    fs::remove_file(&path).expect("temporary protected source is removed");
    fs::create_dir(path).expect("temporary directory replaces the protected source");
    assert_rejected(temporary.path());
}

#[cfg(unix)]
#[test]
fn symlinked_protected_source_fails() {
    use std::os::unix::fs::symlink;

    let temporary = copy_protected_sources();
    let path = temporary.path().join(PROTECTED_SOURCES[0].path);
    let target = temporary.path().join(PROTECTED_SOURCES[1].path);
    fs::remove_file(&path).expect("temporary protected source is removed");
    symlink(target, path).expect("temporary source symlink is created");
    assert_rejected(temporary.path());
}

#[test]
fn restoring_original_bytes_restores_success() {
    let temporary = copy_protected_sources();
    let path = temporary.path().join(PROTECTED_SOURCES[0].path);
    let original = fs::read(&path).expect("protected source is readable");
    let mut changed = original.clone();
    changed.push(b'\n');
    fs::write(&path, changed).expect("temporary source mutation is written");
    assert_rejected(temporary.path());
    fs::write(path, original).expect("original source bytes are restored");
    verify_protected_sources(temporary.path(), &repository_root()).unwrap();
}

#[test]
fn git_process_failure_fails_closed() {
    let temporary = copy_protected_sources();
    let non_repository = tempfile::tempdir().expect("temporary non-repository is available");
    assert!(verify_protected_sources(temporary.path(), non_repository.path()).is_err());
}

#[test]
fn malformed_git_output_fails_closed() {
    assert!(parse_tracked_path("protected.rs", b"protected.rs").is_err());
    assert!(parse_tracked_path("protected.rs", b"other.rs\n").is_err());
    for malformed in [
        b"".as_slice(),
        b"edf175deef0cdc46c9af2f94b208c6f70f89d438".as_slice(),
        b"EDF175DEEF0CDC46C9AF2F94B208C6F70F89D438\n".as_slice(),
        b"not-a-git-object-id\n".as_slice(),
    ] {
        assert!(parse_blob_id("protected.rs", malformed).is_err());
    }
}
