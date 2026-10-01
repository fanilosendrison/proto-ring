"""Load consumer-owned persistent evidence requirement registries."""

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
from proto_ring.governance_routing import ResolvedGovernanceRoute
from proto_ring.governed_objects import GovernedObjectCatalog, GovernedObjectRef

__all__ = [
    "InstantiationKind",
    "EvidenceClassKind",
    "RegistryAuthority",
    "RequirementInstantiation",
    "EvidenceClassAdmission",
    "ContextBinding",
    "PersistentEvidenceRequirement",
    "EvidenceRequirementRegistry",
    "EvidenceRequirementsError",
    "load",
]

_MODEL_VERSION = 1
_REGISTRY_KEYS = frozenset({"model_version", "authority", "requirements"})
_AUTHORITY_KEYS = frozenset({"responsibility", "source"})
_REQUIREMENT_KEYS = frozenset(
    {"responsibility", "instances", "evidence_classes", "subject", "context", "candidates"}
)
_TARGET_KEYS = frozenset({"interface", "object"})
_SINGLE_KEYS = frozenset({"kind"})
_SOURCE_KIND_KEYS = frozenset({"kind", "source"})
_EXPLICIT_CLASS_KEYS = frozenset({"kind", "classes"})
_SOURCE_REFERENCE_KEYS = frozenset({"source"})
_CONTEXT_NOT_REQUIRED_KEYS = frozenset({"required"})
_CONTEXT_REQUIRED_KEYS = frozenset({"required", "source"})


class EvidenceRequirementsError(ValueError):
    """Report controlled failure to load an evidence requirement registry."""


class InstantiationKind(StrEnum):
    SINGLE = "single"
    SOURCE = "source"


class EvidenceClassKind(StrEnum):
    EXPLICIT = "explicit"
    SOURCE = "source"


@dataclass(frozen=True)
class RegistryAuthority:
    responsibility_id: str
    source_id: str


@dataclass(frozen=True)
class RequirementInstantiation:
    kind: InstantiationKind
    source_id: str | None = None


@dataclass(frozen=True)
class EvidenceClassAdmission:
    kind: EvidenceClassKind
    explicit_classes: frozenset[str] | None = None
    source_id: str | None = None


@dataclass(frozen=True)
class ContextBinding:
    required: bool
    source_id: str | None = None


@dataclass(frozen=True)
class PersistentEvidenceRequirement:
    id: str
    responsibility_id: str
    target: GovernedObjectRef | None
    instances: RequirementInstantiation
    evidence_classes: EvidenceClassAdmission
    subject_source_id: str
    context: ContextBinding
    candidate_source_id: str


@dataclass(frozen=True)
class EvidenceRequirementRegistry:
    repository: Path
    carrier: Path
    model_version: int
    authority: RegistryAuthority
    requirements: dict[str, PersistentEvidenceRequirement]


def _mapping(value: object, label: str) -> Mapping[object, object]:
    if not isinstance(value, Mapping):
        raise EvidenceRequirementsError(f"{label} must be a mapping")
    return value


def _exact_keys(
    value: Mapping[object, object], expected: frozenset[str], label: str
) -> None:
    actual = frozenset(value.keys())
    missing = expected - actual
    if missing:
        raise EvidenceRequirementsError(f"{label} is missing key: {sorted(missing)[0]}")
    unexpected = actual - expected
    if unexpected:
        raise EvidenceRequirementsError(
            f"{label} has unexpected key: {sorted(unexpected, key=str)[0]}"
        )


def _identifier(value: object, label: str) -> str:
    if not isinstance(value, str) or value == "":
        raise EvidenceRequirementsError(f"{label} must be a non-empty string")
    return value


def _source_id(
    value: object,
    label: str,
    authority_profile: GovernanceAuthorityProfile,
) -> str:
    source_id = _identifier(value, label)
    if source_id not in authority_profile.sources:
        raise EvidenceRequirementsError(f"unknown governed source: {source_id}")
    return source_id


def _source_reference(
    value: object,
    label: str,
    authority_profile: GovernanceAuthorityProfile,
) -> str:
    declaration = _mapping(value, label)
    _exact_keys(declaration, _SOURCE_REFERENCE_KEYS, label)
    return _source_id(declaration["source"], f"{label}.source", authority_profile)


