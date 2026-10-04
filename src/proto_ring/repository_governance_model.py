"""Construct the canonical logical repository-governance model."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from proto_ring import governance_bootstrap, governance_routing
from proto_ring.governance_bindings import (
    BindingKind,
    GovernanceBindingRegistry,
    ScopeKind,
)
from proto_ring.governance_routing import ResolvedGovernanceRoute
from proto_ring.structured_data import StructuredValue


__all__ = [
    "GovernanceCapability",
    "LogicalGovernanceProvider",
    "ProviderBindingReference",
    "RepositoryGovernanceModel",
    "RepositoryGovernanceModelError",
    "load",
    "validate_binding_capabilities",
]

_PROVIDER_ID = "proto-ring"
_PROVIDER_BINDING_CAPABILITY = "shared_governance_provider"


@dataclass(frozen=True)
class _ModelVersionSpecification:
    provider_binding_route: str
    capabilities: frozenset[str]
    mandatory_capabilities: frozenset[str]
    required_routes: dict[str, frozenset[str]]


_V1_REQUIRED_ROUTES = {
    "architecture_decisions": frozenset({"profile"}),
    "governance_authority": frozenset({"profile"}),
    "governed_objects": frozenset({"profile"}),
    "shared_governance_provider": frozenset({"binding"}),
}
_V2_REQUIRED_ROUTES = {
    "architecture_decisions": frozenset({"profile"}),
    "governance_authority": frozenset({"profile"}),
    "governed_objects": frozenset({"profile"}),
    "shared_governance_provider": frozenset({"registry"}),
    "projection_integrity": frozenset({"registry"}),
    "repository_integrity": frozenset({"profile"}),
    "evidence_requirements": frozenset({"registry"}),
    "authoritative_ref_monotonicity": frozenset({"binding"}),
}
_MODEL_VERSIONS = {
    1: _ModelVersionSpecification(
        "binding",
        frozenset(_V1_REQUIRED_ROUTES),
        frozenset({"shared_governance_provider"}),
        _V1_REQUIRED_ROUTES,
    ),
    2: _ModelVersionSpecification(
        "registry",
        frozenset(_V2_REQUIRED_ROUTES),
        frozenset({"shared_governance_provider", "governance_authority"}),
        _V2_REQUIRED_ROUTES,
    ),
}


class RepositoryGovernanceModelError(ValueError):
    """Report controlled failure to construct canonical repository governance."""


@dataclass(frozen=True)
class ProviderBindingReference:
    capability: str
    route: str


@dataclass(frozen=True)
class LogicalGovernanceProvider:
    id: str
    binding: ProviderBindingReference


@dataclass(frozen=True)
class GovernanceCapability:
    id: str
    configuration: dict[str, StructuredValue]
    routes: dict[str, ResolvedGovernanceRoute]


@dataclass(frozen=True)
class RepositoryGovernanceModel:
    repository: Path
    carrier: Path
    model_version: int
    provider: LogicalGovernanceProvider
    capabilities: dict[str, GovernanceCapability]


def _require_mapping(value: object, label: str) -> Mapping[object, object]:
    if not isinstance(value, Mapping):
        raise RepositoryGovernanceModelError(f"{label} must be a mapping")
    return value


def _require_exact_keys(
    value: Mapping[object, object], expected: frozenset[str], label: str
) -> None:
    actual = frozenset(value.keys())
    missing = expected - actual
    if missing:
        raise RepositoryGovernanceModelError(
            f"{label} is missing key: {sorted(missing)[0]}"
        )
    unexpected = actual - expected
    if unexpected:
        raise RepositoryGovernanceModelError(
            f"{label} has unexpected key: {sorted(unexpected, key=str)[0]}"
        )


def _require_non_empty_string(value: object, label: str) -> str:
    if not isinstance(value, str) or value == "":
        raise RepositoryGovernanceModelError(f"{label} must be a non-empty string")
    return value


def _load_provider(
    governance: Mapping[object, object],
    specification: _ModelVersionSpecification,
) -> LogicalGovernanceProvider:
    provider = _require_mapping(governance["provider"], "provider")
    _require_exact_keys(provider, frozenset({"id", "binding"}), "provider")
    provider_id = _require_non_empty_string(provider["id"], "provider.id")
    if provider_id != _PROVIDER_ID:
        raise RepositoryGovernanceModelError("unsupported logical provider")

    binding = _require_mapping(provider["binding"], "provider.binding")
    _require_exact_keys(
        binding, frozenset({"capability", "route"}), "provider.binding"
    )
    capability = _require_non_empty_string(
        binding["capability"], "provider.binding.capability"
    )
    route = _require_non_empty_string(binding["route"], "provider.binding.route")
    if (
        capability != _PROVIDER_BINDING_CAPABILITY
        or route != specification.provider_binding_route
    ):
        raise RepositoryGovernanceModelError(
            "provider binding must reference shared_governance_provider/"
            f"{specification.provider_binding_route}"
        )
    return LogicalGovernanceProvider(
        id=provider_id,
        binding=ProviderBindingReference(capability=capability, route=route),
    )


def _load_capabilities(
    repository: Path,
    governance: Mapping[object, object],
    model_version: int,
    specification: _ModelVersionSpecification,
) -> dict[str, GovernanceCapability]:
    declarations = _require_mapping(governance["capabilities"], "capabilities")
    capability_ids: list[str] = []
    for raw_capability_id in declarations:
        capability_id = _require_non_empty_string(raw_capability_id, "capability id")
        if capability_id not in specification.capabilities:
            raise RepositoryGovernanceModelError(
                f"unsupported capability: {capability_id}"
            )
        capability_ids.append(capability_id)
    missing_capabilities = specification.mandatory_capabilities - frozenset(
        capability_ids
    )
    if missing_capabilities:
        raise RepositoryGovernanceModelError(
            f"{sorted(missing_capabilities)[0]} capability is required"
        )
    if "governed_objects" in declarations and "governance_authority" not in declarations:
        raise RepositoryGovernanceModelError(
            "governed_objects capability requires governance_authority capability"
        )
    if (
        model_version == 2
        and "projection_integrity" in declarations
        and "repository_integrity" not in declarations
    ):
        raise RepositoryGovernanceModelError(
            "projection_integrity capability requires repository_integrity capability"
        )

    capabilities: dict[str, GovernanceCapability] = {}
    for capability_id in sorted(capability_ids):
        declaration = _require_mapping(
            declarations[capability_id], f"capability {capability_id}"
        )
        _require_exact_keys(
            declaration,
            frozenset({"configuration", "routes"}),
            f"capability {capability_id}",
        )
        configuration = _require_mapping(
            declaration["configuration"],
            f"capability {capability_id}.configuration",
        )
        routes = _require_mapping(
            declaration["routes"], f"capability {capability_id}.routes"
        )
        missing_routes = specification.required_routes[capability_id] - frozenset(
            routes.keys()
        )
        if missing_routes:
            raise RepositoryGovernanceModelError(
                f"capability {capability_id} is missing route: "
                f"{sorted(missing_routes)[0]}"
            )

        route_ids = [
            _require_non_empty_string(raw_route_id, "route id")
            for raw_route_id in routes
        ]
        resolved_routes: dict[str, ResolvedGovernanceRoute] = {}
        for route_id in sorted(route_ids):
            _require_non_empty_string(
                routes[route_id], f"route {capability_id}.{route_id}"
            )
            exact_route = (
                "capabilities",
                capability_id,
                "routes",
                route_id,
            )
            try:
                resolved_routes[route_id] = governance_routing.resolve_path(
                    repository, governance, exact_route
                )
            except governance_routing.GovernanceRoutingError as error:
                raise RepositoryGovernanceModelError(str(error)) from error

        capabilities[capability_id] = GovernanceCapability(
            id=capability_id,
            configuration=cast(dict[str, StructuredValue], dict(configuration)),
            routes=resolved_routes,
        )
    return capabilities


def from_bootstrap(
    bootstrap: governance_bootstrap.GovernanceBootstrap,
) -> RepositoryGovernanceModel:
    """Construct a supported logical model from one canonical bootstrap value."""

    governance = bootstrap.repository_governance
    _require_exact_keys(
        governance,
        frozenset({"model_version", "provider", "capabilities"}),
        "repository_governance",
    )
    model_version = governance["model_version"]
    if type(model_version) is not int or model_version not in _MODEL_VERSIONS:
        raise RepositoryGovernanceModelError("unsupported model_version")
    specification = _MODEL_VERSIONS[model_version]

    provider = _load_provider(governance, specification)
    capabilities = _load_capabilities(
        bootstrap.repository, governance, model_version, specification
    )
    binding_capability = capabilities.get(provider.binding.capability)
    if (
        binding_capability is None
        or provider.binding.route not in binding_capability.routes
    ):
        raise RepositoryGovernanceModelError("provider binding reference is unresolved")

    return RepositoryGovernanceModel(
        repository=bootstrap.repository,
        carrier=bootstrap.carrier,
        model_version=model_version,
        provider=provider,
        capabilities=capabilities,
    )


def load(repository: Path) -> RepositoryGovernanceModel:
    """Load a supported logical model from canonical bootstrap output."""

    try:
        bootstrap = governance_bootstrap.load(repository)
    except governance_bootstrap.GovernanceBootstrapError as error:
        raise RepositoryGovernanceModelError(str(error)) from error
    return from_bootstrap(bootstrap)


def validate_binding_capabilities(
    model: RepositoryGovernanceModel,
    registry: GovernanceBindingRegistry,
) -> None:
    """Validate v2 capability-scoped contract bindings without loading targets."""

    if model.model_version != 2:
        raise RepositoryGovernanceModelError(
            "governance binding capability composition requires model_version 2"
        )
    try:
        if model.repository.resolve() != registry.repository.resolve():
            raise RepositoryGovernanceModelError(
                "governance binding registry repository differs"
            )
    except OSError as error:
        raise RepositoryGovernanceModelError(
            f"cannot resolve repository root: {error}"
        ) from error
    for binding in registry.bindings.values():
        if (
            binding.kind is BindingKind.GOVERNANCE_CONTRACT
            and binding.scope.kind is ScopeKind.CAPABILITY
            and binding.scope.capability_id not in model.capabilities
        ):
            raise RepositoryGovernanceModelError(
                "governance contract binding references undeclared capability: "
                f"{binding.scope.capability_id}"
            )
