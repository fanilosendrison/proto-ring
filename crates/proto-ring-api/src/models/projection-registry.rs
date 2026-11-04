use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{AuthorityReference, Identifier, NativePath, Target, VersionOne};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionMode {
    Reference,
    Generated,
    MechanicallyValidatedMaintained,
    BoundedHistoricalSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionDeclaration {
    pub responsibility: Identifier,
    pub canonical_source: Identifier,
    pub secondary_source: Identifier,
    pub mode: ProjectionMode,
    pub validation: Identifier,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub generator_source: Option<Identifier>,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub boundary_source: Option<Identifier>,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub target: Option<Target>,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub binding: Option<Identifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionsProjection {
    pub model_version: VersionOne,
    pub authority: AuthorityReference,
    pub projections: BTreeMap<Identifier, ProjectionDeclaration>,
    pub carrier: NativePath,
}
