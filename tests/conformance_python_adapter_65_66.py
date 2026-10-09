from __future__ import annotations

from dataclasses import replace
from pathlib import Path

from proto_ring import (
    evidence_requirements,
    governance_authority,
    governance_bindings,
    governed_objects,
    projection_registry,
    repository_integrity,
)
from proto_ring.exact_evidence_binding import EvidenceBinding, EvidenceRequirement, evaluate
from proto_ring.governance_routing import ResolvedGovernanceRoute

from conformance_corpus_test_support import decode_transport, observation
from conformance_python_adapter_66_evidence import binding_status
from conformance_python_adapter_66_support import (
    binding_observation,
    binding_registry,
    integrity_profile,
    projection_observation,
    repository_snapshot,
    requirement_observation,
    validation_observation,
)

OWNED_RESPONSIBILITIES = frozenset({
    "governance-authority.profile", "governed-objects.catalog",
    "governance-bindings.registry", "projection-registry.registry",
    "repository-integrity.profile", "evidence-requirements.registry",
    "exact-evidence-binding.evaluate",
})


def _route(root: Path, path: str, responsibility: str) -> ResolvedGovernanceRoute:
    return ResolvedGovernanceRoute(("capabilities", responsibility, "routes", "profile"), path, root / path)


def _authority(root: Path, data: dict[str, object]) -> governance_authority.GovernanceAuthorityProfile:
    sources = {}
    for source_id, item in data["sources"].items():
        target = None if item.get("repository_target") is None else _route(root, item["repository_target"], source_id)
        sources[source_id] = governance_authority.GovernedSource(source_id, target)
    responsibilities = {}
    for responsibility_id, item in data["responsibilities"].items():
        roles = {source: governance_authority.SourceRole(role) for source, role in item["roles"].items()}
        precedence = tuple(governance_authority.PrecedenceEdge(edge["higher_source"], edge["lower_source"]) for edge in item.get("precedence", []))
        responsibilities[responsibility_id] = governance_authority.GovernedResponsibility(responsibility_id, roles, precedence)
    return governance_authority.GovernanceAuthorityProfile(
        root, root / data["carrier"], 1, sources, responsibilities
    )


def _catalog(root: Path, data: dict[str, object]) -> governed_objects.GovernedObjectCatalog:
    interfaces = {}
    for interface_id, objects in data["interfaces"].items():
        values = {
            object_id: governed_objects.GovernedObject(
                object_id, frozenset(item["responsibilities"]), frozenset()
            ) for object_id, item in objects.items()
        }
        interfaces[interface_id] = governed_objects.GovernedInterface(interface_id, values)
    return governed_objects.GovernedObjectCatalog(root, root / data["carrier"], 1, interfaces)


def _success(value: object):
    return observation("result", value)


def _rejection(responsibility_id: str):
    return observation("controlled_rejection", {"category": f"{responsibility_id}.rejected"})


def _authority_result(root: Path, arguments: dict[str, object]):
    route = _route(root, arguments["path"], "governance_authority")
    try:
        loaded = governance_authority.load(root, route)
    except governance_authority.GovernanceAuthorityError:
        return _rejection("governance-authority.profile")
    query = arguments.get("query")
    result = None
    if query:
        operation = query["operation"]
        if operation == "role_of":
            result = governance_authority.role_of(loaded, query["responsibility"], query["source"])
        elif operation == "authority_sources":
            result = governance_authority.authority_sources(loaded, query["responsibility"])
        elif operation == "outranks":
            result = governance_authority.outranks(loaded, query["responsibility"], query["left"], query["right"])
        elif operation == "source_binding":
            binding = loaded.sources[query["source"]].repository_target
            if binding is not None:
                result = {
                    "declared_path": binding.declared_path,
                    "resolved_target": binding.target.relative_to(root.resolve()).as_posix(),
                    "route": list(binding.route),
                }
    return _success({
        "model_version": loaded.model_version, "sources": sorted(loaded.sources),
        "responsibilities": sorted(loaded.responsibilities), "query_result": result,
    })


