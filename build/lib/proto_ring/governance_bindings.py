"""Load consumer-owned immutable governance binding registries."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
import re

from proto_ring import structured_data
from proto_ring.governance_authority import (
    GovernanceAuthorityProfile,
    SourceRole,
    role_of,
)
from proto_ring.governance_routing import ResolvedGovernanceRoute
from proto_ring.governed_objects import GovernedObjectCatalog, GovernedObjectRef

__all__ = [
    "BindingKind",
    "ScopeKind",
    "ExecutableProviderIdentity",
    "GovernanceContractIdentity",
    "BindingAuthority",
    "GovernanceBindingScope",
    "GovernanceBinding",
    "GovernanceBindingRegistry",
    "GovernanceBindingsError",
    "load",
]

_MODEL_VERSION = 1
_REGISTRY_KEYS = frozenset({"model_version", "source", "bindings"})
_BINDING_KEYS = frozenset({"kind", "scope", "identity", "authority"})
_EXECUTABLE_IDENTITY_KEYS = frozenset({"repository", "commit"})
_CONTRACT_IDENTITY_KEYS = frozenset({"repository", "commit", "path"})
_AUTHORITY_KEYS = frozenset({"responsibility", "source"})
_COMMIT_PATTERN = re.compile(r"[0-9a-f]{40}")


class GovernanceBindingsError(ValueError):
    """Report controlled failure to load a governance binding registry."""


class BindingKind(StrEnum):
    EXECUTABLE_PROVIDER = "executable_provider"
    GOVERNANCE_CONTRACT = "governance_contract"


class ScopeKind(StrEnum):
    LOGICAL_PROVIDER = "logical_provider"
    CAPABILITY = "capability"
    GOVERNED_OBJECT = "governed_object"


@dataclass(frozen=True)
class ExecutableProviderIdentity:
    repository: str
    commit: str


@dataclass(frozen=True)
class GovernanceContractIdentity:
    repository: str
    commit: str
    path: str


@dataclass(frozen=True)
class BindingAuthority:
    responsibility_id: str
    source_id: str


@dataclass(frozen=True)
class GovernanceBindingScope:
    kind: ScopeKind
    capability_id: str | None = None
    governed_object: GovernedObjectRef | None = None


@dataclass(frozen=True)
class GovernanceBinding:
    id: str
    kind: BindingKind
    scope: GovernanceBindingScope
    identity: ExecutableProviderIdentity | GovernanceContractIdentity
    authority: BindingAuthority


@dataclass(frozen=True)
class GovernanceBindingRegistry:
    repository: Path
    carrier: Path
    model_version: int
    source_id: str
    bindings: dict[str, GovernanceBinding]


def _mapping(value: object, label: str) -> Mapping[object, object]:
    if not isinstance(value, Mapping):
        raise GovernanceBindingsError(f"{label} must be a mapping")
    return value


def _exact_keys(
    value: Mapping[object, object], expected: frozenset[str], label: str
) -> None:
    actual = frozenset(value.keys())
    missing = expected - actual
    if missing:
        raise GovernanceBindingsError(f"{label} is missing key: {sorted(missing)[0]}")
    unexpected = actual - expected
    if unexpected:
        raise GovernanceBindingsError(
            f"{label} has unexpected key: {sorted(unexpected, key=str)[0]}"
        )


def _identifier(value: object, label: str) -> str:
    if not isinstance(value, str) or value == "":
        raise GovernanceBindingsError(f"{label} must be a non-empty string")
    return value


def _kind(value: object) -> BindingKind:
    if not isinstance(value, str):
        raise GovernanceBindingsError("binding kind must be a string")
    try:
        return BindingKind(value)
    except ValueError as error:
        raise GovernanceBindingsError(f"unknown binding kind: {value}") from error


def _scope(value: object, kind: BindingKind) -> GovernanceBindingScope:
    declaration = _mapping(value, "binding scope")
    scope_kind_value = declaration.get("kind")
    if not isinstance(scope_kind_value, str):
        raise GovernanceBindingsError("binding scope kind must be a string")
    try:
        scope_kind = ScopeKind(scope_kind_value)
    except ValueError as error:
        raise GovernanceBindingsError(
            f"unknown binding scope kind: {scope_kind_value}"
        ) from error
    if scope_kind is ScopeKind.LOGICAL_PROVIDER:
        _exact_keys(declaration, frozenset({"kind"}), "logical_provider scope")
        scope = GovernanceBindingScope(scope_kind)
    elif scope_kind is ScopeKind.CAPABILITY:
        _exact_keys(
            declaration, frozenset({"kind", "capability"}), "capability scope"
        )
        scope = GovernanceBindingScope(
            scope_kind,
            capability_id=_identifier(
                declaration["capability"], "capability scope capability"
            ),
        )
    else:
        _exact_keys(
            declaration,
            frozenset({"kind", "interface", "object"}),
            "governed_object scope",
        )
        scope = GovernanceBindingScope(
            scope_kind,
            governed_object=GovernedObjectRef(
                _identifier(declaration["interface"], "governed object interface"),
                _identifier(declaration["object"], "governed object identity"),
            ),
        )
    if kind is BindingKind.EXECUTABLE_PROVIDER and (
        scope.kind is not ScopeKind.LOGICAL_PROVIDER
    ):
        raise GovernanceBindingsError(
            "executable_provider binding must use logical_provider scope"
        )
    return scope


def _identity(
    value: object, kind: BindingKind
) -> ExecutableProviderIdentity | GovernanceContractIdentity:
    declaration = _mapping(value, "binding identity")
    expected = (
        _EXECUTABLE_IDENTITY_KEYS
        if kind is BindingKind.EXECUTABLE_PROVIDER
        else _CONTRACT_IDENTITY_KEYS
    )
    _exact_keys(declaration, expected, "binding identity")
    repository = _identifier(declaration["repository"], "identity repository")
    commit = _identifier(declaration["commit"], "identity commit")
    if not _COMMIT_PATTERN.fullmatch(commit):
        raise GovernanceBindingsError(
            "identity commit must be 40 lowercase hexadecimal characters"
        )
    if kind is BindingKind.EXECUTABLE_PROVIDER:
        return ExecutableProviderIdentity(repository, commit)
    return GovernanceContractIdentity(
        repository,
        commit,
        _identifier(declaration["path"], "identity path"),
    )


def _authority(
    value: object,
    authority_profile: GovernanceAuthorityProfile,
    registry_source_id: str,
) -> BindingAuthority:
    declaration = _mapping(value, "binding authority")
    _exact_keys(declaration, _AUTHORITY_KEYS, "binding authority")
    responsibility_id = _identifier(
        declaration["responsibility"], "authority responsibility"
    )
    source_id = _identifier(declaration["source"], "authority source")
    if responsibility_id not in authority_profile.responsibilities:
        raise GovernanceBindingsError(
            f"unknown governed responsibility: {responsibility_id}"
        )
    if source_id not in authority_profile.sources:
        raise GovernanceBindingsError(f"unknown governed source: {source_id}")
    if role_of(authority_profile, responsibility_id, source_id) is not SourceRole.AUTHORITY:
        raise GovernanceBindingsError(
            f"binding source is not authority for responsibility: {source_id}"
        )
    if source_id != registry_source_id and role_of(
        authority_profile, responsibility_id, registry_source_id
    ) is not SourceRole.SECONDARY_REPRESENTATION:
        raise GovernanceBindingsError(
            "registry source must be a secondary representation when binding "
            "authority is a different source"
        )
    return BindingAuthority(responsibility_id, source_id)


def _validate_object_scope(
    scope: GovernanceBindingScope,
    catalog: GovernedObjectCatalog | None,
) -> None:
    reference = scope.governed_object
    if reference is None:
        return
    if catalog is None:
        raise GovernanceBindingsError("governed_object scope requires a catalog")
    interface = catalog.interfaces.get(reference.interface_id)
    if interface is None or reference.object_id not in interface.objects:
        raise GovernanceBindingsError(
            "governed_object scope does not resolve: "
            f"{reference.interface_id}/{reference.object_id}"
        )


def _bindings(
    value: object,
    authority_profile: GovernanceAuthorityProfile,
    registry_source_id: str,
    catalog: GovernedObjectCatalog | None,
) -> dict[str, GovernanceBinding]:
    declarations = _mapping(value, "bindings")
    bindings: dict[str, GovernanceBinding] = {}
    for raw_binding_id, raw_binding in declarations.items():
        binding_id = _identifier(raw_binding_id, "binding id")
        declaration = _mapping(raw_binding, f"binding {binding_id}")
        _exact_keys(declaration, _BINDING_KEYS, f"binding {binding_id}")
        kind = _kind(declaration["kind"])
        scope = _scope(declaration["scope"], kind)
        _validate_object_scope(scope, catalog)
        bindings[binding_id] = GovernanceBinding(
            id=binding_id,
            kind=kind,
            scope=scope,
            identity=_identity(declaration["identity"], kind),
            authority=_authority(
                declaration["authority"], authority_profile, registry_source_id
            ),
        )
    return bindings


def _validate_cardinality(bindings: Mapping[str, GovernanceBinding]) -> None:
    executable_count = sum(
        binding.kind is BindingKind.EXECUTABLE_PROVIDER
        for binding in bindings.values()
    )
    if executable_count != 1:
        raise GovernanceBindingsError(
            "registry must contain exactly one executable_provider binding"
        )


def _validate_contract_ambiguity(bindings: Mapping[str, GovernanceBinding]) -> None:
    revisions: dict[
        tuple[GovernanceBindingScope, str, str], str
    ] = {}
    for binding in bindings.values():
        if binding.kind is not BindingKind.GOVERNANCE_CONTRACT:
            continue
        identity = binding.identity
        if not isinstance(identity, GovernanceContractIdentity):
            raise GovernanceBindingsError("governance_contract identity is invalid")
        key = (binding.scope, identity.repository, identity.path)
        existing = revisions.get(key)
        if existing is not None and existing != identity.commit:
            raise GovernanceBindingsError(
                "conflicting governance contract revisions for the same scope"
            )
        revisions[key] = identity.commit


def _same_repository(repository: Path, other: Path, label: str) -> None:
    try:
        if repository.resolve() != other.resolve():
            raise GovernanceBindingsError(f"{label} repository differs")
    except OSError as error:
        raise GovernanceBindingsError(f"cannot resolve repository root: {error}") from error


def load(
    repository: Path,
    registry_route: ResolvedGovernanceRoute,
    authority_profile: GovernanceAuthorityProfile,
    governed_objects: GovernedObjectCatalog | None = None,
) -> GovernanceBindingRegistry:
    """Load an exact model-version-1 consumer governance binding registry."""

    _same_repository(repository, authority_profile.repository, "governance authority")
    if governed_objects is not None:
        _same_repository(repository, governed_objects.repository, "governed objects")
    try:
        data = registry_route.target.read_bytes()
    except OSError as error:
        raise GovernanceBindingsError(
            f"cannot read governance binding registry: {error}"
        ) from error
    try:
        parsed = structured_data.parse_frontmatter_bytes(data)
    except structured_data.StructuredDataError as error:
        raise GovernanceBindingsError(
            f"cannot parse governance binding registry: {error}"
        ) from error
    if "governance_bindings" not in parsed.metadata:
        raise GovernanceBindingsError("governance_bindings is required")
    registry = _mapping(parsed.metadata["governance_bindings"], "governance_bindings")
    _exact_keys(registry, _REGISTRY_KEYS, "governance_bindings")
    model_version = registry["model_version"]
    if type(model_version) is not int or model_version != _MODEL_VERSION:
        raise GovernanceBindingsError("unsupported model_version")
    source_id = _identifier(registry["source"], "registry source")
    source = authority_profile.sources.get(source_id)
    if source is None:
        raise GovernanceBindingsError(f"unknown registry source: {source_id}")
    if source.repository_target is None:
        raise GovernanceBindingsError("registry source has no repository target")
    try:
        source_target = source.repository_target.target.resolve()
        registry_target = registry_route.target.resolve()
    except OSError as error:
        raise GovernanceBindingsError(f"cannot resolve registry carrier: {error}") from error
    if source_target != registry_target:
        raise GovernanceBindingsError("registry source does not resolve to registry carrier")
    bindings = _bindings(
        registry["bindings"], authority_profile, source_id, governed_objects
    )
    _validate_cardinality(bindings)
    _validate_contract_ambiguity(bindings)
    return GovernanceBindingRegistry(
        repository=repository,
        carrier=registry_route.target,
        model_version=_MODEL_VERSION,
        source_id=source_id,
        bindings=bindings,
    )
