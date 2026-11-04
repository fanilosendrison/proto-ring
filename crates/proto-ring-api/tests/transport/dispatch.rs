use super::{inspection_fixture, request, response};
use proto_ring_api::{
    ApiError, ErrorCode, FailureOutcome, FailureResponse, InspectSelector, Operation, Response,
    VersionOne, decode_request, dispatch, open,
};
use serde_json::json;

#[test]
fn rust_and_json_dispatch_preserve_full_inspection_explanation_and_error_semantics() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    assert!(!super::git_governance::native_units(observed.repository()).is_empty());
    let path = serde_json::to_value(proto_ring_api::NativePath::from_path(&fixture.root).unwrap())
        .unwrap();
    let input = json!({"repository":path,"selector":{"kind":"all"}});
    let bytes = request("inspect", input);
    let typed = decode_request(&bytes).unwrap();
    assert_eq!(
        serde_json::to_value(dispatch(typed)).unwrap(),
        response(&bytes)
    );
    assert_eq!(
        response(&bytes)["value"],
        serde_json::to_value(observed.inspect(InspectSelector::All {}).unwrap()).unwrap()
    );
    for query in [
        json!({"kind":"validation","id":"first"}),
        json!({"kind":"authority_role","responsibility":"missing","source":"missing"}),
    ] {
        let bytes = request("explain", json!({"repository":path,"query":query}));
        assert_eq!(
            response(&bytes)["value"],
            serde_json::to_value(
                observed
                    .explain(serde_json::from_value(query).unwrap())
                    .unwrap()
            )
            .unwrap()
        );
    }
    assert!(!fixture.root.join("EFFECT-MARKER").exists());
    let value = response(&request(
        "inspect",
        json!({"repository":path,"selector":{"kind":"binding","id":"missing"}}),
    ));
    assert_eq!(value["outcome"], "not_found");
    assert_eq!(value["error"]["code"], "subject_not_found");
}
#[test]
fn api03_branches_are_explicit_nonexecuting_failures_without_even_loading_a_repository() {
    for (op, input) in [
        (
            "check_repository_integrity",
            json!({"repository":{"kind":"utf8","value":"/nonexistent/must-not-load"},"expected_observation":null}),
        ),
        (
            "read_execution_output",
            json!({"receipt_id":"b".repeat(64),"validation_id":"arbitrary","instance_index":"18446744073709551615","stream":"stderr","offset":"18446744073709551615","max_bytes":65536}),
        ),
    ] {
        let value = response(&request(op, input));
        assert_eq!(value["operation"], op);
        assert_eq!(value["outcome"], "controlled_failure");
        assert_eq!(value["error"]["code"], "internal_failure");
        assert_eq!(value["error"]["stage"], json!(null));
        assert_eq!(value["error"]["attempt_id"], json!(null));
        assert_eq!(value["error"]["effects_possible"], false);
        assert!(
            value["error"]["detail"]
                .as_str()
                .unwrap()
                .contains("API-03")
        );
    }
}
#[test]
fn independent_exhaustive_failure_matrix_is_enforced_before_emission() {
    let operations = [
        None,
        Some(Operation::Discover),
        Some(Operation::Inspect),
        Some(Operation::Explain),
        Some(Operation::CheckRepositoryIntegrity),
        Some(Operation::ReadExecutionOutput),
    ];
    let outcomes = [
        FailureOutcome::Inapplicable,
        FailureOutcome::NotFound,
        FailureOutcome::AdmissionRefused,
        FailureOutcome::StaleObservation,
        FailureOutcome::IncomparableObservation,
        FailureOutcome::ControlledFailure,
        FailureOutcome::ExecutionIncomplete,
        FailureOutcome::ProtocolError,
    ];
    let codes = [
        ErrorCode::InvalidRequest,
        ErrorCode::UnsupportedVersion,
        ErrorCode::ResponseTooLarge,
        ErrorCode::GovernanceLoadFailed,
        ErrorCode::RepositoryIntegrityNotDeclared,
        ErrorCode::ComponentNotDeclared,
        ErrorCode::SubjectNotFound,
        ErrorCode::RepositoryNotAuthorized,
        ErrorCode::ProfileNotAuthorized,
        ErrorCode::ExecutionPlanNotAuthorized,
        ErrorCode::ExecutionGuaranteeUnavailable,
        ErrorCode::LocalContextInvalid,
        ErrorCode::StaleObservation,
        ErrorCode::IncomparableObservation,
        ErrorCode::OutputNotStarted,
        ErrorCode::OutputExpired,
        ErrorCode::OutputAccessRefused,
        ErrorCode::OutputUnavailable,
        ErrorCode::RangeNotRetained,
        ErrorCode::InvalidOutputRange,
        ErrorCode::ExecutionIncomplete,
        ErrorCode::HostGuardLostAfterStart,
        ErrorCode::ReceiptPersistenceFailed,
        ErrorCode::InternalFailure,
    ];
    for op in operations {
        for outcome in outcomes {
            for code in codes {
                use ErrorCode::*;
                use FailureOutcome as F;
                use Operation as O;
                let expected = matches!(
                    (op, outcome, code),
                    (_, F::ProtocolError, InvalidRequest | UnsupportedVersion)
                        | (Some(_), F::ControlledFailure, InternalFailure)
                        | (
                            Some(O::Inspect | O::Explain),
                            F::Inapplicable,
                            ComponentNotDeclared
                        )
                        | (
                            Some(O::Inspect | O::Explain | O::ReadExecutionOutput),
                            F::NotFound,
                            SubjectNotFound
                        )
                        | (
                            Some(O::Inspect | O::Explain | O::CheckRepositoryIntegrity),
                            F::AdmissionRefused,
                            RepositoryNotAuthorized
                        )
                        | (
                            Some(O::Inspect | O::Explain | O::CheckRepositoryIntegrity),
                            F::ControlledFailure,
                            GovernanceLoadFailed | ResponseTooLarge
                        )
                        | (
                            Some(O::CheckRepositoryIntegrity),
                            F::Inapplicable,
                            RepositoryIntegrityNotDeclared
                        )
                        | (
                            Some(O::CheckRepositoryIntegrity),
                            F::AdmissionRefused,
                            ProfileNotAuthorized
                                | ExecutionPlanNotAuthorized
                                | ExecutionGuaranteeUnavailable
                        )
                        | (
                            Some(O::CheckRepositoryIntegrity),
                            F::StaleObservation,
                            ErrorCode::StaleObservation
                        )
                        | (
                            Some(O::CheckRepositoryIntegrity),
                            F::IncomparableObservation,
                            ErrorCode::IncomparableObservation
                        )
                        | (
                            Some(O::CheckRepositoryIntegrity),
                            F::ControlledFailure,
                            LocalContextInvalid | ReceiptPersistenceFailed
                        )
                        | (
                            Some(O::CheckRepositoryIntegrity),
                            F::ExecutionIncomplete,
                            ErrorCode::ExecutionIncomplete
                                | HostGuardLostAfterStart
                                | ReceiptPersistenceFailed
                        )
                        | (
                            Some(O::ReadExecutionOutput),
                            F::ControlledFailure,
                            OutputNotStarted
                                | OutputExpired
                                | OutputAccessRefused
                                | OutputUnavailable
                                | RangeNotRetained
                                | InvalidOutputRange
                        )
                );
                let incomplete = outcome == F::ExecutionIncomplete;
                let response = Response::Failure(FailureResponse {
                    api_version: VersionOne::V1,
                    operation: op,
                    outcome,
                    error: ApiError {
                        code,
                        stage: None,
                        detail: None,
                        attempt_id: incomplete.then(|| "1".repeat(64).try_into().unwrap()),
                        effects_possible: Some(incomplete),
                    },
                });
                assert_eq!(
                    proto_ring_api::encode_response(&response).is_ok(),
                    expected,
                    "{op:?} {outcome:?} {code:?}"
                );
            }
        }
    }
}
