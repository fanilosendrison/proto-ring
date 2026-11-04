use serde::{Deserialize, Serialize};

use crate::{
    AuthorityReference, BindingDeclaration, CapabilityDeclaration, EvidenceRequirementDeclaration,
    ExplainQuery, GovernedObjectDeclaration, Identifier, ModelVersion, NativePath,
    ProfileReference, ProjectionDeclaration, Target, ValidationDeclaration, VersionOne,
};

/// Shared complete declaration view for inspection and declaration explanations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SubjectProjection {
    Capability {
        id: Identifier,
        model_version: ModelVersion,
        declaration: CapabilityDeclaration,
    },
    Validation {
        id: Identifier,
        profile_ref: ProfileReference,
        declaration: ValidationDeclaration,
    },
    Binding {
        id: Identifier,
        registry_source: Identifier,
        declaration: BindingDeclaration,
    },
    GovernedObject {
        target: Target,
        declaration: GovernedObjectDeclaration,
    },
    Projection {
        id: Identifier,
        registry_model_version: VersionOne,
        registry_authority: AuthorityReference,
        carrier: NativePath,
        declaration: ProjectionDeclaration,
    },
    EvidenceRequirement {
        id: Identifier,
        registry_model_version: VersionOne,
        registry_authority: AuthorityReference,
        carrier: NativePath,
        declaration: EvidenceRequirementDeclaration,
    },
}

impl SubjectProjection {
    /// The subject's exact query; this does not resolve or inspect governance.
    pub fn query(&self) -> ExplainQuery {
        match self {
            Self::Capability { id, .. } => ExplainQuery::Capability { id: id.clone() },
            Self::Validation { id, .. } => ExplainQuery::Validation { id: id.clone() },
            Self::Binding { id, .. } => ExplainQuery::Binding { id: id.clone() },
            Self::GovernedObject { target, .. } => ExplainQuery::GovernedObject {
                target: target.clone(),
            },
            Self::Projection { id, .. } => ExplainQuery::Projection { id: id.clone() },
            Self::EvidenceRequirement { id, .. } => {
                ExplainQuery::EvidenceRequirement { id: id.clone() }
            }
        }
    }
}
