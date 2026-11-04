use proto_ring_engine::projection_registry as engine;

use super::declaration_fields::{authority, id, path, target};
use crate::{ModelError, ProjectionDeclaration, ProjectionMode, ProjectionsProjection, VersionOne};

pub(crate) fn projection(value: &engine::Projection) -> Result<ProjectionDeclaration, ModelError> {
    Ok(ProjectionDeclaration {
        responsibility: id(&value.responsibility)?,
        canonical_source: id(&value.canonical_source)?,
        secondary_source: id(&value.secondary_source)?,
        mode: match value.mode {
            engine::ProjectionMode::Reference => ProjectionMode::Reference,
            engine::ProjectionMode::Generated => ProjectionMode::Generated,
            engine::ProjectionMode::MechanicallyValidatedMaintained => {
                ProjectionMode::MechanicallyValidatedMaintained
            }
            engine::ProjectionMode::BoundedHistoricalSnapshot => {
                ProjectionMode::BoundedHistoricalSnapshot
            }
        },
        validation: id(&value.validation)?,
        generator_source: value.generator_source.as_deref().map(id).transpose()?,
        boundary_source: value.boundary_source.as_deref().map(id).transpose()?,
        target: value.target.as_ref().map(target).transpose()?,
        binding: value.binding.as_deref().map(id).transpose()?,
    })
}

pub(crate) fn projections(
    value: &engine::ProjectionRegistry,
) -> Result<ProjectionsProjection, ModelError> {
    Ok(ProjectionsProjection {
        model_version: VersionOne::try_from(u64::from(value.model_version))?,
        authority: authority(&value.authority.responsibility, &value.authority.source)?,
        projections: value
            .projections
            .iter()
            .map(|(key, value)| Ok((id(key)?, projection(value)?)))
            .collect::<Result<_, ModelError>>()?,
        carrier: path(&value.carrier)?,
    })
}
