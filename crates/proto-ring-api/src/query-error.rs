use crate::{ApiError, ErrorCode, ModelError};

pub(crate) fn component_not_declared() -> ApiError {
    failure(ErrorCode::ComponentNotDeclared, None)
}

pub(crate) fn subject_not_found() -> ApiError {
    failure(ErrorCode::SubjectNotFound, None)
}

pub(crate) fn projection_failure(error: ModelError) -> ApiError {
    failure(ErrorCode::InternalFailure, Some(error.to_string()))
}

fn failure(code: ErrorCode, detail: Option<String>) -> ApiError {
    ApiError {
        code,
        stage: None,
        detail,
        attempt_id: None,
        effects_possible: Some(false),
    }
}
