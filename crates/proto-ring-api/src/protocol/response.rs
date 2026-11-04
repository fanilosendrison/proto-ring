use serde::{Deserialize, Serialize};

use crate::{ApiError, Discovery, ExplanationResult, Inspection, Operation, VersionOne};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultTag {
    Result,
}

/// Only nonexecuting success payloads exist in API-02. API-03 receipt/output
/// success models and their services are deliberately not represented as working.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum SuccessResponse {
    Discover {
        api_version: VersionOne,
        outcome: ResultTag,
        value: Box<Discovery>,
    },
    Inspect {
        api_version: VersionOne,
        outcome: ResultTag,
        value: Box<Inspection>,
    },
    Explain {
        api_version: VersionOne,
        outcome: ResultTag,
        value: Box<ExplanationResult>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureOutcome {
    Inapplicable,
    NotFound,
    AdmissionRefused,
    StaleObservation,
    IncomparableObservation,
    ControlledFailure,
    ExecutionIncomplete,
    ProtocolError,
}

/// Failure syntax. The common response encoder validates the canonical
/// operation/outcome/code matrix before these bytes can be emitted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureResponse {
    pub api_version: VersionOne,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub operation: Option<Operation>,
    pub outcome: FailureOutcome,
    pub error: ApiError,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Success(SuccessResponse),
    Failure(FailureResponse),
}
