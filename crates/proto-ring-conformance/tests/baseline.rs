#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use proto_ring_conformance::harness::{
    CandidateRequest, candidate_requests, serialize_candidate_requests,
};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::MigrationDisposition;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("conformance crate must be in the workspace crates directory")
        .to_path_buf()
}

fn python_executable() -> std::ffi::OsString {
    std::env::var_os("PROTO_RING_PYTHON").unwrap_or_else(|| "python3".into())
}

fn invoke_bridge(repository_root: &Path, request_bytes: &[u8]) -> (Output, Vec<u8>) {
    let request_file = NamedTempFile::new().expect("request file must be created");
    fs::write(request_file.path(), request_bytes).expect("request file must be written");
    let output_file = NamedTempFile::new().expect("output file must be created");
    let output = Command::new(python_executable())
        .current_dir(repository_root)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg("-B")
        .arg("tests/conformance_python_bridge.py")
        .arg("--request")
        .arg(request_file.path())
        .arg("--output")
        .arg(output_file.path())
        .output()
        .expect("Python bridge must start");
    let response = fs::read(output_file.path()).expect("output file must be readable");
    (output, response)
}

fn json_bytes(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("test JSON must serialize");
    bytes.push(b'\n');
    bytes
}

fn assert_bridge_source_is_isolated(repository_root: &Path) {
    let source = fs::read_to_string(repository_root.join("tests/conformance_python_bridge.py"))
        .expect("bridge source must be readable");
    for forbidden in [
        "load_corpus",
        "expected_observation",
        "behavior_classification",
        "frozen_python_observation",
        "later_python_observation",
        "state_distinctions",
        "compare",
        "conformance/v1",
        "index.json",
    ] {
        assert!(!source.contains(forbidden), "bridge contains {forbidden}");
    }
}

fn oracle_field_count(value: &Value) -> usize {
    const ORACLE_FIELDS: &[&str] = &[
        "expected_observation",
        "comparison",
        "derivation",
        "behavior_classification",
        "frozen_python_observation",
        "later_python_observation",
        "resolution",
        "authorities",
        "state_distinctions",
    ];
    match value {
        Value::Object(object) => object
            .iter()
            .map(|(key, value)| {
                usize::from(ORACLE_FIELDS.contains(&key.as_str())) + oracle_field_count(value)
            })
            .sum(),
        Value::Array(values) => values.iter().map(oracle_field_count).sum(),
        _ => 0,
    }
}

fn assert_request_schema(serialized: &[u8], expected_count: usize) {
    let document: Value = serde_json::from_slice(serialized).expect("request must be JSON");
    let requests = document.as_array().expect("request root must be an array");
    assert_eq!(requests.len(), expected_count);
    let expected_keys = BTreeSet::from(["fixture", "responsibility_id", "vector_id"]);
    for request in requests {
        let keys = request
            .as_object()
            .expect("request must be an object")
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        assert_eq!(keys, expected_keys);
    }
    assert_eq!(oracle_field_count(&document), 0);
}

fn one_real_request(corpus: &LoadedCorpus) -> CandidateRequest {
    let matrix = corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == "structured-data.document")
        .expect("structured-data matrix must exist");
    let vector = matrix
        .vectors
        .iter()
        .find(|vector| vector.vector_id == "structured-data.document.positive-integer")
        .expect("positive integer vector must exist");
    CandidateRequest {
        responsibility_id: matrix.responsibility_id.clone(),
        vector_id: vector.vector_id.clone(),
        fixture: vector.fixture.clone(),
    }
}

fn assert_output_schema(repository_root: &Path, request: CandidateRequest) {
    let request_bytes = serialize_candidate_requests(&[request]).expect("request must serialize");
    let (output, response) = invoke_bridge(repository_root, &request_bytes);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: Value = serde_json::from_slice(&response).expect("response must be JSON");
    let records = document.as_array().expect("response root must be an array");
    assert_eq!(records.len(), 1);
    let keys = records[0]
        .as_object()
        .expect("response must contain an object")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        keys,
        BTreeSet::from(["observation", "responsibility_id", "vector_id"])
    );
}

