#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{
    CandidateState, ComparisonStatus, PythonBridge, candidate_requests, run_differential,
};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::{ComparisonKind, Observation};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_conformance::standalone::run_standalone;
use serde::Serialize;

#[derive(Serialize)]
struct LegacyRustRecord {
    responsibility_id: String,
    vector_id: String,
    observation: Observation,
    comparison: ComparisonKind,
    verdict: &'static str,
}

#[derive(Serialize)]
struct ComparisonSummary {
    responsibility_count: usize,
    vector_count: usize,
    missing_ids: Vec<String>,
    additional_ids: Vec<String>,
    observation_divergences: Vec<String>,
    verdict_divergences: Vec<String>,
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("conformance crate belongs to the workspace")
        .to_path_buf()
}

fn comparison_kind_by_id() -> HashMap<String, ComparisonKind> {
    let corpus = load_corpus(&repository_root()).expect("published corpus loads");
    corpus
        .matrices
        .iter()
        .flat_map(|matrix| matrix.vectors.iter())
        .map(|vector| (vector.vector_id.clone(), vector.comparison.kind))
        .collect()
}

fn write_json(path: &Path, value: &impl Serialize) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("evidence serializes");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("evidence file is writable");
}

#[test]
fn legacy_and_standalone_rust_channels_match_by_id() {
    let root = repository_root();
    let corpus = load_corpus(&root).expect("published corpus loads");
    let requests = candidate_requests(&corpus);
    let python = PythonBridge::new(&root)
        .execute(&requests)
        .expect("legacy witness environment runs");
    let legacy = run_differential(&corpus, &python, &ReferenceRustCandidate)
        .expect("legacy differential control runs");
    let standalone =
        run_standalone(&corpus, &ReferenceRustCandidate).expect("standalone Rust control runs");
    let comparison_kinds = comparison_kind_by_id();

    let legacy_records: Vec<_> = legacy
        .into_iter()
        .map(|record| LegacyRustRecord {
            responsibility_id: record.responsibility_id,
            vector_id: record.vector_id.clone(),
            observation: match record.rust_candidate {
                CandidateState::Observation(observation) => observation,
                CandidateState::Unimplemented => panic!("unimplemented: {}", record.vector_id),
            },
            comparison: comparison_kinds[&record.vector_id],
            verdict: match record.rust_vs_expected {
                ComparisonStatus::Match => "MATCH",
                ComparisonStatus::Mismatch => "MISMATCH",
                ComparisonStatus::NotAvailable => "NOT_AVAILABLE",
            },
        })
        .collect();
    let legacy_by_id: HashMap<_, _> = legacy_records
        .iter()
        .map(|record| (record.vector_id.as_str(), record))
        .collect();
    let standalone_by_id: HashMap<_, _> = standalone
        .iter()
        .map(|record| (record.vector_id.as_str(), record))
        .collect();

    let missing_ids: Vec<_> = legacy_by_id
        .keys()
        .filter(|id| !standalone_by_id.contains_key(**id))
        .map(|id| (*id).to_owned())
        .collect();
    let additional_ids: Vec<_> = standalone_by_id
        .keys()
        .filter(|id| !legacy_by_id.contains_key(**id))
        .map(|id| (*id).to_owned())
        .collect();
    let observation_divergences: Vec<_> = legacy_by_id
        .iter()
        .filter(|(id, legacy)| standalone_by_id[**id].observation != legacy.observation)
        .map(|(id, _)| (*id).to_owned())
        .collect();
    let verdict_divergences: Vec<_> = legacy_by_id
        .iter()
        .filter(|(id, legacy)| {
            standalone_by_id[**id].verdict
                != proto_ring_conformance::standalone::StandaloneVerdict::Match
                || legacy.verdict != "MATCH"
        })
        .map(|(id, _)| (*id).to_owned())
        .collect();
    let summary = ComparisonSummary {
        responsibility_count: standalone
            .iter()
            .map(|record| record.responsibility_id.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        vector_count: standalone.len(),
        missing_ids,
        additional_ids,
        observation_divergences,
        verdict_divergences,
    };

    assert_eq!(summary.responsibility_count, 29);
    assert_eq!(summary.vector_count, 1153);
    assert!(summary.missing_ids.is_empty());
    assert!(summary.additional_ids.is_empty());
    assert!(summary.observation_divergences.is_empty());
    assert!(summary.verdict_divergences.is_empty());

    if let Some(directory) = std::env::var_os("PROTO_RING_REVIEW_EVIDENCE_DIR") {
        let directory = PathBuf::from(directory);
        fs::create_dir_all(&directory).expect("evidence directory is creatable");
        write_json(&directory.join("legacy-rust-channel.json"), &legacy_records);
        write_json(&directory.join("standalone-rust-channel.json"), &standalone);
        write_json(&directory.join("channel-comparison.json"), &summary);
    }
}
