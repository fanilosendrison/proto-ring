"""Resolve and execute Repository Integrity runtime projections."""
from __future__ import annotations
import hashlib
import json
import os
import subprocess
from collections.abc import Mapping
from pathlib import Path
from proto_ring.repository_integrity import (
    CommandObligation, ConsumerIntegrityProfile, EvaluationContext, IntegrityProfile,
    IntegrityResult, IntegrityVerdict, ObligationResult, ObligationStatus,
    RepositoryIntegrityError, ValidationResult, _RuntimeValidation, _selected_paths,
)
from proto_ring.repository_state import (
    StateCaptureError, capture, discover_repository_paths, resolve_worktree_root,
)
def _runtime_identity(profile: IntegrityProfile) -> str:
    if profile.persistent_identity is not None:
        return profile.persistent_identity
    obligations = [
        {
            "name": item.name,
            "argv": list(item.argv),
            "undetermined_exit_codes": sorted(item.undetermined_exit_codes),
        }
        for item in profile.obligations
    ]
    payload = {
        "continue_after_non_satisfied": profile.continue_after_non_satisfied,
        "obligations": obligations,
    }
    canonical = json.dumps(
        payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    )
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()
def _context_identity(profile: ConsumerIntegrityProfile,
                      context: EvaluationContext) -> str:
    required = {item.command.environment for item in profile.validations.values()}
    payload = {
        environment_id: context.realizations[environment_id].identity
        if environment_id in context.realizations
        else None
        for environment_id in sorted(required)
    }
    canonical = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()
def _resolve_profile(repository: Path, profile: ConsumerIntegrityProfile,
                     context: EvaluationContext) -> tuple[IntegrityProfile, str]:
    path_definitions = tuple(
        definition
        for definition in profile.validations.values()
        if definition.instances.kind == "repository_paths"
    )
    literal_paths = tuple(sorted({
        selector.value
        for definition in path_definitions
        for selector in definition.instances.selectors
        if selector.kind == "path"
    }))
    resolution_state_identity: str | None = None
    try:
        if path_definitions:
            root = resolve_worktree_root(repository)
            before_resolution = capture(root, literal_paths).identity
            repository_paths = discover_repository_paths(root)
            resolution_state_identity = capture(root, literal_paths).identity
            if before_resolution != resolution_state_identity:
                raise StateCaptureError("repository changed during instance resolution")
        else:
            repository_paths = ()
        discovery_error = None
    except StateCaptureError as error:
        repository_paths = ()
        discovery_error = str(error)
    runtime_validations: list[_RuntimeValidation] = []
    all_obligations: list[CommandObligation] = []
    governed_paths = set(literal_paths)
    for validation_id in profile.order:
        definition = profile.validations[validation_id]
        if definition.instances.kind == "repository_paths" and discovery_error:
            runtime_validations.append(_RuntimeValidation(
                validation_id, definition.prerequisites, (),
                f"instance resolution failed: {discovery_error}",
            ))
            continue
        try:
            if definition.instances.kind == "single":
                argument_sets = (definition.command.arguments,)
            else:
                selected = _selected_paths(definition.instances, repository_paths)
                governed_paths.update(selected)
                if definition.instances.mode == "append_all":
                    argument_sets = (definition.command.arguments + selected,)
                else:
                    argument_sets = tuple(
                        definition.command.arguments + (path,) for path in selected
                    )
        except RepositoryIntegrityError as error:
            runtime_validations.append(_RuntimeValidation(
                validation_id, definition.prerequisites, (),
                f"instance resolution failed: {error}",
            ))
            continue
        realization = context.realizations.get(definition.command.environment)
        obligations: list[CommandObligation] = []
        for arguments in argument_sets:
            if realization is None:
                obligation = CommandObligation(
                    validation_id,
                    arguments,
                    definition.command.undetermined_exit_codes,
                    unavailable_detail=(
                        "validation environment is unavailable: "
                        f"{definition.command.environment}"
                    ),
                )
            else:
                obligation = CommandObligation(
                    validation_id,
                    realization.command_prefix + arguments,
                    definition.command.undetermined_exit_codes,
                    dict(realization.process_environment),
                )
            obligations.append(obligation)
            all_obligations.append(obligation)
        runtime_validations.append(
            _RuntimeValidation(
                validation_id, definition.prerequisites, tuple(obligations)
            )
        )
    return (
        IntegrityProfile(
            tuple(all_obligations),
            profile.continue_after_non_satisfied,
            tuple(runtime_validations),
            profile.identity,
            tuple(sorted(governed_paths)),
            resolution_state_identity,
        ),
        _context_identity(profile, context),
    )
