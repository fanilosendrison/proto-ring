"""Persistent consumer profiles and runtime projections for Repository Integrity."""

from __future__ import annotations

import hashlib
import json
import re
from collections.abc import Mapping
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

from proto_ring import structured_data
from proto_ring.governance_authority import GovernanceAuthorityProfile, SourceRole
from proto_ring.governance_routing import ResolvedGovernanceRoute
from proto_ring.governed_objects import GovernedObjectCatalog, GovernedObjectRef
__all__ = [
    "CommandObligation",
    "ConsumerIntegrityProfile",
    "EvaluationContext",
    "IntegrityProfile",
    "IntegrityResult",
    "IntegrityVerdict",
    "ObligationResult",
    "ObligationStatus",
    "ProfileAuthority",
    "RepositoryIntegrityError",
    "ValidationEnvironmentRealization",
    "ValidationResult",
    "evaluate",
    "evaluate_consumer_profile",
    "load",
]

_MODEL_VERSION = 1
_PROFILE_KEYS = frozenset(
    {"model_version", "authority", "environments", "continue_after_non_satisfied", "validations", "order"}
)
_AUTHORITY_KEYS = frozenset({"responsibility", "source"})
_VALIDATION_KEYS = frozenset({"responsibility", "prerequisites", "instances", "command"})
_TARGET_KEYS = frozenset({"interface", "object"})
_COMMAND_KEYS = frozenset({"kind", "environment", "arguments", "undetermined_exit_codes"})
_SINGLE_KEYS = frozenset({"kind"})
_REPOSITORY_PATH_KEYS = frozenset({"kind", "mode", "selectors"})
_PATH_SELECTOR_KEYS = frozenset({"kind", "path"})
_GLOB_SELECTOR_KEYS = frozenset({"kind", "glob"})

class RepositoryIntegrityError(ValueError):
    """Report controlled failure to load a persistent integrity profile."""

class ObligationStatus(StrEnum):
    SATISFIED = "SATISFIED"
    VIOLATED = "VIOLATED"
    UNDETERMINED = "UNDETERMINED"

class IntegrityVerdict(StrEnum):
    PASS = "PASS"
    NON_PASS = "NON_PASS"
@dataclass(frozen=True)
class CommandObligation:
    name: str
    argv: tuple[str, ...]
    undetermined_exit_codes: frozenset[int] = frozenset()
    environment: Mapping[str, str] | None = None
    unavailable_detail: str | None = None
    def __post_init__(self) -> None:
        if 0 in self.undetermined_exit_codes:
            raise ValueError("0 is not a valid undetermined exit code")
@dataclass(frozen=True)
class _RuntimeValidation:
    validation_id: str
    prerequisites: tuple[str, ...]
    obligations: tuple[CommandObligation, ...]
    resolution_error: str | None = None
@dataclass(frozen=True)
class IntegrityProfile:
    obligations: tuple[CommandObligation, ...]
    continue_after_non_satisfied: bool = True
    validations: tuple[_RuntimeValidation, ...] = ()
    persistent_identity: str | None = None
    governed_paths: tuple[str, ...] = ()
    resolution_state_identity: str | None = None
@dataclass(frozen=True)
class ObligationResult:
    name: str
    status: ObligationStatus
    returncode: int | None
    detail: str | None
@dataclass(frozen=True)
class ValidationResult:
    validation_id: str
    status: ObligationStatus
    obligations: tuple[ObligationResult, ...]
    detail: str | None = None
@dataclass(frozen=True)
class IntegrityResult:
    verdict: IntegrityVerdict
    profile_identity: str
    evaluation_context_identity: str
    baseline_state_identity: str | None
    final_state_identity: str | None
    obligations: tuple[ObligationResult, ...]
    validations: tuple[ValidationResult, ...]
    errors: tuple[str, ...]
@dataclass(frozen=True)
class ProfileAuthority:
    responsibility: str
    source: str
@dataclass(frozen=True)
class RepositoryPathSelector:
    kind: str
    value: str
@dataclass(frozen=True)
class ValidationInstances:
    kind: str
    mode: str | None = None
    selectors: tuple[RepositoryPathSelector, ...] = ()
@dataclass(frozen=True)
class CommandBinding:
    environment: str
    arguments: tuple[str, ...]
    undetermined_exit_codes: frozenset[int]
@dataclass(frozen=True)
class ValidationDefinition:
    validation_id: str
    responsibility: str
    prerequisites: tuple[str, ...]
    target: GovernedObjectRef | None
    instances: ValidationInstances
    command: CommandBinding
@dataclass(frozen=True)
class ConsumerIntegrityProfile:
    repository: Path
    carrier: Path
    model_version: int
    authority: ProfileAuthority
    environments: frozenset[str]
    continue_after_non_satisfied: bool
    validations: Mapping[str, ValidationDefinition]
    order: tuple[str, ...]
    identity: str
