use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Identifier, NativePath, Route, VersionOne};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityReference {
    pub responsibility: Identifier,
    pub source: Identifier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityRole {
    Authority,
    SecondaryRepresentation,
    NonAuthoritative,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoritySource {
    #[serde(deserialize_with = "crate::required_nullable::deserialize")]
    pub repository_target: Option<Route>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Precedence {
    pub higher_source: Identifier,
    pub lower_source: Identifier,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Responsibility {
    pub roles: BTreeMap<Identifier, AuthorityRole>,
    pub precedence: Vec<Precedence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityProjection {
    pub model_version: VersionOne,
    pub sources: BTreeMap<Identifier, AuthoritySource>,
    pub responsibilities: BTreeMap<Identifier, Responsibility>,
    pub carrier: NativePath,
}