def _objects_result(root: Path, arguments: dict[str, object]):
    authority = _authority(root, arguments["semantic_inputs"]["authority"])
    if arguments.get("foreign_authority"):
        authority = replace(authority, repository=root.parent / "foreign")
    route = _route(root, arguments["path"], "governed_objects")
    try:
        loaded = governed_objects.load(root, route, authority)
    except governed_objects.GovernedObjectsError:
        return _rejection("governed-objects.catalog")
    result = {
        "model_version": loaded.model_version,
        "interfaces": {key: sorted(value.objects) for key, value in sorted(loaded.interfaces.items())},
    }
    query = arguments.get("query")
    if query:
        governed_object = loaded.interfaces[query["interface"]].objects[query["object"]]
        if query["operation"] == "object_responsibilities":
            result["query_result"] = sorted(governed_object.responsibilities)
        elif query["operation"] == "object_relations":
            relations = sorted(
                governed_object.relations,
                key=lambda item: (
                    item.relation,
                    item.responsibility,
                    item.target.interface_id,
                    item.target.object_id,
                ),
            )
            result["query_result"] = [
                {
                    "relation": item.relation,
                    "responsibility": item.responsibility,
                    "target_interface": item.target.interface_id,
                    "target_object": item.target.object_id,
                }
                for item in relations
            ]
    return _success(result)


def _bindings_catalog_root(root: Path, arguments: dict[str, object]) -> Path:
    selected = arguments.get("foreign_support_repository")
    if selected is None:
        return root
    if not isinstance(selected, str) or selected != "catalog":
        raise ValueError("invalid foreign_support_repository transport value")
    return root / "foreign"


def _bindings_result(root: Path, arguments: dict[str, object]):
    support = arguments["semantic_inputs"]
    authority = _authority(root, support["authority"])
    catalog = (
        _catalog(_bindings_catalog_root(root, arguments), support["catalog"])
        if arguments.get("with_catalog")
        else None
    )
    route = _route(root, arguments["path"], "governance_bindings")
    before = repository_snapshot(root) if arguments.get("observe_read_only") else None
    try:
        loaded = governance_bindings.load(root, route, authority, catalog)
    except governance_bindings.GovernanceBindingsError:
        return _rejection("governance-bindings.registry")
    value = {
        "model_version": loaded.model_version, "source": loaded.source_id,
        "bindings": sorted(loaded.bindings),
        "kinds": {key: item.kind.value for key, item in sorted(loaded.bindings.items())},
    }
    observed = arguments.get("observe_binding_id")
    if observed is not None:
        value["loaded_binding"] = binding_observation(loaded.bindings[observed])
    if before is not None:
        value["repository_unchanged"] = repository_snapshot(root) == before
    return _success(value)


def _integrity_result(root: Path, arguments: dict[str, object]):
    support = arguments["semantic_inputs"]
    authority = _authority(root, support["authority"])
    catalog = _catalog(root, support["catalog"]) if arguments.get("with_catalog") else None
    route = _route(root, arguments["path"], "repository_integrity")
    before = repository_snapshot(root) if arguments.get("observe_read_only") else None
    try:
        loaded = repository_integrity.load(root, route, authority, catalog)
    except repository_integrity.RepositoryIntegrityError:
        return _rejection("repository-integrity.profile")
    if arguments.get("identity_only"):
        return _success({"identity": loaded.identity})
    value = {
        "model_version": loaded.model_version, "environments": sorted(loaded.environments),
        "order": list(loaded.order), "validations": sorted(loaded.validations),
        "continue_after_non_satisfied": loaded.continue_after_non_satisfied,
        "identity_present": bool(loaded.identity),
        "profile_authority": {
            "responsibility": loaded.authority.responsibility,
            "source": loaded.authority.source,
        },
    }
    observed = arguments.get("observe_validation_id")
    if observed is not None:
        value["loaded_validation"] = validation_observation(loaded.validations[observed])
    observed_many = arguments.get("observe_validation_ids")
    if observed_many is not None:
        value["loaded_validations"] = {
            validation_id: validation_observation(loaded.validations[validation_id])
            for validation_id in observed_many
        }
    if before is not None:
        value["repository_unchanged"] = repository_snapshot(root) == before
    if arguments.get("observe_execution_sentinel"):
        value["execution_sentinel_absent"] = not (root / "execution-sentinel.txt").exists()
    return _success(value)


