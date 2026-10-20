use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{BindingProjection, CompatStructuredValue, EvidenceRegistryValue, TargetValue};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ComposeProjection {
    #[default]
    Summary,
    Complete,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ComposeInput {
    pub repository: String,
    #[serde(default)]
    pub binding_ids: Vec<String>,
    #[serde(default)]
    pub projection: ComposeProjection,
}

#[derive(Serialize)]
pub(crate) struct ComposeValue {
    pub stage: &'static str,
    pub identity_present: bool,
    pub scope_paths: Vec<String>,
    pub components: ComponentPresence,
    pub bindings: BTreeMap<String, BindingProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Box<CompleteStateValue>>,
}

#[derive(Serialize)]
pub(crate) struct ComponentPresence {
    pub repository_governance_model: bool,
    pub governance_authority: bool,
    pub governed_objects: bool,
    pub governance_bindings: bool,
    pub repository_integrity: bool,
    pub projection_registry: bool,
    pub evidence_requirements: bool,
}

#[derive(Serialize)]
pub(crate) struct CompleteStateValue {
    pub repository_governance_model: RepositoryGovernanceModelValue,
    pub governance_authority: GovernanceAuthorityValue,
    pub governed_objects: Option<GovernedObjectsValue>,
    pub governance_bindings: GovernanceBindingsValue,
    pub repository_integrity: Option<RepositoryIntegrityProfileValue>,
    pub projection_registry: Option<ProjectionRegistryValue>,
    pub evidence_requirements: Option<EvidenceRegistryValue>,
}

#[derive(Serialize)]
pub(crate) struct RepositoryGovernanceModelValue {
    pub model_version: u8,
    pub provider: LogicalProviderValue,
    pub capabilities: BTreeMap<String, CapabilityValue>,
}

#[derive(Serialize)]
pub(crate) struct LogicalProviderValue {
    pub id: String,
    pub binding: ProviderBindingValue,
}

#[derive(Serialize)]
pub(crate) struct ProviderBindingValue {
    pub capability: String,
    pub route: String,
}

#[derive(Serialize)]
pub(crate) struct CapabilityValue {
    pub configuration: BTreeMap<String, CompatStructuredValue>,
    pub routes: BTreeMap<String, ResolvedRouteValue>,
}

#[derive(Serialize)]
pub(crate) struct ResolvedRouteValue {
    pub route: Vec<String>,
    pub declared_path: String,
    pub resolved_target: String,
}

#[derive(Serialize)]
pub(crate) struct GovernanceAuthorityValue {
    pub model_version: u8,
    pub sources: BTreeMap<String, GovernedSourceValue>,
    pub responsibilities: BTreeMap<String, GovernedResponsibilityValue>,
}

#[derive(Serialize)]
pub(crate) struct GovernedSourceValue {
    pub repository_target: Option<ResolvedRouteValue>,
}

#[derive(Serialize)]
pub(crate) struct GovernedResponsibilityValue {
    pub roles: BTreeMap<String, &'static str>,
    pub precedence: Vec<PrecedenceEdgeValue>,
}

#[derive(Serialize)]
pub(crate) struct PrecedenceEdgeValue {
    pub higher_source: String,
    pub lower_source: String,
}

#[derive(Serialize)]
pub(crate) struct GovernedObjectsValue {
    pub model_version: u8,
    pub interfaces: BTreeMap<String, GovernedInterfaceValue>,
}

#[derive(Serialize)]
pub(crate) struct GovernedInterfaceValue {
    pub objects: BTreeMap<String, GovernedObjectValue>,
}

#[derive(Serialize)]
pub(crate) struct GovernedObjectValue {
    pub responsibilities: Vec<String>,
    pub relations: Vec<GovernedRelationValue>,
}

#[derive(Serialize)]
pub(crate) struct GovernedRelationValue {
    pub relation: String,
    pub responsibility: String,
    pub target: TargetValue,
}

#[derive(Serialize)]
pub(crate) struct GovernanceBindingsValue {
    pub model_version: u8,
    pub source: String,
    pub bindings: BTreeMap<String, BindingProjection>,
}

#[derive(Serialize)]
pub(crate) struct RepositoryIntegrityProfileValue {
    pub model_version: u8,
    pub authority: ProfileAuthorityValue,
    pub environments: Vec<String>,
    pub continue_after_non_satisfied: bool,
    pub identity_present: bool,
    pub validations: BTreeMap<String, ValidationDefinitionValue>,
    pub order: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ProfileAuthorityValue {
    pub responsibility: String,
    pub source: String,
}

#[derive(Serialize)]
pub(crate) struct ValidationDefinitionValue {
    pub id: String,
    pub responsibility: String,
    pub prerequisites: Vec<String>,
    pub instances: ValidationInstancesValue,
    pub command: CommandBindingValue,
    pub target: Option<TargetValue>,
}

#[derive(Serialize)]
pub(crate) struct ValidationInstancesValue {
    pub kind: &'static str,
    pub mode: Option<&'static str>,
    pub selectors: Vec<RepositoryPathSelectorValue>,
}

#[derive(Serialize)]
pub(crate) struct RepositoryPathSelectorValue {
    pub kind: &'static str,
    pub value: String,
}

#[derive(Serialize)]
pub(crate) struct CommandBindingValue {
    pub environment: String,
    pub arguments: Vec<String>,
    pub undetermined_exit_codes: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ProjectionRegistryValue {
    pub model_version: u8,
    pub authority: ProfileAuthorityValue,
    pub projections: BTreeMap<String, ProjectionValue>,
}

#[derive(Serialize)]
pub(crate) struct ProjectionValue {
    pub id: String,
    pub responsibility: String,
    pub canonical_source: String,
    pub secondary_source: String,
    pub mode: &'static str,
    pub validation: String,
    pub generator_source: Option<String>,
    pub boundary_source: Option<String>,
    pub target: Option<TargetValue>,
    pub binding: Option<String>,
}
