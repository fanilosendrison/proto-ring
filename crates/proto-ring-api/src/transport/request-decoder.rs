use super::{
    REQUEST_LIMIT, contract_schema,
    failure::{error, protocol},
    strict_json,
};
use crate::{ApiError, ErrorCode, FailureOutcome, FailureResponse, Request, VersionOne};

/// Strict wire parsing before any typed dispatch, governance load or effects.
pub fn decode_request(bytes: &[u8]) -> Result<Request, FailureResponse> {
    let (root, complete) = strict_json::root(&bytes[..bytes.len().min(REQUEST_LIMIT + 1)]);
    let invalid = || protocol(root.operation, ErrorCode::InvalidRequest);
    if bytes.len() > REQUEST_LIMIT
        || !complete
        || root.duplicate
        || std::str::from_utf8(bytes).is_err()
    {
        return Err(invalid());
    }
    if root.fields.iter().any(|(key, raw)| {
        !(key == "api_version" && integer_token(raw.get())) && !strict_json::unique(raw.get())
    }) {
        return Err(invalid());
    }
    if let Some((_, raw)) = root.fields.iter().find(|(key, _)| key == "api_version") {
        let text = raw.get();
        if text != "1" {
            if integer_token(text) {
                return Err(protocol(root.operation, ErrorCode::UnsupportedVersion));
            }
            return Err(invalid());
        }
    }
    let operation = root.operation.ok_or_else(invalid)?;
    match contract_schema::valid_request(bytes) {
        Ok(true) => {}
        Ok(false) => return Err(invalid()),
        Err(error) => {
            return Err(FailureResponse {
                api_version: VersionOne::V1,
                operation: Some(operation),
                outcome: FailureOutcome::ControlledFailure,
                error,
            });
        }
    }
    let request: Request = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    validate(&request).map_err(|_| invalid())?;
    Ok(request)
}

// RawValue already validated JSON number grammar. This lexical check avoids
// converting an unsupported integer version through any bounded numeric type.
fn integer_token(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit())
}

pub(super) fn validate(request: &Request) -> Result<(), ApiError> {
    let path = match request {
        Request::Inspect { input, .. } => Some(&input.repository),
        Request::Explain { input, .. } => Some(&input.repository),
        Request::CheckRepositoryIntegrity { input, .. } => Some(&input.repository),
        _ => None,
    };
    if let Some(path) = path {
        path.to_path_buf().map_err(|_| {
            error(
                ErrorCode::InvalidRequest,
                "path is not representable on this platform",
            )
        })?;
    }
    Ok(())
}
