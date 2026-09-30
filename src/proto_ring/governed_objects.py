"""Load consumer-owned governed-object catalogs."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path

from proto_ring import structured_data
from proto_ring.governance_authority import GovernanceAuthorityProfile
from proto_ring.governance_routing import ResolvedGovernanceRoute


__all__ = [
    "GovernedObjectRef",
    "GovernedRelation",
    "GovernedObject",
    "GovernedInterface",
    "GovernedObjectCatalog",
    "GovernedObjectsError",
    "load",
]

_MODEL_VERSION = 1
_PROFILE_KEYS = frozenset({"model_version", "interfaces"})
_INTERFACE_KEYS = frozenset({"objects"})
_OBJECT_KEYS = frozenset({"responsibilities", "relations"})
_RELATION_KEYS = frozenset({"relation", "responsibility", "target"})
_TARGET_KEYS = frozenset({"interface", "object"})


@dataclass(frozen=True)
class GovernedObjectRef:
    interface_id: str
    object_id: str


@dataclass(frozen=True)
class GovernedRelation:
    relation: str
    responsibility: str
    target: GovernedObjectRef


@dataclass(frozen=True)
class GovernedObject:
    id: str
    responsibilities: frozenset[str]
    relations: frozenset[GovernedRelation]


@dataclass(frozen=True)
class GovernedInterface:
    id: str
    objects: dict[str, GovernedObject]


@dataclass(frozen=True)
class GovernedObjectCatalog:
    repository: Path
    carrier: Path
    model_version: int
    interfaces: dict[str, GovernedInterface]


class GovernedObjectsError(ValueError):
    """Report controlled failure to load a governed-object catalog."""


def _mapping(value: object, label: str) -> Mapping[object, object]:
    if not isinstance(value, Mapping):
        raise GovernedObjectsError(f"{label} must be a mapping")
    return value


def _exact_keys(
    value: Mapping[object, object], expected: frozenset[str], label: str
) -> None:
    actual = frozenset(value.keys())
    missing = expected - actual
    if missing:
        raise GovernedObjectsError(f"{label} is missing key: {sorted(missing)[0]}")
    unexpected = actual - expected
    if unexpected:
        raise GovernedObjectsError(
            f"{label} has unexpected key: {sorted(unexpected, key=str)[0]}"
        )


def _identifier(value: object, label: str) -> str:
    if not isinstance(value, str) or value == "":
        raise GovernedObjectsError(f"{label} must be a non-empty string")
    return value


def _responsibilities(
    value: object,
    object_label: str,
    authority_profile: GovernanceAuthorityProfile,
) -> frozenset[str]:
    if not isinstance(value, list):
        raise GovernedObjectsError(f"{object_label}.responsibilities must be a list")
    if not value:
        raise GovernedObjectsError(
            f"{object_label}.responsibilities must not be empty"
        )
    identities: set[str] = set()
    for index, raw_responsibility in enumerate(value):
        responsibility = _identifier(
            raw_responsibility,
            f"{object_label}.responsibilities[{index}]",
        )
        if responsibility in identities:
            raise GovernedObjectsError(
                f"duplicate responsibility on {object_label}: {responsibility}"
            )
        if responsibility not in authority_profile.responsibilities:
            raise GovernedObjectsError(
                f"unknown governed responsibility: {responsibility}"
            )
        identities.add(responsibility)
    return frozenset(identities)


def _relations(
    value: object,
    object_label: str,
    source_responsibilities: frozenset[str],
) -> frozenset[GovernedRelation]:
    if not isinstance(value, list):
        raise GovernedObjectsError(f"{object_label}.relations must be a list")
    relations: set[GovernedRelation] = set()
    for index, raw_relation in enumerate(value):
        label = f"{object_label}.relations[{index}]"
        declaration = _mapping(raw_relation, label)
        _exact_keys(declaration, _RELATION_KEYS, label)
        relation_id = _identifier(declaration["relation"], f"{label}.relation")
        responsibility = _identifier(
            declaration["responsibility"], f"{label}.responsibility"
        )
        if responsibility not in source_responsibilities:
            raise GovernedObjectsError(
                "relation responsibility is not declared by source object: "
                f"{responsibility}"
            )
        target_declaration = _mapping(declaration["target"], f"{label}.target")
        _exact_keys(target_declaration, _TARGET_KEYS, f"{label}.target")
        target = GovernedObjectRef(
            interface_id=_identifier(
                target_declaration["interface"], f"{label}.target.interface"
            ),
            object_id=_identifier(
                target_declaration["object"], f"{label}.target.object"
            ),
        )
        relation = GovernedRelation(relation_id, responsibility, target)
        if relation in relations:
            raise GovernedObjectsError(f"duplicate relation on {object_label}")
        relations.add(relation)
    return frozenset(relations)


def _interfaces(
    value: object,
    authority_profile: GovernanceAuthorityProfile,
) -> dict[str, GovernedInterface]:
    declarations = _mapping(value, "interfaces")
    interfaces: dict[str, GovernedInterface] = {}
    for raw_interface_id, raw_interface in declarations.items():
        interface_id = _identifier(raw_interface_id, "interface id")
        interface_declaration = _mapping(raw_interface, f"interface {interface_id}")
        _exact_keys(
            interface_declaration,
            _INTERFACE_KEYS,
            f"interface {interface_id}",
        )
        object_declarations = _mapping(
            interface_declaration["objects"], f"interface {interface_id}.objects"
        )
        objects: dict[str, GovernedObject] = {}
        for raw_object_id, raw_object in object_declarations.items():
            object_id = _identifier(raw_object_id, "object id")
            object_label = f"object {interface_id}/{object_id}"
            object_declaration = _mapping(raw_object, object_label)
            _exact_keys(object_declaration, _OBJECT_KEYS, object_label)
            responsibilities = _responsibilities(
                object_declaration["responsibilities"],
                object_label,
                authority_profile,
            )
            relations = _relations(
                object_declaration["relations"], object_label, responsibilities
            )
            objects[object_id] = GovernedObject(
                object_id, responsibilities, relations
            )
        interfaces[interface_id] = GovernedInterface(interface_id, objects)
    return interfaces


def _validate_targets(interfaces: Mapping[str, GovernedInterface]) -> None:
    for interface in interfaces.values():
        for governed_object in interface.objects.values():
            for relation in governed_object.relations:
                target_interface = interfaces.get(relation.target.interface_id)
                if target_interface is None:
                    raise GovernedObjectsError(
                        "relation target interface does not exist: "
                        f"{relation.target.interface_id}"
                    )
                if relation.target.object_id not in target_interface.objects:
                    raise GovernedObjectsError(
                        "relation target object does not exist: "
                        f"{relation.target.interface_id}/"
                        f"{relation.target.object_id}"
                    )


def load(
    repository: Path,
    profile_route: ResolvedGovernanceRoute,
    authority_profile: GovernanceAuthorityProfile,
) -> GovernedObjectCatalog:
    """Load an exact model-version-1 governed-object catalog."""

    try:
        repository_root = repository.resolve()
        authority_root = authority_profile.repository.resolve()
    except OSError as error:
        raise GovernedObjectsError(f"cannot resolve repository root: {error}") from error
    if repository_root != authority_root:
        raise GovernedObjectsError(
            "governed-object and governance-authority repositories differ"
        )
    try:
        data = profile_route.target.read_bytes()
    except OSError as error:
        raise GovernedObjectsError(
            f"cannot read governed-object profile: {error}"
        ) from error
    try:
        parsed = structured_data.parse_frontmatter_bytes(data)
    except structured_data.StructuredDataError as error:
        raise GovernedObjectsError(
            f"cannot parse governed-object profile: {error}"
        ) from error
    if "governed_objects" not in parsed.metadata:
        raise GovernedObjectsError("governed_objects is required")
    profile = _mapping(parsed.metadata["governed_objects"], "governed_objects")
    _exact_keys(profile, _PROFILE_KEYS, "governed_objects")
    model_version = profile["model_version"]
    if type(model_version) is not int or model_version != _MODEL_VERSION:
        raise GovernedObjectsError("unsupported model_version")
    interfaces = _interfaces(profile["interfaces"], authority_profile)
    _validate_targets(interfaces)
    return GovernedObjectCatalog(
        repository=repository,
        carrier=profile_route.target,
        model_version=_MODEL_VERSION,
        interfaces=interfaces,
    )
