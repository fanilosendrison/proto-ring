use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{
    CandidateExecutor, CandidateRequest, CandidateState, ComparisonStatus, DifferentialReport,
};
use proto_ring_conformance::loader::LoadedCorpus;
use proto_ring_conformance::model::Observation;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

pub(super) const IMPLEMENTED: &[&str] = &[
    "accepted-adr-body.immutability",
    "adr-metadata.parse",
    "adr-metadata.repository-path",
    "adr-metadata.validation",
    "canonical-adr.resolve",
    "evidence-requirements.registry",
    "exact-evidence-binding.evaluate",
    "governance-authority.profile",
    "governance-bindings.registry",
    "governance-bootstrap.root",
    "governance-routing.resolve",
    "governed-objects.catalog",
    "normative-terminology.definition-discovery",
    "normative-terminology.fingerprint",
    "normative-terminology.inventory",
    "normative-terminology.markdown",
    "normative-terminology.registry",
    "portable-pattern.full-match",
    "projection-registry.registry",
    "repository-governance-model.binding-compatibility",
    "repository-governance-model.load",
    "repository-governance-state.compose",
    "repository-integrity.evaluate",
    "repository-integrity.profile",
    "repository-state.capture",
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
        "rust_candidate/accepted_adr_body.rs",
        "rust_candidate/adr_metadata.rs",
        "rust_candidate/authority_objects.rs",
        "rust_candidate/canonical_adr.rs",
        "rust_candidate/support.rs",
        "rust_candidate/evidence_requirements.rs",
        "rust_candidate/exact_evidence_binding.rs",
        "rust_candidate/foundation.rs",
        "rust_candidate/governance_bindings.rs",
        "rust_candidate/normative_terminology.rs",
        "rust_candidate/portable_pattern.rs",
        "rust_candidate/projection_registry.rs",
        "rust_candidate/repository_governance_state.rs",
        "rust_candidate/repository_governance_state/observe.rs",
        "rust_candidate/repository_governance_state/plan.rs",
        "rust_candidate/repository_integrity.rs",
        "rust_candidate/repository_integrity_evaluation.rs",
        "rust_candidate/repository_state.rs",
        "rust_candidate/repository_state/observe.rs",
        "rust_candidate/repository_state/plan.rs",
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
        "accepted_adr_body/mod.rs",
        "accepted_adr_body/git.rs",
        "accepted_adr_body/history.rs",
        "adr_metadata/mod.rs",
        "adr_metadata/exact_text.rs",
        "adr_metadata/repository_path.rs",
        "adr_metadata/validation.rs",
        "canonical_adr/mod.rs",
        "structured_data.rs",
        "structured_data_yaml.rs",
        "governance_authority.rs",
        "governance_bootstrap.rs",
        "governance_routing.rs",
        "governed_objects.rs",
        "normative_terminology/mod.rs",
        "normative_terminology/unicode14.rs",
        "normative_terminology/registry.rs",
        "normative_terminology/markdown.rs",
        "normative_terminology/matching.rs",
        "normative_terminology/inventory.rs",
        "portable_pattern/mod.rs",
        "portable_pattern/model.rs",
        "portable_pattern/parser.rs",
        "portable_pattern/matcher.rs",
        "repository_governance_model.rs",
        "repository_governance_state/mod.rs",
        "repository_governance_state/compose.rs",
        "repository_governance_state/scope.rs",
        "repository_governance_state/orchestrate.rs",
        "repository_integrity/mod.rs",
        "repository_integrity/model.rs",
        "repository_integrity/selectors.rs",
        "repository_integrity/evaluate.rs",
        "repository_integrity/execution.rs",
        "repository_state/mod.rs",
        "repository_state/git.rs",
        "repository_state/path.rs",
        "repository_state/scope.rs",
        "repository_state/capture.rs",
    ];
    let engine = engine_paths
        .iter()
        .map(|path| {
            fs::read_to_string(repository.join("crates/proto-ring-engine/src").join(path)).unwrap()
        })
        .collect::<String>();
    let implementation = format!("{candidate}{engine}");
    for matrix in &corpus.matrices {
        if IMPLEMENTED.contains(&matrix.responsibility_id.as_str()) {
            for vector in &matrix.vectors {
                assert!(!implementation.contains(&vector.vector_id));
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

pub(super) fn count_status<'a>(
    reports: impl IntoIterator<Item = &'a DifferentialReport>,
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

pub(super) fn count_rust_status_for_responsibilities(
    reports: &[DifferentialReport],
    responsibilities: &HashSet<&str>,
) -> (usize, usize, usize) {
    count_status(
        reports
            .iter()
            .filter(|report| responsibilities.contains(report.responsibility_id.as_str())),
        |report| report.rust_vs_expected,
    )
}

pub(super) fn print_issue64_accounting(vector_count: usize, counts: (usize, usize, usize)) {
    eprintln!(
        "ISSUE64_VECTORS={vector_count}\nISSUE64_RUST_MATCH={}\nISSUE64_RUST_MISMATCH={}\nISSUE64_RUST_NOT_AVAILABLE={}",
        counts.0, counts.1, counts.2
    );
}

pub(super) fn print_rust_accounting(
    responsibility_count: usize,
    implemented_vectors: usize,
    rust: (usize, usize, usize),
    differential: (usize, usize, usize),
) {
    eprintln!(
        "RUST_IMPLEMENTED_RESPONSIBILITIES={responsibility_count}\nRUST_IMPLEMENTED_VECTORS={implemented_vectors}\nRUST_UNIMPLEMENTED_RESPONSIBILITIES=2\nRUST_UNIMPLEMENTED_VECTORS={}",
        1153 - implemented_vectors
    );
    eprintln!(
        "RUST_VS_EXPECTED_MATCH={}\nRUST_VS_EXPECTED_MISMATCH={}\nRUST_VS_EXPECTED_NOT_AVAILABLE={}",
        rust.0, rust.1, rust.2
    );
    eprintln!(
        "PYTHON_VS_RUST_MATCH={}\nPYTHON_VS_RUST_MISMATCH={}\nPYTHON_VS_RUST_NOT_AVAILABLE={}",
        differential.0, differential.1, differential.2
    );
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