def _execute_obligation(root: Path, obligation: CommandObligation,
                        default_environment: Mapping[str, str]) -> ObligationResult:
    if obligation.unavailable_detail is not None:
        return ObligationResult(
            obligation.name,
            ObligationStatus.UNDETERMINED,
            None,
            obligation.unavailable_detail,
        )
    environment = obligation.environment
    if environment is None:
        environment = default_environment
    try:
        completed = subprocess.run(
            list(obligation.argv),
            cwd=os.fspath(root),
            env=dict(environment),
            check=False,
        )
    except (OSError, ValueError, IndexError) as error:
        return ObligationResult(
            obligation.name,
            ObligationStatus.UNDETERMINED,
            None,
            f"command could not be started: {error}",
        )
    if completed.returncode == 0:
        return ObligationResult(obligation.name, ObligationStatus.SATISFIED, 0, None)
    status = (
        ObligationStatus.UNDETERMINED
        if completed.returncode in obligation.undetermined_exit_codes
        else ObligationStatus.VIOLATED
    )
    return ObligationResult(
        obligation.name,
        status,
        completed.returncode,
        f"command exited with {completed.returncode}",
    )
def _aggregate(results: list[ObligationResult]) -> ObligationStatus:
    if any(result.status is ObligationStatus.VIOLATED for result in results):
        return ObligationStatus.VIOLATED
    if any(result.status is ObligationStatus.UNDETERMINED for result in results):
        return ObligationStatus.UNDETERMINED
    return ObligationStatus.SATISFIED
def _skipped(validation: _RuntimeValidation, detail: str) -> ValidationResult:
    obligations = tuple(
        ObligationResult(
            obligation.name, ObligationStatus.UNDETERMINED, None, detail
        )
        for obligation in validation.obligations
    )
    return ValidationResult(
        validation.validation_id, ObligationStatus.UNDETERMINED, obligations, detail
    )
def _runtime_validations(profile: IntegrityProfile) -> tuple[_RuntimeValidation, ...]:
    if profile.validations:
        return profile.validations
    return tuple(
        _RuntimeValidation(obligation.name, (), (obligation,))
        for obligation in profile.obligations
    )
