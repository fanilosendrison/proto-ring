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


def _integrity(root: Path, data: dict[str, object]):
    definitions = {}
    for item in data["validations"]:
        definitions[item["id"]] = repository_integrity.ValidationDefinition(
            item["id"], item["responsibility"], tuple(item["prerequisites"]), None,
            repository_integrity.ValidationInstances("single"),
            repository_integrity.CommandBinding(
                item["environment"], tuple(item["arguments"]),
                frozenset(item["undetermined_exit_codes"]),
            ),
        )
    authority = data["authority"]
    return repository_integrity.ConsumerIntegrityProfile(
        root, root / data["carrier"], 1,
        repository_integrity.ProfileAuthority(authority["responsibility"], authority["source"]),
        frozenset(data["environments"]), data["continue_after_non_satisfied"],
        definitions, tuple(data["order"]), data["identity"],
    )


def _bindings(root: Path, data: dict[str, object]):
    values = {}
    for item in data["bindings"]:
        kind = governance_bindings.BindingKind(item["kind"])
        if kind is governance_bindings.BindingKind.EXECUTABLE_PROVIDER:
            identity = governance_bindings.ExecutableProviderIdentity(item["repository"], item["commit"])
        else:
            identity = governance_bindings.GovernanceContractIdentity(item["repository"], item["commit"], item["path"])
        values[item["id"]] = governance_bindings.GovernanceBinding(
            item["id"], kind,
            governance_bindings.GovernanceBindingScope(
                governance_bindings.ScopeKind(item["scope_kind"]), item.get("capability")
            ),
            identity,
            governance_bindings.BindingAuthority(item["responsibility"], item["source"]),
        )
    return governance_bindings.GovernanceBindingRegistry(
        root, root / data["carrier"], 1, data["source"], values
    )


def _success(value: object):
    return observation("result", value)


def _rejection(responsibility_id: str):
    return observation("controlled_rejection", {"category": f"{responsibility_id}.rejected"})


def _authority_result(root: Path, arguments: dict[str, object]):
    loaded = governance_authority.load(root, _route(root, arguments["path"], "governance_authority"))
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
    loaded = governed_objects.load(root, _route(root, arguments["path"], "governed_objects"), authority)
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


def _bindings_result(root: Path, arguments: dict[str, object]):
    support = arguments["semantic_inputs"]
    carrier = root / str(arguments["path"])
    before = carrier.read_bytes() if arguments.get("observe_read_only") else None
    loaded = governance_bindings.load(
        root, _route(root, arguments["path"], "governance_bindings"),
        _authority(root, support["authority"]),
        _catalog(root, support["catalog"]) if arguments.get("with_catalog") else None,
    )
    value = {
        "model_version": loaded.model_version, "source": loaded.source_id,
        "bindings": sorted(loaded.bindings),
        "kinds": {key: item.kind.value for key, item in sorted(loaded.bindings.items())},
    }
    if before is not None:
        value["carrier_unchanged"] = carrier.read_bytes() == before
    return _success(value)


def _integrity_result(root: Path, arguments: dict[str, object]):
    support = arguments["semantic_inputs"]
    loaded = repository_integrity.load(
        root, _route(root, arguments["path"], "repository_integrity"),
        _authority(root, support["authority"]),
        _catalog(root, support["catalog"]) if arguments.get("with_catalog") else None,
    )
    if arguments.get("identity_only"):
        return _success({"identity": loaded.identity})
    return _success({
        "model_version": loaded.model_version, "environments": sorted(loaded.environments),
        "order": list(loaded.order), "validations": sorted(loaded.validations),
        "continue_after_non_satisfied": loaded.continue_after_non_satisfied,
        "identity_present": bool(loaded.identity),
    })


def _evidence_result(root: Path, arguments: dict[str, object]):
    support = arguments["semantic_inputs"]
    loaded = evidence_requirements.load(
        root, _route(root, arguments["path"], "evidence_requirements"),
        _authority(root, support["authority"]),
        _catalog(root, support["catalog"]) if arguments.get("with_catalog") else None,
    )
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
    return _success({
        "model_version": loaded.model_version, "requirements": sorted(loaded.requirements),
        "authority_source": loaded.authority.source_id,
    })


def _binding_status(requirement_data, binding_data):
    requirement = EvidenceRequirement(
        frozenset(requirement_data["admitted_classes"]),
        None if requirement_data["subject_hex"] is None else bytes.fromhex(requirement_data["subject_hex"]),
        None if requirement_data["context_hex"] is None else bytes.fromhex(requirement_data["context_hex"]),
        requirement_data["context_required"],
    )
    binding = None
    if binding_data is not None:
        binding = EvidenceBinding(
            binding_data["evidence_class"],
            None if binding_data["subject_hex"] is None else bytes.fromhex(binding_data["subject_hex"]),
            None if binding_data["context_hex"] is None else bytes.fromhex(binding_data["context_hex"]),
        )
    return evaluate(requirement, binding).value


def _exact_binding(arguments: dict[str, object]):
    if arguments["operation"] == "evaluate_requirement_history":
        comparisons = arguments.get("comparisons", [arguments])
        return _success({"comparisons": [{
            "label": item["label"],
            "historical": _binding_status(item["historical_requirement"], item["binding"]),
            "current": _binding_status(item["current_requirement"], item["binding"]),
        } for item in comparisons]})
    return _success({"status": _binding_status(arguments["requirement"], arguments["binding"])})


def _projection_result(root: Path, arguments: dict[str, object]):
    support = arguments["semantic_inputs"]
    loaded = projection_registry.load(
        root, _route(root, arguments["path"], "projection_registry"),
        _authority(root, support["authority"]), _integrity(root, support["integrity_profile"]),
        _catalog(root, support["catalog"]) if arguments.get("with_catalog") else None,
        _bindings(root, support["bindings"]) if arguments.get("with_bindings") else None,
    )
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
    return _success(value)


def _dispatch(responsibility_id: str, root: Path | None, arguments: dict[str, object]):
    try:
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
    except (ValueError, OSError, TypeError):
        return _rejection(responsibility_id)
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