fn assert_retirement_only_rejected(repository_root: &Path) {
    let request = json!([{
        "fixture": {},
        "responsibility_id": "shared-governance-provider.check",
        "vector_id": "retirement-only.probe"
    }]);
    let (output, _) = invoke_bridge(repository_root, &json_bytes(&request));
    assert!(!output.status.success());
}

#[test]
fn bridge_rejects_oracle_bearing_and_malformed_requests() {
    let root = repository_root();
    let requests = [
        json!([{
            "responsibility_id": "structured-data.document",
            "vector_id": "probe",
            "fixture": {},
            "expected_observation": {}
        }]),
        json!([{
            "responsibility_id": "structured-data.document",
            "vector_id": "probe"
        }]),
        json!([{"vector_id": "probe", "fixture": {}}]),
        json!({}),
        json!([
            {
                "responsibility_id": "structured-data.document",
                "vector_id": "probe",
                "fixture": {}
            },
            {
                "responsibility_id": "structured-data.document",
                "vector_id": "probe",
                "fixture": {}
            }
        ]),
    ];
    for request in requests {
        let (output, _) = invoke_bridge(&root, &json_bytes(&request));
        assert!(!output.status.success());
    }
}

#[test]
fn bridge_rejects_retirement_only_requests() {
    assert_retirement_only_rejected(&repository_root());
}

#[test]
fn published_corpus_bridge_contract_is_oracle_free() {
    let repository_root = repository_root();
    let corpus = load_corpus(&repository_root).expect("published corpus must load");

    let rust_port = corpus
        .responsibilities
        .iter()
        .filter(|record| record.migration_disposition == MigrationDisposition::RustPort)
        .count();
    let retirement_only: Vec<_> = corpus
        .responsibilities
        .iter()
        .filter(|record| {
            record.migration_disposition == MigrationDisposition::RetireWithoutRustPort
        })
        .collect();
    let vector_count: usize = corpus
        .matrices
        .iter()
        .map(|matrix| matrix.vectors.len())
        .sum();

    assert_eq!(corpus.responsibilities.len(), 30);
    assert_eq!(rust_port, 29);
    assert_eq!(retirement_only.len(), 1);
    assert_eq!(
        retirement_only[0].responsibility_id,
        "shared-governance-provider.check"
    );
    assert!(retirement_only[0].case_file.is_none());
    assert_eq!(corpus.index.case_files.len(), 29);
    assert_eq!(corpus.matrices.len(), 29);
    assert_eq!(vector_count, 1081);
    assert!(
        corpus
            .matrices
            .iter()
            .all(|matrix| matrix.responsibility_id != "shared-governance-provider.check")
    );

    let requests = candidate_requests(&corpus);
    assert_eq!(requests.len(), 1081);
    let serialized = serialize_candidate_requests(&requests).expect("batch request must serialize");
    let request_probe = NamedTempFile::new().expect("request probe must be created");
    fs::write(request_probe.path(), &serialized).expect("request probe must be written");
    let serialized_probe = fs::read(request_probe.path()).expect("request probe must be read");
    assert_request_schema(&serialized_probe, 1081);
    assert_bridge_source_is_isolated(&repository_root);
    assert_output_schema(&repository_root, one_real_request(&corpus));

    eprintln!("PYTHON_BRIDGE_LOADS_CORPUS=no");
    eprintln!("PYTHON_BRIDGE_ORACLE_FIELDS_IN_REQUEST=0");
    eprintln!("PYTHON_BRIDGE_COMPUTES_COMPARISON=no");
    eprintln!("PYTHON_BRIDGE_REQUEST_SCHEMA=PASS");
    eprintln!("PYTHON_BRIDGE_OUTPUT_SCHEMA=PASS");
    eprintln!("PYTHON_BRIDGE_RETIREMENT_ONLY_REJECTED=PASS");
    eprintln!("CORPUS_RESPONSIBILITIES=30");
    eprintln!("RUST_PORT_RESPONSIBILITIES=29");
    eprintln!("RETIRE_WITHOUT_RUST_PORT_RESPONSIBILITIES=1");
    eprintln!("CORPUS_MATRICES=29");
    eprintln!("CORPUS_VECTORS=1081");
}