def evaluate(
    repository: Path,
    profile: IntegrityProfile,
    *,
    env: Mapping[str, str],
    evaluation_context_identity: str,
) -> IntegrityResult:
    """Evaluate a resolved runtime profile against one exact Git worktree."""
    if not evaluation_context_identity:
        raise ValueError("evaluation_context_identity must be non-empty")
    profile_identity = _runtime_identity(profile)
    validations = _runtime_validations(profile)
    try:
        root = resolve_worktree_root(Path(repository))
        baseline_identity = capture(root, profile.governed_paths).identity
    except StateCaptureError as error:
        detail = f"baseline repository state could not be determined: {error}"
        validation_results = tuple(_skipped(validation, detail) for validation in validations)
        return IntegrityResult(
            IntegrityVerdict.NON_PASS,
            profile_identity,
            evaluation_context_identity,
            None,
            None,
            tuple(result for validation in validation_results for result in validation.obligations),
            validation_results,
            (detail,),
        )
    if (
        profile.resolution_state_identity is not None
        and baseline_identity != profile.resolution_state_identity
    ):
        detail = "repository state changed after concrete instance resolution"
        validation_results = tuple(_skipped(item, detail) for item in validations)
        return IntegrityResult(
            IntegrityVerdict.NON_PASS,
            profile_identity,
            evaluation_context_identity,
            baseline_identity,
            baseline_identity,
            tuple(item for result in validation_results for item in result.obligations),
            validation_results,
            (detail,),
        )
    errors: list[str] = []
    validation_results: list[ValidationResult] = []
    aggregate_by_id: dict[str, ObligationStatus] = {}
    final_identity: str | None = baseline_identity
    halted = False
    stop_detail = "obligation skipped after previous non-satisfied result"
    for validation in validations:
        if halted:
            result = _skipped(validation, stop_detail)
        elif validation.resolution_error is not None:
            result = ValidationResult(
                validation.validation_id,
                ObligationStatus.UNDETERMINED,
                (),
                validation.resolution_error,
            )
        elif any(
            aggregate_by_id.get(prerequisite) is not ObligationStatus.SATISFIED
            for prerequisite in validation.prerequisites
        ):
            result = _skipped(
                validation,
                "validation not executed because a prerequisite is not SATISFIED",
            )
        else:
            obligation_results: list[ObligationResult] = []
            for obligation in validation.obligations:
                try:
                    before_identity = capture(root, profile.governed_paths).identity
                except StateCaptureError as error:
                    detail = (
                        "repository state could not be determined before obligation "
                        f"{obligation.name}: {error}"
                    )
                    obligation_results.append(
                        ObligationResult(
                            obligation.name,
                            ObligationStatus.UNDETERMINED,
                            None,
                            detail,
                        )
                    )
                    errors.append(detail)
                    halted = True
                    break
                final_identity = before_identity
                if before_identity != baseline_identity:
                    detail = (
                        f"repository state changed before obligation {obligation.name}"
                    )
                    obligation_results.append(
                        ObligationResult(
                            obligation.name,
                            ObligationStatus.UNDETERMINED,
                            None,
                            "obligation not executed because repository state no longer matches baseline",
                        )
                    )
                    errors.append(detail)
                    halted = True
                    break
                obligation_result = _execute_obligation(root, obligation, env)
                try:
                    after_identity = capture(root, profile.governed_paths).identity
                except StateCaptureError as error:
                    detail = (
                        "repository state could not be determined after obligation "
                        f"{obligation.name}: {error}"
                    )
                    obligation_result = ObligationResult(
                        obligation.name,
                        ObligationStatus.UNDETERMINED,
                        obligation_result.returncode,
                        detail,
                    )
                    errors.append(detail)
                    halted = True
                    obligation_results.append(obligation_result)
                    break
                final_identity = after_identity
                if after_identity != baseline_identity:
                    detail = f"repository state changed during obligation {obligation.name}"
                    obligation_result = ObligationResult(
                        obligation.name,
                        ObligationStatus.VIOLATED,
                        obligation_result.returncode,
                        detail,
                    )
                    errors.append(detail)
                    halted = True
                obligation_results.append(obligation_result)
                if halted:
                    break
            for obligation in validation.obligations[len(obligation_results) :]:
                obligation_results.append(
                    ObligationResult(
                        obligation.name,
                        ObligationStatus.UNDETERMINED,
                        None,
                        stop_detail,
                    )
                )
            status = _aggregate(obligation_results)
            result = ValidationResult(
                validation.validation_id, status, tuple(obligation_results)
            )
        validation_results.append(result)
        aggregate_by_id[validation.validation_id] = result.status
        if (
            result.status is not ObligationStatus.SATISFIED
            and not profile.continue_after_non_satisfied
        ):
            halted = True
    flattened = tuple(
        obligation
        for validation_result in validation_results
        for obligation in validation_result.obligations
    )
    all_satisfied = all(
        result.status is ObligationStatus.SATISFIED for result in validation_results
    )
    verdict = (
        IntegrityVerdict.PASS
        if final_identity == baseline_identity and all_satisfied and not errors
        else IntegrityVerdict.NON_PASS
    )
    return IntegrityResult(
        verdict,
        profile_identity,
        evaluation_context_identity,
        baseline_identity,
        final_identity,
        flattened,
        tuple(validation_results),
        tuple(errors),
    )
def evaluate_consumer_profile(
    repository: Path,
    profile: ConsumerIntegrityProfile,
    context: EvaluationContext,
) -> IntegrityResult:
    """Resolve a persistent profile and evaluate its runtime projection."""
    if Path(repository).resolve() != profile.repository.resolve():
        raise RepositoryIntegrityError("evaluation repository differs from profile repository")
    runtime_profile, context_identity = _resolve_profile(Path(repository), profile, context)
    return evaluate(
        repository,
        runtime_profile,
        env={},
        evaluation_context_identity=context_identity,
    )
