use std::fmt;
use std::path::{Path, PathBuf};

use proto_ring_engine::repository_governance_state::{self, RepositoryGovernanceState};

use crate::{
    ApiError, ObservationProvenance, ObservationReference, ProfileReference,
    load_error::{load_failure, reference_failure},
    reference_derivation,
};

/// One successfully composed, retained canonical governance observation.
///
/// It has no public arbitrary constructor or mutable engine access:
/// ```compile_fail
/// use proto_ring_api::GovernanceObservation;
/// let forged = GovernanceObservation {};
/// ```
/// Its identity-bearing engine state is not serializable through the façade:
/// ```compile_fail
/// fn serialize(observation: &proto_ring_api::GovernanceObservation) {
///     let _ = serde_json::to_string(observation);
/// }
/// ```
pub struct GovernanceObservation {
    state: RepositoryGovernanceState,
    provenance: ObservationProvenance,
    profile_reference: Option<ProfileReference>,
}

/// Load only through the canonical engine composer. No partial observation is
/// returned, and no consumer validator, realization or provider is invoked.
pub fn open(repository: impl AsRef<Path>) -> Result<GovernanceObservation, ApiError> {
    let state = repository_governance_state::load(repository.as_ref()).map_err(load_failure)?;
    let provenance = reference_derivation::provenance(&state).map_err(reference_failure)?;
    let profile_reference =
        reference_derivation::original_profile_reference(&state).map_err(reference_failure)?;
    Ok(GovernanceObservation {
        state,
        provenance,
        profile_reference,
    })
}

impl GovernanceObservation {
    /// Project the retained canonical state without reloading, Git or filesystem
    /// access. Optional-owner absence and a missing subject remain distinct.
    pub fn inspect(&self, selector: crate::InspectSelector) -> Result<crate::Inspection, ApiError> {
        crate::inspection_resolve::inspect(
            &self.state,
            &self.provenance,
            self.profile_reference.as_ref(),
            selector,
        )
    }

    /// Explain retained declarations or canonical authority consultations. This
    /// performs no reload or execution and makes no claim of currentness.
    pub fn explain(
        &self,
        query: crate::ExplainQuery,
    ) -> Result<crate::ExplanationResult, ApiError> {
        crate::explanation_resolve::explain(
            &self.state,
            &self.provenance,
            self.profile_reference.as_ref(),
            query,
        )
    }

    /// Original resolved engine root; this accessor performs no filesystem access.
    pub fn repository(&self) -> &Path {
        &self.state.repository
    }

    /// Original explicit scope in engine order, not a universal file inventory.
    pub fn scope_paths(&self) -> &[PathBuf] {
        &self.state.observed_state.scope_paths
    }

    pub fn reference(&self) -> &ObservationReference {
        &self.provenance.observation
    }

    /// Reference to the entire original profile, not a filtered validation view.
    pub fn profile_reference(&self) -> Option<&ProfileReference> {
        self.profile_reference.as_ref()
    }

    /// Retained paths and reference; equality does not assert a coherent cut or
    /// that the working tree has remained unchanged since opening.
    pub fn provenance(&self) -> &ObservationProvenance {
        &self.provenance
    }
}

impl fmt::Debug for GovernanceObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GovernanceObservation")
            .field("provenance", &self.provenance)
            .field("profile_reference", &self.profile_reference)
            .finish_non_exhaustive()
    }
}
