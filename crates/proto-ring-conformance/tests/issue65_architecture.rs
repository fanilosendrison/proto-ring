#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::fixture;
use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use serde_json::json;

const ISSUE63: &[&str] = &[
    "governance-bootstrap.root",
    "governance-routing.resolve",
    "repository-governance-model.binding-compatibility",
    "repository-governance-model.load",
    "structured-data.document",
    "structured-data.frontmatter",
];
const ISSUE65: &[&str] = &["governance-authority.profile", "governed-objects.catalog"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("conformance crate must be in the workspace")
        .to_path_buf()
}

fn request(corpus: &LoadedCorpus, vector_id: &str) -> CandidateRequest {
    let matrix = corpus
        .matrices
        .iter()
        .find(|matrix| {
            matrix
                .vectors
                .iter()
                .any(|vector| vector.vector_id == vector_id)
        })
        .expect("vector matrix must exist");
    let vector = matrix
        .vectors
        .iter()
        .find(|vector| vector.vector_id == vector_id)
        .expect("vector must exist");
    CandidateRequest {
        responsibility_id: matrix.responsibility_id.clone(),
        vector_id: vector.vector_id.clone(),
        fixture: vector.fixture.clone(),
    }
}

#[test]
fn all_issue65_fixtures_materialize_without_semantic_execution() {
    let corpus = load_corpus(&root()).expect("corpus must load");
    let fixtures: Vec<_> = corpus
        .matrices
        .iter()
        .filter(|matrix| ISSUE65.contains(&matrix.responsibility_id.as_str()))
        .flat_map(|matrix| &matrix.vectors)
        .collect();
    assert_eq!(fixtures.len(), 166);
    for vector in fixtures {
        fixture::materialize(&vector.fixture)
            .unwrap_or_else(|error| panic!("{}: {error}", vector.vector_id));
    }
    eprintln!("ISSUE65_FIXTURES_MATERIALIZED=166/166\nISSUE65_FIXTURE_MATERIALIZATION_ERRORS=0");
}

#[test]
fn issue63_and_issue65_vectors_are_exact_candidate_matches() {
    let corpus = load_corpus(&root()).expect("corpus must load");
    let mut issue63_matches = 0;
    let mut issue65_matches = 0;
    for matrix in &corpus.matrices {
        if !ISSUE63.contains(&matrix.responsibility_id.as_str())
            && !ISSUE65.contains(&matrix.responsibility_id.as_str())
        {
            continue;
        }
        for vector in &matrix.vectors {
            let request = CandidateRequest {
                responsibility_id: matrix.responsibility_id.clone(),
                vector_id: vector.vector_id.clone(),
                fixture: vector.fixture.clone(),
            };
            assert_eq!(
                ReferenceRustCandidate.execute(&request).unwrap(),
                CandidateState::Observation(vector.expected_observation.observation()),
                "{}",
                vector.vector_id
            );
            if ISSUE63.contains(&matrix.responsibility_id.as_str()) {
                issue63_matches += 1;
            } else {
                issue65_matches += 1;
            }
        }
    }
    assert_eq!(issue63_matches, 76);
    assert_eq!(issue65_matches, 166);
    eprintln!("ISSUE63_RUST_MATCH=76\nISSUE63_RUST_MISMATCH=0");
    eprintln!("ISSUE65_RUST_MATCH=166\nISSUE65_RUST_MISMATCH=0");
}

#[test]
fn loaded_state_projections_match_the_published_observations() {
    let corpus = load_corpus(&root()).expect("corpus must load");
    for vector_id in [
        "governance-authority.profile.roles",
        "governance-authority.profile.coauthorities",
        "governance-authority.profile.undeclared-source",
        "governance-authority.profile.unknown-responsibility",
        "governance-authority.profile.source-binding-preserved",
        "governed-objects.catalog.valid-model",
        "governed-objects.catalog.case-distinct",
        "governed-objects.catalog.empty-interfaces",
        "governed-objects.catalog.responsibility-order-r-r2",
        "governed-objects.catalog.relation-order-beta-alpha",
    ] {
        let request = request(&corpus, vector_id);
        let expected = corpus
            .matrices
            .iter()
            .flat_map(|matrix| &matrix.vectors)
            .find(|vector| vector.vector_id == vector_id)
            .expect("vector must exist")
            .expected_observation
            .observation();
        assert_eq!(
            ReferenceRustCandidate
                .execute(&request)
                .expect("candidate execution must succeed"),
            CandidateState::Observation(expected)
        );
    }
    eprintln!("AUTHORITY_RESULT_ORDER=PASS\nAUTHORITY_LOOKUP_TRANSPORT=PASS");
    eprintln!("GOVERNED_OBJECTS_RESULT_ORDER=PASS\nGOVERNED_OBJECTS_CASE_DISTINCT_PROJECTION=PASS");
    eprintln!("LOADED_SOURCE_BINDING_PRESERVED=PASS");
    eprintln!("LOADED_OBJECT_RESPONSIBILITIES_PRESERVED=PASS");
    eprintln!("LOADED_OBJECT_RELATIONS_PRESERVED=PASS");
}

