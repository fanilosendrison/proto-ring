#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Vector;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

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

fn require_match(corpus: &LoadedCorpus, id: &str) {
    let (responsibility_id, vector) = vector(corpus, id);
    let request = CandidateRequest {
        responsibility_id: responsibility_id.to_owned(),
        vector_id: id.to_owned(),
        fixture: vector.fixture.clone(),
    };
    assert_eq!(
        ReferenceRustCandidate.execute(&request).unwrap(),
        CandidateState::Observation(vector.expected_observation.observation())
    );
}

#[test]
fn absolute_declaration_and_final_containment_rejections_remain_distinct() {
    let corpus = load_corpus(&root()).unwrap();
    require_match(&corpus, "governance-routing.resolve.absolute-path");
    eprintln!("ABSOLUTE_DECLARATION_REJECTION=PASS");
    require_match(
        &corpus,
        "governance-routing.resolve.absolute-path-contained-target",
    );
    eprintln!("POST_FIX_ABSOLUTE_CONTAINED_RUST_MATCH=yes");
    eprintln!("ABSOLUTE_CONTAINED_DECLARATION_REJECTION=PASS");
    require_match(&corpus, "governance-routing.resolve.outside-final");
    eprintln!("FINAL_OUTSIDE_CONTAINMENT_REJECTION=PASS");
}
