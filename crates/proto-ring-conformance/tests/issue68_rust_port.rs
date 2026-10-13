#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use proto_ring_conformance::fixture;
use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::{Observation, Vector};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_engine::repository_governance_state::{self, RepositoryGovernanceStage};
use serde_json::Value;

const RESPONSIBILITY: &str = "repository-governance-state.compose";
const BOOTSTRAP_SYMLINK_VECTOR_ID: &str = "repository-governance-state.compose.bootstrap-symlink";

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
        .expect("RepositoryGovernanceState matrix exists")
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

fn bootstrap_symlink_vector(corpus: &LoadedCorpus) -> &Vector {
    vectors(corpus)
        .iter()
        .find(|vector| vector.vector_id == BOOTSTRAP_SYMLINK_VECTOR_ID)
        .expect("published bootstrap symlink vector exists")
}

fn with_hostile_autocrlf(fixture: &Value) -> Value {
    let mut hostile = fixture.clone();
    let steps = hostile
        .get_mut("steps")
        .and_then(Value::as_array_mut)
        .expect("repository plan steps exist");
    for step in steps {
        let step = step.as_object_mut().expect("fixture step is an object");
        if step.get("op").and_then(Value::as_str) != Some("git") {
            continue;
        }
        let environment = step
            .get_mut("environment")
            .and_then(Value::as_object_mut)
            .expect("Git fixture environment exists");
        for (name, value) in [
            ("GIT_CONFIG_COUNT", "1"),
            ("GIT_CONFIG_KEY_0", "core.autocrlf"),
            ("GIT_CONFIG_VALUE_0", "true"),
        ] {
            environment.insert(name.to_owned(), Value::String(value.to_owned()));
        }
    }
    hostile
}

fn derived_clone_plan(vector: &Vector, hostile: bool, explicit_autocrlf: bool) -> Value {
    let mut plan = if hostile {
        with_hostile_autocrlf(&vector.fixture)
    } else {
        vector.fixture.clone()
    };
    let steps = plan
        .get_mut("steps")
        .and_then(Value::as_array_mut)
        .expect("repository plan steps exist");
    let clone_index = steps
        .iter()
        .position(|step| {
            step.get("op").and_then(Value::as_str) == Some("git")
                && step
                    .get("argv")
                    .and_then(Value::as_array)
                    .and_then(|argv| argv.first())
                    .and_then(Value::as_str)
                    == Some("clone")
        })
        .expect("bootstrap symlink plan clones its carrier");
    let argv = steps[clone_index]
        .get_mut("argv")
        .and_then(Value::as_array_mut)
        .expect("clone argv exists");
    *argv.last_mut().expect("clone destination exists") =
        Value::String("external-bootstrap".to_owned());
    if explicit_autocrlf {
        argv.splice(
            0..0,
            [
                Value::String("-c".to_owned()),
                Value::String("core.autocrlf=true".to_owned()),
            ],
        );
    }
    steps.truncate(clone_index + 1);
    plan
}

fn materialized_carrier_bytes(plan: &Value) -> (Vec<u8>, Vec<u8>) {
    let realized = fixture::materialize(plan).expect("derived clone plan materializes");
    let root = realized.root().expect("repository fixture has a root");
    (
        fs::read(root.join("AGENTS.md")).expect("source carrier is readable"),
        fs::read(root.join("external-bootstrap/AGENTS.md")).expect("cloned carrier is readable"),
    )
}

fn carriage_return_count(bytes: &[u8]) -> usize {
    bytes.iter().filter(|byte| **byte == b'\r').count()
}

