use proto_ring_engine::repository_integrity as engine;

use super::declaration_fields::{authority, id, ids, path, target};
use crate::{
    InstanceMode, IntegrityProfileProjection, ModelError, ProfileReference, ValidationCommand,
    ValidationDeclaration, ValidationInstances, ValidationSelector, VersionOne,
};

pub(crate) fn required_profile_reference(
    reference: Option<&ProfileReference>,
) -> Result<&ProfileReference, ModelError> {
    reference.ok_or(ModelError::Inconsistent(
        "loaded integrity profile has no retained public reference",
    ))
}

pub(crate) fn validation(
    value: &engine::ValidationDefinition,
) -> Result<ValidationDeclaration, ModelError> {
    let instances = match &value.instances {
        engine::ValidationInstances::Single => ValidationInstances::Single {},
        engine::ValidationInstances::RepositoryPaths { mode, selectors } => {
            ValidationInstances::RepositoryPaths {
                mode: match mode {
                    engine::RepositoryPathsMode::AppendAll => InstanceMode::AppendAll,
                    engine::RepositoryPathsMode::ForEach => InstanceMode::ForEach,
                },
                selectors: selectors
                    .iter()
                    .map(|selector| {
                        Ok(match selector.kind {
                            engine::SelectorKind::Path => ValidationSelector::Path {
                                value: id(&selector.value)?,
                            },
                            engine::SelectorKind::Glob => ValidationSelector::Glob {
                                value: id(&selector.value)?,
                            },
                        })
                    })
                    .collect::<Result<_, ModelError>>()?,
            }
        }
    };
    Ok(ValidationDeclaration {
        responsibility: id(&value.responsibility)?,
        prerequisites: ids(&value.prerequisites)?,
        instances,
        command: ValidationCommand {
            environment: id(&value.command.environment)?,
            arguments: value.command.arguments.clone(),
            undetermined_exit_codes: ids(&value.command.undetermined_exit_codes)?,
        },
        target: value.target.as_ref().map(target).transpose()?,
    })
}

pub(crate) fn integrity(
    value: &engine::ConsumerIntegrityProfile,
    reference: Option<&ProfileReference>,
) -> Result<IntegrityProfileProjection, ModelError> {
    Ok(IntegrityProfileProjection {
        model_version: VersionOne::try_from(u64::from(value.model_version))?,
        authority: authority(&value.authority.responsibility, &value.authority.source)?,
        environments: ids(&value.environments)?,
        continue_after_non_satisfied: value.continue_after_non_satisfied,
        validations: value
            .validations
            .iter()
            .map(|(key, value)| Ok((id(key)?, validation(value)?)))
            .collect::<Result<_, ModelError>>()?,
        order: ids(&value.order)?,
        profile_ref: required_profile_reference(reference)?.clone(),
        carrier: path(&value.carrier)?,
    })
}
