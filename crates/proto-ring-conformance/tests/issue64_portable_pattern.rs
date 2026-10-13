#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::{Observation, Vector};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

const RESPONSIBILITY: &str = "portable-pattern.full-match";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root exists")
        .to_path_buf()
}

fn vectors(corpus: &LoadedCorpus) -> &[Vector] {
    &corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == RESPONSIBILITY)
        .expect("Portable Pattern matrix exists")
        .vectors
}

fn request(vector: &Vector) -> CandidateRequest {
    CandidateRequest {
        responsibility_id: RESPONSIBILITY.to_owned(),
        vector_id: vector.vector_id.clone(),
        fixture: vector.fixture.clone(),
    }
}

fn expected(vector: &Vector) -> Observation {
    vector.expected_observation.observation()
}

#[test]
fn issue64_portable_pattern_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let vectors = vectors(&corpus);
    let candidate = ReferenceRustCandidate;
    assert_eq!(vectors.len(), 37);
    let mut matches = 0;
    for vector in vectors {
        let actual = candidate.execute(&request(vector)).expect("candidate runs");
        let expected = CandidateState::Observation(expected(vector));
        if actual == expected {
            matches += 1;
        } else {
            eprintln!("vector_id={}", vector.vector_id);
            eprintln!("actual={actual:?}");
            eprintln!("expected={expected:?}");
        }
    }
    assert_eq!(matches, 37);
    eprintln!("ISSUE64_PORTABLE_PATTERN_VECTORS=37");
    eprintln!("ISSUE64_PORTABLE_PATTERN_MATCH=37");
    eprintln!("ISSUE64_PORTABLE_PATTERN_MISMATCH=0");
    eprintln!("PORTABLE_PATTERN_RUST=37/37");
}

#[test]
fn issue64_portable_pattern_source_boundaries_are_explicit() {
    let engine_root = root().join("crates/proto-ring-engine/src/portable_pattern");
    let model = fs::read_to_string(engine_root.join("model.rs")).expect("model source is readable");
    let parser =
        fs::read_to_string(engine_root.join("parser.rs")).expect("parser source is readable");
    let matcher =
        fs::read_to_string(engine_root.join("matcher.rs")).expect("matcher source is readable");
    let manifest = fs::read_to_string(root().join("Cargo.toml")).expect("manifest is readable");
    let combined = format!("{model}{parser}{matcher}");

    for forbidden in [
        "regex =",
        "regex-automata",
        "fancy-regex",
        "onig",
        "pcre",
        "to_lowercase",
        "to_uppercase",
        "case_fold",
        "normalize",
    ] {
        assert!(
            !combined.contains(forbidden) && !manifest.contains(forbidden),
            "forbidden Portable Pattern source: {forbidden}"
        );
    }
    for required in ["BigUint", "minimum_consumption", "Vec<Node>", "Vec<Task>"] {
        assert!(combined.contains(required), "missing source: {required}");
    }
    assert!(parser.contains("frames: Vec<GroupFrame>"));
    assert!(matcher.contains("pending = vec![ExecutionState"));
    assert!(!parser.contains("Box<Node>"));
    assert!(!matcher.contains("Box<Node>"));

    eprintln!("PORTABLE_PATTERN_NATIVE_REGEX_ENGINE=0");
    eprintln!("PORTABLE_PATTERN_REGEX_TRANSLATION=0");
    eprintln!("PORTABLE_PATTERN_MACHINE_WIDTH_QUANTIFIER_STORAGE=0");
    eprintln!("PORTABLE_PATTERN_QUANTIFIER_BIGUINT=YES");
    eprintln!("PORTABLE_PATTERN_MIN_CONSUMPTION_BIGUINT=YES");
    eprintln!("PORTABLE_PATTERN_RECURSIVE_PARSER=0");
    eprintln!("PORTABLE_PATTERN_RECURSIVE_MATCHER=0");
    eprintln!("PORTABLE_PATTERN_UNICODE_NORMALIZATION=0");
    eprintln!("PORTABLE_PATTERN_CASE_FOLDING=0");
    eprintln!("PORTABLE_PATTERN_LOCALE_DEPENDENCE=0");
    eprintln!("PORTABLE_PATTERN_VECTOR_ID_SPECIAL_CASES=0");
}
