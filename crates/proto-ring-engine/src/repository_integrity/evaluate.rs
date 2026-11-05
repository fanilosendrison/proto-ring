use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::backend::{
    Backend, CapturePhase, ObligationPosition, Ordinary, infallible, ordinary_control,
};
use super::control::{AttemptControl, CoherenceCheckpoint, GuardedEvaluation, HostAbort};
use super::execution;
use super::model::{
    EvaluationCapturePhase, EvaluationContextIdentity, IntegrityProfile, IntegrityResult,
    IntegrityVerdict, ObligationResult, ObligationStatus, RuntimeCommandObligation,
    RuntimeValidation, StateRelation, ValidationResult,
};
use super::observations::{Launch, ProcessObservation};
use crate::repository_state::{self, RepositoryState, RepositoryStateIdentity, StateCaptureError};

pub fn evaluate(
    repository: &Path,
    profile: &IntegrityProfile,
    evaluation_context_identity: EvaluationContextIdentity,
) -> IntegrityResult {
    evaluate_with(
        repository,
        profile,
        evaluation_context_identity,
        |_, root, scope| repository_state::capture(root, scope),
        execution::execute,
    )
}

pub(super) fn evaluate_with<C, E>(
    repository: &Path,
    profile: &IntegrityProfile,
    evaluation_context_identity: EvaluationContextIdentity,
    mut capture: C,
    execute: E,
) -> IntegrityResult
where
    C: FnMut(
        EvaluationCapturePhase,
        &Path,
        &[PathBuf],
    ) -> Result<RepositoryState, StateCaptureError>,
    E: FnMut(&Path, &RuntimeCommandObligation) -> ProcessObservation,
{
    let mut backend = Ordinary {
        capture: |phase: CapturePhase, root: &Path, scope: &[PathBuf]| {
            let phase = match phase {
                CapturePhase::Baseline => EvaluationCapturePhase::Baseline,
                CapturePhase::PreObligation => EvaluationCapturePhase::PreObligation,
                CapturePhase::PostObligation => EvaluationCapturePhase::PostObligation,
                // This adapter evaluates only: discovery phases are never requested.
                CapturePhase::BeforeDiscovery | CapturePhase::AfterDiscovery => {
                    return Err(StateCaptureError::new(
                        "discovery capture requested during evaluation",
                    ));
                }
            };
            capture(phase, root, scope)
        },
        discover: repository_state::discover_repository_paths,
        execute,
    };
    infallible(evaluate_core(
        repository,
        profile,
        evaluation_context_identity,
        &mut backend,
        &mut ordinary_control(),
    ))
}

/// Guarded implementation boundary, not a public API-01 checking service.
pub fn evaluate_guarded<B: Backend<Abort = HostAbort>>(
    repository: &Path,
    profile: &IntegrityProfile,
    identity: EvaluationContextIdentity,
    backend: &mut B,
    control: &mut AttemptControl,
) -> GuardedEvaluation {
    match evaluate_core(repository, profile, identity, backend, control) {
        Ok(result) => GuardedEvaluation::Completed { result },
        Err(reason) => GuardedEvaluation::Interrupted {
            reason,
            effects_possible: control.effects_possible(),
        },
    }
}

pub(super) fn evaluate_core<B: Backend>(
    repository: &Path,
    profile: &IntegrityProfile,
    evaluation_context_identity: EvaluationContextIdentity,
    backend: &mut B,
    control: &mut AttemptControl,
) -> Result<IntegrityResult, B::Abort> {
    let result = evaluate_observations(
        repository,
        profile,
        evaluation_context_identity,
        backend,
        control,
    )?;
    // The sole completion boundary also covers all ordinary early returns.
    // It observes the last required attempt, not an invented final capture.
    control.observe::<_, B>(backend.checkpoint(CoherenceCheckpoint::AfterFinalRiObservation))?;
    Ok(result)
}

fn required_capture<B: Backend>(
    backend: &mut B,
    control: &mut AttemptControl,
    phase: CapturePhase,
    root: &Path,
    scope: &[PathBuf],
) -> Result<Result<RepositoryState, StateCaptureError>, B::Abort> {
    control.observe::<_, B>(backend.checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture))?;
    control.observe::<_, B>(backend.capture(phase, root, scope))
}

fn execute_obligation<B: Backend>(
    backend: &mut B,
    control: &mut AttemptControl,
    repository: &Path,
    position: ObligationPosition,
    obligation: &RuntimeCommandObligation,
) -> Result<ObligationResult, B::Abort> {
    if obligation.command_prefix.is_empty() || obligation.process_environment.is_none() {
        return Ok(execution::classify(
            ProcessObservation::NotStarted,
            obligation,
        ));
    }
    control.observe::<_, B>(backend.checkpoint(CoherenceCheckpoint::BeforeEachLaunch))?;
    let observation =
        match control.observe::<_, B>(backend.launch(repository, position, obligation))? {
            Launch::NotStarted => ProcessObservation::NotStarted,
            Launch::Started(process) => {
                control.started(); // Must precede a wait that can abort.
                let completion = control.observe::<_, B>(backend.complete(process, position))?;
                control.observe::<_, B>(
                    backend.checkpoint(CoherenceCheckpoint::AfterEachEffectLifetime),
                )?;
                ProcessObservation::Completed(completion)
            }
        };
    Ok(execution::classify(observation, obligation))
}

