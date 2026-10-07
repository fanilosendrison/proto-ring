from __future__ import annotations

import os
from pathlib import Path
import subprocess
from unittest import mock

from proto_ring import (
    git_whitespace,
    github_authoritative_ref_monotonicity,
    repository_governance_state,
    repository_integrity,
    repository_integrity_evaluation,
    repository_state,
)

from conformance_corpus_test_support import observation
from conformance_fixture_runner import command_argv

OWNED_RESPONSIBILITIES = frozenset({
    "repository-state.capture", "repository-governance-state.compose",
    "repository-integrity.evaluate", "git-whitespace.validate",
    "github-authoritative-ref-monotonicity.observe",
})


def _success(value: object):
    return observation("result", value)


def _rejection(responsibility_id: str):
    return observation("controlled_rejection", {"category": f"{responsibility_id}.rejected"})


def _apply_mutation(root: Path, action: dict[str, object] | None) -> None:
    if not action:
        return
    kind = action["action"]
    path = root / str(action.get("path", ""))
    if kind == "write_utf8":
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(str(action["text"]), encoding="utf-8")
    elif kind == "append_utf8":
        with path.open("a", encoding="utf-8") as stream:
            stream.write(str(action["text"]))
    elif kind == "remove":
        if path.exists() or path.is_symlink():
            path.unlink()
    elif kind == "touch":
        path.parent.mkdir(parents=True, exist_ok=True)
        path.touch()
    elif kind == "chmod":
        path.chmod(int(str(action["mode"]), 8))
    elif kind == "git":
        subprocess.run(["git", *action["argv"]], cwd=root, check=True, capture_output=True)
    else:
        raise ValueError(f"unsupported mutation action: {kind}")


def _state_value(state: repository_state.RepositoryState):
    return {
        "repository_is_root": True,
        "scope_paths": list(state.scope_paths),
        "identity_present": bool(state.identity),
        "identity": state.identity,
    }


def _git_observation(root: Path, participating: list[str] | None = None):
    def git(*args: str) -> str:
        completed = subprocess.run(["git", *args], cwd=root, capture_output=True, check=False)
        return f"{completed.returncode}:{completed.stdout.hex()}:{completed.stderr.hex()}"
    return {
        "status": git("status", "--porcelain=v1", "-z"),
        "head": git("rev-parse", "HEAD"),
        "index": git("ls-files", "--stage", "-z"),
        "config": git("config", "--local", "--list", "-z"),
        "bytes": {
            path: None if not (root / path).is_file() else (root / path).read_bytes().hex()
            for path in participating or []
        },
    }


def _state(root: Path, arguments: dict[str, object]):
    operation = arguments.get("operation", "capture_state")
    scope = tuple(arguments.get("scope_paths", []))
    if operation == "capture_read_only":
        before = _git_observation(root, arguments.get("participating_paths", []))
        state = repository_state.capture(root, scope)
        after = _git_observation(root, arguments.get("participating_paths", []))
        return {"identity": state.identity, "all_equal": before == after, "relations": {key: "EQUAL" if before[key] == after[key] else "NOT_EQUAL" for key in before}}
    if operation == "capture_with_ambient":
        control = repository_state.capture(root, scope)
        with mock.patch.dict(os.environ, arguments["ambient"], clear=False):
            redirected = repository_state.capture(root, scope)
        return {"identity": control.identity, "relation": "EQUAL" if control.identity == redirected.identity else "NOT_EQUAL"}
    fault = arguments.get("fault")
    if fault and fault["kind"] == "repository_mutation_during_capture":
        original = repository_state._capture_path_records
        triggered = False
        def interfering(*args, **kwargs):
            nonlocal triggered
            result = original(*args, **kwargs)
            if not triggered:
                triggered = True
                _apply_mutation(root, fault.get("mutation"))
            return result
        with mock.patch.object(repository_state, "_capture_path_records", side_effect=interfering):
            try:
                repository_state.capture(root, scope)
            except repository_state.StateCaptureError:
                return {"relation": "CONTROLLED_FAILURE"}
        return {"relation": "NO_FAILURE"}
    repository = root / str(arguments.get("repository_subpath", ""))
    try:
        return _state_value(repository_state.capture(repository, scope))
    except repository_state.StateCaptureError:
        if operation == "capture_failure_category":
            return {"relation": "CONTROLLED_FAILURE"}
        raise


