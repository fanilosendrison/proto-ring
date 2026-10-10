#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::{ObservationKind, TransportValue};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use serde_json::Value;

const ISSUE66: &[&str] = &[
    "evidence-requirements.registry",
    "exact-evidence-binding.evaluate",
    "governance-bindings.registry",
    "projection-registry.registry",
    "repository-integrity.profile",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root exists")
        .to_path_buf()
}

fn published_request(vector_id: &str) -> (CandidateRequest, CandidateState) {
    let corpus = load_corpus(&root()).expect("published corpus loads");
    let matrix = corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == "evidence-requirements.registry")
        .expect("Evidence Requirements matrix exists");
    let vector = matrix
        .vectors
        .iter()
        .find(|vector| vector.vector_id == vector_id)
        .expect("published vector exists");
    (
        CandidateRequest {
            responsibility_id: matrix.responsibility_id.clone(),
            vector_id: vector.vector_id.clone(),
            fixture: vector.fixture.clone(),
        },
        CandidateState::Observation(vector.expected_observation.observation()),
    )
}

fn json_record_field_mut<'a>(record: &'a mut Value, name: &str) -> &'a mut Value {
    let entries = record
        .get_mut("value")
        .and_then(Value::as_array_mut)
        .expect("transport record value is an array");
    entries
        .iter_mut()
        .find(|entry| entry["name"] == name)
        .and_then(|entry| entry.get_mut("value"))
        .unwrap_or_else(|| panic!("transport record contains {name}"))
}

fn transport_record_field<'a>(record: &'a TransportValue, name: &str) -> &'a TransportValue {
    let TransportValue::Record(entries) = record else {
        panic!("observation value is a record");
    };
    &entries
        .iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("observation contains {name}"))
        .value
}

#[test]
fn evidence_context_source_mismatch_is_a_harness_error() {
    let vector_id = "evidence-requirements.registry.required-unknown-context";
    let (mut request, expected) = published_request(vector_id);
    let carrier = request.fixture["steps"][0]["text"]
        .as_str()
        .expect("published carrier text exists");
    assert!(
        carrier.contains("context:\n        required: true\n        source: protocol\n"),
        "published persistent requirement declares protocol as context source"
    );

    let unmodified = ReferenceRustCandidate
        .execute(&request)
        .expect("valid published request executes");
    assert_eq!(unmodified, expected);
    let CandidateState::Observation(observation) = unmodified else {
        panic!("published request is implemented");
    };
    assert_eq!(observation.kind, ObservationKind::Result);
    assert_eq!(
        transport_record_field(&observation.value, "context_source_id"),
        &TransportValue::String("protocol".to_owned())
    );
    assert_eq!(
        transport_record_field(&observation.value, "resolved_context_known"),
        &TransportValue::Boolean(false)
    );
    assert_eq!(
        transport_record_field(&observation.value, "binding_status"),
        &TransportValue::String("UNDETERMINED".to_owned())
    );

    request.vector_id = "issue66.evidence-context-source-mismatch".to_owned();
    let invocation = request
        .fixture
        .get_mut("invocation")
        .expect("published invocation exists");
    let resolution = json_record_field_mut(invocation, "consumer_resolution");
    let context = json_record_field_mut(resolution, "context");
    let source = json_record_field_mut(context, "source_id");
    assert_eq!(source["value"], "protocol");
    source["value"] = Value::String("governance".to_owned());
    assert_ne!(source["value"], "protocol");
    assert!(
        ReferenceRustCandidate.execute(&request).is_err(),
        "consumer context source disagreement must be a harness error"
    );
    eprintln!("ISSUE66_EVIDENCE_CONTEXT_SOURCE_MISMATCH=HARNESS_ERROR");
    eprintln!("ISSUE66_REQUIRED_UNKNOWN_CONTEXT_CONTROL=PASS");
}