def _evidence_result(root: Path, arguments: dict[str, object]):
    support = arguments["semantic_inputs"]
    authority = _authority(root, support["authority"])
    catalog = _catalog(root, support["catalog"]) if arguments.get("with_catalog") else None
    route = _route(root, arguments["path"], "evidence_requirements")
    before = repository_snapshot(root) if arguments.get("observe_read_only") else None
    try:
        loaded = evidence_requirements.load(root, route, authority, catalog)
    except evidence_requirements.EvidenceRequirementsError:
        return _rejection("evidence-requirements.registry")
    if arguments["operation"] == "load_and_evaluate_unknown_context":
        persistent = loaded.requirements[arguments["requirement_id"]]
        resolution = arguments["consumer_resolution"]
        context = resolution["context"]
        if context["source_id"] != persistent.context.source_id:
            raise ValueError("consumer context source differs from persistent requirement")
        classes = persistent.evidence_classes.explicit_classes
        requirement = EvidenceRequirement(
            classes,
            bytes.fromhex(resolution["subject"]["identity_hex"])
            if resolution["subject"]["known"] else None,
            bytes.fromhex(context["identity_hex"]) if context["known"] else None,
            persistent.context.required,
        )
        candidate = resolution["candidate"]
        binding = EvidenceBinding(
            candidate["evidence_class"], bytes.fromhex(candidate["subject_hex"]),
            bytes.fromhex(candidate["context_hex"]),
        )
        return _success({
            "registry": "ACCEPTED", "context_required": persistent.context.required,
            "context_source_id": persistent.context.source_id,
            "resolved_context_known": context["known"],
            "binding_status": evaluate(requirement, binding).value,
        })
    value = {
        "model_version": loaded.model_version, "requirements": sorted(loaded.requirements),
        "authority_source": loaded.authority.source_id,
        "registry_authority": {
            "responsibility": loaded.authority.responsibility_id,
            "source": loaded.authority.source_id,
        },
    }
    observed = arguments.get("observe_requirement_id")
    if observed is not None:
        value["loaded_requirement"] = requirement_observation(loaded.requirements[observed])
    if before is not None:
        value["repository_unchanged"] = repository_snapshot(root) == before
    return _success(value)


def _exact_binding(arguments: dict[str, object]):
    if arguments["operation"] == "evaluate_requirement_history":
        comparisons = arguments.get("comparisons", [arguments])
        values = [{
            "label": item["label"],
            "historical": binding_status(item["historical_requirement"], item["binding"]),
            "current": binding_status(item["current_requirement"], item["binding"]),
        } for item in comparisons]
        if any(item["historical"] is None or item["current"] is None for item in values):
            return _rejection("exact-evidence-binding.evaluate")
        return _success({"comparisons": values})
    status = binding_status(arguments["requirement"], arguments["binding"])
    return _rejection("exact-evidence-binding.evaluate") if status is None else _success({"status": status})


def _projection_support_roots(
    root: Path, arguments: dict[str, object]
) -> dict[str, Path]:
    selected = arguments.get("foreign_support_repository")
    allowed = {"integrity_profile", "catalog", "bindings"}
    if selected is not None and (
        not isinstance(selected, str) or selected not in allowed
    ):
        raise ValueError("invalid foreign_support_repository transport value")
    return {
        name: root / "foreign" if selected == name else root
        for name in allowed
    }