#[test]
fn fixture_failure_cannot_become_controlled_rejection() {
    let request = CandidateRequest {
        responsibility_id: "governance-authority.profile".to_owned(),
        vector_id: "synthetic.fixture-failure".to_owned(),
        fixture: json!({
            "kind": "repository_plan",
            "steps": [{"op": "write_utf8", "path": "authority.md"}],
            "invocation": {"type": "record", "value": []}
        }),
    };
    let error = ReferenceRustCandidate
        .execute(&request)
        .expect_err("fixture failure must remain a harness error");
    assert!(error.to_string().contains("fixture failed"));
}

#[test]
fn issue65_source_has_no_vector_branches_or_identity_transformations() {
    let repository = root();
    let engine_root = repository.join("crates/proto-ring-engine/src");
    let candidate_root = repository.join("crates/proto-ring-conformance/src");
    let authority = fs::read_to_string(engine_root.join("governance_authority.rs")).unwrap();
    let objects = fs::read_to_string(engine_root.join("governed_objects.rs")).unwrap();
    let candidate = [
        "rust_candidate.rs",
        "rust_candidate/authority_objects.rs",
        "rust_candidate/authority_objects/support.rs",
        "rust_candidate/foundation.rs",
        "rust_candidate/transport.rs",
    ]
    .iter()
    .map(|path| fs::read_to_string(candidate_root.join(path)).unwrap())
    .collect::<String>();
    let corpus = load_corpus(&repository).expect("corpus must load");
    for vector in corpus
        .matrices
        .iter()
        .filter(|matrix| ISSUE65.contains(&matrix.responsibility_id.as_str()))
        .flat_map(|matrix| &matrix.vectors)
    {
        assert!(!candidate.contains(&vector.vector_id));
        assert!(!authority.contains(&vector.vector_id));
        assert!(!objects.contains(&vector.vector_id));
    }
    for forbidden in [
        ".trim(",
        "to_lowercase",
        "to_uppercase",
        ".split(",
        "split_once",
        "strip_suffix",
        "unicode_normalization",
        "Regex",
    ] {
        assert!(
            !authority.contains(forbidden),
            "authority contains {forbidden}"
        );
        assert!(!objects.contains(forbidden), "objects contain {forbidden}");
        assert!(
            !candidate.contains(forbidden),
            "candidate contains {forbidden}"
        );
    }
    eprintln!("OPAQUE_IDENTITY_IMPLEMENTATION_AUDIT=PASS");
    eprintln!("OPAQUE_IDENTITY_TRANSFORMATION_SITES=0\nRUST_VECTOR_ID_SPECIAL_CASES=0");
}

#[test]
fn production_routing_and_support_rehydration_remain_separate() {
    let repository = root();
    let production =
        fs::read_to_string(repository.join("crates/proto-ring-engine/src/governance_authority.rs"))
            .unwrap();
    let support = fs::read_to_string(
        repository
            .join("crates/proto-ring-conformance/src/rust_candidate/authority_objects/support.rs"),
    )
    .unwrap();
    assert!(production.contains("governance_routing::resolve("));
    for alternate in [
        ".join(",
        ".starts_with(",
        ".strip_prefix(",
        "read_link",
        "symlink_metadata",
    ] {
        assert!(
            !production.contains(alternate),
            "alternate routing: {alternate}"
        );
    }
    for forbidden in [
        "governance_routing::resolve",
        ".canonicalize(",
        "fs::metadata",
        "fs::canonicalize",
    ] {
        assert!(!support.contains(forbidden), "support contains {forbidden}");
    }
    eprintln!("PRODUCTION_GA_REUSES_CANONICAL_ROUTING=PASS");
    eprintln!("GA_REUSES_CANONICAL_ROUTING=PASS");
    eprintln!("PRODUCTION_GA_GOVERNANCE_ROUTING_CALLS>=1");
    eprintln!("PRODUCTION_GA_ALTERNATE_ROUTING_IMPLEMENTATION=0");
    eprintln!("GA_ALTERNATE_ROUTING_IMPLEMENTATION=0");
    eprintln!("SUPPORT_AUTHORITY_GOVERNANCE_ROUTING_CALLS=0");
    eprintln!("SUPPORT_AUTHORITY_REHYDRATES_WITHOUT_ROUTING=PASS");
}
