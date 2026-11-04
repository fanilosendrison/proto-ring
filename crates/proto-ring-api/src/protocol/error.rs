use std::fmt;

use serde::{Deserialize, Serialize};

use crate::HexDigest;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceStage {
    RepositoryState,
    Bootstrap,
    RepositoryGovernanceModel,
    GovernanceAuthority,
    GovernedObjects,
    GovernanceBindings,
    RepositoryIntegrity,
    ProjectionIntegrity,
    EvidenceRequirements,
    Composition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    UnsupportedVersion,
    ResponseTooLarge,
    GovernanceLoadFailed,
    RepositoryIntegrityNotDeclared,
    ComponentNotDeclared,
    SubjectNotFound,
    RepositoryNotAuthorized,
    ProfileNotAuthorized,
    ExecutionPlanNotAuthorized,
    ExecutionGuaranteeUnavailable,
    LocalContextInvalid,
    StaleObservation,
    IncomparableObservation,
    OutputNotStarted,
    OutputExpired,
    OutputAccessRefused,
    OutputUnavailable,
    RangeNotRetained,
    InvalidOutputRange,
    ExecutionIncomplete,
    HostGuardLostAfterStart,
    ReceiptPersistenceFailed,
    InternalFailure,
}

/// Wire diagnostic data. Detail is non-normative and must never be classified
/// by parsing its text. Operation/outcome attribution belongs to the dispatcher.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub code: ErrorCode,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub stage: Option<GovernanceStage>,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub detail: Option<String>,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub attempt_id: Option<HexDigest>,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub effects_possible: Option<bool>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "public API error: {:?}", self.code)
    }
}

impl std::error::Error for ApiError {}
