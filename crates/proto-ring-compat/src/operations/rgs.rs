use std::path::Path;

use proto_ring_engine::governance_authority::GovernanceAuthorityProfile;
use proto_ring_engine::governance_bindings::GovernanceBindingRegistry;
use proto_ring_engine::governance_routing::ResolvedGovernanceRoute;
use proto_ring_engine::governed_objects::{GovernedObjectCatalog, GovernedObjectRef};
use proto_ring_engine::projection_registry::ProjectionRegistry;
use proto_ring_engine::repository_governance_model::RepositoryGovernanceModel;
use proto_ring_engine::repository_governance_state::{
    self, RepositoryGovernanceStage, RepositoryGovernanceState,
};
use proto_ring_engine::repository_integrity::{ConsumerIntegrityProfile, ValidationInstances};

use super::{
    InternalFailure, portable_path, project_binding, project_registry, project_requirement,
    project_structured, rgs_failure,
};
use crate::protocol::*;

pub(super) fn compose(input: ComposeInput) -> Result<Response, InternalFailure> {
    let state = match repository_governance_state::load(Path::new(&input.repository)) {
        Ok(state) => state,
        Err(error) => {
            return Ok(rgs_failure("repository-governance-state.compose", error));
        }
    };
    let scope_paths = state
        .observed_state
        .scope_paths
        .iter()
        .map(|path| path.to_str().map(str::to_owned).ok_or(InternalFailure))
        .collect::<Result<Vec<_>, _>>()?;
    let bindings = input
        .binding_ids
        .into_iter()
        .map(|id| {
            let value = state
                .governance_bindings
                .bindings
                .get(&id)
                .map(project_binding)
                .unwrap_or(BindingProjection::Absent { present: false });
            (id, value)
        })
        .collect();
    let components = ComponentPresence {
        repository_governance_model: true,
        governance_authority: true,
        governed_objects: state.governed_objects.is_some(),
        governance_bindings: true,
        repository_integrity: state.repository_integrity.is_some(),
        projection_registry: state.projection_registry.is_some(),
        evidence_requirements: state.evidence_requirements.is_some(),
    };
    let complete = (input.projection == ComposeProjection::Complete)
        .then(|| project_complete_state(&state))
        .transpose()?
        .map(Box::new);
    Ok(Response::result(
        "repository-governance-state.compose",
        ResultValue::Compose(ComposeValue {
            stage: RepositoryGovernanceStage::Composition.serialized(),
            identity_present: !state.observed_state.identity.0.is_empty(),
            scope_paths,
            components,
            bindings,
            state: complete,
        }),
    ))
}

fn project_complete_state(
    state: &RepositoryGovernanceState,
) -> Result<CompleteStateValue, InternalFailure> {
    let evidence_requirements = state.evidence_requirements.as_ref().map(|registry| {
        let requirements = registry
            .requirements
            .iter()
            .map(|(id, requirement)| (id.clone(), project_requirement(requirement)))
            .collect();
        project_registry(registry, requirements)
    });
    Ok(CompleteStateValue {
        repository_governance_model: project_model(&state.repository_governance_model)?,
        governance_authority: project_authority(&state.governance_authority)?,
        governed_objects: state.governed_objects.as_ref().map(project_objects),
        governance_bindings: project_bindings(&state.governance_bindings),
        repository_integrity: state.repository_integrity.as_ref().map(project_integrity),
        projection_registry: state.projection_registry.as_ref().map(project_projections),
        evidence_requirements,
    })
}

fn project_model(
    model: &RepositoryGovernanceModel,
) -> Result<RepositoryGovernanceModelValue, InternalFailure> {
    let capabilities = model
        .capabilities
        .iter()
        .map(|(id, capability)| {
            let configuration = capability
                .configuration
                .iter()
                .map(|(key, value)| (key.clone(), project_structured(value)))
                .collect();
            let routes = capability
                .routes
                .iter()
                .map(|(id, route)| Ok((id.clone(), project_route(&model.repository_root, route)?)))
                .collect::<Result<_, InternalFailure>>()?;
            Ok((
                id.clone(),
                CapabilityValue {
                    configuration,
                    routes,
                },
            ))
        })
        .collect::<Result<_, InternalFailure>>()?;
    Ok(RepositoryGovernanceModelValue {
        model_version: model.model_version,
        provider: LogicalProviderValue {
            id: model.provider.id.clone(),
            binding: ProviderBindingValue {
                capability: model.provider.binding.capability.clone(),
                route: model.provider.binding.route.clone(),
            },
        },
        capabilities,
    })
}

fn project_authority(
    authority: &GovernanceAuthorityProfile,
) -> Result<GovernanceAuthorityValue, InternalFailure> {
    let sources = authority
        .sources
        .iter()
        .map(|(id, source)| {
            Ok((
                id.clone(),
                GovernedSourceValue {
                    repository_target: source
                        .repository_target
                        .as_ref()
                        .map(|route| project_route(&authority.repository, route))
                        .transpose()?,
                },
            ))
        })
        .collect::<Result<_, InternalFailure>>()?;
    let responsibilities = authority
        .responsibilities
        .iter()
        .map(|(id, responsibility)| {
            let roles = responsibility
                .roles
                .iter()
                .map(|(source, role)| (source.clone(), role.serialized()))
                .collect();
            let precedence = responsibility
                .precedence
                .iter()
                .map(|edge| PrecedenceEdgeValue {
                    higher_source: edge.higher_source.clone(),
                    lower_source: edge.lower_source.clone(),
                })
                .collect();
            (
                id.clone(),
                GovernedResponsibilityValue { roles, precedence },
            )
        })
        .collect();
    Ok(GovernanceAuthorityValue {
        model_version: authority.model_version,
        sources,
        responsibilities,
    })
}