@dataclass(frozen=True)
class ValidationEnvironmentRealization:
    environment_id: str
    identity: str
    command_prefix: tuple[str, ...]
    process_environment: Mapping[str, str]
    def __post_init__(self) -> None:
        if not self.environment_id or not self.identity:
            raise ValueError("environment and realization identities must be non-empty")
        if not self.command_prefix or any(not isinstance(part, str) for part in self.command_prefix):
            raise ValueError("runtime command prefix must contain strings")
        if any(not isinstance(key, str) or not isinstance(value, str) for key, value in self.process_environment.items()):
            raise ValueError("process environment must map strings to strings")
@dataclass(frozen=True)
class EvaluationContext:
    realizations: Mapping[str, ValidationEnvironmentRealization]
    def __post_init__(self) -> None:
        for environment_id, realization in self.realizations.items():
            if environment_id != realization.environment_id:
                raise ValueError("environment realization key does not match its identity")
def _mapping(value: object, label: str) -> Mapping[object, object]:
    if not isinstance(value, Mapping):
        raise RepositoryIntegrityError(f"{label} must be a mapping")
    return value
def _exact_keys(value: Mapping[object, object], expected: frozenset[str], label: str) -> None:
    actual = frozenset(value.keys())
    missing = expected - actual
    if missing:
        raise RepositoryIntegrityError(f"{label} is missing key: {sorted(missing)[0]}")
    unexpected = actual - expected
    if unexpected:
        raise RepositoryIntegrityError(f"{label} has unexpected key: {sorted(unexpected, key=str)[0]}")
def _identifier(value: object, label: str) -> str:
    if not isinstance(value, str) or value == "":
        raise RepositoryIntegrityError(f"{label} must be a non-empty string")
    return value
def _string_list(value: object, label: str) -> tuple[str, ...]:
    if not isinstance(value, list):
        raise RepositoryIntegrityError(f"{label} must be a list")
    return tuple(_identifier(item, f"{label}[{index}]") for index, item in enumerate(value))
def _path_value(value: object, label: str, *, glob: bool) -> str:
    path = _identifier(value, label)
    if path.startswith("/") or "\\" in path or "\x00" in path:
        raise RepositoryIntegrityError(f"{label} must be a relative POSIX path")
    components = path.split("/")
    if any(component in ("", ".", "..") for component in components):
        raise RepositoryIntegrityError(f"{label} has a forbidden path component")
    if glob:
        if "**" in path or any(character in path for character in "?[]{}"):
            raise RepositoryIntegrityError(f"{label} uses unsupported glob syntax")
    elif "*" in path:
        raise RepositoryIntegrityError(f"{label} literal path contains a wildcard")
    return path
def _load_instances(value: object, label: str) -> ValidationInstances:
    declaration = _mapping(value, label)
    kind = _identifier(declaration.get("kind"), f"{label}.kind")
    if kind == "single":
        _exact_keys(declaration, _SINGLE_KEYS, label)
        return ValidationInstances(kind)
    if kind != "repository_paths":
        raise RepositoryIntegrityError(f"unknown instance kind: {kind}")
    _exact_keys(declaration, _REPOSITORY_PATH_KEYS, label)
    mode = _identifier(declaration["mode"], f"{label}.mode")
    if mode not in ("append_all", "for_each"):
        raise RepositoryIntegrityError(f"unknown repository path mode: {mode}")
    raw_selectors = declaration["selectors"]
    if not isinstance(raw_selectors, list):
        raise RepositoryIntegrityError(f"{label}.selectors must be a list")
    selectors: list[RepositoryPathSelector] = []
    for index, raw_selector in enumerate(raw_selectors):
        selector_label = f"{label}.selectors[{index}]"
        selector = _mapping(raw_selector, selector_label)
        selector_kind = _identifier(selector.get("kind"), f"{selector_label}.kind")
        if selector_kind == "path":
            _exact_keys(selector, _PATH_SELECTOR_KEYS, selector_label)
            selected = _path_value(selector["path"], f"{selector_label}.path", glob=False)
        elif selector_kind == "glob":
            _exact_keys(selector, _GLOB_SELECTOR_KEYS, selector_label)
            selected = _path_value(selector["glob"], f"{selector_label}.glob", glob=True)
        else:
            raise RepositoryIntegrityError(f"unknown selector kind: {selector_kind}")
        selectors.append(RepositoryPathSelector(selector_kind, selected))
    return ValidationInstances(kind, mode, tuple(selectors))
