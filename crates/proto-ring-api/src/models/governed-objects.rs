use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Identifier, NativePath, VersionOne};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub interface: Identifier,
    pub object: Identifier,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectRelation {
    pub relation: Identifier,
    pub responsibility: Identifier,
    pub target: Target,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GovernedObjectDeclaration {
    pub responsibilities: Vec<Identifier>,
    pub relations: Vec<ObjectRelation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GovernedInterface {
    pub objects: BTreeMap<Identifier, GovernedObjectDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectsProjection {
    pub model_version: VersionOne,
    pub interfaces: BTreeMap<Identifier, GovernedInterface>,
    pub carrier: NativePath,
}
