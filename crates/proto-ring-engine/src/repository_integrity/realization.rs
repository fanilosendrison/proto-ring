use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::backend::{Backend, CapturePhase, Ordinary, infallible, ordinary_control};
use super::control::{AttemptControl, CoherenceCheckpoint, HostAbort, Interruption};
use super::model::{
    ConsumerIntegrityProfile, EvaluationContext, IntegrityProfile, RealizedIntegrityProfile,
    RepositoryPathsMode, RuntimeCommandObligation, RuntimeValidation, ValidationInstances,
    evaluation_context_identity,
};
use super::selectors::{path_to_posix, select_repository_paths};
use super::{RepositoryIntegrityError, execution, failure};
use crate::repository_state::{self, RepositoryState, StateCaptureError};

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
    discover: D,
) -> Result<RealizedIntegrityProfile, RepositoryIntegrityError>
where
    C: FnMut(&Path, &[PathBuf]) -> Result<RepositoryState, StateCaptureError>,
    D: FnMut(&Path, &[PathBuf]) -> Result<Vec<PathBuf>, StateCaptureError>,
{
    let mut backend = Ordinary {
        capture: |_: CapturePhase, root: &Path, scope: &[PathBuf]| capture(root, scope),
        discover,
        execute: execution::execute,
    };
    infallible(realize_core(
        repository,
        profile,
        context,
        &mut backend,
        &mut ordinary_control(),
    ))
}

/// Guarded implementation hook. The outer error never becomes discovery_failed.
/// The caller retains control across realization, evaluation and finalization.
pub fn realize_guarded<B: Backend<Abort = HostAbort>>(
    repository: &Path,
    profile: &ConsumerIntegrityProfile,
    context: &EvaluationContext,
    backend: &mut B,
    control: &mut AttemptControl,
) -> Result<Result<RealizedIntegrityProfile, RepositoryIntegrityError>, Interruption> {
    realize_core(repository, profile, context, backend, control)
        .map_err(|reason| control.interrupted(reason))
}

pub(super) fn realize_core<B: Backend>(
    repository: &Path,
    profile: &ConsumerIntegrityProfile,
    context: &EvaluationContext,
    backend: &mut B,
    control: &mut AttemptControl,
) -> Result<Result<RealizedIntegrityProfile, RepositoryIntegrityError>, B::Abort> {
    control.observe::<_, B>(backend.checkpoint(CoherenceCheckpoint::BeforeRealization))?;
    let result = realize(repository, profile, context, backend, control)?;
    // Even ordinary validation errors cross this boundary; an abort never does.
    control.observe::<_, B>(backend.checkpoint(CoherenceCheckpoint::AfterRealization))?;
    Ok(result)
}

fn realize<B: Backend>(
    repository: &Path,
    profile: &ConsumerIntegrityProfile,
    context: &EvaluationContext,
    backend: &mut B,
    control: &mut AttemptControl,
) -> Result<Result<RealizedIntegrityProfile, RepositoryIntegrityError>, B::Abort> {
    let root = match repository_state::resolve_worktree_root(repository) {
        Ok(root) => root,
        Err(error) => {
            return Ok(Err(failure(format!(
                "cannot resolve evaluation repository: {error}"
            ))));
        }
    };
    if root != profile.repository {
        return Ok(Err(failure(
            "evaluation repository differs from profile repository",
        )));
    }
    let context_identity = match evaluation_context_identity(profile, context) {
        Ok(identity) => identity,
        Err(error) => return Ok(Err(error)),
    };
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
    // Resolve the outer channel before mapping ordinary failures to failed resolution.
    let discovery = if has_path_instances {
        discover(backend, control, &root, &literal_scope)?
    } else {
        Ok(None)
    };
    let (repository_paths, resolution_state_identity, discovery_failed) = match discovery {
        Ok(Some((before, paths, after))) if before.identity == after.identity => {
            (paths, Some(after.identity), false)
        }
        Ok(None) => (Vec::new(), None, false),
        _ => (Vec::new(), None, true),
    };
    // All remaining work is ordinary profile construction, without host operations.
    Ok((|| {
        let mut governed_paths = literal_paths;
        let mut validations = Vec::with_capacity(profile.order.len());
        for validation_id in &profile.order {
            let definition = profile
                .validations
                .get(validation_id)
                .ok_or_else(|| failure("profile order references absent validation"))?;
            let argument_sets = match &definition.instances {
                ValidationInstances::Single => Some(vec![definition.command.arguments.clone()]),
                ValidationInstances::RepositoryPaths { .. } if discovery_failed => None,
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
                            undetermined_exit_codes: definition
                                .command
                                .undetermined_exit_codes
                                .clone(),
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
    })())
}

type Discovery =
    Result<Option<(RepositoryState, Vec<PathBuf>, RepositoryState)>, StateCaptureError>;
fn discover<B: Backend>(
    backend: &mut B,
    control: &mut AttemptControl,
    root: &Path,
    scope: &[PathBuf],
) -> Result<Discovery, B::Abort> {
    let before = match control.observe::<_, B>(backend.capture(
        CapturePhase::BeforeDiscovery,
        root,
        scope,
    ))? {
        Ok(state) => state,
        Err(error) => return Ok(Err(error)),
    };
    let paths = match control.observe::<_, B>(backend.discover(root, &[]))? {
        Ok(paths) => paths,
        Err(error) => return Ok(Err(error)),
    };
    let after =
        control.observe::<_, B>(backend.capture(CapturePhase::AfterDiscovery, root, scope))?;
    Ok(after.map(|after| Some((before, paths, after))))
}
