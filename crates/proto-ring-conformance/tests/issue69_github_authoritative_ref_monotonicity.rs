#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Vector;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

const RESPONSIBILITY: &str = "github-authoritative-ref-monotonicity.observe";
const VECTOR_COUNT: usize = 20;

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
        .expect("Wave C matrix exists")
        .vectors
}

#[test]
fn issue69_wave_c_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let matrix = vectors(&corpus);
    assert_eq!(matrix.len(), VECTOR_COUNT);
    let actual_ids: BTreeSet<_> = matrix
        .iter()
        .map(|vector| vector.vector_id.clone())
        .collect();
    let expected_ids: BTreeSet<_> = [
        "additional-rule",
        "bound-absent",
        "http-error",
        "io-error",
        "malformed-binding",
        "malformed-item",
        "malformed-json",
        "malformed-provider-data",
        "malformed-utf8",
        "missing-deletion",
        "missing-non-fast-forward",
        "network-error",
        "other-ruleset",
        "pagination",
        "pagination-incomplete",
        "provider-unavailable",
        "required-rules",
        "timeout",
        "tls-error",
        "wrong-top-level",
    ]
    .into_iter()
    .map(|suffix| format!("{RESPONSIBILITY}.{suffix}"))
    .collect();
    assert_eq!(actual_ids, expected_ids);

    let candidate = ReferenceRustCandidate;
    let mut matches = 0;
    for vector in matrix {
        let request = CandidateRequest {
            responsibility_id: RESPONSIBILITY.to_owned(),
            vector_id: vector.vector_id.clone(),
            fixture: vector.fixture.clone(),
        };
        let actual = candidate.execute(&request).expect("candidate runs");
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
    eprintln!("GITHUB_ARM_RUST=20/20");
    eprintln!("ISSUE69_WAVE_C_RUST=20/20");
}

