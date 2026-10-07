#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use proto_ring_conformance::harness::{
    CandidateExecutor, CandidateRequest, CandidateState, ComparisonStatus, PythonBridge,
    candidate_requests, implemented_responsibilities, run_differential,
    serialize_candidate_requests,
};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::{MigrationDisposition, Observation};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use serde_json::{Value, json};
use tempfile::NamedTempFile;
const IMPLEMENTED: &[&str] = &[
    "governance-authority.profile",
    "governance-bootstrap.root",
    "governance-routing.resolve",
    "governed-objects.catalog",
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

fn invoke_bridge(request: &[u8]) -> (Output, Vec<u8>) {
    let request_file = NamedTempFile::new().unwrap();
    fs::write(request_file.path(), request).unwrap();
    let output_file = NamedTempFile::new().unwrap();
    let output =
        Command::new(std::env::var_os("PROTO_RING_PYTHON").unwrap_or_else(|| "python3".into()))
            .current_dir(root())
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .args(["-B", "tests/conformance_python_bridge.py", "--request"])
            .arg(request_file.path())
            .arg("--output")
            .arg(output_file.path())
            .output()
            .unwrap();
    (output, fs::read(output_file.path()).unwrap())
}

fn json_bytes(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn assert_bridge_isolation(serialized: &[u8]) {
    let source = fs::read_to_string(root().join("tests/conformance_python_bridge.py")).unwrap();
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
    let document: Value = serde_json::from_slice(serialized).unwrap();
    let requests = document.as_array().unwrap();
    assert_eq!(requests.len(), 596);
    let expected = BTreeSet::from(["fixture", "responsibility_id", "vector_id"]);
    let forbidden = [
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
    for request in requests {
        let object = request.as_object().unwrap();
        assert_eq!(
            object.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            expected
        );
        assert!(object.keys().all(|key| !forbidden.contains(&key.as_str())));
    }
}

#[test]
fn bridge_rejects_oracle_malformed_duplicate_and_retirement_requests() {
    let probes = [
        json!([{"responsibility_id":"structured-data.document","vector_id":"probe","fixture":{},"expected_observation":{}}]),
        json!([{"responsibility_id":"structured-data.document","vector_id":"probe"}]),
        json!([{"vector_id":"probe","fixture":{}}]),
        json!({}),
        json!([{"responsibility_id":"structured-data.document","vector_id":"probe","fixture":{}},
               {"responsibility_id":"structured-data.document","vector_id":"probe","fixture":{}}]),
        json!([{"responsibility_id":"shared-governance-provider.check","vector_id":"probe","fixture":{}}]),
    ];
    for probe in probes {
        assert!(!invoke_bridge(&json_bytes(&probe)).0.status.success());
    }
}

fn vector<'a>(
    corpus: &'a LoadedCorpus,
    id: &str,
) -> (&'a str, &'a proto_ring_conformance::model::Vector) {
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

fn request_for(corpus: &LoadedCorpus, id: &str) -> CandidateRequest {
    let (responsibility_id, vector) = vector(corpus, id);
    CandidateRequest {
        responsibility_id: responsibility_id.to_owned(),
        vector_id: id.to_owned(),
        fixture: vector.fixture.clone(),
    }
}

fn expected_observation(corpus: &LoadedCorpus, id: &str) -> Observation {
    vector(corpus, id).1.expected_observation.observation()
}

fn assert_projection_regressions(corpus: &LoadedCorpus) {
    let candidate = ReferenceRustCandidate;
    for id in [
        "structured-data.frontmatter.valid-mapping",
        "structured-data.frontmatter.immediate-close",
        "repository-governance-model.load.valid-model-v1",
        "repository-governance-model.load.valid-model-v2",
    ] {
        let state = candidate.execute(&request_for(corpus, id)).unwrap();
        assert_eq!(
            state,
            CandidateState::Observation(expected_observation(corpus, id))
        );
    }
}

fn assert_source_boundaries(corpus: &LoadedCorpus) {
    let repository = root();
    let candidate = [
        "rust_candidate.rs",
        "rust_candidate/authority_objects.rs",
        "rust_candidate/authority_objects/support.rs",
        "rust_candidate/foundation.rs",
        "rust_candidate/transport.rs",
    ]
    .iter()
    .map(|path| {
        let source = repository.join("crates/proto-ring-conformance/src");
        fs::read_to_string(source.join(path)).unwrap()
    })
    .collect::<String>();
    let engine_paths = [
        "lib.rs",
        "structured_data.rs",
        "structured_data_yaml.rs",
        "governance_authority.rs",
        "governance_bootstrap.rs",
        "governance_routing.rs",
        "governed_objects.rs",
        "repository_governance_model.rs",
    ];
    let engine = engine_paths
        .iter()
        .map(|path| {
            fs::read_to_string(repository.join("crates/proto-ring-engine/src").join(path)).unwrap()
        })
        .collect::<String>();
    for matrix in &corpus.matrices {
        if IMPLEMENTED.contains(&matrix.responsibility_id.as_str()) {
            for vector in &matrix.vectors {
                assert!(!candidate.contains(&vector.vector_id));
            }
        }
    }
    for forbidden in ["YamlLoader", "Yaml::Integer", "Yaml::Boolean", "Yaml::Null"] {
        assert!(!engine.contains(forbidden));
    }
    assert!(engine.contains("parse_frontmatter_bytes"));
    assert!(engine.contains("governance_bootstrap::load"));
    assert!(engine.contains("governance_routing::resolve"));
}

fn count_status(
    reports: &[proto_ring_conformance::harness::DifferentialReport],
    channel: fn(&proto_ring_conformance::harness::DifferentialReport) -> ComparisonStatus,
) -> (usize, usize, usize) {
    let mut counts = (0, 0, 0);
    for report in reports {
        match channel(report) {
            ComparisonStatus::Match => counts.0 += 1,
            ComparisonStatus::Mismatch => counts.1 += 1,
            ComparisonStatus::NotAvailable => counts.2 += 1,
        }
    }
    counts
}

fn require_matches(reports: &[proto_ring_conformance::harness::DifferentialReport], ids: &[&str]) {
    for id in ids {
        let report = reports
            .iter()
            .find(|report| report.vector_id == *id)
            .unwrap();
        assert_eq!(report.rust_vs_expected, ComparisonStatus::Match);
    }
}

#[test]
fn absolute_contained_target_is_rejected_by_reference_candidate() {
    let corpus = load_corpus(&root()).unwrap();
    let id = "governance-routing.resolve.absolute-path-contained-target";
    let state = ReferenceRustCandidate
        .execute(&request_for(&corpus, id))
        .unwrap();
    assert_eq!(
        state,
        CandidateState::Observation(expected_observation(&corpus, id))
    );
}

#[test]
fn published_corpus_runs_through_reference_candidate() {
    let corpus = load_corpus(&root()).unwrap();
    let rust_port = corpus
        .responsibilities
        .iter()
        .filter(|record| record.migration_disposition == MigrationDisposition::RustPort)
        .count();
    let retired: Vec<_> = corpus
        .responsibilities
        .iter()
        .filter(|record| {
            record.migration_disposition == MigrationDisposition::RetireWithoutRustPort
        })
        .collect();
    let vectors: usize = corpus
        .matrices
        .iter()
        .map(|matrix| matrix.vectors.len())
        .sum();
    assert_eq!(
        (
            corpus.responsibilities.len(),
            rust_port,
            retired.len(),
            corpus.matrices.len(),
            vectors
        ),
        (30, 29, 1, 29, 596)
    );
    assert_eq!(
        retired[0].responsibility_id,
        "shared-governance-provider.check"
    );

    let requests = candidate_requests(&corpus);
    let serialized = serialize_candidate_requests(&requests).unwrap();
    assert_bridge_isolation(&serialized);
    assert_projection_regressions(&corpus);
    assert_source_boundaries(&corpus);
    let python = PythonBridge::new(&root()).execute(&requests).unwrap();
    assert_eq!(python.len(), 596);
    let reports = run_differential(&corpus, &python, &ReferenceRustCandidate).unwrap();

    let python_counts = count_status(&reports, |report| report.python_vs_expected);
    let rust_counts = count_status(&reports, |report| report.rust_vs_expected);
    let differential_counts = count_status(&reports, |report| report.python_vs_rust);
    for report in &reports {
        if report.rust_vs_expected == ComparisonStatus::Mismatch {
            eprintln!(
                "RUST_MISMATCH={} actual={:?} expected={:?}",
                report.vector_id, report.rust_candidate, report.expected_observation
            );
        }
    }
    assert_eq!(python_counts, (596, 0, 0));
    assert_eq!(rust_counts, (239, 0, 357));
    assert_eq!(differential_counts, (239, 0, 357));
    let implemented_vectors = reports
        .iter()
        .filter(|report| matches!(report.rust_candidate, CandidateState::Observation(_)))
        .count();
    let implemented_responsibility_count = implemented_responsibilities(&reports);
    let implemented_ids: HashSet<_> = reports
        .iter()
        .filter_map(|report| {
            matches!(report.rust_candidate, CandidateState::Observation(_))
                .then_some(report.responsibility_id.as_str())
        })
        .collect();
    assert_eq!(implemented_ids, IMPLEMENTED.iter().copied().collect());
    assert_eq!(
        (implemented_responsibility_count, implemented_vectors),
        (8, 239)
    );

    require_matches(
        &reports,
        &[
            "structured-data.document.beyond-i64",
            "structured-data.document.int-limit-641",
            "structured-data.document.int-limit-4301",
        ],
    );
    require_matches(
        &reports,
        &[
            "governance-routing.resolve.missing-parent",
            "governance-routing.resolve.file-parent",
            "governance-routing.resolve.file-trailing-slash",
            "governance-routing.resolve.file-dot",
            "governance-routing.resolve.broken-symlink",
            "governance-routing.resolve.symlink-cycle",
            "governance-routing.resolve.symlink-parent",
            "governance-routing.resolve.outside-final",
            "governance-routing.resolve.absolute-path-contained-target",
        ],
    );
    require_matches(
        &reports,
        &[
            "repository-governance-model.load.valid-model-v1",
            "repository-governance-model.load.valid-model-v2",
            "repository-governance-model.load.unsupported-version",
            "repository-governance-model.load.unknown-root-key",
            "repository-governance-model.load.malformed-capability",
        ],
    );
    require_matches(
        &reports,
        &[
            "repository-governance-model.binding-compatibility.compatible-binding",
            "repository-governance-model.binding-compatibility.missing-required-route",
            "repository-governance-model.binding-compatibility.undeclared-capability",
        ],
    );

    let metrics: HashMap<&str, usize> = HashMap::from([
        ("implemented", implemented_vectors),
        ("unimplemented", 596 - implemented_vectors),
    ]);
    eprintln!(
        "PYTHON_BRIDGE_LOADS_CORPUS=no\nPYTHON_BRIDGE_ORACLE_FIELDS_IN_REQUEST=0\nPYTHON_BRIDGE_COMPUTES_COMPARISON=no\nPYTHON_BRIDGE_REQUEST_SCHEMA=PASS\nPYTHON_BRIDGE_OUTPUT_SCHEMA=PASS\nPYTHON_BRIDGE_RETIREMENT_ONLY_REJECTED=PASS"
    );
    eprintln!(
        "CORPUS_RESPONSIBILITIES=30\nRUST_PORT_RESPONSIBILITIES=29\nRETIRE_WITHOUT_RUST_PORT_RESPONSIBILITIES=1\nCORPUS_MATRICES=29\nCORPUS_VECTORS=596"
    );
    eprintln!("ISSUE63_RESPONSIBILITIES=6\nISSUE63_VECTORS=74");
    eprintln!("ISSUE65_RESPONSIBILITIES=2\nISSUE65_VECTORS=165");
    eprintln!(
        "PYTHON_OBSERVATIONS=596\nPYTHON_VS_EXPECTED_MATCH={}\nPYTHON_VS_EXPECTED_MISMATCH={}",
        python_counts.0, python_counts.1
    );
    eprintln!(
        "RUST_IMPLEMENTED_RESPONSIBILITIES={implemented_responsibility_count}\nRUST_IMPLEMENTED_VECTORS={}\nRUST_UNIMPLEMENTED_RESPONSIBILITIES=21\nRUST_UNIMPLEMENTED_VECTORS={}",
        metrics["implemented"], metrics["unimplemented"]
    );
    eprintln!(
        "RUST_VS_EXPECTED_MATCH={}\nRUST_VS_EXPECTED_MISMATCH={}\nRUST_VS_EXPECTED_NOT_AVAILABLE={}",
        rust_counts.0, rust_counts.1, rust_counts.2
    );
    eprintln!(
        "PYTHON_VS_RUST_MATCH={}\nPYTHON_VS_RUST_MISMATCH={}\nPYTHON_VS_RUST_NOT_AVAILABLE={}",
        differential_counts.0, differential_counts.1, differential_counts.2
    );
    eprintln!(
        "RETIREMENT_ONLY_EXECUTIONS=0\nRUST_VECTOR_ID_SPECIAL_CASES=0\nYAML_IMPLICIT_TYPING_DEPENDENCY=0"
    );
    eprintln!(
        "UNBOUNDED_INTEGER_CASES=3/3\nROUTING_HOSTILE_PATH_CASES=9/9\nRGM_VERSION_CASES=5/5\nRGM_BINDING_COMPATIBILITY_CASES=3/3"
    );
    eprintln!(
        "FRONTMATTER_RESULT_ORDER=PASS\nIMMEDIATE_CLOSE_RESULT_ORDER=PASS\nRGM_RESULT_ORDER=PASS\nRGM_MODEL_VERSION_TRANSPORT=PASS"
    );
    eprintln!("DIFFERENTIAL_HARNESS=PASS");
}
