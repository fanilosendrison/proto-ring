#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use proto_ring_conformance::compare::{ComparisonOutcome, compare_observation};
use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

const WITNESSES: &[&str] = &[
    "repository-governance-model.load.projection-integrity-requires-repository-integrity",
    "repository-governance-model.load.projection-integrity-with-repository-integrity",
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("conformance crate must be in the workspace crates directory")
        .to_path_buf()
}

#[test]
fn projection_integrity_dependency_has_two_current_rust_witnesses() {
    let corpus = load_corpus(&repository_root()).expect("published corpus must load");
    let matrix = corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == "repository-governance-model.load")
        .expect("Repository Governance Model matrix must exist");
    let candidate = ReferenceRustCandidate;

    for vector_id in WITNESSES {
        let vector = matrix
            .vectors
            .iter()
            .find(|vector| vector.vector_id == *vector_id)
            .expect("witness vector must exist");
        let request = CandidateRequest {
            responsibility_id: matrix.responsibility_id.clone(),
            vector_id: vector.vector_id.clone(),
            fixture: vector.fixture.clone(),
        };
        let CandidateState::Observation(actual) = candidate
            .execute(&request)
            .expect("Rust witness execution must succeed")
        else {
            panic!("RGM responsibility must be implemented")
        };
        assert_eq!(
            compare_observation(
                &actual,
                &vector.expected_observation.observation(),
                &vector.comparison,
            ),
            ComparisonOutcome::Match
        );
    }

    eprintln!("RGM_WITNESS_RUST_MATCH=2/2");
}
