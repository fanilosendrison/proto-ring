//! Exact-state-bound, read-only repository governance composition.

mod compose;
mod orchestrate;
mod scope;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use crate::evidence_requirements::EvidenceRequirementRegistry;
use crate::governance_authority::GovernanceAuthorityProfile;
use crate::governance_bindings::GovernanceBindingRegistry;
use crate::governance_bootstrap::GovernanceBootstrap;
use crate::governed_objects::GovernedObjectCatalog;
use crate::projection_registry::ProjectionRegistry;
use crate::repository_governance_model::RepositoryGovernanceModel;
use crate::repository_integrity::ConsumerIntegrityProfile;
use crate::repository_state::RepositoryState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepositoryGovernanceStage {
    RepositoryState,
    Bootstrap,
    RepositoryGovernanceModel,
    GovernanceAuthority,
    GovernedObjects,
    GovernanceBindings,
    RepositoryIntegrity,
    ProjectionIntegrity,
    EvidenceRequirements,
    Composition,
}

impl RepositoryGovernanceStage {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::RepositoryState => "repository_state",
            Self::Bootstrap => "bootstrap",
            Self::RepositoryGovernanceModel => "repository_governance_model",
            Self::GovernanceAuthority => "governance_authority",
            Self::GovernedObjects => "governed_objects",
            Self::GovernanceBindings => "governance_bindings",
            Self::RepositoryIntegrity => "repository_integrity",
            Self::ProjectionIntegrity => "projection_integrity",
            Self::EvidenceRequirements => "evidence_requirements",
            Self::Composition => "composition",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryGovernanceDiagnostic {
    pub stage: RepositoryGovernanceStage,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryGovernanceStateError {
    pub diagnostic: RepositoryGovernanceDiagnostic,
}

impl Display for RepositoryGovernanceStateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}: {}",
            self.diagnostic.stage.serialized(),
            self.diagnostic.detail
        )
    }
}

impl Error for RepositoryGovernanceStateError {}

#[derive(Clone, Debug)]
pub struct RepositoryGovernanceState {
    pub repository: PathBuf,
    pub observed_state: RepositoryState,
    pub bootstrap: GovernanceBootstrap,
    pub repository_governance_model: RepositoryGovernanceModel,
    pub governance_authority: GovernanceAuthorityProfile,
    pub governed_objects: Option<GovernedObjectCatalog>,
    pub governance_bindings: GovernanceBindingRegistry,
    pub repository_integrity: Option<ConsumerIntegrityProfile>,
    pub projection_registry: Option<ProjectionRegistry>,
    pub evidence_requirements: Option<EvidenceRequirementRegistry>,
}

pub fn load(
    repository: &Path,
) -> Result<RepositoryGovernanceState, RepositoryGovernanceStateError> {
    orchestrate::load(repository)
}

pub(super) fn failure(
    stage: RepositoryGovernanceStage,
    detail: impl Into<String>,
) -> RepositoryGovernanceStateError {
    RepositoryGovernanceStateError {
        diagnostic: RepositoryGovernanceDiagnostic {
            stage,
            detail: detail.into(),
        },
    }
}

#[cfg(feature = "conformance-internal")]
#[doc(hidden)]
pub mod conformance_internal {
    use std::path::{Path, PathBuf};

    use crate::repository_state::StateCaptureError;

    use super::{RepositoryGovernanceState, RepositoryGovernanceStateError, orchestrate};

    pub fn load_with_after_initial_capture_hook<F>(
        repository: &Path,
        hook: F,
    ) -> Result<RepositoryGovernanceState, RepositoryGovernanceStateError>
    where
        F: FnOnce(&Path) -> Result<(), StateCaptureError>,
    {
        orchestrate::load_with_after_initial_capture_hook(repository, hook)
    }

    pub fn load_with_scope_overrides(
        repository: &Path,
        discovery_scope: &[PathBuf],
        authoritative_scope: &[PathBuf],
    ) -> Result<RepositoryGovernanceState, RepositoryGovernanceStateError> {
        orchestrate::load_with_scope_overrides(repository, discovery_scope, authoritative_scope)
    }
}
