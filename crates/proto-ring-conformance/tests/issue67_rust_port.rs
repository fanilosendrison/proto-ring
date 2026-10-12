#![forbid(unsafe_code)]
#[path = "issue67_rust_port/diagnostics.rs"]
mod diagnostics;
use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Observation;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_engine::repository_state;
use proto_ring_engine::repository_state::conformance_internal;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
const RESPONSIBILITY: &str = "repository-state.capture";
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root exists")
        .to_path_buf()
}
fn repository_state_requests(corpus: &LoadedCorpus) -> Vec<CandidateRequest> {
    let matrix = corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == RESPONSIBILITY)
        .expect("RepositoryState matrix exists");
    matrix
        .vectors
        .iter()
        .map(|vector| CandidateRequest {
            responsibility_id: RESPONSIBILITY.to_owned(),
            vector_id: vector.vector_id.clone(),
            fixture: vector.fixture.clone(),
        })
        .collect()
}
fn repository_state_expected(corpus: &LoadedCorpus) -> Vec<Observation> {
    corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == RESPONSIBILITY)
        .expect("RepositoryState matrix exists")
        .vectors
        .iter()
        .map(|vector| vector.expected_observation.observation())
        .collect()
}
fn execute_repository_state(corpus: &LoadedCorpus) -> Vec<CandidateState> {
    let candidate = ReferenceRustCandidate;
    repository_state_requests(corpus)
        .iter()
        .map(|request| candidate.execute(request).expect("Rust candidate executes"))
        .collect()
}
fn git(repository: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .status()
        .expect("Git executes");
    assert!(status.success(), "Git command failed: {arguments:?}");
}
fn initialize_committed_repository(repository: &Path) {
    git(repository, &["init", "-q", "-b", "main"]);
    git(repository, &["config", "user.name", "Corpus Author"]);
    git(
        repository,
        &["config", "user.email", "corpus@example.invalid"],
    );
    fs::write(repository.join("tracked.txt"), b"tracked\n").expect("tracked file is written");
    git(repository, &["add", "tracked.txt"]);
    git(repository, &["commit", "-q", "-m", "baseline"]);
}
fn initialized_repository() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    initialize_committed_repository(temporary.path());
    temporary
}
#[cfg(unix)]
#[test]
fn issue67_exact_root_preserves_git_line_terminator_like_path_bytes() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let temporary = tempfile::tempdir().expect("temporary root exists");
    for (suffix, label) in [
        (b'\r', "ISSUE67_EXACT_ROOT_CR_SUFFIX"),
        (b'\n', "ISSUE67_EXACT_ROOT_LF_SUFFIX"),
    ] {
        let mut name = b"repository-".to_vec();
        name.push(suffix);
        let repository = temporary.path().join(OsString::from_vec(name));
        fs::create_dir(&repository).expect("exact repository path is created");
        git(&repository, &["init", "-q", "-b", "main"]);
        let resolved = repository_state::resolve_worktree_root(&repository)
            .expect("exact worktree root resolves");
        assert_eq!(
            resolved,
            repository.canonicalize().expect("path canonicalizes")
        );
        eprintln!("{label}=PASS");
    }
}
#[test]
fn issue67_untracked_gitignore_cannot_silently_exclude_governed_path() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    fs::write(repository.join(".gitignore"), b"hidden.txt\n")
        .expect("untracked ignore policy is written");
    fs::write(repository.join("hidden.txt"), b"hidden\n").expect("hidden file is written");
    assert!(repository_state::capture(repository, &[]).is_err());
    eprintln!("ISSUE67_UNTRACKED_GITIGNORE_FAIL_CLOSED=PASS");
}
#[test]
fn issue67_explicit_scope_rejects_git_implementation_state() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    for relative in [".git", ".git/config", "nested/.git/config"] {
        assert!(
            repository_state::capture(repository, &[PathBuf::from(relative)]).is_err(),
            "{relative} must be rejected",
        );
    }
    for relative in [".gitignore", ".github"] {
        repository_state::capture(repository, &[PathBuf::from(relative)])
            .expect("exact .git prefix lookalike remains admissible");
    }
    eprintln!("ISSUE67_EXPLICIT_DOT_GIT_REJECTED=PASS");
    eprintln!("ISSUE67_DOT_GIT_DESCENDANT_REJECTED=PASS");
    eprintln!("ISSUE67_NESTED_DOT_GIT_COMPONENT_REJECTED=PASS");
    eprintln!("ISSUE67_DOT_GIT_PREFIX_FALSE_POSITIVES=0");
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
fn issue67_git_admin_alias_respects_host_filesystem_identity() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    let aliases = [".GIT", ".GiT"];
    let host_alias = same_filesystem_object(&repository.join(".git"), &repository.join(".GIT"));
    eprintln!(
        "HOST_DOT_GIT_CASE_ALIAS={}",
        if host_alias { "yes" } else { "no" }
    );
    if host_alias {
        for alias in aliases {
            if same_filesystem_object(&repository.join(".git"), &repository.join(alias)) {
                assert!(repository_state::capture(repository, &[PathBuf::from(alias)]).is_err());
            }
        }
        assert!(repository_state::capture(repository, &[PathBuf::from(".GIT/config")]).is_err());
        assert!(
            repository_state::discover_repository_paths(repository, &[PathBuf::from(".GIT")])
                .is_err()
        );
        eprintln!("ISSUE67_HOST_PHYSICAL_DOT_GIT_ALIAS_REJECTED=PASS");
    } else {
        fs::write(repository.join(".GIT"), b"ordinary").expect("distinct case variant is written");
        let before = repository_state::capture(repository, &[PathBuf::from(".GIT")])
            .expect("distinct case variant is observable");
        fs::write(repository.join(".GIT"), b"changed").expect("case variant is mutated");
        let after = repository_state::capture(repository, &[PathBuf::from(".GIT")])
            .expect("mutated case variant is observable");
        assert_ne!(before.identity, after.identity);
        eprintln!("ISSUE67_DISTINCT_DOT_GIT_CASE_VARIANT_OBSERVABLE=PASS");
    }
    for relative in [".gitignore", ".github"] {
        repository_state::capture(repository, &[PathBuf::from(relative)])
            .expect("prefix lookalike remains admissible");
    }
}
#[cfg(unix)]
#[test]
fn issue67_final_symlink_to_git_dir_is_observed_as_symlink_not_followed() {
    use std::os::unix::fs::symlink;
    let temporary = initialized_repository();
    let repository = temporary.path();
    symlink(".git", repository.join("git-link")).expect("Git link is created");
    let before = repository_state::capture(repository, &[PathBuf::from("git-link")])
        .expect("final symlink object is observable");
    fs::remove_file(repository.join("git-link")).expect("old link is removed");
    symlink("tracked.txt", repository.join("git-link")).expect("new link is created");
    let after = repository_state::capture(repository, &[PathBuf::from("git-link")])
        .expect("changed final symlink object is observable");
    assert_ne!(before.identity, after.identity);
    symlink(".git/objects", repository.join("git-objects-link"))
        .expect("Git objects link is created");
    repository_state::capture(repository, &[PathBuf::from("git-objects-link")])
        .expect("final Git descendant symlink object is observable");
    eprintln!("ISSUE67_FINAL_SYMLINK_TO_DOT_GIT_OBJECT_ONLY=PASS");
    eprintln!("ISSUE67_FINAL_SYMLINK_TO_DOT_GIT_DESCENDANT_OBJECT_ONLY=PASS");
}
#[cfg(unix)]
fn repository_with_nested_git_alias(target: &str) -> tempfile::TempDir {
    use std::os::unix::fs::symlink;
    let temporary = initialized_repository();
    fs::create_dir(temporary.path().join("nested")).expect("nested directory is created");
    symlink(target, temporary.path().join("nested/link")).expect("nested alias is created");
    temporary
}
#[cfg(unix)]
#[test]
fn issue67_nested_intermediate_alias_to_root_git_is_rejected() {
    use std::os::unix::fs::symlink;
    let temporary = repository_with_nested_git_alias("../.git");
    assert!(
        repository_state::capture(temporary.path(), &[PathBuf::from("nested/link/config")])
            .is_err()
    );
    symlink(".git", temporary.path().join("git-parent")).expect("root alias is created");
    assert!(
        repository_state::capture(temporary.path(), &[PathBuf::from("git-parent/config")]).is_err()
    );
    eprintln!("ISSUE67_NESTED_ALIAS_TO_ROOT_DOT_GIT_REJECTED=PASS");
    eprintln!("ISSUE67_INTERMEDIATE_SYMLINK_TO_DOT_GIT_REJECTED=PASS");
}
#[cfg(unix)]
#[test]
fn issue67_nested_intermediate_alias_to_git_descendant_is_rejected() {
    let temporary = repository_with_nested_git_alias("../.git/objects");
    assert!(
        repository_state::capture(temporary.path(), &[PathBuf::from("nested/link/pack")]).is_err()
    );
    eprintln!("ISSUE67_NESTED_ALIAS_TO_DOT_GIT_DESCENDANT_REJECTED=PASS");
}
#[cfg(unix)]
#[test]
fn issue67_git_alias_with_absent_child_still_rejects() {
    let temporary = repository_with_nested_git_alias("../.git");
    assert!(
        repository_state::capture(
            temporary.path(),
            &[PathBuf::from("nested/link/definitely-absent")]
        )
        .is_err()
    );
    eprintln!("ISSUE67_DOT_GIT_ALIAS_ABSENT_CHILD_REJECTED=PASS");
}
#[test]
fn issue67_nested_absent_scope_is_observable() {
    let temporary = initialized_repository();
    let scope = [PathBuf::from("missing/child")];
    let first = repository_state::capture(temporary.path(), &scope).expect("nested absence works");
    let second = repository_state::capture(temporary.path(), &scope).expect("absence is stable");
    assert_eq!(first.scope_paths, scope);
    assert_eq!(first.identity, second.identity);
    fs::create_dir(temporary.path().join("missing")).expect("parent is created");
    fs::write(temporary.path().join("missing/child"), b"present").expect("child is created");
    let present = repository_state::capture(temporary.path(), &scope).expect("presence works");
    assert_ne!(first.identity, present.identity);
    eprintln!("ISSUE67_NESTED_ABSENT_SCOPE=PASS");
    eprintln!("ISSUE67_NESTED_ABSENT_TO_PRESENT_IDENTITY_CHANGE=PASS");
}
#[test]
fn issue67_nested_absent_after_existing_safe_parent_is_observable() {
    let temporary = initialized_repository();
    fs::create_dir(temporary.path().join("safe")).expect("safe parent exists");
    repository_state::capture(temporary.path(), &[PathBuf::from("safe/missing/child")])
        .expect("absence below safe parent is observable");
    eprintln!("ISSUE67_NESTED_ABSENT_AFTER_SAFE_PARENT=PASS");
}
#[cfg(unix)]
#[test]
fn issue67_absent_prefix_to_git_alias_fails_closed() {
    use std::os::unix::fs::symlink;
    let temporary = initialized_repository();
    let result = conformance_internal::capture_with_validation_hook(
        temporary.path(),
        &[PathBuf::from("missing/config")],
        |_| {
            symlink(".git", temporary.path().join("missing"))
                .map_err(|error| repository_state::StateCaptureError::new(error.to_string()))
        },
    );
    assert!(result.is_err());
    eprintln!("ISSUE67_ABSENT_PREFIX_TO_DOT_GIT_ALIAS_FAILS_CLOSED=PASS");
}
#[test]
fn issue67_git_trace_targets_cannot_create_files() {
    let temporary = initialized_repository();
    let repository = temporary.path();
    let targets = tempfile::tempdir().expect("trace target directory exists");
    let names = [
        "GIT_TRACE",
        "GIT_TRACE_PERFORMANCE",
        "GIT_TRACE_SETUP",
        "GIT_TRACE2",
        "GIT_TRACE2_EVENT",
        "GIT_TRACE2_PERF",
    ];
    let ambient: Vec<_> = names
        .iter()
        .map(|name| {
            (
                OsString::from(name),
                targets.path().join(name).into_os_string(),
            )
        })
        .collect();
    conformance_internal::capture_with_ambient_environment(repository, &[], &ambient)
        .expect("trace environment is sanitized before successful capture");
    assert!(names.iter().all(|name| !targets.path().join(name).exists()));
    eprintln!("ISSUE67_GIT_TRACE_FILE_SIDE_EFFECTS=0");
    eprintln!("ISSUE67_GIT_TRACE2_FILE_SIDE_EFFECTS=0");
}
#[test]
fn issue67_rust_port_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let requests = repository_state_requests(&corpus);
    let expected = repository_state_expected(&corpus);
    let actual = execute_repository_state(&corpus);
    assert_eq!(requests.len(), 53);
    diagnostics::assert_all_match(&requests, &actual, &expected);
    eprintln!("ISSUE67_RUST_VECTORS=53");
    eprintln!("ISSUE67_RUST_MATCH=53");
    eprintln!("ISSUE67_RUST_MISMATCH=0");
}
fn assert_repository_state_vectors(ids: &[&str]) {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let requests = repository_state_requests(&corpus);
    let expected = repository_state_expected(&corpus);
    let actual = execute_repository_state(&corpus);
    for id in ids {
        let index = requests
            .iter()
            .position(|request| request.vector_id == *id)
            .expect("vector exists");
        assert_eq!(
            actual[index],
            CandidateState::Observation(expected[index].clone())
        );
    }
}
#[test]
fn issue67_hostile_capture_vectors_fail_closed() {
    for (id, label) in [
        (
            "repository-state.capture.concurrent-mutation",
            "ISSUE67_STRUCTURAL_MUTATION",
        ),
        (
            "repository-state.capture.concurrent-content-only-mutation",
            "ISSUE67_CONTENT_ONLY_MUTATION",
        ),
        (
            "repository-state.capture.concurrent-path-membership-mutation",
            "ISSUE67_PATH_MEMBERSHIP_MUTATION",
        ),
    ] {
        assert_repository_state_vectors(&[id]);
        eprintln!("{label}=PASS");
    }
}
#[test]
fn issue67_identity_is_opaque() {
    assert_repository_state_vectors(&[
        "repository-state.capture.scope-order-duplicate-identity",
        "repository-state.capture.bytes-change",
    ]);
}
#[test]
fn issue67_ignore_policy_vectors_match() {
    assert_repository_state_vectors(&[
        "repository-state.capture.tracked-repository-ignore",
        "repository-state.capture.repository-ignore-untracked-excluded",
        "repository-state.capture.info-exclude-untracked-still-governed",
        "repository-state.capture.ambient-global-ignore-config",
        "repository-state.capture.untracked",
        "repository-state.capture.untracked-content-mutation",
    ]);
}
#[test]
fn issue67_ambient_git_redirects_are_neutralized() {
    assert_repository_state_vectors(&[
        "repository-state.capture.ambient-redirect",
        "repository-state.capture.ambient-worktree-redirect",
        "repository-state.capture.ambient-object-database-redirect",
        "repository-state.capture.ambient-namespace-redirect",
        "repository-state.capture.ambient-global-ignore-config",
    ]);
}
