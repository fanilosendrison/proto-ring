use proto_ring_api::{
    ApiError, ErrorCode, FailureOutcome, FailureResponse, Operation, RESPONSE_LIMIT, Response,
    VersionOne, encode_response,
};

fn failure(operation: Operation, outcome: FailureOutcome, code: ErrorCode) -> Response {
    Response::Failure(FailureResponse {
        api_version: VersionOne::V1,
        operation: Some(operation),
        outcome,
        error: ApiError {
            code,
            stage: None,
            detail: None,
            attempt_id: None,
            effects_possible: Some(false),
        },
    })
}

#[test]
fn invalid_code_outcome_pairs_are_rejected_with_valid_controls_for_each_operation() {
    for (operation, outcome, code) in [
        (
            Operation::Discover,
            FailureOutcome::NotFound,
            ErrorCode::SubjectNotFound,
        ),
        (
            Operation::Inspect,
            FailureOutcome::ControlledFailure,
            ErrorCode::InvalidRequest,
        ),
        (
            Operation::Explain,
            FailureOutcome::ProtocolError,
            ErrorCode::SubjectNotFound,
        ),
    ] {
        let error = encode_response(&failure(operation, outcome, code)).unwrap_err();
        assert_eq!(error.code, ErrorCode::InternalFailure);
        assert!(
            encode_response(&failure(
                operation,
                FailureOutcome::ControlledFailure,
                ErrorCode::InternalFailure
            ))
            .is_ok()
        );
    }
}

#[test]
fn overflow_validates_the_small_replacement_not_the_invalid_unbounded_original() {
    // This original is deliberately invalid AND too large. Once the byte budget
    // fails, only the replacement's canonical matrix may decide emission.
    let mut original = failure(
        Operation::Inspect,
        FailureOutcome::ControlledFailure,
        ErrorCode::SubjectNotFound,
    );
    let Response::Failure(value) = &mut original else {
        unreachable!()
    };
    value.error.detail = Some("x".repeat(RESPONSE_LIMIT));
    let encoded = encode_response(&original).unwrap();
    assert!(encoded.len() < 1024);
    let replacement: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(replacement["operation"], "inspect");
    assert_eq!(replacement["outcome"], "controlled_failure");
    assert_eq!(replacement["error"]["code"], "internal_failure");
    assert_eq!(replacement["error"]["stage"], serde_json::Value::Null);
    assert_eq!(replacement["error"]["attempt_id"], serde_json::Value::Null);
    assert_eq!(replacement["error"]["effects_possible"], false);
    assert!(replacement.get("value").is_none());
}