#[test]
fn issue68_hostile_autocrlf_preserves_bootstrap_symlink_observation() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let vector = bootstrap_symlink_vector(&corpus);
    let mut hostile_request = request(vector);
    hostile_request.fixture = with_hostile_autocrlf(&vector.fixture);
    let actual = ReferenceRustCandidate
        .execute(&hostile_request)
        .expect("candidate runs");
    assert_eq!(actual, CandidateState::Observation(expected(vector)));
    eprintln!("BOOTSTRAP_SYMLINK_STAGE=repository_state");
    eprintln!("BOOTSTRAP_SYMLINK_STATUS=CONTROLLED_FAILURE");
    eprintln!("HOSTILE_AUTOCRLF_RGS_OBSERVATION=PASS");
}

#[test]
fn issue68_hostile_autocrlf_preserves_clone_bytes() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let vector = bootstrap_symlink_vector(&corpus);
    let (source, cloned) = materialized_carrier_bytes(&derived_clone_plan(vector, true, false));
    assert_eq!(carriage_return_count(&source), 0);
    assert_eq!(carriage_return_count(&cloned), 0);
    assert_eq!(source, cloned);
    eprintln!("SOURCE_AGENTS_CR_COUNT=0");
    eprintln!("CLONED_AGENTS_CR_COUNT=0");
    eprintln!("SOURCE_BYTES_EQUAL_CLONE_BYTES=YES");
    eprintln!("HOSTILE_AUTOCRLF_BYTE_IDENTITY=PASS");
}

#[test]
fn issue68_later_explicit_git_config_takes_precedence() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let vector = bootstrap_symlink_vector(&corpus);
    let (source, cloned) = materialized_carrier_bytes(&derived_clone_plan(vector, false, true));
    assert_eq!(carriage_return_count(&source), 0);
    assert!(carriage_return_count(&cloned) > 0);
    assert_ne!(source, cloned);
    eprintln!("EXPLICIT_LATER_GIT_CONFIG_PRECEDENCE=PASS");
}

#[test]
fn issue68_rust_port_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let vectors = vectors(&corpus);
    let candidate = ReferenceRustCandidate;
    assert_eq!(vectors.len(), 25);
    let matching = vectors
        .iter()
        .filter(|vector| {
            candidate.execute(&request(vector)).expect("candidate runs")
                == CandidateState::Observation(expected(vector))
        })
        .count();
    assert_eq!(matching, 25);
    eprintln!("ISSUE68_RUST_VECTORS=25");
    eprintln!("ISSUE68_RUST_MATCH=25");
    eprintln!("ISSUE68_RUST_MISMATCH=0");
}

#[test]
fn issue68_component_failures_preserve_owned_stages() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    for (suffix, stage) in [
        (
            "model-v1",
            RepositoryGovernanceStage::RepositoryGovernanceModel,
        ),
        (
            "malformed-authority",
            RepositoryGovernanceStage::GovernanceAuthority,
        ),
        (
            "malformed-projection",
            RepositoryGovernanceStage::ProjectionIntegrity,
        ),
        (
            "undeclared-capability-binding",
            RepositoryGovernanceStage::Composition,
        ),
    ] {
        let vector = vectors(&corpus)
            .iter()
            .find(|vector| vector.vector_id.ends_with(suffix))
            .expect("focused vector exists");
        let realized = fixture::materialize(&vector.fixture).expect("fixture materializes");
        let error = repository_governance_state::load(
            realized.root().expect("repository fixture has a root"),
        )
        .expect_err("focused composition fails");
        assert_eq!(error.diagnostic.stage, stage, "{suffix}");
        assert!(!error.diagnostic.detail.is_empty(), "{suffix}");
    }
    eprintln!("ISSUE68_STAGE_ATTRIBUTION=PASS");
}

