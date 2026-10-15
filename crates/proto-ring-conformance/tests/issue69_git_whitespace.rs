#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Vector;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

const RESPONSIBILITY: &str = "git-whitespace.validate";
const VECTOR_COUNT: usize = 15;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root exists")
        .to_path_buf()
}

fn vectors(corpus: &LoadedCorpus) -> &[Vector] {
    &corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == RESPONSIBILITY)
        .expect("Wave B matrix exists")
        .vectors
}

#[test]
fn issue69_wave_b_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let matrix = vectors(&corpus);
    assert_eq!(matrix.len(), VECTOR_COUNT);
    let actual_ids: BTreeSet<_> = matrix
        .iter()
        .map(|vector| vector.vector_id.clone())
        .collect();
    let expected_ids: BTreeSet<_> = [
        "clean",
        "git-unavailable",
        "payload-malformed-json",
        "payload-malformed-utf8",
        "payload-missing-path",
        "payload-missing-sha",
        "payload-nonexistent",
        "payload-nonobject",
        "payload-unavailable",
        "pull-request",
        "push-range",
        "root-push",
        "staged",
        "unknown-event",
        "unstaged",
    ]
    .into_iter()
    .map(|suffix| format!("{RESPONSIBILITY}.{suffix}"))
    .collect();
    assert_eq!(actual_ids, expected_ids);

    let candidate = ReferenceRustCandidate;
    let mut matches = 0;
    for vector in matrix {
        let request = CandidateRequest {
            responsibility_id: RESPONSIBILITY.to_owned(),
            vector_id: vector.vector_id.clone(),
            fixture: vector.fixture.clone(),
        };
        let actual = candidate.execute(&request).expect("candidate runs");
        let expected = CandidateState::Observation(vector.expected_observation.observation());
        if actual == expected {
            matches += 1;
        } else {
            eprintln!("vector_id={}", vector.vector_id);
            eprintln!("actual={actual:?}");
            eprintln!("expected={expected:?}");
        }
    }
    assert_eq!(matches, VECTOR_COUNT);
    eprintln!("GIT_WHITESPACE_RUST=15/15");
    eprintln!("ISSUE69_WAVE_B_RUST=15/15");
}

#[test]
fn issue69_wave_b_source_boundaries_are_explicit() {
    let repository = root();
    let engine =
        fs::read_to_string(repository.join("crates/proto-ring-engine/src/git_whitespace.rs"))
            .unwrap();
    let adapter = fs::read_to_string(
        repository.join("crates/proto-ring-conformance/src/rust_candidate/git_whitespace.rs"),
    )
    .unwrap();
    let registration =
        fs::read_to_string(repository.join("crates/proto-ring-conformance/src/rust_candidate.rs"))
            .unwrap();
    let implementation = format!("{engine}{adapter}{registration}");

    assert_eq!(engine.matches("Command::new(\"git\")").count(), 1);
    assert_eq!(engine.matches("\"--check\"").count(), 4);
    assert!(engine.contains("&[\"diff\", \"--check\"]"));
    assert!(engine.contains("&[\"diff\", \"--cached\", \"--check\"]"));
    assert!(engine.contains("format!(\"{before}..{after}\")"));
    assert!(engine.contains("format!(\"{base}...{head}\")"));
    assert!(engine.contains("&[\"hash-object\", \"-t\", \"tree\", \"--stdin\"]"));
    assert!(!engine.contains("4b825dc642cb6eb9a060e54bf8d69288fbee4904"));
    assert!(!engine.contains("\"-w\""));

    assert!(engine.contains("environment.get(EVENT_NAME)"));
    assert!(engine.contains("environment.get(EVENT_PATH)"));
    assert!(!engine.contains("std::env"));
    assert!(engine.contains("std::str::from_utf8"));
    assert!(engine.contains("serde_json::from_str"));
    assert!(engine.contains("value.as_object()"));
    assert!(engine.contains("GitExecutionFailure::ProcessStart"));
    assert!(engine.contains("check_with_runner"));
    assert!(engine.contains("UnavailableGitRunner"));

    for forbidden in [
        "Command::new(\"sh\")",
        "Command::new(\"bash\")",
        "regex::",
        "\"hash-object\", \"-w\"",
        "\"add\"",
        "\"commit\"",
        "\"update-ref\"",
        "\"checkout\"",
    ] {
        assert!(
            !engine.contains(forbidden),
            "forbidden mechanism: {forbidden}"
        );
    }
    assert!(adapter.contains("git_process_start_unavailable"));
    assert!(adapter.contains("resolve_fixture_event_path"));
    assert!(adapter.contains("diagnostic.kind == DiagnosticKind::GitProcessStartFailure"));

    let corpus = load_corpus(&repository).unwrap();
    for vector in vectors(&corpus) {
        assert!(!implementation.contains(&vector.vector_id));
    }
    for source in [
        "crates/proto-ring-engine/src/lib.rs",
        "crates/proto-ring-engine/src/git_whitespace.rs",
        "crates/proto-ring-conformance/src/rust_candidate.rs",
        "crates/proto-ring-conformance/src/rust_candidate/git_whitespace.rs",
        "crates/proto-ring-conformance/tests/issue69_git_whitespace.rs",
        "crates/proto-ring-conformance/tests/corpus.rs",
        "crates/proto-ring-conformance/tests/corpus/support.rs",
    ] {
        let lines = fs::read_to_string(repository.join(source))
            .unwrap()
            .lines()
            .count();
        assert!(lines <= 400, "{source} has {lines} lines");
    }

    eprintln!("GIT_WHITESPACE_AUTHORITY=GIT_DIFF_CHECK");
    eprintln!("UNSTAGED_CHECK=YES");
    eprintln!("STAGED_CHECK=YES");
    eprintln!("PUSH_RANGE=TWO_DOT");
    eprintln!("PULL_REQUEST_RANGE=TRIPLE_DOT");
    eprintln!("EMPTY_TREE_HARDCODED=NO");
    eprintln!("EMPTY_TREE_OBJECT_FORMAT_AGNOSTIC=YES");
    eprintln!("EVENT_SELECTION_AMBIENT_ENVIRONMENT=NO");
    eprintln!("EVENT_SELECTION_EXPLICIT_MAPPING_ONLY=YES");
    eprintln!("EVENT_PAYLOAD_UTF8_REQUIRED=YES");
    eprintln!("EVENT_PAYLOAD_JSON_OBJECT_REQUIRED=YES");
    eprintln!("PROCESS_START_FAILURE_CONTROLLED=YES");
    eprintln!("REPOSITORY_MUTATION=NO");
    eprintln!("SHELL_EXECUTION=NO");
    eprintln!("VECTOR_ID_SPECIAL_CASES=NO");
}
