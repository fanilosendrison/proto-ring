use std::path::Path;

use crate::evidence_requirements::{self, EvidenceRequirementRegistry};
use crate::governance_authority::{self, GovernanceAuthorityProfile};
use crate::governance_bindings::{self, BindingKind, GovernanceBindingRegistry, ScopeKind};
use crate::governance_bootstrap::{self, GovernanceBootstrap};
use crate::governance_routing::ResolvedGovernanceRoute;
use crate::governed_objects::{self, GovernedObjectCatalog};
use crate::projection_registry::{self, ProjectionRegistry};
use crate::repository_governance_model::{self, RepositoryGovernanceModel};
use crate::repository_integrity::{self, ConsumerIntegrityProfile};

use super::{RepositoryGovernanceStage, RepositoryGovernanceStateError, failure};

pub(super) struct Composition {
    pub(super) bootstrap: GovernanceBootstrap,
    pub(super) repository_governance_model: RepositoryGovernanceModel,
    pub(super) governance_authority: GovernanceAuthorityProfile,
    pub(super) governed_objects: Option<GovernedObjectCatalog>,
    pub(super) governance_bindings: GovernanceBindingRegistry,
    pub(super) repository_integrity: Option<ConsumerIntegrityProfile>,
    pub(super) projection_registry: Option<ProjectionRegistry>,
    pub(super) evidence_requirements: Option<EvidenceRequirementRegistry>,
}

pub(super) fn compose(root: &Path) -> Result<Composition, RepositoryGovernanceStateError> {
    let bootstrap = governance_bootstrap::load(root)
        .map_err(|error| failure(RepositoryGovernanceStage::Bootstrap, error.to_string()))?;
    let model = repository_governance_model::from_bootstrap(root, &bootstrap).map_err(|error| {
        failure(
            RepositoryGovernanceStage::RepositoryGovernanceModel,
            error.to_string(),
        )
    })?;
    if model.model_version != 2 {
        return Err(failure(
            RepositoryGovernanceStage::RepositoryGovernanceModel,
            "RepositoryGovernanceState requires model version 2",
        ));
    }

    let authority =
        governance_authority::load(root, route(&model, "governance_authority", "profile")?)
            .map_err(|error| {
                failure(
                    RepositoryGovernanceStage::GovernanceAuthority,
                    error.to_string(),
                )
            })?;

    let governed_objects = if model.capabilities.contains_key("governed_objects") {
        Some(
            governed_objects::load(
                root,
                route(&model, "governed_objects", "profile")?,
                &authority,
            )
            .map_err(|error| {
                failure(
                    RepositoryGovernanceStage::GovernedObjects,
                    error.to_string(),
                )
            })?,
        )
    } else {
        None
    };

    let bindings = governance_bindings::load(
        root,
        route(&model, "shared_governance_provider", "registry")?,
        &authority,
        governed_objects.as_ref(),
    )
    .map_err(|error| {
        failure(
            RepositoryGovernanceStage::GovernanceBindings,
            error.to_string(),
        )
    })?;
    validate_binding_capabilities(&model, &bindings)?;

    let integrity = if model.capabilities.contains_key("repository_integrity") {
        Some(
            repository_integrity::load(
                root,
                route(&model, "repository_integrity", "profile")?,
                &authority,
                governed_objects.as_ref(),
            )
            .map_err(|error| {
                failure(
                    RepositoryGovernanceStage::RepositoryIntegrity,
                    error.to_string(),
                )
            })?,
        )
    } else {
        None
    };

    let projections = if model.capabilities.contains_key("projection_integrity") {
        let integrity = integrity.as_ref().ok_or_else(|| {
            failure(
                RepositoryGovernanceStage::Composition,
                "projection_integrity requires a loaded repository_integrity profile",
            )
        })?;
        Some(
            projection_registry::load(
                root,
                route(&model, "projection_integrity", "registry")?,
                &authority,
                integrity,
                governed_objects.as_ref(),
                Some(&bindings),
            )
            .map_err(|error| {
                failure(
                    RepositoryGovernanceStage::ProjectionIntegrity,
                    error.to_string(),
                )
            })?,
        )
    } else {
        None
    };

    let evidence = if model.capabilities.contains_key("evidence_requirements") {
        Some(
            evidence_requirements::load(
                root,
                route(&model, "evidence_requirements", "registry")?,
                &authority,
                governed_objects.as_ref(),
            )
            .map_err(|error| {
                failure(
                    RepositoryGovernanceStage::EvidenceRequirements,
                    error.to_string(),
                )
            })?,
        )
    } else {
        None
    };

    Ok(Composition {
        bootstrap,
        repository_governance_model: model,
        governance_authority: authority,
        governed_objects,
        governance_bindings: bindings,
        repository_integrity: integrity,
        projection_registry: projections,
        evidence_requirements: evidence,
    })
}

fn route<'a>(
    model: &'a RepositoryGovernanceModel,
    capability_id: &str,
    route_id: &str,
) -> Result<&'a ResolvedGovernanceRoute, RepositoryGovernanceStateError> {
    model
        .capabilities
        .get(capability_id)
        .and_then(|capability| capability.routes.get(route_id))
        .ok_or_else(|| {
            failure(
                RepositoryGovernanceStage::Composition,
                format!("validated model is missing route {capability_id}/{route_id}"),
            )
        })
}

fn validate_binding_capabilities(
    model: &RepositoryGovernanceModel,
    bindings: &GovernanceBindingRegistry,
) -> Result<(), RepositoryGovernanceStateError> {
    let capability_ids = bindings
        .bindings
        .values()
        .filter(|binding| {
            binding.kind == BindingKind::GovernanceContract
                && binding.scope.kind == ScopeKind::Capability
        })
        .map(|binding| {
            binding.scope.capability.clone().ok_or_else(|| {
                failure(
                    RepositoryGovernanceStage::Composition,
                    "capability-scoped binding has no capability ID",
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    repository_governance_model::validate_binding_capabilities(model, &capability_ids)
        .map_err(|error| failure(RepositoryGovernanceStage::Composition, error.to_string()))
}
