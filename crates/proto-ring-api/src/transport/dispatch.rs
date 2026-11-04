use super::{
    discover,
    failure::{error, failure, internal},
    request_decoder::validate,
};
use crate::{
    ApiError, ErrorCode, Operation, Request, Response, ResultTag, SuccessResponse, VersionOne, open,
};

/// One common typed dispatcher. API-03 branches do not open a repository,
/// consult local context, execute a command or read a receipt/output artifact.
pub fn dispatch(request: Request) -> Response {
    let operation = request.operation();
    match validate(&request).and_then(|()| perform(request)) {
        Ok(value) => Response::Success(value),
        Err(error) => failure(operation, error),
    }
}
fn perform(request: Request) -> Result<SuccessResponse, ApiError> {
    Ok(match request {
        Request::Discover { .. } => SuccessResponse::Discover {
            api_version: VersionOne::V1,
            outcome: ResultTag::Result,
            value: Box::new(discover()?),
        },
        Request::Inspect { input, .. } => {
            let path = input
                .repository
                .to_path_buf()
                .map_err(|_| error(ErrorCode::InvalidRequest, "invalid repository path"))?;
            SuccessResponse::Inspect {
                api_version: VersionOne::V1,
                outcome: ResultTag::Result,
                value: Box::new(open(path)?.inspect(input.selector)?),
            }
        }
        Request::Explain { input, .. } => {
            let path = input
                .repository
                .to_path_buf()
                .map_err(|_| error(ErrorCode::InvalidRequest, "invalid repository path"))?;
            SuccessResponse::Explain {
                api_version: VersionOne::V1,
                outcome: ResultTag::Result,
                value: Box::new(open(path)?.explain(input.query)?),
            }
        }
        Request::CheckRepositoryIntegrity { .. } | Request::ReadExecutionOutput { .. } => {
            return Err(internal(
                "API-03 service is missing in this API-02 development build",
            ));
        }
    })
}
impl Request {
    pub fn operation(&self) -> Operation {
        match self {
            Self::Discover { .. } => Operation::Discover,
            Self::Inspect { .. } => Operation::Inspect,
            Self::Explain { .. } => Operation::Explain,
            Self::CheckRepositoryIntegrity { .. } => Operation::CheckRepositoryIntegrity,
            Self::ReadExecutionOutput { .. } => Operation::ReadExecutionOutput,
        }
    }
}
