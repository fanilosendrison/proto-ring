#![forbid(unsafe_code)]

use arm::conformance_internal::{
    self as control, ControlFailure, with_provider_observation_forbidden,
};
use proto_ring_conformance::fixture;
use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::Vector;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_engine::github_authoritative_ref_monotonicity::{self as arm, ArmStatus};
use proto_ring_engine::repository_governance_state::conformance_internal as rgs;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn vector(responsibility: &str, suffix: &str) -> Vector {
    load_corpus(&root())
        .unwrap()
        .matrices
        .into_iter()
        .find(|matrix| matrix.responsibility_id == responsibility)
        .unwrap()
        .vectors
        .into_iter()
        .find(|vector| vector.vector_id.ends_with(suffix))
        .unwrap()
}

fn rgs_vector() -> Vector {
    vector(
        "repository-governance-state.compose",
        ".no-provider-contact",
    )
}

fn request(vector: &Vector, responsibility: &str) -> CandidateRequest {
    CandidateRequest {
        responsibility_id: responsibility.to_owned(),
        vector_id: vector.vector_id.clone(),
        fixture: vector.fixture.clone(),
    }
}

#[test]
fn t1_canonical_rgs_measured_zero() {
    let vector = rgs_vector();
    let actual = ReferenceRustCandidate
        .execute(&request(&vector, "repository-governance-state.compose"))
        .unwrap();
    assert_eq!(
        actual,
        CandidateState::Observation(vector.expected_observation.observation())
    );
}

#[test]
fn t2_public_arm_entry_is_recorded_before_binding_parsing() {
    let temporary = tempfile::tempdir().unwrap();
    // An absent local binding also prevents network access if this regression fails.
    let path = temporary.path().join("missing-binding.md");
    let error = with_provider_observation_forbidden(|| {
        let observation = arm::observe(&path);
        assert_eq!(observation.status, ArmStatus::Undetermined);
        assert_eq!(
            observation.diagnostics[0].kind,
            arm::ArmDiagnosticKind::ProviderObservationUnavailable
        );
    })
    .unwrap_err();
    assert_eq!(error.kind(), ControlFailure::InvocationsObserved);
    assert_eq!(error.invocations(), Some(1));
}

#[test]
fn t3_injected_arm_entry_does_not_reach_transport() {
    let temporary = tempfile::tempdir().unwrap();
    let mut transport_calls = 0;
    let error = with_provider_observation_forbidden(|| {
        let observed =
            control::observe_with_transport(&temporary.path().join("missing.md"), |_| {
                transport_calls += 1;
                panic!("prohibited entry must not invoke transport")
            });
        assert_eq!(observed.status, ArmStatus::Undetermined);
    })
    .unwrap_err();
    assert_eq!(error.invocations(), Some(1));
    assert_eq!(transport_calls, 0);
}

#[test]
fn t4_invocation_during_real_rgs_construction_survives_ignored_arm_result() {
    let vector = rgs_vector();
    let materialized = fixture::materialize(&vector.fixture["base"]).unwrap();
    let root = materialized.root().unwrap();
    let error = with_provider_observation_forbidden(|| {
        let loaded = rgs::load_with_after_initial_capture_hook(root, |root| {
            let _ignored = arm::observe(&root.join("missing-binding.md"));
            Ok(())
        });
        assert!(
            loaded.is_ok(),
            "the ignored ARM result must not be the RGS failure oracle"
        );
    })
    .unwrap_err();
    assert_eq!(error.kind(), ControlFailure::InvocationsObserved);
    assert_eq!(error.invocations(), Some(1));
}

#[test]
fn t5_zero_one_and_repeated_call_accounting() {
    let (value, receipt) = with_provider_observation_forbidden(|| 42).unwrap();
    assert_eq!(value, 42);
    assert_eq!(receipt.invocations(), 0);
    let temporary = tempfile::tempdir().unwrap();
    for count in [1, 3] {
        let error = with_provider_observation_forbidden(|| {
            for _ in 0..count {
                let _ = arm::observe(&temporary.path().join("missing.md"));
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), ControlFailure::InvocationsObserved);
        assert_eq!(error.invocations(), Some(count));
    }
}

#[test]
fn t6_nested_scope_invalidates_outer_zero() {
    let mut nested_ran = false;
    let error = with_provider_observation_forbidden(|| {
        let nested = with_provider_observation_forbidden(|| nested_ran = true);
        assert_eq!(nested.unwrap_err().kind(), ControlFailure::NestedScope);
    })
    .unwrap_err();
    assert!(!nested_ran);
    assert_eq!(error.kind(), ControlFailure::NestedScope);
    assert_eq!(error.invocations(), Some(0));
    // Overflow and missing internal state are tested privately in the engine.
}

#[test]
fn t7_unwind_cannot_leave_a_receipt_or_contaminate_next_scope() {
    let mut returned_receipt = None;
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        returned_receipt = Some(with_provider_observation_forbidden(|| {
            panic!("local unwind")
        }));
    }));
    assert!(panic.is_err());
    assert!(returned_receipt.is_none());
    assert_eq!(
        with_provider_observation_forbidden(|| ())
            .unwrap()
            .1
            .invocations(),
        0
    );
}

