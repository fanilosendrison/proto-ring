"""Resolve exact consumer-supplied routes to contained repository targets."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

from proto_ring.adr_metadata import AdrMetadataError, repository_path

__all__ = [
    "GovernanceRoutingError",
    "ResolvedGovernanceRoute",
    "resolve_path",
]


class GovernanceRoutingError(ValueError):
    """Report a controlled failure to resolve an exact governance route."""


@dataclass(frozen=True)
class ResolvedGovernanceRoute:
    route: tuple[str, ...]
    declared_path: str
    target: Path


def resolve_path(
    repository: Path,
    routing: Mapping[str, object],
    route: Sequence[str],
) -> ResolvedGovernanceRoute:
    """Resolve one exact route to an available contained repository target."""

    if not isinstance(routing, Mapping):
        raise GovernanceRoutingError("routing must be a mapping")
    if isinstance(route, (str, bytes)) or not isinstance(route, Sequence):
        raise GovernanceRoutingError("route must be a sequence of segments")
    if not route:
        raise GovernanceRoutingError("route must not be empty")
    if any(not isinstance(segment, str) or segment == "" for segment in route):
        raise GovernanceRoutingError("route segments must be non-empty strings")

    exact_route = tuple(route)
    current: object = routing
    for segment in exact_route[:-1]:
        if not isinstance(current, Mapping):
            raise GovernanceRoutingError(
                f"route cannot traverse non-mapping segment: {segment}"
            )
        if segment not in current:
            raise GovernanceRoutingError(f"route segment is missing: {segment}")
        current = current[segment]

    leaf = exact_route[-1]
    if not isinstance(current, Mapping):
        raise GovernanceRoutingError(
            f"route cannot traverse non-mapping leaf container: {leaf}"
        )
    if leaf not in current:
        raise GovernanceRoutingError(f"route leaf is missing: {leaf}")
    declared_path = current[leaf]
    if not isinstance(declared_path, str) or declared_path == "":
        raise GovernanceRoutingError("route leaf must be a non-empty string")

    try:
        target = repository_path(repository, declared_path)
    except AdrMetadataError as error:
        raise GovernanceRoutingError(str(error)) from error
    if not target.exists():
        raise GovernanceRoutingError(
            f"routed repository target does not exist: {declared_path}"
        )

    return ResolvedGovernanceRoute(
        route=exact_route,
        declared_path=declared_path,
        target=target,
    )
