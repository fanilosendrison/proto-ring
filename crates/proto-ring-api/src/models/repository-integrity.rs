use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{AuthorityReference, Identifier, NativePath, ProfileReference, Target, VersionOne};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValidationSelector {
    Path { value: Identifier },
    Glob { value: Identifier },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceMode {
    AppendAll,
    ForEach,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValidationInstances {
    Single {},
    RepositoryPaths {
        mode: InstanceMode,
        selectors: Vec<ValidationSelector>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationCommand {
    pub environment: Identifier,
    pub arguments: Vec<String>,
    pub undetermined_exit_codes: Vec<Identifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationDeclaration {
    pub responsibility: Identifier,
    pub prerequisites: Vec<Identifier>,
    pub instances: ValidationInstances,
    pub command: ValidationCommand,
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub target: Option<Target>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrityProfileProjection {
    pub model_version: VersionOne,
    pub authority: AuthorityReference,
    pub environments: Vec<Identifier>,
    pub continue_after_non_satisfied: bool,
    pub validations: BTreeMap<Identifier, ValidationDeclaration>,
    pub order: Vec<Identifier>,
    pub profile_ref: ProfileReference,
    pub carrier: NativePath,
}
