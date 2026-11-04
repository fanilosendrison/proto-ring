use proto_ring_engine::repository_governance_state::RepositoryGovernanceState;

use crate::{
    ApiError, ComponentProjection, Components, ExplicitNull, InspectSelector, Inspection,
    InspectionResult, ModelError, ObservationProvenance, Presence, Present, ProfileReference,
    SubjectProjection, projections,
    query_error::{component_not_declared, projection_failure},
    subject_resolution,
};

pub(crate) fn inspect(
    state: &RepositoryGovernanceState,
    provenance: &ObservationProvenance,
    profile_reference: Option<&ProfileReference>,
    selector: InspectSelector,
) -> Result<Inspection, ApiError> {
    let result = match selector {
        InspectSelector::All {} => all(state, profile_reference)
            .map(|value| InspectionResult::All(Box::new(value)))
            .map_err(projection_failure),
        InspectSelector::Bootstrap {} => {
            component(projections::bootstrap(state).map(ComponentProjection::Bootstrap))
        }
        InspectSelector::Model {} => component(
            projections::model(&state.repository_governance_model).map(ComponentProjection::Model),
        ),
        InspectSelector::Authority {} => component(
            projections::authority(&state.governance_authority).map(ComponentProjection::Authority),
        ),
        InspectSelector::Bindings {} => component(
            projections::bindings(&state.governance_bindings).map(ComponentProjection::Bindings),
        ),
        InspectSelector::GovernedObjects {} => component(
            projections::objects(
                state
                    .governed_objects
                    .as_ref()
                    .ok_or_else(component_not_declared)?,
            )
            .map(ComponentProjection::GovernedObjects),
        ),
        InspectSelector::RepositoryIntegrity {} => component(
            projections::integrity(
                state
                    .repository_integrity
                    .as_ref()
                    .ok_or_else(component_not_declared)?,
                profile_reference,
            )
            .map(ComponentProjection::RepositoryIntegrity),
        ),
        InspectSelector::Projections {} => component(
            projections::projections(
                state
                    .projection_registry
                    .as_ref()
                    .ok_or_else(component_not_declared)?,
            )
            .map(ComponentProjection::Projections),
        ),
        InspectSelector::EvidenceRequirements {} => component(
            projections::evidence(
                state
                    .evidence_requirements
                    .as_ref()
                    .ok_or_else(component_not_declared)?,
            )
            .map(ComponentProjection::EvidenceRequirements),
        ),
        InspectSelector::Capability { id } => subject(subject_resolution::capability(state, &id)),
        InspectSelector::Validation { id } => subject(subject_resolution::validation(
            state,
            profile_reference,
            &id,
        )),
        InspectSelector::Binding { id } => subject(subject_resolution::binding(state, &id)),
        InspectSelector::GovernedObject { target } => {
            subject(subject_resolution::governed_object(state, &target))
        }
    }?;
    Inspection::new(provenance.clone(), result).map_err(projection_failure)
}

fn component(value: Result<ComponentProjection, ModelError>) -> Result<InspectionResult, ApiError> {
    value
        .map(|value| InspectionResult::Component(Box::new(value)))
        .map_err(projection_failure)
}

fn subject(value: Result<SubjectProjection, ApiError>) -> Result<InspectionResult, ApiError> {
    value.map(|value| InspectionResult::Subject(Box::new(value)))
}

fn optional<T>(value: Option<Result<T, ModelError>>) -> Result<Presence<T>, ModelError> {
    Ok(match value.transpose()? {
        Some(value) => Presence::Present(value),
        None => Presence::Absent(ExplicitNull),
    })
}

fn all(
    state: &RepositoryGovernanceState,
    profile_reference: Option<&ProfileReference>,
) -> Result<Components, ModelError> {
    Ok(Components {
        bootstrap: Present::new(projections::bootstrap(state)?),
        model: Present::new(projections::model(&state.repository_governance_model)?),
        authority: Present::new(projections::authority(&state.governance_authority)?),
        governed_objects: optional(state.governed_objects.as_ref().map(projections::objects))?,
        bindings: Present::new(projections::bindings(&state.governance_bindings)?),
        repository_integrity: optional(
            state
                .repository_integrity
                .as_ref()
                .map(|profile| projections::integrity(profile, profile_reference)),
        )?,
        projections: optional(
            state
                .projection_registry
                .as_ref()
                .map(projections::projections),
        )?,
        evidence_requirements: optional(
            state
                .evidence_requirements
                .as_ref()
                .map(projections::evidence),
        )?,
    })
}
