#![forbid(unsafe_code)]

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::{Observation, ObservationKind, TransportValue, Vector};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

// Compile the actual adapter and its existing dependencies, not a test replica.
// This retains fixture ownership for post-load filesystem assertions without
// adding a public conformance API. Executor observations are checked separately.
use proto_ring_conformance::model;
#[allow(dead_code)]
#[path = "../src"]
mod live_fixture {
    pub mod fixture;
}
use live_fixture::fixture;
#[allow(dead_code)]
#[path = "../src/rust_candidate"]
mod rust_candidate {
    use super::support;
    pub(crate) use super::transport;
    mod repository_integrity;

    pub(super) fn observe(
        fixture: &crate::fixture::MaterializedFixture,
    ) -> Result<crate::model::Observation, crate::fixture::FixtureError> {
        repository_integrity::execute(fixture)
    }

    #[test]
    fn t6_metadata_errors_fail_closed() {
        use repository_integrity::profile_transport::sentinel_absent;
        use std::io::{Error, ErrorKind};

        assert!(sentinel_absent(Err(Error::from(ErrorKind::NotFound))).unwrap());
        for kind in [
            ErrorKind::PermissionDenied,
            ErrorKind::Interrupted,
            ErrorKind::Other,
        ] {
            let error = sentinel_absent(Err(Error::new(kind, "controlled metadata error")))
                .expect_err("unexpected inspection error must be a harness failure");
            assert!(
                error
                    .to_string()
                    .contains("execution sentinel metadata inspection failed")
            );
            assert!(error.to_string().contains("controlled metadata error"));
        }
    }
}
#[allow(dead_code)]
#[path = "../src/rust_candidate/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "../src/rust_candidate/transport.rs"]
pub(crate) mod transport;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn canonical() -> Vector {
    load_corpus(&root())
        .expect("existing corpus loads")
        .matrices
        .into_iter()
        .find(|matrix| matrix.responsibility_id == "repository-integrity.profile")
        .expect("profile matrix")
        .vectors
        .into_iter()
        .find(|vector| vector.vector_id == "repository-integrity.profile.load-does-not-execute")
        .expect("canonical sentinel vector")
}

fn execute(fixture: Value) -> Observation {
    let request = CandidateRequest {
        responsibility_id: "repository-integrity.profile".to_owned(),
        vector_id: "issue133.local-observation".to_owned(),
        fixture,
    };
    match ReferenceRustCandidate
        .execute(&request)
        .expect("candidate succeeds")
    {
        CandidateState::Observation(observation) => {
            assert_eq!(observation.kind, ObservationKind::Result);
            observation
        }
        CandidateState::Unimplemented => panic!("profile adapter must be implemented"),
    }
}

fn flag(fixture: &mut Value, value: Option<Value>) {
    let fields = fixture["invocation"]["value"].as_array_mut().unwrap();
    fields.retain(|entry| entry["name"] != "observe_execution_sentinel");
    if let Some(value) = value {
        fields.push(json!({"name": "observe_execution_sentinel", "value": value}));
    }
}

fn expected_sentinel(observation: &mut Observation, value: Option<bool>) {
    let TransportValue::Record(entries) = &mut observation.value else {
        panic!("expected record");
    };
    if let Some(value) = value {
        entries
            .iter_mut()
            .find(|entry| entry.name == "execution_sentinel_absent")
            .expect("canonical sentinel expectation")
            .value = TransportValue::Boolean(value);
    } else {
        entries.retain(|entry| entry.name != "execution_sentinel_absent");
    }
}

#[test]
fn t1_absent_sentinel_is_measured_without_execution() {
    let vector = canonical();
    let materialized = fixture::materialize(&vector.fixture).unwrap();
    let sentinel = materialized.root().unwrap().join("execution-sentinel.txt");
    assert_eq!(
        fs::symlink_metadata(&sentinel).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    let actual = rust_candidate::observe(&materialized).unwrap();
    assert_eq!(actual, vector.expected_observation.observation());
    assert_eq!(
        fs::symlink_metadata(&sentinel).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    assert_eq!(execute(vector.fixture), actual);
}

#[test]
fn t2_preexisting_sentinel_is_present_and_unchanged() {
    let vector = canonical();
    let mut fixture_value = vector.fixture;
    fixture_value["steps"].as_array_mut().unwrap().push(json!({
        "op": "write_utf8", "path": "execution-sentinel.txt", "text": "fixture-owned sentinel\n"
    }));
    let materialized = fixture::materialize(&fixture_value).unwrap();
    let sentinel = materialized.root().unwrap().join("execution-sentinel.txt");
    assert_eq!(fs::read(&sentinel).unwrap(), b"fixture-owned sentinel\n");
    let actual = rust_candidate::observe(&materialized).unwrap();
    let mut expected = vector.expected_observation.observation();
    expected_sentinel(&mut expected, Some(false));
    assert_eq!(actual, expected);
    assert_eq!(fs::read(&sentinel).unwrap(), b"fixture-owned sentinel\n");
    assert_eq!(execute(fixture_value), expected);
}

#[test]
fn t3_false_flag_omits_observation() {
    let vector = canonical();
    let mut fixture_value = vector.fixture;
    flag(
        &mut fixture_value,
        Some(json!({"type": "boolean", "value": false})),
    );
    let mut expected = vector.expected_observation.observation();
    expected_sentinel(&mut expected, None);
    assert_eq!(execute(fixture_value), expected);
}

#[test]
fn t4_absent_flag_omits_observation() {
    let vector = canonical();
    let mut fixture_value = vector.fixture;
    flag(&mut fixture_value, None);
    let mut expected = vector.expected_observation.observation();
    expected_sentinel(&mut expected, None);
    assert_eq!(execute(fixture_value), expected);
}

#[test]
fn t5_invalid_flag_is_a_harness_error_even_for_rejected_profile() {
    let vector = canonical();
    let mut fixture_value = vector.fixture;
    flag(
        &mut fixture_value,
        Some(json!({"type": "string", "value": "true"})),
    );
    let request = |fixture| CandidateRequest {
        responsibility_id: "repository-integrity.profile".to_owned(),
        vector_id: "issue133.invalid-flag".to_owned(),
        fixture,
    };
    assert!(
        ReferenceRustCandidate
            .execute(&request(fixture_value.clone()))
            .is_err()
    );
    fixture_value["steps"][0]["text"] = json!("malformed profile");
    assert!(
        ReferenceRustCandidate
            .execute(&request(fixture_value))
            .is_err()
    );
}

#[test]
fn t7_original_canonical_vector_matches_independent_expectation() {
    let vector = canonical();
    let expected = vector.expected_observation.observation();
    assert_eq!(execute(vector.fixture), expected);
}
