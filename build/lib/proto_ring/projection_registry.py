"""Load consumer-owned persistent Projection Registries."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

from proto_ring import structured_data
from proto_ring.governance_authority import (
    GovernanceAuthorityProfile,
    SourceRole,
    role_of,
)
from proto_ring.governance_bindings import GovernanceBindingRegistry
from proto_ring.governance_routing import ResolvedGovernanceRoute
from proto_ring.governed_objects import GovernedObjectCatalog, GovernedObjectRef
from proto_ring.repository_integrity import ConsumerIntegrityProfile

__all__ = [
    "ProjectionMode",
    "RegistryAuthority",
    "Projection",
    "ProjectionRegistry",
    "ProjectionRegistryError",
    "load",
]

_MODEL_VERSION = 1
_REGISTRY_KEYS = frozenset({"model_version", "authority", "projections"})
_AUTHORITY_KEYS = frozenset({"responsibility", "source"})
_PROJECTION_KEYS = frozenset(
    {"responsibility", "canonical_source", "secondary_source", "mode", "validation"}
)
_TARGET_KEYS = frozenset({"interface", "object"})
_OPTIONAL_KEYS = frozenset({"generator_source", "boundary_source", "target", "binding"})


class ProjectionRegistryError(ValueError):
    """Report controlled failure to load a Projection Registry."""


class ProjectionMode(StrEnum):
    REFERENCE = "reference"
    GENERATED = "generated"
    MECHANICALLY_VALIDATED_MAINTAINED = "mechanically_validated_maintained"
    BOUNDED_HISTORICAL_SNAPSHOT = "bounded_historical_snapshot"


@dataclass(frozen=True)
class RegistryAuthority:
    responsibility_id: str
    source_id: str


@dataclass(frozen=True)
class Projection:
    id: str
    responsibility_id: str
    canonical_source_id: str
    secondary_source_id: str
    mode: ProjectionMode
    validation_id: str
    generator_source_id: str | None = None
    boundary_source_id: str | None = None
    target: GovernedObjectRef | None = None
    binding_id: str | None = None


@dataclass(frozen=True)
class ProjectionRegistry:
    repository: Path
    carrier: Path
    model_version: int
    authority: RegistryAuthority
    projections: dict[str, Projection]


def _mapping(value: object, label: str) -> Mapping[object, object]:
    if not isinstance(value, Mapping):
        raise ProjectionRegistryError(f"{label} must be a mapping")
    return value


def _exact_keys(
    value: Mapping[object, object], expected: frozenset[str], label: str
) -> None:
    actual = frozenset(value.keys())
    missing = expected - actual
    if missing:
        raise ProjectionRegistryError(f"{label} is missing key: {sorted(missing)[0]}")
    unexpected = actual - expected
    if unexpected:
        raise ProjectionRegistryError(
            f"{label} has unexpected key: {sorted(unexpected, key=str)[0]}"
        )


def _identifier(value: object, label: str) -> str:
    if not isinstance(value, str) or value == "":
        raise ProjectionRegistryError(f"{label} must be a non-empty string")
    return value


def _same_repository(repository: Path, other: Path, label: str) -> None:
    try:
        if repository.resolve() != other.resolve():
            raise ProjectionRegistryError(f"{label} repository differs")
    except OSError as error:
        raise ProjectionRegistryError(f"cannot resolve repository root: {error}") from error


def _source(
    value: object,
    label: str,
    authority_profile: GovernanceAuthorityProfile,
) -> str:
    source_id = _identifier(value, label)
    if source_id not in authority_profile.sources:
        raise ProjectionRegistryError(f"unknown governed source: {source_id}")
    return source_id


def _require_role(
    authority_profile: GovernanceAuthorityProfile,
    responsibility_id: str,
    source_id: str,
    expected: SourceRole,
    label: str,
) -> None:
    if role_of(authority_profile, responsibility_id, source_id) is not expected:
        raise ProjectionRegistryError(
            f"{label} must have role {expected.value} for responsibility"
        )


def _registry_authority(
    value: object,
    route: ResolvedGovernanceRoute,
    authority_profile: GovernanceAuthorityProfile,
) -> RegistryAuthority:
    declaration = _mapping(value, "registry authority")
    _exact_keys(declaration, _AUTHORITY_KEYS, "registry authority")
    responsibility_id = _identifier(
        declaration["responsibility"], "authority.responsibility"
    )
    if responsibility_id not in authority_profile.responsibilities:
        raise ProjectionRegistryError(
            f"unknown governed responsibility: {responsibility_id}"
        )
    source_id = _source(declaration["source"], "authority.source", authority_profile)
    _require_role(
        authority_profile, responsibility_id, source_id, SourceRole.AUTHORITY,
        "registry authority source",
    )
    governed_source = authority_profile.sources[source_id]
    if governed_source.repository_target is None:
        raise ProjectionRegistryError("registry authority source has no repository target")
    try:
        if governed_source.repository_target.target.resolve() != route.target.resolve():
            raise ProjectionRegistryError(
                "registry authority source does not resolve to registry carrier"
            )
    except OSError as error:
        raise ProjectionRegistryError(f"cannot resolve registry carrier: {error}") from error
    return RegistryAuthority(responsibility_id, source_id)


def _mode(value: object) -> ProjectionMode:
    raw_mode = _identifier(value, "projection mode")
    try:
        return ProjectionMode(raw_mode)
    except ValueError as error:
        raise ProjectionRegistryError(f"unknown projection mode: {raw_mode}") from error


def _target(
    value: object,
    catalog: GovernedObjectCatalog | None,
) -> GovernedObjectRef:
    declaration = _mapping(value, "projection target")
    _exact_keys(declaration, _TARGET_KEYS, "projection target")
    reference = GovernedObjectRef(
        _identifier(declaration["interface"], "target.interface"),
        _identifier(declaration["object"], "target.object"),
    )
    if catalog is None:
        raise ProjectionRegistryError("projection target requires governed objects")
    interface = catalog.interfaces.get(reference.interface_id)
    if interface is None or reference.object_id not in interface.objects:
        raise ProjectionRegistryError(
            f"governed object does not exist: {reference.interface_id}/{reference.object_id}"
        )
    return reference


def _mode_sources(
    declaration: Mapping[object, object],
    mode: ProjectionMode,
    responsibility_id: str,
    authority_profile: GovernanceAuthorityProfile,
) -> tuple[str | None, str | None]:
    generator_source_id: str | None = None
    boundary_source_id: str | None = None
    if mode is ProjectionMode.GENERATED:
        if "generator_source" not in declaration:
            raise ProjectionRegistryError("generated projection requires generator_source")
        generator_source_id = _source(
            declaration["generator_source"], "generator_source", authority_profile
        )
        _require_role(
            authority_profile,
            responsibility_id,
            generator_source_id,
            SourceRole.NON_AUTHORITATIVE,
            "generator source",
        )
    elif "generator_source" in declaration:
        raise ProjectionRegistryError("generator_source is valid only for generated mode")
    if mode is ProjectionMode.BOUNDED_HISTORICAL_SNAPSHOT:
        if "boundary_source" not in declaration:
            raise ProjectionRegistryError(
                "bounded historical projection requires boundary_source"
            )
        boundary_source_id = _source(
            declaration["boundary_source"], "boundary_source", authority_profile
        )
        _require_role(
            authority_profile,
            responsibility_id,
            boundary_source_id,
            SourceRole.AUTHORITY,
            "boundary source",
        )
    elif "boundary_source" in declaration:
        raise ProjectionRegistryError(
            "boundary_source is valid only for bounded historical mode"
        )
    return generator_source_id, boundary_source_id


def _projection(
    projection_id: str,
    value: object,
    authority_profile: GovernanceAuthorityProfile,
    integrity_profile: ConsumerIntegrityProfile,
    catalog: GovernedObjectCatalog | None,
    bindings: GovernanceBindingRegistry | None,
) -> Projection:
    label = f"projection {projection_id}"
    declaration = _mapping(value, label)
    actual_optional = frozenset(declaration.keys()) & _OPTIONAL_KEYS
    _exact_keys(declaration, _PROJECTION_KEYS | actual_optional, label)
    responsibility_id = _identifier(
        declaration["responsibility"], f"{label}.responsibility"
    )
    if responsibility_id not in authority_profile.responsibilities:
        raise ProjectionRegistryError(
            f"unknown governed responsibility: {responsibility_id}"
        )
    canonical_source_id = _source(
        declaration["canonical_source"], f"{label}.canonical_source", authority_profile
    )
    secondary_source_id = _source(
        declaration["secondary_source"], f"{label}.secondary_source", authority_profile
    )
    if canonical_source_id == secondary_source_id:
        raise ProjectionRegistryError("canonical and secondary sources must differ")
    _require_role(
        authority_profile, responsibility_id, canonical_source_id,
        SourceRole.AUTHORITY, "canonical source",
    )
    _require_role(
        authority_profile, responsibility_id, secondary_source_id,
        SourceRole.SECONDARY_REPRESENTATION, "secondary source",
    )
    mode = _mode(declaration["mode"])
    validation_id = _identifier(declaration["validation"], f"{label}.validation")
    if validation_id not in integrity_profile.validations:
        raise ProjectionRegistryError(f"unknown ValidationId: {validation_id}")
    generator_source_id, boundary_source_id = _mode_sources(
        declaration, mode, responsibility_id, authority_profile
    )
    target = _target(declaration["target"], catalog) if "target" in declaration else None
    binding_id = _identifier(declaration["binding"], f"{label}.binding") if "binding" in declaration else None
    if binding_id is not None:
        if bindings is None:
            raise ProjectionRegistryError("projection binding requires Governance Bindings")
        binding = bindings.bindings.get(binding_id)
        if binding is None:
            raise ProjectionRegistryError(f"unknown BindingId: {binding_id}")
        if binding.authority.responsibility_id != responsibility_id:
            raise ProjectionRegistryError("projection and binding responsibilities differ")
        if binding.authority.source_id != canonical_source_id:
            raise ProjectionRegistryError("projection canonical source differs from binding authority")
    return Projection(
        projection_id, responsibility_id, canonical_source_id, secondary_source_id,
        mode, validation_id, generator_source_id, boundary_source_id, target, binding_id,
    )


def _reject_active_ambiguity(projections: Mapping[str, Projection]) -> None:
    active: dict[
        tuple[str, str, GovernedObjectRef | None, str | None],
        tuple[str, ProjectionMode],
    ] = {}
    for projection in projections.values():
        if projection.mode is ProjectionMode.BOUNDED_HISTORICAL_SNAPSHOT:
            continue
        key = (
            projection.responsibility_id,
            projection.secondary_source_id,
            projection.target,
            projection.binding_id,
        )
        declaration = (projection.canonical_source_id, projection.mode)
        existing = active.get(key)
        if existing is not None and existing != declaration:
            raise ProjectionRegistryError(
                "conflicting active projection declarations for the same effective scope"
            )
        active[key] = declaration


def load(
    repository: Path,
    registry_route: ResolvedGovernanceRoute,
    authority_profile: GovernanceAuthorityProfile,
    integrity_profile: ConsumerIntegrityProfile,
    governed_objects: GovernedObjectCatalog | None = None,
    governance_bindings: GovernanceBindingRegistry | None = None,
) -> ProjectionRegistry:
    """Load one exact model-version-1 Projection Registry without execution."""

    _same_repository(repository, authority_profile.repository, "governance authority")
    _same_repository(repository, integrity_profile.repository, "repository integrity")
    if governed_objects is not None:
        _same_repository(repository, governed_objects.repository, "governed objects")
    if governance_bindings is not None:
        _same_repository(repository, governance_bindings.repository, "governance bindings")
    try:
        data = registry_route.target.read_bytes()
    except OSError as error:
        raise ProjectionRegistryError(f"cannot read Projection Registry: {error}") from error
    try:
        parsed = structured_data.parse_frontmatter_bytes(data)
    except structured_data.StructuredDataError as error:
        raise ProjectionRegistryError(f"cannot parse Projection Registry: {error}") from error
    if "projection_registry" not in parsed.metadata:
        raise ProjectionRegistryError("projection_registry is required")
    registry = _mapping(parsed.metadata["projection_registry"], "projection_registry")
    _exact_keys(registry, _REGISTRY_KEYS, "projection_registry")
    version = registry["model_version"]
    if type(version) is not int or version != _MODEL_VERSION:
        raise ProjectionRegistryError("unsupported model_version")
    authority = _registry_authority(
        registry["authority"], registry_route, authority_profile
    )
    declarations = _mapping(registry["projections"], "projections")
    projections = {
        _identifier(raw_id, "ProjectionId"): _projection(
            _identifier(raw_id, "ProjectionId"), raw_value, authority_profile,
            integrity_profile, governed_objects, governance_bindings,
        )
        for raw_id, raw_value in declarations.items()
    }
    _reject_active_ambiguity(projections)
    return ProjectionRegistry(
        repository, registry_route.target, _MODEL_VERSION, authority, projections
    )