def _projection_result(root: Path, arguments: dict[str, object]):
    support = arguments["semantic_inputs"]
    support_roots = _projection_support_roots(root, arguments)
    authority = _authority(root, support["authority"])
    integrity = integrity_profile(
        support_roots["integrity_profile"], support["integrity_profile"]
    )
    catalog = (
        _catalog(support_roots["catalog"], support["catalog"])
        if arguments.get("with_catalog") else None
    )
    bindings = (
        binding_registry(support_roots["bindings"], support["bindings"])
        if arguments.get("with_bindings") else None
    )
    route = _route(root, arguments["path"], "projection_registry")
    before = repository_snapshot(root) if arguments.get("observe_read_only") else None
    try:
        loaded = projection_registry.load(
            root, route, authority, integrity, catalog, bindings
        )
    except projection_registry.ProjectionRegistryError:
        return _rejection("projection-registry.registry")
    value = {
        "model_version": loaded.model_version, "projections": sorted(loaded.projections),
        "modes": {key: item.mode.value for key, item in sorted(loaded.projections.items())},
    }
    if arguments.get("qualified_observation"):
        value["registry"] = "ACCEPTED"
        value["loaded"] = {
            key: {"canonical_source": item.canonical_source_id, "secondary_source": item.secondary_source_id, "mode": item.mode.value}
            for key, item in sorted(loaded.projections.items())
        }
    if arguments.get("observe_registry_authority"):
        value["registry_authority"] = {
            "responsibility": loaded.authority.responsibility_id,
            "source": loaded.authority.source_id,
        }
    observed = arguments.get("observe_projection_id")
    if observed is not None:
        value["loaded_projection"] = projection_observation(loaded.projections[observed])
    if before is not None:
        value["repository_unchanged"] = repository_snapshot(root) == before
    return _success(value)


def precheck_support(
    responsibility_id: str, root: Path | None, arguments: dict[str, object]
) -> None:
    if responsibility_id == "exact-evidence-binding.evaluate":
        return
    support = arguments["semantic_inputs"]
    authority = _authority(root, support["authority"])
    if responsibility_id == "governance-bindings.registry":
        if arguments.get("with_catalog"):
            _catalog(_bindings_catalog_root(root, arguments), support["catalog"])
        return
    if responsibility_id in {
        "repository-integrity.profile",
        "evidence-requirements.registry",
    }:
        if arguments.get("with_catalog"):
            _catalog(root, support["catalog"])
        return
    if responsibility_id == "projection-registry.registry":
        support_roots = _projection_support_roots(root, arguments)
        integrity_profile(
            support_roots["integrity_profile"], support["integrity_profile"]
        )
        if arguments.get("with_catalog"):
            _catalog(support_roots["catalog"], support["catalog"])
        if arguments.get("with_bindings"):
            binding_registry(support_roots["bindings"], support["bindings"])
        return
    raise ValueError(f"unsupported #66 responsibility: {responsibility_id}")


def _dispatch(responsibility_id: str, root: Path | None, arguments: dict[str, object]):
    if responsibility_id == "governance-authority.profile":
        return _authority_result(root, arguments)
    if responsibility_id == "governed-objects.catalog":
        return _objects_result(root, arguments)
    if responsibility_id == "governance-bindings.registry":
        return _bindings_result(root, arguments)
    if responsibility_id == "projection-registry.registry":
        return _projection_result(root, arguments)
    if responsibility_id == "repository-integrity.profile":
        return _integrity_result(root, arguments)
    if responsibility_id == "evidence-requirements.registry":
        return _evidence_result(root, arguments)
    if responsibility_id == "exact-evidence-binding.evaluate":
        return _exact_binding(arguments)
    raise ValueError(f"unsupported responsibility: {responsibility_id}")


def execute(responsibility_id, fixture):
    if responsibility_id not in OWNED_RESPONSIBILITIES:
        raise ValueError(f"responsibility is not owned by adapter 65/66: {responsibility_id}")
    raw = fixture.run(lambda root, arguments: _dispatch(responsibility_id, root, arguments))
    if responsibility_id == "repository-integrity.profile" and isinstance(raw, dict) and "invocations" in raw:
        values = {key: decode_transport(value["value"]) for key, value in raw["invocations"].items()}
        if all("identity" in value for value in values.values()):
            left, right = values
            return _success({"profiles_valid": True, "identity_relation": {"left": left, "right": right, "relation": "EQUAL" if values[left]["identity"] == values[right]["identity"] else "NOT_EQUAL"}})
    return raw