def _state_relation(invocations: dict[str, object]):
    left, right = list(invocations)[:2]
    relation = "EQUAL" if invocations[left]["identity"] == invocations[right]["identity"] else "NOT_EQUAL"
    return _success({"left": left, "right": right, "relation": relation})


def _rgs_value(state, extra: dict[str, object] | None = None):
    value = {
        "stage": "composition",
        "scope_paths": list(state.observed_state.scope_paths),
        "identity_present": bool(state.observed_state.identity),
        "optional_components": {
            "governed_objects": state.governed_objects is not None,
            "repository_integrity": state.repository_integrity is not None,
            "projection_registry": state.projection_registry is not None,
            "evidence_requirements": state.evidence_requirements is not None,
        },
        "identity": state.observed_state.identity,
    }
    value.update(extra or {})
    return value


def _rgs(root: Path, arguments: dict[str, object]):
    fault = arguments.get("fault")
    before = _git_observation(root, arguments.get("participating_paths", []))
    provider_calls = 0
    try:
        if fault and fault["kind"] == "observation_scope_mutation_during_governance_state_construction":
            scopes = [tuple(fault["before_scope"]), tuple(fault["after_scope"])]
            calls = 0
            def scope(*_args, **_kwargs):
                nonlocal calls
                value = scopes[min(calls, 1)]
                calls += 1
                return value
            with mock.patch.object(repository_governance_state, "_observation_scope", side_effect=scope):
                state = repository_governance_state.load(root)
        elif fault and fault["kind"] == "repository_mutation_during_governance_state_construction":
            real = repository_governance_state.governance_bootstrap.load
            calls = 0
            def mutate(repository):
                nonlocal calls
                calls += 1
                if calls == 2:
                    _apply_mutation(root, fault.get("mutation"))
                return real(repository)
            with mock.patch.object(repository_governance_state.governance_bootstrap, "load", side_effect=mutate):
                state = repository_governance_state.load(root)
        elif fault and fault["kind"] == "provider_network_access_forbidden":
            def forbidden(*_args, **_kwargs):
                nonlocal provider_calls
                provider_calls += 1
                raise AssertionError("provider access forbidden by fixture")
            with mock.patch("urllib.request.urlopen", side_effect=forbidden):
                state = repository_governance_state.load(root)
        else:
            state = repository_governance_state.load(root)
    except repository_governance_state.RepositoryGovernanceStateError as error:
        if arguments.get("qualified_observation"):
            return {"status": "CONTROLLED_FAILURE", "stage": error.diagnostic.stage.value}
        if fault:
            return {"relation": "CONTROLLED_FAILURE"}
        raise
    if not arguments.get("qualified_observation"):
        return _rgs_value(state)
    after = _git_observation(root, arguments.get("participating_paths", []))
    extra = {
        "capabilities": sorted(state.repository_governance_model.capabilities),
        "provider_interactions": provider_calls,
        "marker_absent": not (root / str(arguments.get("marker_path", "__none__"))).exists(),
        "purity": {key: before[key] == after[key] for key in before},
    }
    return _rgs_value(state, extra)


def _action_argv(action: dict[str, object]) -> tuple[str, ...]:
    return command_argv({key: value for key, value in action.items() if key not in {"id", "undetermined_exit_codes"}})


def _legacy_integrity_profile(arguments: dict[str, object]):
    obligations = tuple(repository_integrity.CommandObligation(
        action["id"], _action_argv(action), frozenset(action.get("undetermined_exit_codes", []))
    ) for action in arguments["actions"])
    return repository_integrity.IntegrityProfile(
        obligations, bool(arguments.get("continue_after_non_satisfied", True))
    )


def _consumer_profile(root: Path, value: dict[str, object]):
    definitions = {}
    for item in value["validations"]:
        instances = item["instances"]
        selectors = tuple(repository_integrity.RepositoryPathSelector(
            selector["kind"], selector["value"]
        ) for selector in instances.get("selectors", []))
        definition = repository_integrity.ValidationDefinition(
            item["id"], item["responsibility"], tuple(item["prerequisites"]), None,
            repository_integrity.ValidationInstances(
                instances["kind"], instances.get("mode"), selectors
            ),
            repository_integrity.CommandBinding(
                item["command"]["environment"], tuple(item["command"]["arguments"]),
                frozenset(item["command"]["undetermined_exit_codes"]),
            ),
        )
        definitions[item["id"]] = definition
    authority = value["authority"]
    return repository_integrity.ConsumerIntegrityProfile(
        root.resolve(), root / value["carrier"], 1,
        repository_integrity.ProfileAuthority(authority["responsibility"], authority["source"]),
        frozenset(value["environments"]), value["continue_after_non_satisfied"],
        definitions, tuple(value["order"]), value["identity"],
    )


