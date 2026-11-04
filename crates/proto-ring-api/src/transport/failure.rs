use super::{RESPONSE_LIMIT, bounded_buffer, contract_schema};
use crate::{
    ApiError, ErrorCode, FailureOutcome, FailureResponse, Operation, Response, VersionOne,
};

pub(super) fn error(code: ErrorCode, detail: &str) -> ApiError {
    ApiError {
        code,
        stage: None,
        detail: Some(detail.into()),
        attempt_id: None,
        effects_possible: Some(false),
    }
}
pub(super) fn internal(detail: &str) -> ApiError {
    error(ErrorCode::InternalFailure, detail)
}
pub(super) fn protocol(operation: Option<Operation>, code: ErrorCode) -> FailureResponse {
    FailureResponse {
        api_version: VersionOne::V1,
        operation,
        outcome: FailureOutcome::ProtocolError,
        error: error(code, "request does not satisfy the API v1 boundary"),
    }
}

/// The schema owns the permitted outcome/code matrix, including metadata guards.
/// Validate candidate envelopes rather than maintaining a parallel matrix.
pub(super) fn failure(operation: Operation, error: ApiError) -> Response {
    if let Ok(api) = contract_schema::canonical()
        && let Some(branches) = api["$defs"]["response"]["oneOf"].as_array()
        && let Ok(operation_value) = serde_json::to_value(operation)
    {
        for branch in branches {
            if branch["properties"]["operation"]["const"] != operation_value {
                continue;
            }
            let Ok(outcome) =
                serde_json::from_value(branch["properties"]["outcome"]["const"].clone())
            else {
                continue;
            };
            let candidate = Response::Failure(FailureResponse {
                api_version: VersionOne::V1,
                operation: Some(operation),
                outcome,
                error: error.clone(),
            });
            if bounded_buffer::serialize(&candidate, RESPONSE_LIMIT).is_ok()
                && contract_schema::valid_response(&candidate).unwrap_or(false)
            {
                return candidate;
            }
        }
    }
    // If embedded-contract processing itself fails, final encoding must still
    // validate this fallback. It will return an internal error rather than emit
    // any envelope not permitted by the canonical schema.
    Response::Failure(FailureResponse {
        api_version: VersionOne::V1,
        operation: Some(operation),
        outcome: FailureOutcome::ControlledFailure,
        error: internal("cannot construct a canonical error response"),
    })
}
