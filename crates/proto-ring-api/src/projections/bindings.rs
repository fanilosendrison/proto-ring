use proto_ring_engine::governance_bindings as engine;

use super::declaration_fields::{authority, id, path, target_ids};
use crate::{
    BindingDeclaration, BindingIdentity, BindingKind, BindingScope, BindingsProjection, ModelError,
    VersionOne,
};

pub(crate) fn binding(value: &engine::GovernanceBinding) -> Result<BindingDeclaration, ModelError> {
    let scope =
        match value.scope.kind {
            engine::ScopeKind::LogicalProvider => BindingScope::LogicalProvider {},
            engine::ScopeKind::Capability => BindingScope::Capability {
                capability: id(value.scope.capability.as_deref().ok_or(
                    ModelError::Inconsistent("loaded binding has no capability scope ID"),
                )?)?,
            },
            engine::ScopeKind::GovernedObject => BindingScope::GovernedObject {
                target: target_ids(
                    value
                        .scope
                        .interface
                        .as_deref()
                        .ok_or(ModelError::Inconsistent(
                            "loaded binding has no scope interface",
                        ))?,
                    value
                        .scope
                        .object
                        .as_deref()
                        .ok_or(ModelError::Inconsistent(
                            "loaded binding has no scope object",
                        ))?,
                )?,
            },
        };
    let identity = match &value.identity {
        engine::GovernanceBindingIdentity::ExecutableProvider(value) => {
            BindingIdentity::ExecutableProvider {
                repository: id(&value.repository)?,
                commit: id(&value.commit)?,
            }
        }
        engine::GovernanceBindingIdentity::GovernanceContract(value) => {
            BindingIdentity::GovernanceContract {
                repository: id(&value.repository)?,
                commit: id(&value.commit)?,
                path: id(&value.path)?,
            }
        }
    };
    Ok(BindingDeclaration {
        kind: match value.kind {
            engine::BindingKind::ExecutableProvider => BindingKind::ExecutableProvider,
            engine::BindingKind::GovernanceContract => BindingKind::GovernanceContract,
        },
        scope,
        identity,
        authority: authority(&value.authority.responsibility, &value.authority.source)?,
    })
}

pub(crate) fn bindings(
    value: &engine::GovernanceBindingRegistry,
) -> Result<BindingsProjection, ModelError> {
    Ok(BindingsProjection {
        model_version: VersionOne::try_from(u64::from(value.model_version))?,
        source: id(&value.source)?,
        bindings: value
            .bindings
            .iter()
            .map(|(key, value)| Ok((id(key)?, binding(value)?)))
            .collect::<Result<_, ModelError>>()?,
        carrier: path(&value.carrier)?,
    })
}