def _context(value: dict[str, object]):
    realizations = {}
    for environment_id, item in value["realizations"].items():
        prefix = tuple(item["command_prefix"]) if "command_prefix" in item else command_argv(item["action"])
        realizations[environment_id] = repository_integrity.ValidationEnvironmentRealization(
            environment_id, item["identity"], prefix, item["process_environment"]
        )
    return repository_integrity.EvaluationContext(realizations)


def _evaluate_with_fault(root: Path, fault, operation):
    if not fault:
        return operation()
    kind = fault["kind"]
    if kind == "repository_path_discovery_unavailable":
        with mock.patch.object(repository_integrity_evaluation, "discover_repository_paths", side_effect=repository_state.StateCaptureError("fixture discovery unavailable")):
            return operation()
    if kind in {"repository_state_unavailable_before_obligation", "repository_state_unavailable_after_obligation"}:
        real = repository_integrity_evaluation.capture
        calls = 0
        fail_at = 2 if kind.endswith("before_obligation") else 3
        def capture(*args, **kwargs):
            nonlocal calls
            calls += 1
            if calls == fail_at:
                raise repository_state.StateCaptureError("fixture state unavailable")
            return real(*args, **kwargs)
        with mock.patch.object(repository_integrity_evaluation, "capture", side_effect=capture):
            return operation()
    if kind == "repository_mutation_during_integrity_evaluation":
        if fault["phase"] != "after_baseline_before_obligation":
            raise ValueError("unsupported integrity mutation phase")
        real = repository_integrity_evaluation.capture
        calls = 0
        def capture(*args, **kwargs):
            nonlocal calls
            calls += 1
            result = real(*args, **kwargs)
            if calls == 1:
                _apply_mutation(root, fault.get("mutation"))
            return result
        with mock.patch.object(repository_integrity_evaluation, "capture", side_effect=capture):
            return operation()
    return operation()


def _integrity_value(result, root: Path, observed: list[str], unavailable: bool, executed: list[list[str]] | None = None):
    return {
        "verdict": result.verdict.value,
        "validations": [{
            "id": item.validation_id, "status": item.status.value,
            "obligations": [{"id": obligation.name, "status": obligation.status.value} for obligation in item.obligations],
        } for item in result.validations],
        "obligations": [{"id": item.name, "status": item.status.value} for item in result.obligations],
        "state_relation": "UNAVAILABLE" if unavailable or result.baseline_state_identity is None or result.final_state_identity is None else ("EQUAL" if result.baseline_state_identity == result.final_state_identity else "NOT_EQUAL"),
        "side_effects": {path: {"present": (root / path).exists(), "text": (root / path).read_text(encoding="utf-8") if (root / path).is_file() else None} for path in observed},
        "executed_arguments": executed or [],
    }


def _integrity(root: Path, arguments: dict[str, object]):
    fault = arguments.get("fault")
    if arguments.get("operation") != "evaluate_consumer_profile":
        result = _evaluate_with_fault(root, fault, lambda: repository_integrity.evaluate(
            root, _legacy_integrity_profile(arguments), env={}, evaluation_context_identity="corpus"
        ))
        value = _integrity_value(result, root, [str(arguments.get("marker_path", "__none__"))], bool(arguments.get("state_unavailable_phase")))
        if arguments.get("qualified_execution_observation"):
            obligations = [{"id": item.name, "status": item.status.value, "executed": item.returncode is not None} for item in result.obligations]
            return _success({"verdict": value["verdict"], "obligations": obligations,
                "state_relation": value["state_relation"], "state_coherence_halt": value["state_relation"] == "NOT_EQUAL" and not any(item["executed"] for item in obligations),
                "marker_absent": not value["side_effects"][str(arguments["marker_path"])]["present"]})
        return _success({"verdict": value["verdict"], "obligations": value["obligations"],
            "state_relation": value["state_relation"], "marker_absent": not value["side_effects"][str(arguments.get("marker_path", "__none__"))]["present"]})
    profile = _consumer_profile(root, arguments["profile"])
    values, identities, names = [], [], []
    for context in arguments["contexts"]:
        executed: list[list[str]] = []
        real_run = subprocess.run
        def record_run(argv, *args, **kwargs):
            command = list(argv)
            if "--command-action" in command:
                executed.append(command[command.index("--command-action") + 2 :])
            return real_run(argv, *args, **kwargs)
        with mock.patch.dict(os.environ, arguments.get("ambient_environment", {}), clear=False), mock.patch.object(repository_integrity_evaluation.subprocess, "run", side_effect=record_run):
            result = _evaluate_with_fault(root, fault, lambda context=context: repository_integrity.evaluate_consumer_profile(root, profile, _context(context)))
        identities.append(result.evaluation_context_identity); names.append(str(context["name"]))
        values.append(_integrity_value(result, root, arguments.get("observe_paths", []), bool(fault and fault["kind"].startswith("repository_state_unavailable_")), executed))
    if len(values) == 1:
        return _success(values[0])
    return _success({"evaluations": values, "identity_relations": [{"left": names[0], "right": names[index], "relation": "EQUAL" if identities[0] == identities[index] else "NOT_EQUAL"} for index in (1, 2)]})