#[test]
fn issue68_default_build_excludes_conformance_seams() {
    let temporary = tempfile::tempdir().expect("probe root exists");
    fs::create_dir(temporary.path().join("src")).expect("probe source directory exists");
    fs::write(
        temporary.path().join("src/main.rs"),
        r#"use proto_ring_engine::repository_governance_state::conformance_internal;
fn main() {
    let _ = conformance_internal::load_with_scope_overrides(
        std::path::Path::new("."),
        &[],
        &[],
    );
}
"#,
    )
    .expect("probe source is written");
    write_probe_manifest(temporary.path(), false);
    let default = check_probe(temporary.path());
    assert!(!default.status.success());
    let stderr = String::from_utf8_lossy(&default.stderr);
    assert!(stderr.contains("conformance_internal"));

    write_probe_manifest(temporary.path(), true);
    let featured = check_probe(temporary.path());
    assert!(
        featured.status.success(),
        "feature probe failed: {}",
        String::from_utf8_lossy(&featured.stderr)
    );
    eprintln!("RGS_DEFAULT_CONFORMANCE_SEAM=0");
    eprintln!("RGS_FEATURED_CONFORMANCE_SEAM=PASS");
}

fn write_probe_manifest(root_path: &Path, feature: bool) {
    let engine = root().join("crates/proto-ring-engine");
    let feature = if feature {
        ", features = [\"conformance-internal\"]"
    } else {
        ""
    };
    fs::write(
        root_path.join("Cargo.toml"),
        format!(
            "[workspace]\n\n[package]\nname = \"issue68-feature-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nproto-ring-engine = {{ path = '{}'{feature} }}\n",
            engine.display()
        ),
    )
    .expect("probe manifest is written");
}

fn check_probe(root_path: &Path) -> std::process::Output {
    Command::new(env!("CARGO"))
        .args(["check", "--offline", "--quiet"])
        .current_dir(root_path)
        .env("CARGO_TARGET_DIR", root_path.join("target"))
        .output()
        .expect("Cargo feature probe executes")
}

#[test]
fn issue68_engine_source_preserves_non_execution_boundaries() {
    let source_root = root().join("crates/proto-ring-engine/src");
    let source = [
        "repository_governance_state/mod.rs",
        "repository_governance_state/compose.rs",
        "repository_governance_state/scope.rs",
        "repository_governance_state/orchestrate.rs",
    ]
    .iter()
    .map(|path| fs::read_to_string(source_root.join(path)).expect("RGS source is readable"))
    .collect::<String>();

    for forbidden in [
        "reqwest",
        "ureq",
        "TcpStream",
        "Command::new",
        "exact_evidence_binding",
        "evaluate(",
        "read_dir",
        "write(",
        "create_dir",
        "coherence_basis",
        "coherence_proven",
        "snapshot_verified",
        "governance_routing::resolve",
    ] {
        assert!(
            !source.contains(forbidden),
            "forbidden RGS source: {forbidden}"
        );
    }
    for required in [
        "repository_state::capture",
        "repository_governance_model::from_bootstrap",
        "governance_authority::load",
        "governance_bindings::load",
        "validate_binding_capabilities",
    ] {
        assert!(source.contains(required), "missing RGS source: {required}");
    }
    eprintln!("RGS_PROVIDER_NETWORK_CALLS=0");
    eprintln!("RGS_INTEGRITY_EVALUATION_CALLS=0");
    eprintln!("RGS_ADR_ENUMERATION_CALLS=0");
    eprintln!("RGS_REPOSITORY_MUTATION_CALLS=0");
    eprintln!("RGS_ARTIFACT_GENERATION_CALLS=0");
    eprintln!("RGS_COHERENCE_PROOF_FIELD=0");
    eprintln!("RGS_COHERENCE_BASIS_INPUT=0");
    eprintln!("RGS_SECOND_ROUTING_INTERPRETER=0");
    eprintln!("RGS_USES_REPOSITORY_STATE_CAPTURE=YES");
    eprintln!("RGS_USES_RGM_FROM_BOOTSTRAP=YES");
    eprintln!("RGS_USES_EXISTING_COMPONENT_LOADERS=YES");
    eprintln!("RGS_USES_EXISTING_BINDING_CAPABILITY_VALIDATION=YES");
}
