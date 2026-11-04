use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Operation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    PureProjection,
    LocalObservation,
    ConsumerCommandExecution,
    ReadHostOutput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Precondition {
    RepositoryRead,
    RgsLoad,
    QueryResolution,
    CoherentWorkspace,
    RiDeclared,
    ProfileAdmission,
    Realization,
    EffectPlanAdmission,
    ReceiptBudget,
    ReceiptAccess,
    RetainedOutput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Applicability {
    Always,
    RgsLoad,
    QueryResolution,
    RiDeclared,
    RetainedOutput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authorization {
    None,
    RepositoryRead,
    EffectPlanAdmission,
    ReceiptAccess,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntry {
    pub operation: Operation,
    pub effects: Vec<Effect>,
    pub preconditions: Vec<Precondition>,
    pub applicability: Applicability,
    pub authorization: Authorization,
    // JSON Schema documents are intentionally untyped JSON, not declarations.
    // Runtime discovery mechanically derives these from the embedded contract.
    pub request_schema: Value,
    pub response_schema: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaDraft {
    #[serde(rename = "https://json-schema.org/draft/2020-12/schema")]
    Draft202012,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Discovery {
    pub operations: [CatalogEntry; 5],
    pub schema_draft: SchemaDraft,
}
