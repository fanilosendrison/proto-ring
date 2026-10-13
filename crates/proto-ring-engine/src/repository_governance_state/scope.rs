use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use crate::governance_routing::ResolvedGovernanceRoute;
use crate::repository_state;

use super::compose::Composition;
use super::{RepositoryGovernanceStage, RepositoryGovernanceStateError, failure};

pub(super) fn observation_scope(
    root: &Path,
    composition: &Composition,
) -> Result<Vec<PathBuf>, RepositoryGovernanceStateError> {
    let mut paths = BTreeSet::new();
    paths.insert(PathBuf::from("AGENTS.md"));
    paths.insert(bootstrap_target(root)?);

    for capability in composition
        .repository_governance_model
        .capabilities
        .values()
    {
        for route in capability.routes.values() {
            add_route(root, route, &mut paths)?;
        }
    }

    add_carrier(root, &composition.governance_authority.carrier, &mut paths)?;
    if let Some(catalog) = &composition.governed_objects {
        add_carrier(root, &catalog.carrier, &mut paths)?;
    }
    add_carrier(root, &composition.governance_bindings.carrier, &mut paths)?;
    if let Some(integrity) = &composition.repository_integrity {
        add_carrier(root, &integrity.carrier, &mut paths)?;
    }
    if let Some(registry) = &composition.projection_registry {
        add_carrier(root, &registry.carrier, &mut paths)?;
    }
    if let Some(registry) = &composition.evidence_requirements {
        add_carrier(root, &registry.carrier, &mut paths)?;
    }

    for source in composition.governance_authority.sources.values() {
        if let Some(route) = &source.repository_target {
            add_route(root, route, &mut paths)?;
        }
    }

    Ok(paths.into_iter().collect())
}

fn bootstrap_target(root: &Path) -> Result<PathBuf, RepositoryGovernanceStateError> {
    let root_dir = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        failure(
            RepositoryGovernanceStage::RepositoryState,
            format!("cannot open exact worktree capability: {error}"),
        )
    })?;
    let target = root_dir.canonicalize("AGENTS.md").map_err(|error| {
        failure(
            RepositoryGovernanceStage::RepositoryState,
            format!("cannot resolve bootstrap carrier target: {error}"),
        )
    })?;
    if target.as_os_str().is_empty() {
        return Err(failure(
            RepositoryGovernanceStage::Composition,
            "bootstrap carrier target cannot be represented as repository-relative state",
        ));
    }
    Ok(target)
}

fn add_route(
    root: &Path,
    route: &ResolvedGovernanceRoute,
    paths: &mut BTreeSet<PathBuf>,
) -> Result<(), RepositoryGovernanceStateError> {
    add_carrier(root, &route.target, paths)?;
    match repository_state::explicit_scope_path_identity(root, &route.declared_path) {
        Ok(Some(path)) => {
            paths.insert(path);
        }
        Ok(None) => {}
        Err(error) => {
            return Err(failure(
                RepositoryGovernanceStage::RepositoryState,
                error.to_string(),
            ));
        }
    }
    Ok(())
}

fn add_carrier(
    root: &Path,
    target: &Path,
    paths: &mut BTreeSet<PathBuf>,
) -> Result<(), RepositoryGovernanceStateError> {
    let relative = target.strip_prefix(root).map_err(|_| {
        failure(
            RepositoryGovernanceStage::Composition,
            "resolved target is not contained by the exact repository root",
        )
    })?;
    if relative.as_os_str().is_empty() {
        return Err(failure(
            RepositoryGovernanceStage::Composition,
            "resolved target cannot be represented as repository-relative state",
        ));
    }
    paths.insert(relative.to_path_buf());
    Ok(())
}
