use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{
    CandidateExecutor, CandidateRequest, CandidateState, ComparisonStatus, DifferentialReport,
};
use proto_ring_conformance::loader::LoadedCorpus;
use proto_ring_conformance::model::Observation;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

pub(super) const IMPLEMENTED: &[&str] = &[
    "evidence-requirements.registry",
    "exact-evidence-binding.evaluate",
    "governance-authority.profile",
    "governance-bindings.registry",
    "governance-bootstrap.root",
    "governance-routing.resolve",
    "governed-objects.catalog",
    "projection-registry.registry",
    "repository-governance-model.binding-compatibility",
    "repository-governance-model.load",
    "repository-integrity.profile",
    "structured-data.document",
    "structured-data.frontmatter",
];

pub(super) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
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

pub(super) fn request_for(corpus: &LoadedCorpus, id: &str) -> CandidateRequest {
    let (responsibility_id, vector) = vector(corpus, id);
    CandidateRequest {
        responsibility_id: responsibility_id.to_owned(),
        vector_id: id.to_owned(),
        fixture: vector.fixture.clone(),
    }
}

pub(super) fn expected_observation(corpus: &LoadedCorpus, id: &str) -> Observation {
    vector(corpus, id).1.expected_observation.observation()
}

pub(super) fn assert_projection_regressions(corpus: &LoadedCorpus) {
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

pub(super) fn assert_source_boundaries(corpus: &LoadedCorpus) {
    let repository = root();
    let candidate = [
        "rust_candidate.rs",
        "rust_candidate/authority_objects.rs",
        "rust_candidate/support.rs",
        "rust_candidate/evidence_requirements.rs",
        "rust_candidate/exact_evidence_binding.rs",
        "rust_candidate/foundation.rs",
        "rust_candidate/governance_bindings.rs",
        "rust_candidate/projection_registry.rs",
        "rust_candidate/repository_integrity.rs",
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

pub(super) fn count_status(
    reports: &[DifferentialReport],
    channel: fn(&DifferentialReport) -> ComparisonStatus,
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

pub(super) fn require_matches(reports: &[DifferentialReport], ids: &[&str]) {
    for id in ids {
        let report = reports
            .iter()
            .find(|report| report.vector_id == *id)
            .unwrap();
        assert_eq!(report.rust_vs_expected, ComparisonStatus::Match);
    }
}
