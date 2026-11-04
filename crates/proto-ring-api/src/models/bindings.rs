use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{AuthorityReference, Identifier, NativePath, Target, VersionOne};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingKind {
    ExecutableProvider,
    GovernanceContract,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BindingScope {
    LogicalProvider {},
    Capability { capability: Identifier },
    GovernedObject { target: Target },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BindingIdentity {
    ExecutableProvider {
        repository: Identifier,
        commit: Identifier,
    },
    GovernanceContract {
        repository: Identifier,
        commit: Identifier,
        path: Identifier,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindingDeclaration {
    pub kind: BindingKind,
    pub scope: BindingScope,
    pub identity: BindingIdentity,
    pub authority: AuthorityReference,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindingsProjection {
    pub model_version: VersionOne,
    pub source: Identifier,
    pub bindings: BTreeMap<Identifier, BindingDeclaration>,
    pub carrier: NativePath,
}
