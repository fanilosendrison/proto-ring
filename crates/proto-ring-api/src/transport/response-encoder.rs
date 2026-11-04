use super::{
    RESPONSE_LIMIT,
    bounded_buffer::{self, EncodingFailure},
    contract_schema,
    failure::{error, failure, internal},
};
use crate::{ApiError, ErrorCode, Operation, Response, SuccessResponse};

fn operation(response: &Response) -> Option<Operation> {
    match response {
        Response::Success(SuccessResponse::Discover { .. }) => Some(Operation::Discover),
        Response::Success(SuccessResponse::Inspect { .. }) => Some(Operation::Inspect),
        Response::Success(SuccessResponse::Explain { .. }) => Some(Operation::Explain),
        Response::Failure(value) => value.operation,
    }
}
/// Serialize within the byte budget before any stdout write. Overflow discards
/// the entire candidate success, never a declaration subset or truncated JSON.
pub fn encode_response(response: &Response) -> Result<Vec<u8>, ApiError> {
    match bounded_buffer::serialize(response, RESPONSE_LIMIT) {
        Ok(bytes) => validated(response, bytes),
        Err(EncodingFailure::Limit) => {
            let operation = operation(response)
                .ok_or_else(|| internal("unassigned error response exceeds byte budget"))?;
            let error = if matches!(
                response,
                Response::Success(
                    SuccessResponse::Inspect { .. } | SuccessResponse::Explain { .. }
                )
            ) {
                error(
                    ErrorCode::ResponseTooLarge,
                    "complete observation exceeds the response byte limit",
                )
            } else {
                internal("response exceeds the byte limit")
            };
            let replacement = failure(operation, error);
            let bytes = bounded_buffer::serialize(&replacement, RESPONSE_LIMIT)
                .map_err(|_| internal("cannot encode bounded failure"))?;
            validated(&replacement, bytes)
        }
        Err(EncodingFailure::Internal) => Err(internal("response serialization failed")),
    }
}

fn validated(response: &Response, bytes: Vec<u8>) -> Result<Vec<u8>, ApiError> {
    if !contract_schema::valid_response(response)? {
        return Err(internal(
            "response violates the canonical operation/outcome/code schema",
        ));
    }
    Ok(bytes)
}
