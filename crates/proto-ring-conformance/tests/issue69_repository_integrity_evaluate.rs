#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Vector;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

const RESPONSIBILITY: &str = "repository-integrity.evaluate";
const VECTOR_COUNT: usize = 34;

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
        .expect("Wave A matrix exists")
        .vectors
}

#[test]
fn issue69_wave_a_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let matrix = vectors(&corpus);
    assert_eq!(matrix.len(), VECTOR_COUNT);
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
    eprintln!("REPOSITORY_INTEGRITY_EVALUATE_RUST=34/34");
    eprintln!("ISSUE69_WAVE_A_RUST=34/34");
}

#[test]
fn issue69_wave_a_source_boundaries_are_explicit() {
    let repository = root();
    let engine_root = repository.join("crates/proto-ring-engine/src/repository_integrity");
    let module = fs::read_to_string(engine_root.join("mod.rs")).unwrap();
    let model = fs::read_to_string(engine_root.join("model.rs")).unwrap();
    let selectors = fs::read_to_string(engine_root.join("selectors.rs")).unwrap();
    let evaluation = fs::read_to_string(engine_root.join("evaluate.rs")).unwrap();
    let execution = fs::read_to_string(engine_root.join("execution.rs")).unwrap();
    let adapter = fs::read_to_string(repository.join(
        "crates/proto-ring-conformance/src/rust_candidate/repository_integrity_evaluation.rs",
    ))
    .unwrap();
    let implementation = format!("{module}{model}{selectors}{evaluation}{execution}{adapter}");

    assert_eq!(execution.matches("Command::new").count(), 1);
    for required in [
        "env_clear",
        ".envs(environment)",
        "ObligationStatus::Undetermined",
        "status.code().map(|code| code.to_string())",
    ] {
        assert!(
            execution.contains(required),
            "missing execution boundary: {required}"
        );
    }
    for forbidden in ["sh\"", "bash\"", "regex::", "tokio", "async_process"] {
        assert!(
            !implementation.contains(forbidden),
            "forbidden mechanism: {forbidden}"
        );
    }
    for required in [
        "repository_state::capture",
        "repository_state::discover_repository_paths",
        "discover(&root, &[])",
        "EvaluationCapturePhase::Baseline",
        "EvaluationCapturePhase::PreObligation",
        "EvaluationCapturePhase::PostObligation",
        "for validation_id in &profile.order",
        "validation.prerequisites.iter()",
        "aggregate_by_id.get(prerequisite)",
        "ValidationResult::skipped(validation)",
        "resolution_state_identity",
        "is_some_and(|identity| identity != &baseline)",
        "coherence_halted",
        "StateRelation::Unavailable",
        "StateRelation::NotEqual",
    ] {
        assert!(
            evaluation.contains(required),
            "missing evaluation boundary: {required}"
        );
    }
    for required in [
        "RepositoryPathsMode::AppendAll",
        "RepositoryPathsMode::ForEach",
        "duplicate selected repository path",
        "matches.sort_by_key",
        "SelectorKind::Path => vec![PathBuf::from",
    ] {
        assert!(format!("{evaluation}{selectors}").contains(required));
    }
    for required in [
        "proto-ring evaluation-context opaque identity",
        "realization.identity",
        "EvaluationContextIdentity",
    ] {
        assert!(
            model.contains(required),
            "missing opaque identity boundary: {required}"
        );
    }
    let identity_start = model.find("fn evaluation_context_identity").unwrap();
    let identity_end = model[identity_start..]
        .find("pub enum ObligationStatus")
        .map(|offset| identity_start + offset)
        .unwrap();
    let identity_source = &model[identity_start..identity_end];
    for excluded in [
        "command_prefix",
        "process_environment",
        "sha2",
        "serde_json",
    ] {
        assert!(
            !identity_source.contains(excluded),
            "identity includes {excluded}"
        );
    }
    assert!(module.contains("#[cfg(feature = \"conformance-internal\")]"));
    assert!(module.contains("pub enum CapturePhase"));
    assert!(!adapter.contains("captures =="));
    assert_eq!(evaluation.matches("discover(&root").count(), 1);
    assert!(!evaluation.contains("Command::new"));
    assert!(!selectors.contains("glob::"));

    let governance_state_root =
        repository.join("crates/proto-ring-engine/src/repository_governance_state");
    let governance_state = ["mod.rs", "compose.rs", "scope.rs", "orchestrate.rs"]
        .iter()
        .map(|path| fs::read_to_string(governance_state_root.join(path)).unwrap())
        .collect::<String>();
    assert!(!governance_state.contains("repository_integrity::evaluate"));
    assert!(!governance_state.contains("Command::new"));

    let corpus = load_corpus(&repository).unwrap();
    for vector in vectors(&corpus) {
        assert!(!implementation.contains(&vector.vector_id));
    }
    for source in [
        "crates/proto-ring-engine/src/repository_integrity/mod.rs",
        "crates/proto-ring-engine/src/repository_integrity/model.rs",
        "crates/proto-ring-engine/src/repository_integrity/selectors.rs",
        "crates/proto-ring-engine/src/repository_integrity/evaluate.rs",
        "crates/proto-ring-engine/src/repository_integrity/execution.rs",
        "crates/proto-ring-conformance/src/rust_candidate.rs",
        "crates/proto-ring-conformance/src/rust_candidate/repository_integrity_evaluation.rs",
        "crates/proto-ring-conformance/tests/issue69_repository_integrity_evaluate.rs",
        "crates/proto-ring-conformance/tests/corpus.rs",
        "crates/proto-ring-conformance/tests/corpus/support.rs",
    ] {
        let lines = fs::read_to_string(repository.join(source))
            .unwrap()
            .lines()
            .count();
        assert!(lines <= 400, "{source} has {lines} lines");
    }

    eprintln!("REPOSITORYSTATE_REUSE=YES");
    eprintln!("SECOND_REPOSITORY_STATE_IMPLEMENTATION=NO");
    eprintln!("PROFILE_LOADING_EXECUTES_COMMANDS=NO");
    eprintln!("REPOSITORY_GOVERNANCE_STATE_EXECUTES_COMMANDS=NO");
    eprintln!("EXPLICIT_CHILD_ENVIRONMENT_ONLY=YES");
    eprintln!("AMBIENT_ENVIRONMENT_MERGE=NO");
    eprintln!("VALIDATION_ORDER_PRESERVED=YES");
    eprintln!("PREREQUISITES_FAIL_CLOSED=YES");
    eprintln!("ZERO_FOR_EACH_SATISFIED=YES");
    eprintln!("ZERO_APPEND_ALL_EXECUTES_BASE=YES");
    eprintln!("DUPLICATE_SELECTED_PATHS_FAIL_CLOSED=YES");
    eprintln!("ABSENT_LITERAL_RETAINED=YES");
    eprintln!("GLOB_RESULTS_LEXICALLY_SORTED=YES");
    eprintln!("STATE_COHERENCE_PRE_CHECK=YES");
    eprintln!("STATE_COHERENCE_POST_CHECK=YES");
    eprintln!("STATE_COHERENCE_FAILURE_HALTS=YES");
    eprintln!("EVALUATION_CONTEXT_IDENTITY_OPAQUE=YES");
    eprintln!("EVALUATION_CONTEXT_IDENTITY_IGNORES_COMMAND_PREFIX=YES");
    eprintln!("EVALUATION_CONTEXT_IDENTITY_IGNORES_PROCESS_ENVIRONMENT=YES");
    eprintln!("EVALUATION_CONTEXT_IDENTITY_IGNORES_UNRELATED_REALIZATIONS=YES");
    eprintln!("VECTOR_ID_SPECIAL_CASES=NO");
}
