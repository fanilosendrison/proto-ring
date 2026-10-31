#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{
    CandidateExecutor, CandidateRequest, CandidateState, candidate_requests,
};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::{
    CaseMatrix, ComparisonKind, ComparisonRule, CorpusIndex, ExpectedObservation,
    MigrationDisposition, Observation, ObservationKind, ResponsibilityRecordSummary,
    TransportValue, Vector,
};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_conformance::standalone::{
    CandidateExecution, StandaloneVerdict, evaluate_standalone_executions, run_standalone,
};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("conformance crate belongs to the workspace")
        .to_path_buf()
}

fn observation(value: &str) -> Observation {
    Observation {
        kind: ObservationKind::Result,
        value: TransportValue::String(value.to_owned()),
    }
}

fn synthetic_corpus() -> LoadedCorpus {
    LoadedCorpus {
        index: CorpusIndex {
            responsibility_files: vec!["responsibilities/synthetic.json".to_owned()],
            case_files: vec!["cases/synthetic.json".to_owned()],
        },
        responsibilities: vec![ResponsibilityRecordSummary {
            responsibility_id: "synthetic.responsibility".to_owned(),
            migration_disposition: MigrationDisposition::RustPort,
            case_file: Some("cases/synthetic.json".to_owned()),
        }],
        matrices: vec![CaseMatrix {
            responsibility_id: "synthetic.responsibility".to_owned(),
            vectors: vec![Vector {
                vector_id: "synthetic.vector".to_owned(),
                fixture: serde_json::Value::Null,
                expected_observation: ExpectedObservation {
                    kind: ObservationKind::Result,
                    value: TransportValue::String("expected".to_owned()),
                    derivation: serde_json::Value::Null,
                },
                comparison: ComparisonRule {
                    kind: ComparisonKind::Exact,
                },
            }],
        }],
    }
}

fn request_by_id(corpus: &LoadedCorpus, vector_id: &str) -> CandidateRequest {
    candidate_requests(corpus)
        .into_iter()
        .find(|request| request.vector_id == vector_id)
        .expect("requested corpus vector must exist")
}

fn record_entry_value_mut<'a>(
    value: &'a mut serde_json::Value,
    name: &str,
) -> &'a mut serde_json::Value {
    value["value"]
        .as_array_mut()
        .expect("transport record must contain entries")
        .iter_mut()
        .find(|entry| entry["name"] == name)
        .expect("transport record entry must exist")
        .get_mut("value")
        .expect("transport record entry must contain a value")
}

fn remove_record_entry(value: &mut serde_json::Value, name: &str) {
    value["value"]
        .as_array_mut()
        .expect("transport record must contain entries")
        .retain(|entry| entry["name"] != name);
}

fn synthetic_execution(state: CandidateState) -> CandidateExecution {
    CandidateExecution {
        responsibility_id: "synthetic.responsibility".to_owned(),
        vector_id: "synthetic.vector".to_owned(),
        state,
    }
}

#[test]
fn standalone_path_has_no_python_bridge_or_differential_dependency() {
    let source = include_str!("../src/standalone.rs");
    assert!(!source.contains("PythonBridge"));
    assert!(!source.contains("run_differential"));
}

#[test]
fn candidate_request_contains_only_candidate_inputs() {
    let request = CandidateRequest {
        responsibility_id: "responsibility".to_owned(),
        vector_id: "vector".to_owned(),
        fixture: serde_json::json!({"input": true}),
    };
    let serialized = serde_json::to_value(request).expect("request serializes");
    let keys: HashSet<_> = serialized
        .as_object()
        .expect("request is an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        HashSet::from(["responsibility_id", "vector_id", "fixture"])
    );
}

#[test]
fn standalone_reference_candidate_executes_the_complete_corpus() {
    let corpus = load_corpus(&repository_root()).expect("published corpus loads");
    let records = run_standalone(&corpus, &ReferenceRustCandidate)
        .expect("standalone Rust control must conform");
    let responsibilities: HashSet<_> = records
        .iter()
        .map(|record| record.responsibility_id.as_str())
        .collect();
    let vector_ids: HashSet<_> = records
        .iter()
        .map(|record| record.vector_id.as_str())
        .collect();

    assert_eq!(responsibilities.len(), 29);
    assert_eq!(records.len(), 1153);
    assert_eq!(vector_ids.len(), 1153);
    assert!(
        records
            .iter()
            .all(|record| record.verdict == StandaloneVerdict::Match)
    );

    if let Some(path) = std::env::var_os("PROTO_RING_STANDALONE_EVIDENCE_PATH") {
        let mut bytes = serde_json::to_vec_pretty(&records).expect("evidence serializes");
        bytes.push(b'\n');
        fs::write(path, bytes).expect("standalone evidence path is writable");
    }
}

#[test]
fn false_observation_fails_the_control() {
    let error = evaluate_standalone_executions(
        &synthetic_corpus(),
        vec![synthetic_execution(CandidateState::Observation(
            observation("wrong"),
        ))],
    )
    .expect_err("wrong observation must fail");
    assert!(error.to_string().contains("observation mismatch"));
}

#[test]
fn missing_result_fails_the_control() {
    let error = evaluate_standalone_executions(&synthetic_corpus(), Vec::new())
        .expect_err("missing result must fail");
    assert!(error.to_string().contains("missing candidate result"));
}

#[test]
fn duplicate_result_fails_the_control() {
    let execution = synthetic_execution(CandidateState::Observation(observation("expected")));
    let error =
        evaluate_standalone_executions(&synthetic_corpus(), vec![execution.clone(), execution])
            .expect_err("duplicate result must fail");
    assert!(error.to_string().contains("duplicate candidate result"));
}

#[test]
fn unimplemented_candidate_fails_the_control() {
    let error = evaluate_standalone_executions(
        &synthetic_corpus(),
        vec![synthetic_execution(CandidateState::Unimplemented)],
    )
    .expect_err("unimplemented result must fail");
    assert!(error.to_string().contains("unimplemented candidate"));
}

#[test]
fn issue66_support_failure_is_distinct_from_semantic_rejection() {
    let corpus = load_corpus(&repository_root()).expect("published corpus loads");
    let mut malformed = request_by_id(&corpus, "governance-bindings.registry.valid-registry");
    let semantic_inputs =
        record_entry_value_mut(&mut malformed.fixture["invocation"], "semantic_inputs");
    let authority = record_entry_value_mut(semantic_inputs, "authority");
    remove_record_entry(authority, "sources");
    ReferenceRustCandidate
        .execute(&malformed)
        .expect_err("missing harness support must fail execution");

    let rejection = request_by_id(&corpus, "governance-bindings.registry.root-non-mapping");
    let CandidateState::Observation(observation) = ReferenceRustCandidate
        .execute(&rejection)
        .expect("semantic rejection must be a candidate observation")
    else {
        panic!("semantic rejection responsibility must be implemented")
    };
    assert_eq!(observation.kind, ObservationKind::ControlledRejection);
}

#[test]
fn fixture_preparation_failure_remains_a_harness_error() {
    let mut corpus = synthetic_corpus();
    corpus.matrices[0].responsibility_id = "repository-state.capture".to_owned();
    corpus.matrices[0].vectors[0].fixture = serde_json::Value::Null;
    let error = run_standalone(&corpus, &ReferenceRustCandidate)
        .expect_err("fixture failure must not become a semantic rejection");
    assert!(!error.to_string().contains("observation mismatch"));
}
