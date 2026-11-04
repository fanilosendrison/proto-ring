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
    let bytes = match bounded_buffer::serialize(response, RESPONSE_LIMIT) {
        Ok(bytes) => bytes,
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
            bounded_buffer::serialize(&failure(operation, error), RESPONSE_LIMIT)
                .map_err(|_| internal("cannot encode bounded failure"))?
        }
        Err(EncodingFailure::Internal) => return Err(internal("response serialization failed")),
    };
    if !contract_schema::valid_response(&bytes)? {
        return Err(internal(
            "response violates the canonical operation/outcome/code schema",
        ));
    }
    Ok(bytes)
}
