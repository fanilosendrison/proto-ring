use proto_ring_engine::{
    governance_routing::ResolvedGovernanceRoute, repository_governance_model as engine,
};

use super::{
    declaration_fields::{id, ids, path},
    structured_value::structured_value,
};
use crate::{
    CapabilityDeclaration, ModelError, ModelProjection, ModelVersion, Provider, ProviderBinding,
    ProviderId, Route,
};

pub(super) fn route(value: &ResolvedGovernanceRoute) -> Result<Route, ModelError> {
    Ok(Route {
        segments: ids(&value.route)?,
        declared_path: id(&value.declared_path)?,
        resolved_target: path(&value.target)?,
    })
}

pub(crate) fn capability(value: &engine::Capability) -> Result<CapabilityDeclaration, ModelError> {
    Ok(CapabilityDeclaration {
        // Configuration keys are opaque and may be empty.
        configuration: value
            .configuration
            .iter()
            .map(|(key, value)| Ok((key.clone(), structured_value(value)?)))
            .collect::<Result<_, ModelError>>()?,
        routes: value
            .routes
            .iter()
            .map(|(key, value)| Ok((id(key)?, route(value)?)))
            .collect::<Result<_, ModelError>>()?,
    })
}

pub(crate) fn model(
    value: &engine::RepositoryGovernanceModel,
) -> Result<ModelProjection, ModelError> {
    let provider_id = match value.provider.id.as_str() {
        "proto-ring" => ProviderId::ProtoRing,
        _ => return Err(ModelError::InvalidScalar("loaded provider ID")),
    };
    Ok(ModelProjection {
        model_version: ModelVersion::try_from(u64::from(value.model_version))?,
        provider: Provider {
            id: provider_id,
            binding: ProviderBinding {
                capability: id(&value.provider.binding.capability)?,
                route: id(&value.provider.binding.route)?,
            },
        },
        capabilities: value
            .capabilities
            .iter()
            .map(|(key, value)| Ok((id(key)?, capability(value)?)))
            .collect::<Result<_, ModelError>>()?,
    })
}
