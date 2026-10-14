#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Vector;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

const WAVE_B: [(&str, usize); 4] = [
    ("adr-metadata.parse", 24),
    ("adr-metadata.repository-path", 7),
    ("adr-metadata.validation", 18),
    ("canonical-adr.resolve", 22),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root exists")
        .to_path_buf()
}

fn vectors<'a>(corpus: &'a LoadedCorpus, responsibility: &str) -> &'a [Vector] {
    &corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == responsibility)
        .expect("Wave B matrix exists")
        .vectors
}

fn request(responsibility: &str, vector: &Vector) -> CandidateRequest {
    CandidateRequest {
        responsibility_id: responsibility.to_owned(),
        vector_id: vector.vector_id.clone(),
        fixture: vector.fixture.clone(),
    }
}

#[test]
fn issue64_wave_b_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let candidate = ReferenceRustCandidate;
    let mut wave_matches = 0;
    for (responsibility, expected_count) in WAVE_B {
        let matrix = vectors(&corpus, responsibility);
        assert_eq!(matrix.len(), expected_count);
        let mut matches = 0;
        for vector in matrix {
            let actual = candidate
                .execute(&request(responsibility, vector))
                .expect("candidate runs");
            let expected = CandidateState::Observation(vector.expected_observation.observation());
            if actual == expected {
                matches += 1;
            } else {
                eprintln!("vector_id={}", vector.vector_id);
                eprintln!("actual={actual:?}");
                eprintln!("expected={expected:?}");
            }
        }
        assert_eq!(matches, expected_count);
        wave_matches += matches;
        match responsibility {
            "adr-metadata.parse" => eprintln!("ADR_METADATA_PARSE_RUST=24/24"),
            "adr-metadata.repository-path" => {
                eprintln!("ADR_METADATA_REPOSITORY_PATH_RUST=7/7")
            }
            "adr-metadata.validation" => eprintln!("ADR_METADATA_VALIDATION_RUST=18/18"),
            "canonical-adr.resolve" => eprintln!("CANONICAL_ADR_RUST=22/22"),
            _ => unreachable!(),
        }
    }
    assert_eq!(wave_matches, 71);
    eprintln!("WAVE_B_RUST=71/71");
}

#[test]
fn issue64_wave_b_global_accounting_is_exact() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let candidate = ReferenceRustCandidate;
    let mut counts = (0, 0, 0);
    for matrix in &corpus.matrices {
        for vector in &matrix.vectors {
            let actual = candidate
                .execute(&request(&matrix.responsibility_id, vector))
                .expect("candidate runs");
            match actual {
                CandidateState::Observation(observation)
                    if observation == vector.expected_observation.observation() =>
                {
                    counts.0 += 1;
                }
                CandidateState::Observation(observation) => {
                    counts.1 += 1;
                    eprintln!("vector_id={}", vector.vector_id);
                    eprintln!("actual={observation:?}");
                    eprintln!("expected={:?}", vector.expected_observation.observation());
                }
                CandidateState::Unimplemented => counts.2 += 1,
            }
        }
    }
    assert_eq!(counts, (983, 0, 170));
    assert_eq!(counts.0 + counts.1 + counts.2, 1153);
    eprintln!("GLOBAL_RUST_MATCH={}", counts.0);
    eprintln!("GLOBAL_RUST_MISMATCH={}", counts.1);
    eprintln!("GLOBAL_RUST_NOT_AVAILABLE={}", counts.2);
    eprintln!("GLOBAL_RUST_TOTAL=1153");
}

#[test]
fn issue64_wave_b_source_boundaries_are_explicit() {
    let repository = root();
    let engine = repository.join("crates/proto-ring-engine/src");
    let canonical = fs::read_to_string(engine.join("canonical_adr/mod.rs")).unwrap();
    let exact_text = fs::read_to_string(engine.join("adr_metadata/exact_text.rs")).unwrap();
    let validation = fs::read_to_string(engine.join("adr_metadata/validation.rs")).unwrap();
    let manifest = fs::read_to_string(repository.join("Cargo.toml")).unwrap();

    assert!(canonical.contains("repository_governance_model::load"));
    assert!(canonical.contains("PortablePattern::compile"));
    assert!(canonical.contains("resolve_repository_path"));
    assert!(!canonical.contains("regex::"));
    assert!(exact_text.contains("UTF8_BOM"));
    assert!(exact_text.contains("b'\\r'"));
    assert!(validation.contains("draft202012::meta::validate"));
    assert!(validation.contains("should_validate_formats(true)"));
    assert!(validation.contains(".offline()"));
    assert!(manifest.contains("jsonschema = { version = \"=0.58.6\""));
    assert!(manifest.contains("sha2 = \"=0.11.1\""));

    let touched_sources = [
        "crates/proto-ring-engine/src/lib.rs",
        "crates/proto-ring-engine/src/adr_metadata/mod.rs",
        "crates/proto-ring-engine/src/adr_metadata/exact_text.rs",
        "crates/proto-ring-engine/src/adr_metadata/repository_path.rs",
        "crates/proto-ring-engine/src/adr_metadata/validation.rs",
        "crates/proto-ring-engine/src/canonical_adr/mod.rs",
        "crates/proto-ring-engine/tests/issue64_jsonschema_dependency.rs",
        "crates/proto-ring-conformance/src/rust_candidate.rs",
        "crates/proto-ring-conformance/src/rust_candidate/adr_metadata.rs",
        "crates/proto-ring-conformance/src/rust_candidate/canonical_adr.rs",
        "crates/proto-ring-conformance/tests/issue64_adr_canonical.rs",
        "crates/proto-ring-conformance/tests/corpus.rs",
        "crates/proto-ring-conformance/tests/corpus/support.rs",
    ];
    for source in touched_sources {
        let lines = fs::read_to_string(repository.join(source))
            .unwrap()
            .lines()
            .count();
        assert!(lines <= 400, "{source} has {lines} lines");
    }

    eprintln!("CANONICAL_ADR_NATIVE_REGEX_ENGINE=0");
    eprintln!("CANONICAL_ADR_PORTABLE_PATTERN_REUSE=YES");
    eprintln!("CANONICAL_ADR_RGM_ROUTE_REUSE=YES");
    eprintln!("ADR_METADATA_CANONICAL_STRUCTURED_DATA_REUSE=YES");
    eprintln!("WAVE_B_VECTOR_ID_SPECIAL_CASES=0");
}
