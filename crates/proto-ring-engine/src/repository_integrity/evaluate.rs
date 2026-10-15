use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::repository_state::{self, RepositoryState, RepositoryStateIdentity, StateCaptureError};

use super::execution::{self, ProcessOutcome};
use super::model::{
    ConsumerIntegrityProfile, EvaluationCapturePhase, EvaluationContext, EvaluationContextIdentity,
    IntegrityProfile, IntegrityResult, IntegrityVerdict, ObligationResult, ObligationStatus,
    RealizedIntegrityProfile, RepositoryPathsMode, RuntimeCommandObligation, RuntimeValidation,
    StateRelation, ValidationInstances, ValidationResult, evaluation_context_identity,
};
use super::selectors::{path_to_posix, select_repository_paths};
use super::{RepositoryIntegrityError, failure};

pub fn realize_consumer_profile(
    repository: &Path,
    profile: &ConsumerIntegrityProfile,
    context: &EvaluationContext,
) -> Result<RealizedIntegrityProfile, RepositoryIntegrityError> {
    realize_with(
        repository,
        profile,
        context,
        repository_state::capture,
        repository_state::discover_repository_paths,
    )
}

pub(super) fn realize_with<C, D>(
    repository: &Path,
    profile: &ConsumerIntegrityProfile,
    context: &EvaluationContext,
    mut capture: C,
    mut discover: D,
) -> Result<RealizedIntegrityProfile, RepositoryIntegrityError>
where
    C: FnMut(&Path, &[PathBuf]) -> Result<RepositoryState, StateCaptureError>,
    D: FnMut(&Path, &[PathBuf]) -> Result<Vec<PathBuf>, StateCaptureError>,
{
    let root = repository_state::resolve_worktree_root(repository)
        .map_err(|error| failure(format!("cannot resolve evaluation repository: {error}")))?;
    if root != profile.repository {
        return Err(failure(
            "evaluation repository differs from profile repository",
        ));
    }
    let context_identity = evaluation_context_identity(profile, context)?;
    let literal_paths: BTreeSet<_> = profile
        .validations
        .values()
        .flat_map(|definition| match &definition.instances {
            ValidationInstances::RepositoryPaths { selectors, .. } => selectors
                .iter()
                .filter(|selector| selector.kind == super::model::SelectorKind::Path)
                .map(|selector| PathBuf::from(&selector.value))
                .collect(),
            ValidationInstances::Single => Vec::new(),
        })
        .collect();
    let has_path_instances = profile.validations.values().any(|definition| {
        matches!(
            definition.instances,
            ValidationInstances::RepositoryPaths { .. }
        )
    });
    let literal_scope: Vec<_> = literal_paths.iter().cloned().collect();
    let (repository_paths, resolution_state_identity, discovery_failed) = if has_path_instances {
        match capture(&root, &literal_scope).and_then(|before| {
            let paths = discover(&root, &[])?;
            capture(&root, &literal_scope).map(|after| (before, paths, after))
        }) {
            Ok((before, paths, after)) if before.identity == after.identity => {
                (paths, Some(after.identity), false)
            }
            _ => (Vec::new(), None, true),
        }
    } else {
        (Vec::new(), None, false)
    };
    let mut governed_paths = literal_paths;
    let mut validations = Vec::with_capacity(profile.order.len());
    for validation_id in &profile.order {
        let definition = profile
            .validations
            .get(validation_id)
            .ok_or_else(|| failure("profile order references absent validation"))?;
        let argument_sets = match &definition.instances {
            ValidationInstances::Single => Some(vec![definition.command.arguments.clone()]),
            ValidationInstances::RepositoryPaths { mode, .. } if discovery_failed => None,
            ValidationInstances::RepositoryPaths { mode, .. } => {
                match select_repository_paths(&definition.instances, &repository_paths) {
                    Ok(selected) => {
                        governed_paths.extend(selected.iter().cloned());
                        let selected_text = selected
                            .iter()
                            .map(|path| {
                                path_to_posix(path)
                                    .ok_or_else(|| failure("selected path is not UTF-8"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        Some(match mode {
                            RepositoryPathsMode::AppendAll => {
                                let mut arguments = definition.command.arguments.clone();
                                arguments.extend(selected_text);
                                vec![arguments]
                            }
                            RepositoryPathsMode::ForEach => selected_text
                                .into_iter()
                                .map(|path| {
                                    let mut arguments = definition.command.arguments.clone();
                                    arguments.push(path);
                                    arguments
                                })
                                .collect(),
                        })
                    }
                    Err(_) => None,
                }
            }
        };
        let realization = context.realizations.get(&definition.command.environment);
        let obligations = argument_sets
            .as_ref()
            .map(|sets| {
                sets.iter()
                    .map(|arguments| RuntimeCommandObligation {
                        validation_id: validation_id.clone(),
                        command_prefix: realization
                            .map(|item| item.command_prefix.clone())
                            .unwrap_or_default(),
                        arguments: arguments.clone(),
                        undetermined_exit_codes: definition.command.undetermined_exit_codes.clone(),
                        process_environment: realization
                            .map(|item| item.process_environment.clone()),
                    })
                    .collect()
            })
            .unwrap_or_default();
        validations.push(RuntimeValidation {
            id: validation_id.clone(),
            prerequisites: definition.prerequisites.clone(),
            obligations,
            resolution_failed: argument_sets.is_none(),
        });
    }
    Ok(RealizedIntegrityProfile {
        profile: IntegrityProfile {
            continue_after_non_satisfied: profile.continue_after_non_satisfied,
            validations,
            identity: profile.identity.clone(),
            governed_paths: governed_paths.into_iter().collect(),
            resolution_state_identity,
        },
        evaluation_context_identity: context_identity,
    })
}

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
    mut execute: E,
) -> IntegrityResult
where
    C: FnMut(
        EvaluationCapturePhase,
        &Path,
        &[PathBuf],
    ) -> Result<RepositoryState, StateCaptureError>,
    E: FnMut(&Path, &RuntimeCommandObligation) -> ProcessOutcome,
{
    let baseline = match capture(
        EvaluationCapturePhase::Baseline,
        repository,
        &profile.governed_paths,
    ) {
        Ok(state) => state.identity,
        Err(_) => {
            return completed_result(
                profile,
                evaluation_context_identity,
                None,
                None,
                StateRelation::Unavailable,
                skipped_validations(&profile.validations),
                true,
            );
        }
    };
    if profile
        .resolution_state_identity
        .as_ref()
        .is_some_and(|identity| identity != &baseline)
    {
        return completed_result(
            profile,
            evaluation_context_identity,
            Some(baseline.clone()),
            Some(baseline),
            StateRelation::NotEqual,
            skipped_validations(&profile.validations),
            true,
        );
    }
    let mut aggregate_by_id = BTreeMap::new();
    let mut validation_results = Vec::with_capacity(profile.validations.len());
    let mut final_identity = Some(baseline.clone());
    let mut relation = StateRelation::Equal;
    let mut coherence_halted = false;
    let mut policy_stopped = false;
    for validation in &profile.validations {
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
            for obligation in &validation.obligations {
                let before = match capture(
                    EvaluationCapturePhase::PreObligation,
                    repository,
                    &profile.governed_paths,
                ) {
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
                let mut obligation_result = execute(repository, obligation).into_result(obligation);
                match capture(
                    EvaluationCapturePhase::PostObligation,
                    repository,
                    &profile.governed_paths,
                ) {
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
    completed_result(
        profile,
        evaluation_context_identity,
        Some(baseline),
        final_identity,
        relation,
        validation_results,
        coherence_halted,
    )
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