def _load_command(value: object, label: str, environments: frozenset[str]) -> CommandBinding:
    declaration = _mapping(value, label)
    _exact_keys(declaration, _COMMAND_KEYS, label)
    if declaration["kind"] != "command":
        raise RepositoryIntegrityError("unknown command binding kind")
    environment = _identifier(declaration["environment"], f"{label}.environment")
    if environment not in environments:
        raise RepositoryIntegrityError(f"undeclared validation environment: {environment}")
    raw_arguments = declaration["arguments"]
    if not isinstance(raw_arguments, list) or any(
        not isinstance(argument, str) for argument in raw_arguments
    ):
        raise RepositoryIntegrityError(f"{label}.arguments must be a string list")
    arguments = tuple(raw_arguments)
    raw_codes = declaration["undetermined_exit_codes"]
    if not isinstance(raw_codes, list) or any(type(code) is not int for code in raw_codes):
        raise RepositoryIntegrityError(f"{label}.undetermined_exit_codes must be an integer list")
    if len(set(raw_codes)) != len(raw_codes) or 0 in raw_codes:
        raise RepositoryIntegrityError(f"{label}.undetermined_exit_codes is invalid")
    return CommandBinding(environment, arguments, frozenset(raw_codes))
def _target(value: object, label: str) -> GovernedObjectRef:
    declaration = _mapping(value, label)
    _exact_keys(declaration, _TARGET_KEYS, label)
    return GovernedObjectRef(
        _identifier(declaration["interface"], f"{label}.interface"),
        _identifier(declaration["object"], f"{label}.object"),
    )
def _validate_target(target: GovernedObjectRef, responsibility: str, catalog: GovernedObjectCatalog | None) -> None:
    if catalog is None:
        raise RepositoryIntegrityError("governed-object catalog is required for a validation target")
    interface = catalog.interfaces.get(target.interface_id)
    governed_object = None if interface is None else interface.objects.get(target.object_id)
    if governed_object is None:
        raise RepositoryIntegrityError(f"governed object does not exist: {target.interface_id}/{target.object_id}")
    if responsibility not in governed_object.responsibilities:
        raise RepositoryIntegrityError("governed object does not participate in validation responsibility")
def _identity(profile_data: Mapping[object, object]) -> str:
    normalized = dict(profile_data)
    normalized["environments"] = sorted(profile_data["environments"])
    normalized_validations: dict[str, object] = {}
    for validation_id, raw_value in _mapping(
        profile_data["validations"], "validations"
    ).items():
        value = dict(_mapping(raw_value, f"validation {validation_id}"))
        value["prerequisites"] = sorted(value["prerequisites"])
        command = dict(_mapping(value["command"], "command"))
        command["undetermined_exit_codes"] = sorted(
            command["undetermined_exit_codes"]
        )
        value["command"] = command
        normalized_validations[str(validation_id)] = value
    normalized["validations"] = normalized_validations
    canonical = json.dumps(
        normalized, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    )
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()
def _validate_graph(validations: Mapping[str, ValidationDefinition], order: tuple[str, ...]) -> None:
    if len(set(order)) != len(order) or set(order) != set(validations):
        raise RepositoryIntegrityError("order must contain every ValidationId exactly once")
    positions = {validation_id: index for index, validation_id in enumerate(order)}
    for validation in validations.values():
        if len(set(validation.prerequisites)) != len(validation.prerequisites):
            raise RepositoryIntegrityError(f"duplicate prerequisite for {validation.validation_id}")
        for prerequisite in validation.prerequisites:
            if prerequisite not in validations:
                raise RepositoryIntegrityError(f"unknown prerequisite: {prerequisite}")
            if prerequisite == validation.validation_id:
                raise RepositoryIntegrityError("validation cannot depend on itself")
            if positions[prerequisite] >= positions[validation.validation_id]:
                raise RepositoryIntegrityError("prerequisite must precede its dependent")
