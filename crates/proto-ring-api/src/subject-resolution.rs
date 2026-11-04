use proto_ring_engine::repository_governance_state::RepositoryGovernanceState;

use crate::{
    ApiError, Identifier, ModelVersion, ProfileReference, SubjectProjection, Target, VersionOne,
    projections,
    query_error::{component_not_declared, projection_failure, subject_not_found},
};

pub(crate) fn capability(
    state: &RepositoryGovernanceState,
    id: &Identifier,
) -> Result<SubjectProjection, ApiError> {
    let model = &state.repository_governance_model;
    let value = model
        .capabilities
        .get(id.as_str())
        .ok_or_else(subject_not_found)?;
    Ok(SubjectProjection::Capability {
        id: id.clone(),
        model_version: ModelVersion::try_from(u64::from(model.model_version))
            .map_err(projection_failure)?,
        declaration: projections::capability(value).map_err(projection_failure)?,
    })
}

pub(crate) fn validation(
    state: &RepositoryGovernanceState,
    profile_reference: Option<&ProfileReference>,
    id: &Identifier,
) -> Result<SubjectProjection, ApiError> {
    let profile = state
        .repository_integrity
        .as_ref()
        .ok_or_else(component_not_declared)?;
    let value = profile
        .validations
        .get(id.as_str())
        .ok_or_else(subject_not_found)?;
    Ok(SubjectProjection::Validation {
        id: id.clone(),
        profile_ref: projections::required_profile_reference(profile_reference)
            .map_err(projection_failure)?
            .clone(),
        declaration: projections::validation(value).map_err(projection_failure)?,
    })
}

pub(crate) fn binding(
    state: &RepositoryGovernanceState,
    id: &Identifier,
) -> Result<SubjectProjection, ApiError> {
    let registry = &state.governance_bindings;
    let value = registry
        .bindings
        .get(id.as_str())
        .ok_or_else(subject_not_found)?;
    Ok(SubjectProjection::Binding {
        id: id.clone(),
        registry_source: Identifier::try_from(registry.source.as_str())
            .map_err(projection_failure)?,
        declaration: projections::binding(value).map_err(projection_failure)?,
    })
}

pub(crate) fn governed_object(
    state: &RepositoryGovernanceState,
    target: &Target,
) -> Result<SubjectProjection, ApiError> {
    let catalog = state
        .governed_objects
        .as_ref()
        .ok_or_else(component_not_declared)?;
    let value = catalog
        .interfaces
        .get(target.interface.as_str())
        .and_then(|interface| interface.objects.get(target.object.as_str()))
        .ok_or_else(subject_not_found)?;
    Ok(SubjectProjection::GovernedObject {
        target: target.clone(),
        declaration: projections::governed_object(value).map_err(projection_failure)?,
    })
}

pub(crate) fn projection(
    state: &RepositoryGovernanceState,
    id: &Identifier,
) -> Result<SubjectProjection, ApiError> {
    let registry = state
        .projection_registry
        .as_ref()
        .ok_or_else(component_not_declared)?;
    let value = registry
        .projections
        .get(id.as_str())
        .ok_or_else(subject_not_found)?;
    Ok(SubjectProjection::Projection {
        id: id.clone(),
        registry_model_version: VersionOne::try_from(u64::from(registry.model_version))
            .map_err(projection_failure)?,
        registry_authority: projections::authority_reference(
            &registry.authority.responsibility,
            &registry.authority.source,
        )
        .map_err(projection_failure)?,
        carrier: projections::carrier_path(&registry.carrier).map_err(projection_failure)?,
        declaration: projections::projection(value).map_err(projection_failure)?,
    })
}

pub(crate) fn evidence_requirement(
    state: &RepositoryGovernanceState,
    id: &Identifier,
) -> Result<SubjectProjection, ApiError> {
    let registry = state
        .evidence_requirements
        .as_ref()
        .ok_or_else(component_not_declared)?;
    let value = registry
        .requirements
        .get(id.as_str())
        .ok_or_else(subject_not_found)?;
    Ok(SubjectProjection::EvidenceRequirement {
        id: id.clone(),
        registry_model_version: VersionOne::try_from(u64::from(registry.model_version))
            .map_err(projection_failure)?,
        registry_authority: projections::authority_reference(
            &registry.authority.responsibility,
            &registry.authority.source,
        )
        .map_err(projection_failure)?,
        carrier: projections::carrier_path(&registry.carrier).map_err(projection_failure)?,
        declaration: projections::evidence_requirement(value).map_err(projection_failure)?,
    })
}
