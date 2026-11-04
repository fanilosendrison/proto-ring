use serde::{Deserialize, Serialize};

use crate::{
    ExplainQuery, HexDigest, Identifier, InspectSelector, NativePath, ObservationReference,
    ReadLimit, U64Decimal, VersionOne,
};

/// Typed syntax only, not a raw JSON decoder or execution authorization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Discover {
        api_version: VersionOne,
        input: DiscoverInput,
    },
    Inspect {
        api_version: VersionOne,
        input: InspectInput,
    },
    Explain {
        api_version: VersionOne,
        input: ExplainInput,
    },
    CheckRepositoryIntegrity {
        api_version: VersionOne,
        input: CheckInput,
    },
    ReadExecutionOutput {
        api_version: VersionOne,
        input: OutputInput,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoverInput {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectInput {
    pub repository: NativePath,
    pub selector: InspectSelector,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplainInput {
    pub repository: NativePath,
    pub query: ExplainQuery,
}

/// API-03 integration syntax; no execution service is provided by this model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckInput {
    pub repository: NativePath,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub expected_observation: Option<ObservationReference>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// API-03 integration syntax; constructing it does not read an artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputInput {
    pub receipt_id: HexDigest,
    pub validation_id: Identifier,
    pub instance_index: U64Decimal,
    pub stream: OutputStream,
    pub offset: U64Decimal,
    pub max_bytes: ReadLimit,
}
