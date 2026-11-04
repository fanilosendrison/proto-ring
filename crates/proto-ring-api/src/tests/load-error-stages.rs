use proto_ring_engine::repository_governance_state::{
    RepositoryGovernanceDiagnostic, RepositoryGovernanceStage, RepositoryGovernanceStateError,
};

use super::{load_failure, reference_failure};
use crate::{ErrorCode, GovernanceStage, ModelError};

#[test]
fn exact_stage_conversion_is_independent_of_diagnostic_text() {
    for (engine, public) in [
        (
            RepositoryGovernanceStage::RepositoryState,
            GovernanceStage::RepositoryState,
        ),
        (
            RepositoryGovernanceStage::Bootstrap,
            GovernanceStage::Bootstrap,
        ),
        (
            RepositoryGovernanceStage::RepositoryGovernanceModel,
            GovernanceStage::RepositoryGovernanceModel,
        ),
        (
            RepositoryGovernanceStage::GovernanceAuthority,
            GovernanceStage::GovernanceAuthority,
        ),
        (
            RepositoryGovernanceStage::GovernedObjects,
            GovernanceStage::GovernedObjects,
        ),
        (
            RepositoryGovernanceStage::GovernanceBindings,
            GovernanceStage::GovernanceBindings,
        ),
        (
            RepositoryGovernanceStage::RepositoryIntegrity,
            GovernanceStage::RepositoryIntegrity,
        ),
        (
            RepositoryGovernanceStage::ProjectionIntegrity,
            GovernanceStage::ProjectionIntegrity,
        ),
        (
            RepositoryGovernanceStage::EvidenceRequirements,
            GovernanceStage::EvidenceRequirements,
        ),
        (
            RepositoryGovernanceStage::Composition,
            GovernanceStage::Composition,
        ),
    ] {
        let detail = "bootstrap composition repository_integrity: not a classifier";
        let error = load_failure(RepositoryGovernanceStateError {
            diagnostic: RepositoryGovernanceDiagnostic {
                stage: engine,
                detail: detail.to_owned(),
            },
        });
        assert_eq!(error.code, ErrorCode::GovernanceLoadFailed);
        assert_eq!(error.stage, Some(public));
        assert_eq!(error.detail.as_deref(), Some(detail));
        assert_eq!(error.attempt_id, None);
        assert_eq!(error.effects_possible, Some(false));
    }
}

#[test]
fn reference_failure_does_not_manufacture_an_engine_stage_or_partial_state() {
    let error = reference_failure(ModelError::Inconsistent(
        "duplicate RGS scope path identity",
    ));
    assert_eq!(error.code, ErrorCode::InternalFailure);
    assert_eq!(error.stage, None);
    assert_eq!(error.attempt_id, None);
    assert_eq!(error.effects_possible, Some(false));
}