def _whitespace(root: Path, arguments: dict[str, object]):
    environment = {str(key): str(value) for key, value in arguments.get("environment", {}).items()}
    if "GITHUB_EVENT_PATH" in environment and not os.path.isabs(environment["GITHUB_EVENT_PATH"]):
        environment["GITHUB_EVENT_PATH"] = os.fspath(root / environment["GITHUB_EVENT_PATH"])
    fault = arguments.get("fault")
    if fault and fault["kind"] == "git_process_start_unavailable":
        with mock.patch.object(git_whitespace.subprocess, "run", side_effect=OSError("git unavailable")):
            errors = git_whitespace.check(root, env=environment)
    else:
        errors = git_whitespace.check(root, env=environment)
    domains = []
    labels = [error.split(":", 1)[0] for error in errors]
    for label, domain in (("unstaged whitespace check", "unstaged"), ("staged whitespace check", "staged"), ("GitHub push committed-range whitespace check", "push_committed_range"), ("GitHub pull-request committed-range whitespace check", "pull_request_committed_range")):
        if label in labels:
            domains.append(domain)
    if fault and fault["kind"] == "git_process_start_unavailable":
        domains = ["git_observation"]
    elif errors and not domains:
        domains = ["range_determination"]
    return _success({"status": "PASS" if not errors else "NON_PASS", "domains": domains})


def _provider(root: Path, arguments: dict[str, object], fixture):
    assert fixture.provider is not None
    with mock.patch("urllib.request.urlopen", side_effect=fixture.provider.urlopen):
        result = github_authoritative_ref_monotonicity.check(root / arguments["binding_path"])
    return _success({"status": result.status.value})


def _dispatch(responsibility_id: str, root: Path | None, arguments: dict[str, object], fixture):
    try:
        if responsibility_id == "repository-state.capture":
            return _state(root, arguments)
        if responsibility_id == "repository-governance-state.compose":
            return _rgs(root, arguments)
        if responsibility_id == "repository-integrity.evaluate":
            return _integrity(root, arguments)
        if responsibility_id == "git-whitespace.validate":
            return _whitespace(root, arguments)
        if responsibility_id == "github-authoritative-ref-monotonicity.observe":
            return _provider(root, arguments, fixture)
    except (ValueError, OSError, TypeError, repository_state.StateCaptureError, repository_governance_state.RepositoryGovernanceStateError):
        return _rejection(responsibility_id)
    raise ValueError(f"unsupported responsibility: {responsibility_id}")


def execute(responsibility_id, fixture):
    if responsibility_id not in OWNED_RESPONSIBILITIES:
        raise ValueError(f"responsibility is not owned by adapter 67/69: {responsibility_id}")
    raw = fixture.run(lambda root, arguments: _dispatch(responsibility_id, root, arguments, fixture))
    if isinstance(raw, dict) and set(raw) == {"kind", "value"}:
        return raw
    if responsibility_id == "repository-governance-state.compose" and isinstance(raw, dict):
        if "invocations" in raw:
            values = raw["invocations"]
            return _state_relation(values)
        if "identity" in raw:
            raw = {key: value for key, value in raw.items() if key != "identity"}
        return _success(raw)
    if responsibility_id == "repository-state.capture" and isinstance(raw, dict):
        if "invocations" in raw and all("identity" in value for value in raw["invocations"].values()):
            return _state_relation(raw["invocations"])
        if "identity" in raw:
            raw = {key: value for key, value in raw.items() if key != "identity"}
        return _success(raw)
    return raw
