use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Identifier, ModelVersion, NativePath, StructuredValue};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub segments: Vec<Identifier>,
    pub declared_path: Identifier,
    pub resolved_target: NativePath,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityDeclaration {
    pub configuration: BTreeMap<String, StructuredValue>,
    pub routes: BTreeMap<Identifier, Route>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderId {
    #[serde(rename = "proto-ring")]
    ProtoRing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderBinding {
    pub capability: Identifier,
    pub route: Identifier,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub id: ProviderId,
    pub binding: ProviderBinding,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelProjection {
    pub model_version: ModelVersion,
    pub provider: Provider,
    pub capabilities: BTreeMap<Identifier, CapabilityDeclaration>,
}
