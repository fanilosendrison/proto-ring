mod evaluate;
mod execution;
mod identity;
mod load;
mod model;
mod selectors;

use std::error::Error;
use std::fmt::{Display, Formatter};

pub use evaluate::{evaluate, realize_consumer_profile};
pub use load::load;
pub use model::{
    CommandBinding, ConsumerIntegrityProfile, EvaluationContext, EvaluationContextIdentity,
    IntegrityProfile, IntegrityResult, IntegrityVerdict, ObligationResult, ObligationStatus,
    ProfileAuthority, ProfileIdentity, RealizedIntegrityProfile, RepositoryPathSelector,
    RepositoryPathsMode, RuntimeCommandObligation, RuntimeValidation, SelectorKind, StateRelation,
    ValidationDefinition, ValidationEnvironmentRealization, ValidationInstances, ValidationResult,
};

#[cfg(feature = "conformance-internal")]
#[doc(hidden)]
pub mod conformance_internal {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use crate::repository_state::{RepositoryState, StateCaptureError};

    use super::model::EvaluationCapturePhase;
    use super::{
        ConsumerIntegrityProfile, EvaluationContext, EvaluationContextIdentity, IntegrityProfile,
        IntegrityResult, RealizedIntegrityProfile, RepositoryIntegrityError,
        RuntimeCommandObligation, evaluate, execution,
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum CapturePhase {
        Baseline,
        PreObligation,
        PostObligation,
    }

    pub fn evaluate_with_capture_and_ambient<C>(
        repository: &Path,
        profile: &IntegrityProfile,
        evaluation_context_identity: EvaluationContextIdentity,
        capture: C,
        ambient_seed: &BTreeMap<String, String>,
    ) -> IntegrityResult
    where
        C: FnMut(CapturePhase, &Path, &[PathBuf]) -> Result<RepositoryState, StateCaptureError>,
    {
        evaluate_with_execution_observer(
            repository,
            profile,
            evaluation_context_identity,
            capture,
            ambient_seed,
            |_, _| {},
        )
    }

    pub fn evaluate_with_execution_observer<C, O>(
        repository: &Path,
        profile: &IntegrityProfile,
        evaluation_context_identity: EvaluationContextIdentity,
        mut capture: C,
        ambient_seed: &BTreeMap<String, String>,
        mut observe_execution: O,
    ) -> IntegrityResult
    where
        C: FnMut(CapturePhase, &Path, &[PathBuf]) -> Result<RepositoryState, StateCaptureError>,
        O: FnMut(&RuntimeCommandObligation, bool),
    {
        evaluate::evaluate_with(
            repository,
            profile,
            evaluation_context_identity,
            |phase, root, scope| {
                let phase = match phase {
                    EvaluationCapturePhase::Baseline => CapturePhase::Baseline,
                    EvaluationCapturePhase::PreObligation => CapturePhase::PreObligation,
                    EvaluationCapturePhase::PostObligation => CapturePhase::PostObligation,
                };
                capture(phase, root, scope)
            },
            |root: &Path, obligation: &RuntimeCommandObligation| {
                let outcome = execution::execute_with_ambient(root, obligation, ambient_seed);
                observe_execution(obligation, outcome.started());
                outcome
            },
        )
    }

    pub fn realize_with_discovery<D>(
        repository: &Path,
        profile: &ConsumerIntegrityProfile,
        context: &EvaluationContext,
        discover: D,
    ) -> Result<RealizedIntegrityProfile, RepositoryIntegrityError>
    where
        D: FnMut(&Path, &[PathBuf]) -> Result<Vec<PathBuf>, StateCaptureError>,
    {
        evaluate::realize_with(
            repository,
            profile,
            context,
            crate::repository_state::capture,
            discover,
        )
    }
}

#[derive(Debug)]
pub struct RepositoryIntegrityError(String);
impl Display for RepositoryIntegrityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl Error for RepositoryIntegrityError {}
pub(super) fn failure(message: impl Into<String>) -> RepositoryIntegrityError {
    RepositoryIntegrityError(message.into())
}