#[test]
fn t8_simultaneous_threads_have_independent_state() {
    let (ready_tx, ready_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let mut releases = Vec::new();
    let mut finishes = Vec::new();
    let mut workers = Vec::new();
    for count in [0_u64, 2] {
        let ready = ready_tx.clone();
        let done = done_tx.clone();
        let (release_tx, release_rx) = mpsc::channel();
        let (finish_tx, finish_rx) = mpsc::channel();
        releases.push(release_tx);
        finishes.push(finish_tx);
        workers.push(std::thread::spawn(move || {
            let temporary = tempfile::tempdir().unwrap();
            let result = with_provider_observation_forbidden(|| {
                ready.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(30)).unwrap();
                for _ in 0..count {
                    let _ = arm::observe(&temporary.path().join("missing.md"));
                }
                done.send(()).unwrap();
                finish_rx.recv_timeout(Duration::from_secs(30)).unwrap();
            });
            if count == 0 {
                assert_eq!(result.unwrap().1.invocations(), 0);
            } else {
                assert_eq!(result.unwrap_err().invocations(), Some(count));
            }
            assert!(with_provider_observation_forbidden(|| ()).is_ok());
        }));
    }
    for _ in 0..2 {
        ready_rx.recv_timeout(Duration::from_secs(30)).unwrap();
    }
    for release in releases {
        release.send(()).unwrap();
    }
    for _ in 0..2 {
        done_rx.recv_timeout(Duration::from_secs(30)).unwrap();
    }
    for finish in finishes {
        finish.send(()).unwrap();
    }
    for worker in workers {
        worker.join().unwrap();
    }
}

#[test]
fn t9_unarmed_injected_transport_preserves_existing_semantics() {
    let vector = vector(
        "github-authoritative-ref-monotonicity.observe",
        ".required-rules",
    );
    let fixture = fixture::materialize(&json!({
        "kind": "repository_plan",
        "steps": [{"op": "write_hex", "path": "binding.md", "hex": vector.fixture["binding"]["bytes_hex"]}],
        "invocation": {"type": "null", "value": null}
    })).unwrap();
    let mut calls = 0;
    let observed = control::observe_with_transport(
        &fixture.root().unwrap().join("binding.md"),
        |_| {
            calls += 1;
            Ok(control::HttpResponse {
            status: 200,
            body: br#"[{"type":"deletion","ruleset_id":7},{"type":"non_fast_forward","ruleset_id":7}]"#.to_vec(),
        })
        },
    );
    assert_eq!(observed.status, ArmStatus::Satisfied);
    assert_eq!(calls, 1);
    assert_eq!(
        ReferenceRustCandidate
            .execute(&request(
                &vector,
                "github-authoritative-ref-monotonicity.observe"
            ))
            .unwrap(),
        CandidateState::Observation(vector.expected_observation.observation())
    );
}

#[test]
fn t10_candidate_control_failure_is_harness_error_not_semantic_zero() {
    let vector = rgs_vector();
    let mut candidate_failed = false;
    let error = with_provider_observation_forbidden(|| {
        let outcome = ReferenceRustCandidate
            .execute(&request(&vector, "repository-governance-state.compose"));
        let failure = outcome.expect_err("nested candidate control must fail as a harness error");
        assert!(
            failure
                .to_string()
                .contains("ARM observation control NestedScope")
        );
        candidate_failed = true;
    })
    .unwrap_err();
    assert!(candidate_failed);
    assert_eq!(error.kind(), ControlFailure::NestedScope);
    assert!(
        ReferenceRustCandidate
            .execute(&request(&vector, "repository-governance-state.compose"))
            .is_ok()
    );
}