def _instances(
    value: object,
    authority_profile: GovernanceAuthorityProfile,
) -> RequirementInstantiation:
    declaration = _mapping(value, "requirement instances")
    raw_kind = _identifier(declaration.get("kind"), "requirement instances.kind")
    try:
        kind = InstantiationKind(raw_kind)
    except ValueError as error:
        raise EvidenceRequirementsError(f"unknown instantiation kind: {raw_kind}") from error
    if kind is InstantiationKind.SINGLE:
        _exact_keys(declaration, _SINGLE_KEYS, "single instances")
        return RequirementInstantiation(kind)
    _exact_keys(declaration, _SOURCE_KIND_KEYS, "source instances")
    return RequirementInstantiation(
        kind,
        _source_id(declaration["source"], "instances.source", authority_profile),
    )


def _evidence_classes(
    value: object,
    authority_profile: GovernanceAuthorityProfile,
) -> EvidenceClassAdmission:
    declaration = _mapping(value, "evidence classes")
    raw_kind = _identifier(declaration.get("kind"), "evidence classes.kind")
    try:
        kind = EvidenceClassKind(raw_kind)
    except ValueError as error:
        raise EvidenceRequirementsError(f"unknown evidence class kind: {raw_kind}") from error
    if kind is EvidenceClassKind.SOURCE:
        _exact_keys(declaration, _SOURCE_KIND_KEYS, "source evidence classes")
        return EvidenceClassAdmission(
            kind,
            source_id=_source_id(
                declaration["source"], "evidence_classes.source", authority_profile
            ),
        )
    _exact_keys(declaration, _EXPLICIT_CLASS_KEYS, "explicit evidence classes")
    raw_classes = declaration["classes"]
    if not isinstance(raw_classes, list) or not raw_classes:
        raise EvidenceRequirementsError("explicit evidence classes must be a non-empty list")
    classes = tuple(
        _identifier(item, f"evidence_classes.classes[{index}]")
        for index, item in enumerate(raw_classes)
    )
    if len(set(classes)) != len(classes):
        raise EvidenceRequirementsError("explicit evidence classes must be duplicate-free")
    return EvidenceClassAdmission(kind, explicit_classes=frozenset(classes))


def _context(
    value: object,
    authority_profile: GovernanceAuthorityProfile,
) -> ContextBinding:
    declaration = _mapping(value, "context binding")
    required = declaration.get("required")
    if type(required) is not bool:
        raise EvidenceRequirementsError("context.required must be boolean")
    if not required:
        _exact_keys(declaration, _CONTEXT_NOT_REQUIRED_KEYS, "context not required")
        return ContextBinding(False)
    _exact_keys(declaration, _CONTEXT_REQUIRED_KEYS, "context required")
    return ContextBinding(
        True,
        _source_id(declaration["source"], "context.source", authority_profile),
    )


def _target(value: object, label: str) -> GovernedObjectRef:
    declaration = _mapping(value, label)
    _exact_keys(declaration, _TARGET_KEYS, label)
    return GovernedObjectRef(
        _identifier(declaration["interface"], f"{label}.interface"),
        _identifier(declaration["object"], f"{label}.object"),
    )


def _validate_target(
    target: GovernedObjectRef,
    responsibility_id: str,
    catalog: GovernedObjectCatalog | None,
) -> None:
    if catalog is None:
        raise EvidenceRequirementsError("requirement target requires governed objects")
    interface = catalog.interfaces.get(target.interface_id)
    governed_object = None if interface is None else interface.objects.get(target.object_id)
    if governed_object is None:
        raise EvidenceRequirementsError(
            f"governed object does not exist: {target.interface_id}/{target.object_id}"
        )
    if responsibility_id not in governed_object.responsibilities:
        raise EvidenceRequirementsError(
            "governed object does not participate in requirement responsibility"
        )


