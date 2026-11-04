use proto_ring_engine::{
    governance_authority::{self, ResponsibilityLookup, RoleLookup, SourceRole},
    repository_governance_state::RepositoryGovernanceState,
};

use crate::{
    ApiError, ExplainQuery, Explanation, ExplanationResult, ObservationProvenance,
    PrecedenceRelation, ProfileReference, RoleConsultation, subject_resolution,
};

/// Only adapt canonical consultation results and retained declaration views.
/// In particular, do not derive an inverse relation or elect an authority.
pub(crate) fn explain(
    state: &RepositoryGovernanceState,
    provenance: &ObservationProvenance,
    profile_reference: Option<&ProfileReference>,
    query: ExplainQuery,
) -> Result<ExplanationResult, ApiError> {
    let explanation = match query {
        ExplainQuery::AuthorityRole {
            responsibility,
            source,
        } => {
            let role = match governance_authority::role_of(
                &state.governance_authority,
                responsibility.as_str(),
                source.as_str(),
            ) {
                RoleLookup::Role(SourceRole::Authority) => RoleConsultation::Authority,
                RoleLookup::Role(SourceRole::SecondaryRepresentation) => {
                    RoleConsultation::SecondaryRepresentation
                }
                RoleLookup::Role(SourceRole::NonAuthoritative) => {
                    RoleConsultation::NonAuthoritative
                }
                RoleLookup::UnknownResponsibility => RoleConsultation::UnknownResponsibility,
                RoleLookup::Undeclared => RoleConsultation::Undeclared,
            };
            Explanation::AuthorityRole {
                responsibility,
                source,
                role,
            }
        }
        ExplainQuery::AuthorityPrecedence {
            responsibility,
            higher_source,
            lower_source,
        } => {
            let relation = match governance_authority::outranks(
                &state.governance_authority,
                responsibility.as_str(),
                higher_source.as_str(),
                lower_source.as_str(),
            ) {
                ResponsibilityLookup::Value(true) => PrecedenceRelation::Outranks,
                ResponsibilityLookup::Value(false) => PrecedenceRelation::DoesNotOutrank,
                ResponsibilityLookup::UnknownResponsibility => {
                    PrecedenceRelation::UnknownResponsibility
                }
            };
            Explanation::AuthorityPrecedence {
                responsibility,
                higher_source,
                lower_source,
                relation,
            }
        }
        ExplainQuery::Capability { id } => {
            Explanation::Declaration(Box::new(subject_resolution::capability(state, &id)?))
        }
        ExplainQuery::Validation { id } => Explanation::Declaration(Box::new(
            subject_resolution::validation(state, profile_reference, &id)?,
        )),
        ExplainQuery::Binding { id } => {
            Explanation::Declaration(Box::new(subject_resolution::binding(state, &id)?))
        }
        ExplainQuery::GovernedObject { target } => Explanation::Declaration(Box::new(
            subject_resolution::governed_object(state, &target)?,
        )),
        ExplainQuery::Projection { id } => {
            Explanation::Declaration(Box::new(subject_resolution::projection(state, &id)?))
        }
        ExplainQuery::EvidenceRequirement { id } => Explanation::Declaration(Box::new(
            subject_resolution::evidence_requirement(state, &id)?,
        )),
    };
    Ok(ExplanationResult::new(provenance.clone(), explanation))
}
