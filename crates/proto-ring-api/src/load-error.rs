use proto_ring_engine::repository_governance_state::{
    RepositoryGovernanceStage, RepositoryGovernanceStateError,
};

use crate::{ApiError, ErrorCode, GovernanceStage, ModelError};

pub(crate) fn load_failure(error: RepositoryGovernanceStateError) -> ApiError {
    let stage = match error.diagnostic.stage {
        RepositoryGovernanceStage::RepositoryState => GovernanceStage::RepositoryState,
        RepositoryGovernanceStage::Bootstrap => GovernanceStage::Bootstrap,
        RepositoryGovernanceStage::RepositoryGovernanceModel => {
            GovernanceStage::RepositoryGovernanceModel
        }
        RepositoryGovernanceStage::GovernanceAuthority => GovernanceStage::GovernanceAuthority,
        RepositoryGovernanceStage::GovernedObjects => GovernanceStage::GovernedObjects,
        RepositoryGovernanceStage::GovernanceBindings => GovernanceStage::GovernanceBindings,
        RepositoryGovernanceStage::RepositoryIntegrity => GovernanceStage::RepositoryIntegrity,
        RepositoryGovernanceStage::ProjectionIntegrity => GovernanceStage::ProjectionIntegrity,
        RepositoryGovernanceStage::EvidenceRequirements => GovernanceStage::EvidenceRequirements,
        RepositoryGovernanceStage::Composition => GovernanceStage::Composition,
    };
    ApiError {
        code: ErrorCode::GovernanceLoadFailed,
        stage: Some(stage),
        detail: Some(error.diagnostic.detail),
        attempt_id: None,
        effects_possible: Some(false),
    }
}

#[cfg(test)]
#[path = "tests/load-error-stages.rs"]
mod tests;

pub(crate) fn reference_failure(error: ModelError) -> ApiError {
    // A successful engine load followed by an unrepresentable public reference
    // is an internal failure, not a fabricated engine load stage or partial RGS.
    ApiError {
        code: ErrorCode::InternalFailure,
        stage: None,
        detail: Some(error.to_string()),
        attempt_id: None,
        effects_possible: Some(false),
    }
}
