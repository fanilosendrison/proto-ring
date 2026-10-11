#![forbid(unsafe_code)]

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Observation;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_engine::repository_state;
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
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    let repository = temporary.path();
    git(repository, &["init", "-q", "-b", "main"]);
    git(repository, &["config", "user.name", "Corpus Author"]);
    git(
        repository,
        &["config", "user.email", "corpus@example.invalid"],
    );
    fs::write(repository.join("tracked.txt"), b"tracked\n").expect("tracked file is written");
    git(repository, &["add", "tracked.txt"]);
    git(repository, &["commit", "-q", "-m", "baseline"]);
    fs::write(repository.join(".gitignore"), b"hidden.txt\n")
        .expect("untracked ignore policy is written");
    fs::write(repository.join("hidden.txt"), b"hidden\n").expect("hidden file is written");
    assert!(repository_state::capture(repository, &[]).is_err());
    eprintln!("ISSUE67_UNTRACKED_GITIGNORE_FAIL_CLOSED=PASS");
}

#[test]
fn issue67_rust_port_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let requests = repository_state_requests(&corpus);
    let expected = repository_state_expected(&corpus);
    let actual = execute_repository_state(&corpus);
    assert_eq!(requests.len(), 53);
    let matching = actual
        .iter()
        .zip(&expected)
        .filter(|(state, expected)| **state == CandidateState::Observation((*expected).clone()))
        .count();
    assert_eq!(matching, 53);
    eprintln!("ISSUE67_RUST_VECTORS=53");
    eprintln!("ISSUE67_RUST_MATCH=53");
    eprintln!("ISSUE67_RUST_MISMATCH=0");
}

#[test]
fn issue67_hostile_capture_vectors_fail_closed() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let requests = repository_state_requests(&corpus);
    let expected = repository_state_expected(&corpus);
    let actual = execute_repository_state(&corpus);
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
        let index = requests
            .iter()
            .position(|request| request.vector_id == id)
            .expect("hostile vector exists");
        assert_eq!(
            actual[index],
            CandidateState::Observation(expected[index].clone())
        );
        eprintln!("{label}=PASS");
    }
}

#[test]
fn issue67_identity_is_opaque() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let requests = repository_state_requests(&corpus);
    let expected = repository_state_expected(&corpus);
    let actual = execute_repository_state(&corpus);
    for id in [
        "repository-state.capture.scope-order-duplicate-identity",
        "repository-state.capture.bytes-change",
    ] {
        let index = requests
            .iter()
            .position(|request| request.vector_id == id)
            .expect("identity vector exists");
        assert_eq!(
            actual[index],
            CandidateState::Observation(expected[index].clone())
        );
    }
}

#[test]
fn issue67_ignore_policy_vectors_match() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let requests = repository_state_requests(&corpus);
    let expected = repository_state_expected(&corpus);
    let actual = execute_repository_state(&corpus);
    for id in [
        "repository-state.capture.tracked-repository-ignore",
        "repository-state.capture.repository-ignore-untracked-excluded",
        "repository-state.capture.info-exclude-untracked-still-governed",
        "repository-state.capture.ambient-global-ignore-config",
        "repository-state.capture.untracked",
        "repository-state.capture.untracked-content-mutation",
    ] {
        let index = requests
            .iter()
            .position(|request| request.vector_id == id)
            .expect("ignore-policy vector exists");
        assert_eq!(
            actual[index],
            CandidateState::Observation(expected[index].clone())
        );
    }
}

#[test]
fn issue67_ambient_git_redirects_are_neutralized() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let requests = repository_state_requests(&corpus);
    let expected = repository_state_expected(&corpus);
    let actual = execute_repository_state(&corpus);
    for id in [
        "repository-state.capture.ambient-redirect",
        "repository-state.capture.ambient-worktree-redirect",
        "repository-state.capture.ambient-object-database-redirect",
        "repository-state.capture.ambient-namespace-redirect",
        "repository-state.capture.ambient-global-ignore-config",
    ] {
        let index = requests
            .iter()
            .position(|request| request.vector_id == id)
            .expect("ambient vector exists");
        assert_eq!(
            actual[index],
            CandidateState::Observation(expected[index].clone())
        );
    }
}
