#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use proto_ring_conformance::fixture::{self, MaterializedFixture};
use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::{TransportValue, Vector};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use serde_json::json;

const IMPLEMENTED: &[&str] = &[
    "governance-bootstrap.root",
    "governance-routing.resolve",
    "repository-governance-model.binding-compatibility",
    "repository-governance-model.load",
    "structured-data.document",
    "structured-data.frontmatter",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}

fn vector<'a>(corpus: &'a LoadedCorpus, id: &str) -> (&'a str, &'a Vector) {
    corpus
        .matrices
        .iter()
        .find_map(|matrix| {
            matrix
                .vectors
                .iter()
                .find(|vector| vector.vector_id == id)
                .map(|vector| (matrix.responsibility_id.as_str(), vector))
        })
        .unwrap()
}

fn field<'a>(value: &'a TransportValue, name: &str) -> &'a TransportValue {
    fixture::record_field(value, name).unwrap()
}

fn declared_path(realized: &MaterializedFixture) -> PathBuf {
    let routing = field(realized.arguments(), "routing");
    let governance = field(routing, "governance");
    let responsibility = field(governance, "responsibility");
    PathBuf::from(fixture::string(field(responsibility, "path")).unwrap())
}

#[test]
fn fixture_failure_escapes_as_harness_error() {
    let request = CandidateRequest {
        responsibility_id: "structured-data.document".to_owned(),
        vector_id: "fixture-boundary.invalid-mkdir".to_owned(),
        fixture: json!({
            "kind": "repository_plan",
            "environment": {},
            "faults": [],
            "steps": [{"op": "mkdir"}],
            "invocation": {
                "type": "record",
                "value": [{
                    "name": "operation",
                    "value": {"type": "string", "value": "parse_document"}
                }]
            }
        }),
    };
    let error = match ReferenceRustCandidate.execute(&request) {
        Err(error) => error,
        Ok(CandidateState::Observation(observation)) => {
            panic!("fixture failure became semantic observation: {observation:?}")
        }
        Ok(CandidateState::Unimplemented) => panic!("implemented responsibility was unavailable"),
    };
    let message = error.to_string();
    assert!(message.contains(&request.vector_id));
    assert!(message.contains("missing fixture property: path"));
    eprintln!("FIXTURE_FAILURE_ESCAPES_AS_HARNESS_ERROR=PASS");
    eprintln!("FIXTURE_FAILURE_AS_SEMANTIC_OBSERVATION=0");
}

#[test]
fn absolute_contained_fixture_materializes_with_bound_capture() {
    let corpus = load_corpus(&root()).unwrap();
    let (_, vector) = vector(
        &corpus,
        "governance-routing.resolve.absolute-path-contained-target",
    );
    let realized = fixture::materialize(&vector.fixture).unwrap();
    let repository = realized.root().unwrap().canonicalize().unwrap();
    let declared = declared_path(&realized);
    let expected = repository.join("config/target.yaml");

    assert!(declared.is_absolute());
    assert!(declared.exists());
    assert_eq!(
        declared.canonicalize().unwrap(),
        expected.canonicalize().unwrap()
    );
    assert!(declared.canonicalize().unwrap().starts_with(&repository));
    eprintln!("ABSOLUTE_CONTAINED_FIXTURE_MATERIALIZED=PASS");
    eprintln!("ABSOLUTE_CONTAINED_CAPTURE_BOUND=PASS");
    eprintln!("ABSOLUTE_CONTAINED_DECLARED_PATH_IS_ABSOLUTE=PASS");
    eprintln!("ABSOLUTE_CONTAINED_TARGET_EXISTS=PASS");
    eprintln!("ABSOLUTE_CONTAINED_FINAL_TARGET_INSIDE_REPOSITORY=PASS");
}

#[test]
fn every_issue_63_fixture_materializes() {
    let corpus = load_corpus(&root()).unwrap();
    let mut materialized = 0;
    let mut failures = Vec::new();
    for matrix in &corpus.matrices {
        if !IMPLEMENTED.contains(&matrix.responsibility_id.as_str()) {
            continue;
        }
        for vector in &matrix.vectors {
            match fixture::materialize(&vector.fixture) {
                Ok(_) => materialized += 1,
                Err(error) => failures.push(format!(
                    "{} {}: {}",
                    matrix.responsibility_id, vector.vector_id, error
                )),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(materialized, 74);
    eprintln!("ISSUE63_FIXTURES_MATERIALIZED={materialized}/74");
    eprintln!("ISSUE63_FIXTURE_MATERIALIZATION_ERRORS={}", failures.len());
}
