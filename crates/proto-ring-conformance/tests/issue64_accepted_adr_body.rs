#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Vector;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

const RESPONSIBILITY: &str = "accepted-adr-body.immutability";
const VECTOR_COUNT: usize = 22;

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
        .expect("Wave D matrix exists")
        .vectors
}

fn request(vector: &Vector) -> CandidateRequest {
    CandidateRequest {
        responsibility_id: RESPONSIBILITY.to_owned(),
        vector_id: vector.vector_id.clone(),
        fixture: vector.fixture.clone(),
    }
}

#[test]
fn issue64_wave_d_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let matrix = vectors(&corpus);
    assert_eq!(matrix.len(), VECTOR_COUNT);
    let candidate = ReferenceRustCandidate;
    let mut matches = 0;
    for vector in matrix {
        let actual = candidate.execute(&request(vector)).expect("candidate runs");
        let expected = CandidateState::Observation(vector.expected_observation.observation());
        if actual == expected {
            matches += 1;
        } else {
            eprintln!("vector_id={}", vector.vector_id);
            eprintln!("actual={actual:?}");
            eprintln!("expected={expected:?}");
        }
    }
    assert_eq!(matches, VECTOR_COUNT);
    eprintln!("ACCEPTED_ADR_BODY_RUST=22/22");
    eprintln!("WAVE_D_RUST=22/22");
}

#[test]
fn issue64_wave_d_source_boundaries_are_explicit() {
    let repository = root();
    let engine_root = repository.join("crates/proto-ring-engine/src/accepted_adr_body");
    let git = fs::read_to_string(engine_root.join("git.rs")).unwrap();
    let history = fs::read_to_string(engine_root.join("history.rs")).unwrap();
    let orchestration = fs::read_to_string(engine_root.join("mod.rs")).unwrap();
    let adapter = fs::read_to_string(
        repository.join("crates/proto-ring-conformance/src/rust_candidate/accepted_adr_body.rs"),
    )
    .unwrap();
    let production = format!("{git}{history}{orchestration}");

    assert_eq!(production.matches("Command::new(\"git\")").count(), 1);
    assert_eq!(git.matches("Command::new(\"git\")").count(), 1);
    for required in [
        "GIT_NO_REPLACE_OBJECTS",
        "GIT_CONFIG_NOSYSTEM",
        "GIT_CONFIG_GLOBAL",
        "env_clear",
        "git_environment_name",
        "rev-list",
        "--first-parent",
        "--reverse",
        "diff-tree",
        "--no-renames",
        "\"-z\"",
        "ls-tree",
        "cat-file",
        "blob",
        "--is-shallow-repository",
    ] {
        assert!(git.contains(required), "missing Git boundary: {required}");
    }
    for read_only in ["rev-parse", "rev-list", "diff-tree", "ls-tree", "cat-file"] {
        assert!(git.contains(&format!("\"{read_only}\"")));
    }
    for forbidden in [
        "\"add\"",
        "\"commit\"",
        "\"checkout\"",
        "\"merge\"",
        "\"replace\"",
        "\"reset\"",
        "\"update-index\"",
        "\"write-tree\"",
        "\"log\"",
        "\"show\"",
    ] {
        assert!(
            !git.contains(forbidden),
            "Git write/forbidden operation: {forbidden}"
        );
    }
    for required in [
        "first_parent_commits",
        "first_parent(&commit)",
        "changed_paths",
        "REGULAR_FILE_MODES",
        "[\"100644\", \"100755\"]",
        "std::str::from_utf8(basename)",
        "adr_metadata::parse_adr",
    ] {
        assert!(
            history.contains(required),
            "missing history boundary: {required}"
        );
    }
    for required in [
        "repository_governance_model::load",
        "architecture_decisions",
        "adr_directory",
        "filename_pattern",
        "PortablePattern::compile",
        "canonical_adr::resolve",
    ] {
        assert!(
            orchestration.contains(required),
            "missing reuse boundary: {required}"
        );
    }
    assert!(adapter.contains("AcceptedBodyDiagnosticKind::BodyChanged"));
    assert!(!adapter.contains("diagnostic.detail"));
    for forbidden in ["regex::", "sha2", "sha256", "git2", "gix::"] {
        assert!(
            !production.contains(forbidden),
            "forbidden mechanism: {forbidden}"
        );
    }

    let corpus = load_corpus(&repository).unwrap();
    for vector in vectors(&corpus) {
        assert!(!format!("{production}{adapter}").contains(&vector.vector_id));
    }
    for source in [
        "crates/proto-ring-engine/src/lib.rs",
        "crates/proto-ring-engine/src/accepted_adr_body/mod.rs",
        "crates/proto-ring-engine/src/accepted_adr_body/git.rs",
        "crates/proto-ring-engine/src/accepted_adr_body/history.rs",
        "crates/proto-ring-conformance/src/rust_candidate.rs",
        "crates/proto-ring-conformance/src/rust_candidate/accepted_adr_body.rs",
        "crates/proto-ring-conformance/tests/issue64_accepted_adr_body.rs",
        "crates/proto-ring-conformance/tests/corpus.rs",
        "crates/proto-ring-conformance/tests/corpus/support.rs",
    ] {
        let lines = fs::read_to_string(repository.join(source))
            .unwrap()
            .lines()
            .count();
        assert!(lines <= 400, "{source} has {lines} lines");
    }

    eprintln!("FIRST_PARENT_HISTORY=YES");
    eprintln!("REVERSE_HISTORY_ORDER=YES");
    eprintln!("RENAME_HEURISTICS=NO");
    eprintln!("NUL_PATH_TRANSPORT=YES");
    eprintln!("HISTORICAL_BLOB_BY_OBJECT_ID=YES");
    eprintln!("GIT_NO_REPLACE_OBJECTS=YES");
    eprintln!("GIT_WRITE_OPERATIONS=NO");
    eprintln!("NON_UTF8_GLOBAL_DECODE=NO");
    eprintln!("DIRECT_CHILD_FILTER_BEFORE_UTF8_DECODE=YES");
    eprintln!("PORTABLE_PATTERN_REUSE=YES");
    eprintln!("NATIVE_REGEX_ENGINE=NO");
    eprintln!("ADR_METADATA_PARSER_REUSE=YES");
    eprintln!("CANONICAL_ADR_CURRENT_RESOLUTION_REUSE=YES");
    eprintln!("REGULAR_FILE_MODES=100644,100755");
    eprintln!("HISTORICAL_SYMLINK_FOLLOWING=NO");
    eprintln!("SHALLOW_HISTORY_FAIL_CLOSED=YES");
    eprintln!("VECTOR_ID_SPECIAL_CASES=NO");
}