fn project_objects(catalog: &GovernedObjectCatalog) -> GovernedObjectsValue {
    let interfaces = catalog
        .interfaces
        .iter()
        .map(|(id, interface)| {
            let objects = interface
                .objects
                .iter()
                .map(|(id, object)| {
                    let responsibilities = object.responsibilities.iter().cloned().collect();
                    let relations = object
                        .relations
                        .iter()
                        .map(|relation| GovernedRelationValue {
                            relation: relation.relation.clone(),
                            responsibility: relation.responsibility.clone(),
                            target: project_target(&relation.target),
                        })
                        .collect();
                    (
                        id.clone(),
                        GovernedObjectValue {
                            responsibilities,
                            relations,
                        },
                    )
                })
                .collect();
            (id.clone(), GovernedInterfaceValue { objects })
        })
        .collect();
    GovernedObjectsValue {
        model_version: catalog.model_version,
        interfaces,
    }
}

fn project_bindings(registry: &GovernanceBindingRegistry) -> GovernanceBindingsValue {
    GovernanceBindingsValue {
        model_version: registry.model_version,
        source: registry.source.clone(),
        bindings: registry
            .bindings
            .iter()
            .map(|(id, binding)| (id.clone(), project_binding(binding)))
            .collect(),
    }
}

fn project_integrity(profile: &ConsumerIntegrityProfile) -> RepositoryIntegrityProfileValue {
    let validations = profile
        .validations
        .iter()
        .map(|(id, definition)| {
            let (mode, selectors) = match &definition.instances {
                ValidationInstances::Single => (None, Vec::new()),
                ValidationInstances::RepositoryPaths { mode, selectors } => (
                    Some(mode.serialized()),
                    selectors
                        .iter()
                        .map(|selector| RepositoryPathSelectorValue {
                            kind: selector.kind.serialized(),
                            value: selector.value.clone(),
                        })
                        .collect(),
                ),
            };
            (
                id.clone(),
                ValidationDefinitionValue {
                    id: definition.id.clone(),
                    responsibility: definition.responsibility.clone(),
                    prerequisites: definition.prerequisites.iter().cloned().collect(),
                    instances: ValidationInstancesValue {
                        kind: definition.instances.kind(),
                        mode,
                        selectors,
                    },
                    command: CommandBindingValue {
                        environment: definition.command.environment.clone(),
                        arguments: definition.command.arguments.clone(),
                        undetermined_exit_codes: definition
                            .command
                            .undetermined_exit_codes
                            .iter()
                            .cloned()
                            .collect(),
                    },
                    target: definition.target.as_ref().map(project_target),
                },
            )
        })
        .collect();
    RepositoryIntegrityProfileValue {
        model_version: profile.model_version,
        authority: ProfileAuthorityValue {
            responsibility: profile.authority.responsibility.clone(),
            source: profile.authority.source.clone(),
        },
        environments: profile.environments.iter().cloned().collect(),
        continue_after_non_satisfied: profile.continue_after_non_satisfied,
        identity_present: !profile.identity.0.is_empty(),
        validations,
        order: profile.order.clone(),
    }
}

fn project_projections(registry: &ProjectionRegistry) -> ProjectionRegistryValue {
    ProjectionRegistryValue {
        model_version: registry.model_version,
        authority: ProfileAuthorityValue {
            responsibility: registry.authority.responsibility.clone(),
            source: registry.authority.source.clone(),
        },
        projections: registry
            .projections
            .iter()
            .map(|(id, projection)| {
                (
                    id.clone(),
                    ProjectionValue {
                        id: projection.id.clone(),
                        responsibility: projection.responsibility.clone(),
                        canonical_source: projection.canonical_source.clone(),
                        secondary_source: projection.secondary_source.clone(),
                        mode: projection.mode.serialized(),
                        validation: projection.validation.clone(),
                        generator_source: projection.generator_source.clone(),
                        boundary_source: projection.boundary_source.clone(),
                        target: projection.target.as_ref().map(project_target),
                        binding: projection.binding.clone(),
                    },
                )
            })
            .collect(),
    }
}

fn project_route(
    repository: &Path,
    route: &ResolvedGovernanceRoute,
) -> Result<ResolvedRouteValue, InternalFailure> {
    let relative = route
        .target
        .strip_prefix(repository)
        .map_err(|_| InternalFailure)?;
    Ok(ResolvedRouteValue {
        route: route.route.clone(),
        declared_path: route.declared_path.clone(),
        resolved_target: portable_path(relative).map_err(|_| InternalFailure)?,
    })
}

fn project_target(target: &GovernedObjectRef) -> TargetValue {
    TargetValue {
        interface: target.interface_id.clone(),
        object: target.object_id.clone(),
    }
}