#[test]
fn issue69_wave_c_source_boundaries_are_explicit() {
    let repository = root();
    let engine_root =
        repository.join("crates/proto-ring-engine/src/github_authoritative_ref_monotonicity");
    let module = fs::read_to_string(engine_root.join("mod.rs")).unwrap();
    let binding = fs::read_to_string(engine_root.join("binding.rs")).unwrap();
    let http = fs::read_to_string(engine_root.join("http.rs")).unwrap();
    let observation = fs::read_to_string(engine_root.join("observation.rs")).unwrap();
    let adapter = fs::read_to_string(repository.join(
        "crates/proto-ring-conformance/src/rust_candidate/github_authoritative_ref_monotonicity.rs",
    ))
    .unwrap();
    let registration =
        fs::read_to_string(repository.join("crates/proto-ring-conformance/src/rust_candidate.rs"))
            .unwrap();
    let implementation = format!("{module}{binding}{http}{observation}{adapter}{registration}");

    let generic_contract =
        fs::read_to_string(repository.join("docs/contracts/authoritative-ref-monotonicity.md"))
            .unwrap();
    assert!(!generic_contract.contains("GitHub"));
    assert!(
        http.contains("https://api.github.com") || observation.contains("https://api.github.com")
    );
    assert!(binding.contains("structured_data::parse_frontmatter_bytes"));
    assert!(!binding.contains("contract_sha"));
    assert!(observation.contains("ArmStatus::Violated, ArmDiagnosticKind::BindingInvalid"));
    assert!(observation.contains("ArmStatus::Undetermined"));
    assert!(observation.contains("\"deletion\", \"non_fast_forward\""));
    assert!(observation.contains("if ruleset_id == binding.ruleset_id"));

    assert!(observation.contains("pub(super) const PAGE_SIZE: usize = 100"));
    assert!(observation.contains("pub(super) const MAX_PAGES: usize = 100"));
    assert!(observation.contains("for page in 1..=MAX_PAGES"));
    assert!(observation.contains("if items.len() < PAGE_SIZE"));
    assert!(observation.contains("std::str::from_utf8(body)"));
    assert!(observation.contains("serde_json::from_str(text)"));
    assert!(observation.contains("value.as_array()"));
    assert!(observation.contains("item.as_object()"));
    assert!(
        observation.find(".get(\"type\")").unwrap()
            < observation
                .find("if ruleset_id == binding.ruleset_id")
                .unwrap()
    );

    let workspace_manifest = fs::read_to_string(repository.join("Cargo.toml")).unwrap();
    let engine_manifest =
        fs::read_to_string(repository.join("crates/proto-ring-engine/Cargo.toml")).unwrap();
    assert!(workspace_manifest.contains(
        "ureq = { version = \"=3.4.2\", default-features = false, features = [\"rustls\"] }"
    ));
    for forbidden in [
        "\"json\"",
        "\"cookies\"",
        "\"multipart\"",
        "\"charset\"",
        "\"socks-proxy\"",
        "\"native-tls\"",
        "\"platform-verifier\"",
    ] {
        assert!(!workspace_manifest.contains(forbidden));
    }
    assert!(engine_manifest.contains("ureq.workspace = true"));
    assert!(http.contains(".https_only(true)"));
    assert!(http.contains("Duration::from_secs(10)"));
    assert!(http.contains(".http_status_as_error(false)"));
    assert!(http.contains("TlsProvider::Rustls"));
    assert!(http.contains("RootCerts::WebPki"));
    assert!(http.contains(".read_to_vec()"));
    assert!(!http.contains("read_to_string"));
    for forbidden in ["GITHUB_TOKEN", "GH_TOKEN", "Authorization"] {
        assert!(!format!("{module}{binding}{http}{observation}").contains(forbidden));
    }

    assert!(adapter.contains("tempfile::tempdir"));
    assert!(adapter.contains("conformance_internal::observe_with_transport"));
    assert!(!adapter.contains("github_authoritative_ref_monotonicity::observe("));
    assert!(adapter.contains("self.position != expected"));
    assert!(adapter.contains("self.requests.len() != expected"));
    let corpus = load_corpus(&repository).unwrap();
    for vector in vectors(&corpus) {
        assert!(!implementation.contains(&vector.vector_id));
    }

    let governance_state_root =
        repository.join("crates/proto-ring-engine/src/repository_governance_state");
    let governance_state = ["mod.rs", "compose.rs", "scope.rs", "orchestrate.rs"]
        .iter()
        .map(|path| fs::read_to_string(governance_state_root.join(path)).unwrap())
        .collect::<String>();
    assert!(!governance_state.contains("github_authoritative_ref_monotonicity"));
    assert!(!governance_state.contains("ureq"));

    for source in [
        "crates/proto-ring-engine/src/lib.rs",
        "crates/proto-ring-engine/src/github_authoritative_ref_monotonicity/mod.rs",
        "crates/proto-ring-engine/src/github_authoritative_ref_monotonicity/binding.rs",
        "crates/proto-ring-engine/src/github_authoritative_ref_monotonicity/http.rs",
        "crates/proto-ring-engine/src/github_authoritative_ref_monotonicity/observation.rs",
        "crates/proto-ring-conformance/src/rust_candidate.rs",
        "crates/proto-ring-conformance/src/rust_candidate/github_authoritative_ref_monotonicity.rs",
        "crates/proto-ring-conformance/tests/issue69_github_authoritative_ref_monotonicity.rs",
        "crates/proto-ring-conformance/tests/corpus.rs",
        "crates/proto-ring-conformance/tests/corpus/support.rs",
    ] {
        let lines = fs::read_to_string(repository.join(source))
            .unwrap()
            .lines()
            .count();
        assert!(lines <= 400, "{source} has {lines} lines");
    }

    eprintln!("ARM_GENERIC_CONTRACT_PROVIDER_NEUTRAL=YES");
    eprintln!("GITHUB_IMPLEMENTATION_PROVIDER_SPECIFIC=YES");
    eprintln!("LOCAL_BINDING_CONTRACT_PIN_DUPLICATION=NO");
    eprintln!("MALFORMED_BINDING_STATUS=VIOLATED");
    eprintln!("PROVIDER_UNAVAILABLE_STATUS=UNDETERMINED");
    eprintln!("MALFORMED_PROVIDER_DATA_STATUS=UNDETERMINED");
    eprintln!("REQUIRED_RULE_DELETION=YES");
    eprintln!("REQUIRED_RULE_NON_FAST_FORWARD=YES");
    eprintln!("BOUND_RULESET_ID_FILTER=YES");
    eprintln!("OTHER_RULESET_CANNOT_SATISFY=YES");
    eprintln!("ADDITIONAL_BOUND_RULES_ALLOWED=YES");
    eprintln!("PAGINATION_PAGE_SIZE=100");
    eprintln!("PAGINATION_COMPLETE_ONLY_WHEN_SHORT_PAGE=YES");
    eprintln!("PARTIAL_PAGINATION_CANNOT_SATISFY=YES");
    eprintln!("MAX_PAGES=100");
    eprintln!("HTTP_RESPONSE_UTF8_STRICT=YES");
    eprintln!("HTTP_RESPONSE_JSON_STRICT=YES");
    eprintln!("HTTP_TOP_LEVEL_ARRAY_REQUIRED=YES");
    eprintln!("EVERY_RULE_ITEM_VALIDATED=YES");
    eprintln!("LIVE_HTTP_CLIENT=ureq-3.4.2");
    eprintln!("LIVE_HTTP_HTTPS_ONLY=YES");
    eprintln!("LIVE_HTTP_GLOBAL_TIMEOUT_SECONDS=10");
    eprintln!("LIVE_HTTP_TLS=rustls-webpki");
    eprintln!("AMBIENT_GITHUB_TOKEN_DISCOVERY=NO");
    eprintln!("LIVE_PROVIDER_ACCESS_DURING_CONFORMANCE=NO");
    eprintln!("REPOSITORY_GOVERNANCE_STATE_PROVIDER_INTERACTIONS=0");
    eprintln!("VECTOR_ID_SPECIAL_CASES=NO");
}