#[test]
fn issue66_rust_port_matches_published_vectors() {
    let corpus = load_corpus(&root()).expect("published corpus loads");
    let candidate = ReferenceRustCandidate;
    let mut matches = 0;
    let mut mismatches = Vec::new();
    let mut vectors = 0;
    let mut exact_evidence_binding_vectors = 0;
    let mut exact_evidence_binding_matches = 0;
    let mut evidence_requirements_vectors = 0;
    let mut evidence_requirements_matches = 0;
    let mut governance_bindings_vectors = 0;
    let mut governance_bindings_matches = 0;
    let mut repository_integrity_vectors = 0;
    let mut repository_integrity_matches = 0;
    let mut new_s1_gap_matches = 0;
    let mut new_s3_gap_matches = 0;
    let mut new_s4_gap_matches = 0;
    for matrix in &corpus.matrices {
        if !ISSUE66.contains(&matrix.responsibility_id.as_str()) {
            continue;
        }
        for vector in &matrix.vectors {
            vectors += 1;
            if matrix.responsibility_id == "exact-evidence-binding.evaluate" {
                exact_evidence_binding_vectors += 1;
            }
            if matrix.responsibility_id == "evidence-requirements.registry" {
                evidence_requirements_vectors += 1;
            }
            if matrix.responsibility_id == "governance-bindings.registry" {
                governance_bindings_vectors += 1;
            }
            if matrix.responsibility_id == "repository-integrity.profile" {
                repository_integrity_vectors += 1;
            }
            let request = CandidateRequest {
                responsibility_id: matrix.responsibility_id.clone(),
                vector_id: vector.vector_id.clone(),
                fixture: vector.fixture.clone(),
            };
            let actual = candidate.execute(&request).expect("candidate executes");
            let expected = CandidateState::Observation(vector.expected_observation.observation());
            if actual == expected {
                matches += 1;
                if matrix.responsibility_id == "exact-evidence-binding.evaluate" {
                    exact_evidence_binding_matches += 1;
                }
                if matrix.responsibility_id == "evidence-requirements.registry" {
                    evidence_requirements_matches += 1;
                }
                if matrix.responsibility_id == "governance-bindings.registry" {
                    governance_bindings_matches += 1;
                }
                if matrix.responsibility_id == "repository-integrity.profile" {
                    repository_integrity_matches += 1;
                }
                if vector.vector_id == "governance-bindings.registry.foreign-catalog-repository" {
                    new_s1_gap_matches += 1;
                }
                if vector.vector_id == "repository-integrity.profile.foreign-catalog-repository" {
                    new_s3_gap_matches += 1;
                }
                if vector.vector_id == "evidence-requirements.registry.foreign-catalog-repository" {
                    new_s4_gap_matches += 1;
                }
            } else {
                mismatches.push(format!(
                    "{}\n  expected={expected:?}\n  actual={actual:?}",
                    vector.vector_id
                ));
            }
        }
    }
    for mismatch in &mismatches {
        eprintln!("ISSUE66_RUST_MISMATCH: {mismatch}");
    }
    eprintln!(
        "ISSUE66_RUST_VECTORS={vectors}\nISSUE66_RUST_MATCH={matches}\nISSUE66_RUST_MISMATCH={}\nGOVERNANCE_BINDINGS_RUST_MATCH={governance_bindings_matches}/{governance_bindings_vectors}\nREPOSITORY_INTEGRITY_RUST_MATCH={repository_integrity_matches}/{repository_integrity_vectors}\nEVIDENCE_REQUIREMENTS_RUST_MATCH={evidence_requirements_matches}/{evidence_requirements_vectors}\nEXACT_EVIDENCE_BINDING_RUST_MATCH={exact_evidence_binding_matches}/{exact_evidence_binding_vectors}\nNEW_S1_GAP_RUST_MATCH={new_s1_gap_matches}/1\nNEW_S3_GAP_RUST_MATCH={new_s3_gap_matches}/1\nNEW_S4_GAP_RUST_MATCH={new_s4_gap_matches}/1",
        mismatches.len()
    );
    assert_eq!(
        vectors, 555,
        "mechanically selected #66 vector count changed"
    );
    assert_eq!(
        exact_evidence_binding_vectors, 44,
        "Exact Evidence Binding vector count changed"
    );
    assert_eq!(
        evidence_requirements_vectors, 120,
        "Evidence Requirements vector count changed"
    );
    assert_eq!(
        governance_bindings_vectors, 109,
        "Governance Bindings vector count changed"
    );
    assert_eq!(
        repository_integrity_vectors, 173,
        "Repository Integrity vector count changed"
    );
    assert!(mismatches.is_empty());
    assert_eq!(matches, 555);
    assert_eq!(governance_bindings_matches, 109);
    assert_eq!(repository_integrity_matches, 173);
    assert_eq!(evidence_requirements_matches, 120);
    assert_eq!(exact_evidence_binding_matches, 44);
    assert_eq!(new_s1_gap_matches, 1);
    assert_eq!(new_s3_gap_matches, 1);
    assert_eq!(new_s4_gap_matches, 1);
}