def _same_repository(repository: Path, other: Path, label: str) -> None:
    try:
        if repository.resolve() != other.resolve():
            raise EvidenceRequirementsError(f"{label} repository differs")
    except OSError as error:
        raise EvidenceRequirementsError(f"cannot resolve repository root: {error}") from error


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
    source_id = _source_id(declaration["source"], "authority.source", authority_profile)
    if responsibility_id not in authority_profile.responsibilities:
        raise EvidenceRequirementsError(
            f"unknown governed responsibility: {responsibility_id}"
        )
    if role_of(authority_profile, responsibility_id, source_id) is not SourceRole.AUTHORITY:
        raise EvidenceRequirementsError("registry source is not authority for responsibility")
    source = authority_profile.sources[source_id]
    if source.repository_target is None:
        raise EvidenceRequirementsError("registry authority source has no repository target")
    try:
        if source.repository_target.target.resolve() != route.target.resolve():
            raise EvidenceRequirementsError(
                "registry authority source does not resolve to registry carrier"
            )
    except OSError as error:
        raise EvidenceRequirementsError(f"cannot resolve registry carrier: {error}") from error
    return RegistryAuthority(responsibility_id, source_id)


def _requirements(
    value: object,
    authority_profile: GovernanceAuthorityProfile,
    catalog: GovernedObjectCatalog | None,
) -> dict[str, PersistentEvidenceRequirement]:
    declarations = _mapping(value, "requirements")
    requirements: dict[str, PersistentEvidenceRequirement] = {}
    for raw_id, raw_requirement in declarations.items():
        requirement_id = _identifier(raw_id, "EvidenceRequirementId")
        label = f"requirement {requirement_id}"
        declaration = _mapping(raw_requirement, label)
        expected = _REQUIREMENT_KEYS | ({"target"} if "target" in declaration else set())
        _exact_keys(declaration, frozenset(expected), label)
        responsibility_id = _identifier(
            declaration["responsibility"], f"{label}.responsibility"
        )
        if responsibility_id not in authority_profile.responsibilities:
            raise EvidenceRequirementsError(
                f"unknown governed responsibility: {responsibility_id}"
            )
        target = _target(declaration["target"], f"{label}.target") if "target" in declaration else None
        if target is not None:
            _validate_target(target, responsibility_id, catalog)
        requirements[requirement_id] = PersistentEvidenceRequirement(
            requirement_id,
            responsibility_id,
            target,
            _instances(declaration["instances"], authority_profile),
            _evidence_classes(declaration["evidence_classes"], authority_profile),
            _source_reference(declaration["subject"], f"{label}.subject", authority_profile),
            _context(declaration["context"], authority_profile),
            _source_reference(declaration["candidates"], f"{label}.candidates", authority_profile),
        )
    return requirements


def load(
    repository: Path,
    registry_route: ResolvedGovernanceRoute,
    authority_profile: GovernanceAuthorityProfile,
    governed_objects: GovernedObjectCatalog | None = None,
) -> EvidenceRequirementRegistry:
    """Load an exact model-version-1 persistent Evidence Requirement Registry."""

    _same_repository(repository, authority_profile.repository, "governance authority")
    if governed_objects is not None:
        _same_repository(repository, governed_objects.repository, "governed objects")
    try:
        data = registry_route.target.read_bytes()
    except OSError as error:
        raise EvidenceRequirementsError(
            f"cannot read evidence requirement registry: {error}"
        ) from error
    try:
        parsed = structured_data.parse_frontmatter_bytes(data)
    except structured_data.StructuredDataError as error:
        raise EvidenceRequirementsError(
            f"cannot parse evidence requirement registry: {error}"
        ) from error
    if "evidence_requirements" not in parsed.metadata:
        raise EvidenceRequirementsError("evidence_requirements is required")
    registry = _mapping(parsed.metadata["evidence_requirements"], "evidence_requirements")
    _exact_keys(registry, _REGISTRY_KEYS, "evidence_requirements")
    version = registry["model_version"]
    if type(version) is not int or version != _MODEL_VERSION:
        raise EvidenceRequirementsError("unsupported model_version")
    authority = _registry_authority(
        registry["authority"], registry_route, authority_profile
    )
    return EvidenceRequirementRegistry(
        repository,
        registry_route.target,
        _MODEL_VERSION,
        authority,
        _requirements(registry["requirements"], authority_profile, governed_objects),
    )