fn evaluate_observations<B: Backend>(
    repository: &Path,
    profile: &IntegrityProfile,
    evaluation_context_identity: EvaluationContextIdentity,
    backend: &mut B,
    control: &mut AttemptControl,
) -> Result<IntegrityResult, B::Abort> {
    let baseline = match required_capture(
        backend,
        control,
        CapturePhase::Baseline,
        repository,
        &profile.governed_paths,
    )? {
        Ok(state) => state.identity,
        Err(_) => {
            return Ok(completed_result(
                profile,
                evaluation_context_identity,
                None,
                None,
                StateRelation::Unavailable,
                skipped_validations(&profile.validations),
                true,
            ));
        }
    };
    if profile
        .resolution_state_identity
        .as_ref()
        .is_some_and(|identity| identity != &baseline)
    {
        return Ok(completed_result(
            profile,
            evaluation_context_identity,
            Some(baseline.clone()),
            Some(baseline),
            StateRelation::NotEqual,
            skipped_validations(&profile.validations),
            true,
        ));
    }
    let mut aggregate_by_id = BTreeMap::new();
    let mut validation_results = Vec::with_capacity(profile.validations.len());
    let mut final_identity = Some(baseline.clone());
    let mut relation = StateRelation::Equal;
    let mut coherence_halted = false;
    let mut policy_stopped = false;
    for (validation_position, validation) in profile.validations.iter().enumerate() {
        let result = if coherence_halted || policy_stopped {
            ValidationResult::skipped(validation)
        } else if validation.resolution_failed {
            ValidationResult {
                id: validation.id.clone(),
                status: ObligationStatus::Undetermined,
                obligations: Vec::new(),
            }
        } else if validation.prerequisites.iter().any(|prerequisite| {
            aggregate_by_id.get(prerequisite) != Some(&ObligationStatus::Satisfied)
        }) {
            ValidationResult::skipped(validation)
        } else {
            let mut obligations = Vec::with_capacity(validation.obligations.len());
            for (instance, obligation) in validation.obligations.iter().enumerate() {
                let before = match required_capture(
                    backend,
                    control,
                    CapturePhase::PreObligation,
                    repository,
                    &profile.governed_paths,
                )? {
                    Ok(state) => state.identity,
                    Err(_) => {
                        relation = StateRelation::Unavailable;
                        final_identity = None;
                        coherence_halted = true;
                        obligations.push(ObligationResult::undetermined(obligation));
                        break;
                    }
                };
                final_identity = Some(before.clone());
                if before != baseline {
                    relation = StateRelation::NotEqual;
                    coherence_halted = true;
                    obligations.push(ObligationResult::undetermined(obligation));
                    break;
                }
                let mut obligation_result = execute_obligation(
                    backend,
                    control,
                    repository,
                    ObligationPosition {
                        validation: validation_position,
                        instance,
                    },
                    obligation,
                )?;
                match required_capture(
                    backend,
                    control,
                    CapturePhase::PostObligation,
                    repository,
                    &profile.governed_paths,
                )? {
                    Err(_) => {
                        obligation_result.status = ObligationStatus::Undetermined;
                        relation = StateRelation::Unavailable;
                        final_identity = None;
                        coherence_halted = true;
                    }
                    Ok(state) if state.identity != baseline => {
                        obligation_result.status = ObligationStatus::Violated;
                        relation = StateRelation::NotEqual;
                        final_identity = Some(state.identity);
                        coherence_halted = true;
                    }
                    Ok(state) => final_identity = Some(state.identity),
                }
                obligations.push(obligation_result);
                if coherence_halted {
                    break;
                }
            }
            obligations.extend(
                validation.obligations[obligations.len()..]
                    .iter()
                    .map(ObligationResult::undetermined),
            );
            ValidationResult {
                id: validation.id.clone(),
                status: ObligationStatus::aggregate(&obligations),
                obligations,
            }
        };
        aggregate_by_id.insert(validation.id.clone(), result.status);
        if result.status != ObligationStatus::Satisfied
            && !profile.continue_after_non_satisfied
            && !coherence_halted
        {
            policy_stopped = true;
        }
        validation_results.push(result);
    }
    Ok(completed_result(
        profile,
        evaluation_context_identity,
        Some(baseline),
        final_identity,
        relation,
        validation_results,
        coherence_halted,
    ))
}

fn skipped_validations(validations: &[RuntimeValidation]) -> Vec<ValidationResult> {
    validations.iter().map(ValidationResult::skipped).collect()
}

fn completed_result(
    profile: &IntegrityProfile,
    evaluation_context_identity: EvaluationContextIdentity,
    baseline_state_identity: Option<RepositoryStateIdentity>,
    final_state_identity: Option<RepositoryStateIdentity>,
    state_relation: StateRelation,
    validations: Vec<ValidationResult>,
    state_coherence_halt: bool,
) -> IntegrityResult {
    let obligations = validations
        .iter()
        .flat_map(|validation| validation.obligations.iter().cloned())
        .collect();
    let all_satisfied = validations
        .iter()
        .all(|validation| validation.status == ObligationStatus::Satisfied);
    IntegrityResult {
        verdict: if all_satisfied && state_relation == StateRelation::Equal && !state_coherence_halt
        {
            IntegrityVerdict::Pass
        } else {
            IntegrityVerdict::NonPass
        },
        profile_identity: profile.identity.clone(),
        evaluation_context_identity,
        baseline_state_identity,
        final_state_identity,
        state_relation,
        obligations,
        validations,
        state_coherence_halt,
    }
}
