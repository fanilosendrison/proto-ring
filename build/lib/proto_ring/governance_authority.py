"""Load and query consumer-owned governance-authority declarations."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

from proto_ring import governance_routing, structured_data
from proto_ring.governance_routing import ResolvedGovernanceRoute


__all__ = [
    "GovernanceAuthorityError",
    "SourceRole",
    "AuthorityLookupState",
    "GovernedSource",
    "PrecedenceEdge",
    "GovernedResponsibility",
    "GovernanceAuthorityProfile",
    "load",
    "role_of",
    "authority_sources",
    "sources_with_role",
    "outranks",
]

_MODEL_VERSION = 1
_PROFILE_KEYS = frozenset({"model_version", "sources", "responsibilities"})
_SOURCE_KEYS = frozenset({"repository_target"})
_RESPONSIBILITY_KEYS = frozenset({"roles", "precedence"})
_PRECEDENCE_KEYS = frozenset({"higher_source", "lower_source"})


class GovernanceAuthorityError(ValueError):
    """Report controlled failure to load a governance-authority profile."""


class SourceRole(StrEnum):
    AUTHORITY = "authority"
    SECONDARY_REPRESENTATION = "secondary_representation"
    NON_AUTHORITATIVE = "non_authoritative"


class AuthorityLookupState(StrEnum):
    UNKNOWN_RESPONSIBILITY = "unknown_responsibility"
    UNDECLARED = "undeclared"


@dataclass(frozen=True)
class GovernedSource:
    id: str
    repository_target: ResolvedGovernanceRoute | None


@dataclass(frozen=True)
class PrecedenceEdge:
    higher_source: str
    lower_source: str


@dataclass(frozen=True)
class GovernedResponsibility:
    id: str
    roles: dict[str, SourceRole]
    precedence: tuple[PrecedenceEdge, ...]


@dataclass(frozen=True)
class GovernanceAuthorityProfile:
    repository: Path
    carrier: Path
    model_version: int
    sources: dict[str, GovernedSource]
    responsibilities: dict[str, GovernedResponsibility]


def _mapping(value: object, label: str) -> Mapping[object, object]:
    if not isinstance(value, Mapping):
        raise GovernanceAuthorityError(f"{label} must be a mapping")
    return value


def _exact_keys(
    value: Mapping[object, object], expected: frozenset[str], label: str
) -> None:
    actual = frozenset(value.keys())
    missing = expected - actual
    if missing:
        raise GovernanceAuthorityError(
            f"{label} is missing key: {sorted(missing)[0]}"
        )
    unexpected = actual - expected
    if unexpected:
        raise GovernanceAuthorityError(
            f"{label} has unexpected key: {sorted(unexpected, key=str)[0]}"
        )


def _identifier(value: object, label: str) -> str:
    if not isinstance(value, str) or value == "":
        raise GovernanceAuthorityError(f"{label} must be a non-empty string")
    return value


def _load_sources(
    repository: Path, declarations: Mapping[object, object]
) -> dict[str, GovernedSource]:
    sources: dict[str, GovernedSource] = {}
    for raw_source_id, raw_declaration in declarations.items():
        source_id = _identifier(raw_source_id, "source id")
        declaration = _mapping(raw_declaration, f"source {source_id}")
        if declaration:
            _exact_keys(declaration, _SOURCE_KEYS, f"source {source_id}")
            _identifier(
                declaration["repository_target"],
                f"source {source_id}.repository_target",
            )
            try:
                target = governance_routing.resolve_path(
                    repository,
                    declarations,
                    (source_id, "repository_target"),
                )
            except governance_routing.GovernanceRoutingError as error:
                raise GovernanceAuthorityError(str(error)) from error
        else:
            target = None
        sources[source_id] = GovernedSource(source_id, target)
    return sources


def _load_roles(
    raw_roles: object,
    responsibility_id: str,
    sources: Mapping[str, GovernedSource],
) -> dict[str, SourceRole]:
    declarations = _mapping(raw_roles, f"responsibility {responsibility_id}.roles")
    roles: dict[str, SourceRole] = {}
    for raw_source_id, raw_role in declarations.items():
        source_id = _identifier(raw_source_id, "role source id")
        if source_id not in sources:
            raise GovernanceAuthorityError(
                f"role source is not declared: {source_id}"
            )
        if not isinstance(raw_role, str):
            raise GovernanceAuthorityError(
                f"role for {source_id} must be a source-role string"
            )
        try:
            roles[source_id] = SourceRole(raw_role)
        except ValueError as error:
            raise GovernanceAuthorityError(
                f"unknown source role: {raw_role}"
            ) from error
    return roles


def _load_precedence(
    raw_precedence: object,
    responsibility_id: str,
    roles: Mapping[str, SourceRole],
) -> tuple[PrecedenceEdge, ...]:
    if not isinstance(raw_precedence, list):
        raise GovernanceAuthorityError(
            f"responsibility {responsibility_id}.precedence must be a sequence"
        )
    edges: list[PrecedenceEdge] = []
    identities: set[tuple[str, str]] = set()
    for index, raw_edge in enumerate(raw_precedence):
        label = f"responsibility {responsibility_id}.precedence[{index}]"
        edge = _mapping(raw_edge, label)
        _exact_keys(edge, _PRECEDENCE_KEYS, label)
        higher = _identifier(edge["higher_source"], f"{label}.higher_source")
        lower = _identifier(edge["lower_source"], f"{label}.lower_source")
        if higher == lower:
            raise GovernanceAuthorityError("precedence self edge is invalid")
        for endpoint in (higher, lower):
            if endpoint not in roles:
                raise GovernanceAuthorityError(
                    f"precedence endpoint is not a declared role source: {endpoint}"
                )
            if roles[endpoint] is not SourceRole.AUTHORITY:
                raise GovernanceAuthorityError(
                    f"precedence endpoint is not an authority: {endpoint}"
                )
        identity = (higher, lower)
        if identity in identities:
            raise GovernanceAuthorityError("duplicate precedence edge")
        identities.add(identity)
        edges.append(PrecedenceEdge(higher, lower))
    _reject_cycle(edges)
    return tuple(edges)


def _reject_cycle(edges: list[PrecedenceEdge]) -> None:
    successors: dict[str, set[str]] = {}
    indegree: dict[str, int] = {}
    for edge in edges:
        successors.setdefault(edge.higher_source, set()).add(edge.lower_source)
        indegree.setdefault(edge.higher_source, 0)
        indegree[edge.lower_source] = indegree.get(edge.lower_source, 0) + 1
    pending = [source_id for source_id, degree in indegree.items() if degree == 0]
    visited = 0
    while pending:
        source_id = pending.pop()
        visited += 1
        for successor in successors.get(source_id, set()):
            indegree[successor] -= 1
            if indegree[successor] == 0:
                pending.append(successor)
    if visited != len(indegree):
        raise GovernanceAuthorityError("precedence cycle is invalid")


def _load_responsibilities(
    declarations: Mapping[object, object],
    sources: Mapping[str, GovernedSource],
) -> dict[str, GovernedResponsibility]:
    responsibilities: dict[str, GovernedResponsibility] = {}
    for raw_responsibility_id, raw_declaration in declarations.items():
        responsibility_id = _identifier(raw_responsibility_id, "responsibility id")
        declaration = _mapping(
            raw_declaration, f"responsibility {responsibility_id}"
        )
        _exact_keys(
            declaration,
            _RESPONSIBILITY_KEYS,
            f"responsibility {responsibility_id}",
        )
        roles = _load_roles(declaration["roles"], responsibility_id, sources)
        precedence = _load_precedence(
            declaration["precedence"], responsibility_id, roles
        )
        if (
            SourceRole.SECONDARY_REPRESENTATION in roles.values()
            and SourceRole.AUTHORITY not in roles.values()
        ):
            raise GovernanceAuthorityError(
                "secondary representation requires an authority for the same "
                f"responsibility: {responsibility_id}"
            )
        responsibilities[responsibility_id] = GovernedResponsibility(
            responsibility_id, roles, precedence
        )
    return responsibilities


def load(
    repository: Path,
    profile_route: ResolvedGovernanceRoute,
) -> GovernanceAuthorityProfile:
    """Load an exact model-version-1 authority profile from its routed carrier."""

    try:
        data = profile_route.target.read_bytes()
    except OSError as error:
        raise GovernanceAuthorityError(
            f"cannot read governance-authority profile: {error}"
        ) from error
    try:
        parsed = structured_data.parse_frontmatter_bytes(data)
    except structured_data.StructuredDataError as error:
        raise GovernanceAuthorityError(
            f"cannot parse governance-authority profile: {error}"
        ) from error
    if "governance_authority" not in parsed.metadata:
        raise GovernanceAuthorityError("governance_authority is required")
    profile = _mapping(parsed.metadata["governance_authority"], "governance_authority")
    _exact_keys(profile, _PROFILE_KEYS, "governance_authority")
    model_version = profile["model_version"]
    if type(model_version) is not int or model_version != _MODEL_VERSION:
        raise GovernanceAuthorityError("unsupported model_version")
    source_declarations = _mapping(profile["sources"], "sources")
    responsibility_declarations = _mapping(
        profile["responsibilities"], "responsibilities"
    )
    sources = _load_sources(repository, source_declarations)
    responsibilities = _load_responsibilities(
        responsibility_declarations, sources
    )
    used_sources = {
        source_id
        for responsibility in responsibilities.values()
        for source_id in responsibility.roles
    }
    unused_sources = set(sources) - used_sources
    if unused_sources:
        raise GovernanceAuthorityError(
            f"declared source is unused: {sorted(unused_sources)[0]}"
        )
    return GovernanceAuthorityProfile(
        repository=repository,
        carrier=profile_route.target,
        model_version=_MODEL_VERSION,
        sources=sources,
        responsibilities=responsibilities,
    )


def role_of(
    profile: GovernanceAuthorityProfile,
    responsibility_id: str,
    source_id: str,
) -> SourceRole | AuthorityLookupState:
    responsibility = profile.responsibilities.get(responsibility_id)
    if responsibility is None:
        return AuthorityLookupState.UNKNOWN_RESPONSIBILITY
    return responsibility.roles.get(source_id, AuthorityLookupState.UNDECLARED)


def sources_with_role(
    profile: GovernanceAuthorityProfile,
    responsibility_id: str,
    role: SourceRole,
) -> frozenset[str] | AuthorityLookupState:
    responsibility = profile.responsibilities.get(responsibility_id)
    if responsibility is None:
        return AuthorityLookupState.UNKNOWN_RESPONSIBILITY
    return frozenset(
        source_id
        for source_id, declared_role in responsibility.roles.items()
        if declared_role is role
    )


def authority_sources(
    profile: GovernanceAuthorityProfile,
    responsibility_id: str,
) -> frozenset[str] | AuthorityLookupState:
    return sources_with_role(profile, responsibility_id, SourceRole.AUTHORITY)


def outranks(
    profile: GovernanceAuthorityProfile,
    responsibility_id: str,
    higher_source: str,
    lower_source: str,
) -> bool | AuthorityLookupState:
    responsibility = profile.responsibilities.get(responsibility_id)
    if responsibility is None:
        return AuthorityLookupState.UNKNOWN_RESPONSIBILITY
    successors: dict[str, set[str]] = {}
    for edge in responsibility.precedence:
        successors.setdefault(edge.higher_source, set()).add(edge.lower_source)
    pending = list(successors.get(higher_source, set()))
    visited: set[str] = set()
    while pending:
        current = pending.pop()
        if current == lower_source:
            return True
        if current not in visited:
            visited.add(current)
            pending.extend(successors.get(current, set()))
    return False