def load(repository: Path, profile_route: ResolvedGovernanceRoute, authority_profile: GovernanceAuthorityProfile, governed_objects: GovernedObjectCatalog | None = None) -> ConsumerIntegrityProfile:
    """Load and validate one canonical consumer-owned model-version-1 profile."""
    try:
        root = repository.resolve()
        if authority_profile.repository.resolve() != root:
            raise RepositoryIntegrityError("integrity and authority repositories differ")
        if governed_objects is not None and governed_objects.repository.resolve() != root:
            raise RepositoryIntegrityError("integrity and governed-object repositories differ")
        data = profile_route.target.read_bytes()
    except OSError as error:
        raise RepositoryIntegrityError(f"cannot read Repository Integrity profile: {error}") from error
    try:
        parsed = structured_data.parse_frontmatter_bytes(data)
    except structured_data.StructuredDataError as error:
        raise RepositoryIntegrityError(f"cannot parse Repository Integrity profile: {error}") from error
    if "repository_integrity" not in parsed.metadata:
        raise RepositoryIntegrityError("repository_integrity is required")
    raw_profile = _mapping(parsed.metadata["repository_integrity"], "repository_integrity")
    _exact_keys(raw_profile, _PROFILE_KEYS, "repository_integrity")
    if type(raw_profile["model_version"]) is not int or raw_profile["model_version"] != _MODEL_VERSION:
        raise RepositoryIntegrityError("unsupported model_version")
    raw_authority = _mapping(raw_profile["authority"], "authority")
    _exact_keys(raw_authority, _AUTHORITY_KEYS, "authority")
    authority = ProfileAuthority(_identifier(raw_authority["responsibility"], "authority.responsibility"), _identifier(raw_authority["source"], "authority.source"))
    responsibility = authority_profile.responsibilities.get(authority.responsibility)
    source = authority_profile.sources.get(authority.source)
    if responsibility is None or responsibility.roles.get(authority.source) is not SourceRole.AUTHORITY:
        raise RepositoryIntegrityError("profile source is not authority for its responsibility")
    if source is None or source.repository_target is None or source.repository_target.target.resolve() != profile_route.target.resolve():
        raise RepositoryIntegrityError("profile carrier is not the authoritative routed source")
    environment_sequence = _string_list(raw_profile["environments"], "environments")
    if len(set(environment_sequence)) != len(environment_sequence):
        raise RepositoryIntegrityError("duplicate ValidationEnvironmentId")
    environments = frozenset(environment_sequence)
    continuation = raw_profile["continue_after_non_satisfied"]
    if type(continuation) is not bool:
        raise RepositoryIntegrityError("continue_after_non_satisfied must be boolean")
    raw_validations = _mapping(raw_profile["validations"], "validations")
    validations: dict[str, ValidationDefinition] = {}
    for raw_id, raw_validation in raw_validations.items():
        validation_id = _identifier(raw_id, "ValidationId")
        declaration = _mapping(raw_validation, f"validation {validation_id}")
        expected = _VALIDATION_KEYS | ({"target"} if "target" in declaration else set())
        _exact_keys(declaration, frozenset(expected), f"validation {validation_id}")
        validation_responsibility = _identifier(declaration["responsibility"], f"validation {validation_id}.responsibility")
        if validation_responsibility not in authority_profile.responsibilities:
            raise RepositoryIntegrityError(f"unknown governed responsibility: {validation_responsibility}")
        target = _target(declaration["target"], f"validation {validation_id}.target") if "target" in declaration else None
        if target is not None:
            _validate_target(target, validation_responsibility, governed_objects)
        validations[validation_id] = ValidationDefinition(
            validation_id,
            validation_responsibility,
            _string_list(declaration["prerequisites"], f"validation {validation_id}.prerequisites"),
            target,
            _load_instances(declaration["instances"], f"validation {validation_id}.instances"),
            _load_command(declaration["command"], f"validation {validation_id}.command", environments),
        )
    order = _string_list(raw_profile["order"], "order")
    _validate_graph(validations, order)
    return ConsumerIntegrityProfile(root, profile_route.target, _MODEL_VERSION, authority, environments, continuation, validations, order, _identity(raw_profile))
def _glob_expression(pattern: str) -> re.Pattern[str]:
    components = []
    for component in pattern.split("/"):
        components.append(
            "".join(
                "[^/]*" if character == "*" else re.escape(character)
                for character in component
            )
        )
    return re.compile("^" + "/".join(components) + "$")
def _selected_paths(
    instances: ValidationInstances, repository_paths: tuple[str, ...]
) -> tuple[str, ...]:
    selected: list[str] = []
    seen: set[str] = set()
    for selector in instances.selectors:
        if selector.kind == "path":
            matches = [selector.value]
        else:
            expression = _glob_expression(selector.value)
            matches = sorted(
                path for path in repository_paths if expression.fullmatch(path)
            )
        for path in matches:
            if path in seen:
                raise RepositoryIntegrityError(
                    f"duplicate selected repository path: {path}"
                )
            seen.add(path)
            selected.append(path)
    return tuple(selected)
def evaluate(repository: Path, profile: IntegrityProfile, *,
             env: Mapping[str, str],
             evaluation_context_identity: str) -> IntegrityResult:
    from proto_ring.repository_integrity_evaluation import evaluate as evaluate_impl
    return evaluate_impl(
        repository, profile, env=env,
        evaluation_context_identity=evaluation_context_identity)
def evaluate_consumer_profile(repository: Path,
                              profile: ConsumerIntegrityProfile,
                              context: EvaluationContext) -> IntegrityResult:
    from proto_ring.repository_integrity_evaluation import evaluate_consumer_profile as evaluate_consumer_profile_impl
    return evaluate_consumer_profile_impl(repository, profile, context)
