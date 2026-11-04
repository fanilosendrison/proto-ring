use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{AuthorityReference, Identifier, NativePath, Target, VersionOne};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceInstanceKind {
    Single,
    Source,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceInstances {
    pub kind: EvidenceInstanceKind,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub source: Option<Identifier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClassesKind {
    Explicit,
    Source,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceClasses {
    pub kind: EvidenceClassesKind,
    pub classes: Vec<Identifier>,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub source: Option<Identifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceContext {
    pub required: bool,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub source: Option<Identifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRequirementDeclaration {
    pub responsibility: Identifier,
    pub instances: EvidenceInstances,
    pub evidence_classes: EvidenceClasses,
    pub subject_source: Identifier,
    pub context: EvidenceContext,
    pub candidate_source: Identifier,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub target: Option<Target>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceProjection {
    pub model_version: VersionOne,
    pub authority: AuthorityReference,
    pub requirements: BTreeMap<Identifier, EvidenceRequirementDeclaration>,
    pub carrier: NativePath,
}
