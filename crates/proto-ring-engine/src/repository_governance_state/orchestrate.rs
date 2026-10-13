use std::path::Path;
#[cfg(feature = "conformance-internal")]
use std::path::PathBuf;

use crate::repository_state;
#[cfg(feature = "conformance-internal")]
use crate::repository_state::StateCaptureError;

use super::compose::{self, Composition};
use super::{
    RepositoryGovernanceStage, RepositoryGovernanceState, RepositoryGovernanceStateError, failure,
    scope,
};

pub(super) fn load(
    repository: &Path,
) -> Result<RepositoryGovernanceState, RepositoryGovernanceStateError> {
    #[cfg(feature = "conformance-internal")]
    {
        let mut controls = ConformanceControls::production();
        load_internal(repository, &mut controls)
    }
    #[cfg(not(feature = "conformance-internal"))]
    {
        load_internal(repository)
    }
}

fn load_internal(
    repository: &Path,
    #[cfg(feature = "conformance-internal")] controls: &mut ConformanceControls<'_>,
) -> Result<RepositoryGovernanceState, RepositoryGovernanceStateError> {
    let root = repository_state::resolve_worktree_root(repository)
        .map_err(|error| state_failure(error.to_string()))?;

    let discovery = compose::compose(&root)?;
    let mut discovery_scope = scope::observation_scope(&root, &discovery)?;
    #[cfg(feature = "conformance-internal")]
    if let Some(replacement) = controls.discovery_scope {
        discovery_scope = replacement.to_vec();
    }

    let initial_state = repository_state::capture(&root, &discovery_scope)
        .map_err(|error| state_failure(error.to_string()))?;
    #[cfg(feature = "conformance-internal")]
    if let Some(hook) = controls.after_initial_capture.as_mut() {
        hook(&root).map_err(|error| state_failure(error.to_string()))?;
    }

    let authoritative = compose::compose(&root)?;
    let mut authoritative_scope = scope::observation_scope(&root, &authoritative)?;
    #[cfg(feature = "conformance-internal")]
    if let Some(replacement) = controls.authoritative_scope {
        authoritative_scope = replacement.to_vec();
    }

    if discovery_scope != authoritative_scope {
        return Err(failure(
            RepositoryGovernanceStage::Composition,
            "observation scope changed during state construction",
        ));
    }

    let final_state = repository_state::capture(&root, &discovery_scope)
        .map_err(|error| state_failure(error.to_string()))?;
    if initial_state.identity != final_state.identity {
        return Err(state_failure(
            "repository state changed during governance composition",
        ));
    }

    Ok(state(root, final_state, authoritative))
}

fn state(
    repository: std::path::PathBuf,
    observed_state: repository_state::RepositoryState,
    composition: Composition,
) -> RepositoryGovernanceState {
    RepositoryGovernanceState {
        repository,
        observed_state,
        bootstrap: composition.bootstrap,
        repository_governance_model: composition.repository_governance_model,
        governance_authority: composition.governance_authority,
        governed_objects: composition.governed_objects,
        governance_bindings: composition.governance_bindings,
        repository_integrity: composition.repository_integrity,
        projection_registry: composition.projection_registry,
        evidence_requirements: composition.evidence_requirements,
    }
}

fn state_failure(detail: impl Into<String>) -> RepositoryGovernanceStateError {
    failure(RepositoryGovernanceStage::RepositoryState, detail)
}

#[cfg(feature = "conformance-internal")]
type AfterInitialCapture<'a> = dyn FnMut(&Path) -> Result<(), StateCaptureError> + 'a;

#[cfg(feature = "conformance-internal")]
struct ConformanceControls<'a> {
    after_initial_capture: Option<&'a mut AfterInitialCapture<'a>>,
    discovery_scope: Option<&'a [PathBuf]>,
    authoritative_scope: Option<&'a [PathBuf]>,
}

#[cfg(feature = "conformance-internal")]
impl ConformanceControls<'_> {
    fn production() -> Self {
        Self {
            after_initial_capture: None,
            discovery_scope: None,
            authoritative_scope: None,
        }
    }
}

#[cfg(feature = "conformance-internal")]
pub(super) fn load_with_after_initial_capture_hook<F>(
    repository: &Path,
    hook: F,
) -> Result<RepositoryGovernanceState, RepositoryGovernanceStateError>
where
    F: FnOnce(&Path) -> Result<(), StateCaptureError>,
{
    let mut hook = Some(hook);
    let mut adapter = |root: &Path| {
        hook.take()
            .ok_or_else(|| StateCaptureError::new("initial-capture hook executed more than once"))?(
            root,
        )
    };
    let mut controls = ConformanceControls {
        after_initial_capture: Some(&mut adapter),
        discovery_scope: None,
        authoritative_scope: None,
    };
    load_internal(repository, &mut controls)
}

#[cfg(feature = "conformance-internal")]
pub(super) fn load_with_scope_overrides(
    repository: &Path,
    discovery_scope: &[PathBuf],
    authoritative_scope: &[PathBuf],
) -> Result<RepositoryGovernanceState, RepositoryGovernanceStateError> {
    let mut controls = ConformanceControls {
        after_initial_capture: None,
        discovery_scope: Some(discovery_scope),
        authoritative_scope: Some(authoritative_scope),
    };
    load_internal(repository, &mut controls)
}
