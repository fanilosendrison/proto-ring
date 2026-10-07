from __future__ import annotations

from pathlib import Path
import re
from unittest import mock

from proto_ring import (
    accepted_adr_body,
    adr_metadata,
    canonical_adr,
    governance_bootstrap,
    governance_routing,
    governance_bindings,
    normative_terminology,
    normative_terminology_inventory,
    normative_terminology_matching,
    portable_pattern,
    repository_governance_model,
    structured_data,
)

from conformance_corpus_test_support import observation, transport

OWNED_RESPONSIBILITIES = frozenset({
    "structured-data.document", "structured-data.frontmatter",
    "governance-bootstrap.root", "governance-routing.resolve",
    "repository-governance-model.load",
    "repository-governance-model.binding-compatibility", "adr-metadata.parse",
    "adr-metadata.validation", "adr-metadata.repository-path",
    "portable-pattern.full-match", "canonical-adr.resolve",
    "accepted-adr-body.immutability", "normative-terminology.registry",
    "normative-terminology.markdown",
    "normative-terminology.definition-discovery",
    "normative-terminology.fingerprint", "normative-terminology.inventory",
})


def _rejection(responsibility_id: str):
    return observation("controlled_rejection", {"category": f"{responsibility_id}.rejected"})


def _success(value: object):
    return observation("result", value)


def _route(arguments: dict[str, object], root: Path):
    return governance_routing.resolve_path(
        root, arguments["routing"], tuple(arguments["route"])
    )


def _model_value(model: repository_governance_model.RepositoryGovernanceModel):
    return {
        "model_version": model.model_version,
        "provider_id": model.provider.id,
        "provider_binding_capability": model.provider.binding.capability,
        "provider_binding_route": model.provider.binding.route,
        "capabilities": sorted(model.capabilities),
        "required_routes": {
            capability_id: sorted(capability.routes)
            for capability_id, capability in sorted(model.capabilities.items())
        },
    }


def _binding_registry(root: Path, data: dict[str, object]):
    values = {}
    for item in data["bindings"]:
        scope = governance_bindings.GovernanceBindingScope(
            governance_bindings.ScopeKind(item["scope_kind"]), item.get("capability")
        )
        if item["kind"] == "governance_contract":
            identity = governance_bindings.GovernanceContractIdentity(item["repository"], item["commit"], item["path"])
        else:
            identity = governance_bindings.ExecutableProviderIdentity(item["repository"], item["commit"])
        values[item["id"]] = governance_bindings.GovernanceBinding(
            item["id"], governance_bindings.BindingKind(item["kind"]), scope, identity,
            governance_bindings.BindingAuthority(item["responsibility"], item["source"]),
        )
    return governance_bindings.GovernanceBindingRegistry(root, root / data["carrier"], 1, data["source"], values)


def _portable(arguments: dict[str, object]):
    compiled = portable_pattern.PortablePattern(str(arguments["pattern"]))
    matched = compiled.full_match(str(arguments["candidate"]))
    if matched is None:
        return _success({"matched": False, "captures": []})
    captures = [matched.group(index) for index in range(1, compiled.capture_count + 1)]
    named = {name: matched.group(name) for name in compiled.capture_names}
    return _success({"matched": True, "captures": captures, "named_captures": named})


def _adr_parse(arguments: dict[str, object]):
    operation = arguments["operation"]
    data = bytes.fromhex(str(arguments["bytes_hex"]))
    if operation == "load_yaml":
        with mock.patch.object(Path, "read_bytes", return_value=data):
            value = adr_metadata.load_yaml(Path("fixture.yaml"))
        return _success(value)
    if operation == "parse_adr":
        metadata, body = adr_metadata.parse_adr_bytes(data)
        return _success({"metadata": metadata, "body_hex": body.hex()})
    if operation == "decision_body":
        return _success({"body_hex": adr_metadata.decision_body_bytes(data).hex()})
    if operation == "h1":
        return _success({"text": adr_metadata.h1_text(data)})
    if operation == "preserved_payload":
        return _success({"payload_hex": adr_metadata.preserved_payload_bytes(data).hex()})
    if operation == "sha256":
        return _success({"sha256": adr_metadata.sha256_hex(data)})
    raise ValueError(f"unknown ADR parse operation: {operation}")


def _adr_validation(arguments: dict[str, object]):
    operation = arguments["operation"]
    value = arguments.get("value")
    if operation in {"schema_errors", "schema_error_locations"}:
        errors = adr_metadata.schema_errors(value, arguments["schema"], arguments["overlay_schema"])
        if operation == "schema_error_locations":
            locations = []
            for error in errors:
                match = re.match(r"^(base|overlay) schema (\$[^:]*):", error)
                locations.append({"schema": match.group(1), "path": match.group(2)} if match else {"schema": "schema", "path": "$"})
            return _success({"rejected": bool(errors), "errors": locations})
        return _success({"errors": errors})
    if operation == "require_mapping":
        return _success(adr_metadata.require_mapping(value, "fixture"))
    if operation == "require_string":
        return _success({"text": adr_metadata.require_string(value, "fixture")})
    if operation == "require_string_list":
        return _success({"items": adr_metadata.require_string_list(value, "fixture")})
    if operation in {"relation_target_errors", "relation_target_categories"}:
        errors = adr_metadata.relation_target_errors(
            source_id=arguments["source_id"], relation_type=arguments["relation_type"],
            targets=arguments["targets"], known_ids=arguments["known_ids"],
        )
        if operation == "relation_target_categories":
            categories = []
            for error in errors:
                categories.append({"category": "self_reference"} if "itself" in error else {"category": "missing_target", "target": error.rsplit(" ", 1)[-1]})
            return _success({"errors": categories})
        return _success({"errors": errors})
    raise ValueError(f"unknown ADR validation operation: {operation}")


def _terminology_format(arguments: dict[str, object]):
    value = arguments["format"]
    return normative_terminology.RegistryFormat(
        value["start_marker"], value["end_marker"], tuple(value["headers"]),
        value["key_pattern"], value["anchor_link_pattern"], value["anchor_pattern"],
    )


def _terminology_entry(value: dict[str, object]):
    return normative_terminology.RegistryEntry(
        value["key"], value["canonical_term"], value["anchor"],
        tuple(value["aliases"]), tuple(value["deprecated"]), value["term_structure"],
    )


def _terminology_policy(registry_format, data):
    section_kind = data["section_from_heading"]["kind"]
    def section(heading):
        if section_kind == "identity":
            return heading
        match = re.match(r"^(\d+(?:\.\d+)*[A-Z]?)\b", heading)
        return match.group(1) if match else ""
    location = data["canonical_location"]
    def canonical(entry, block):
        if location["kind"] == "anchor_matches_key":
            return entry.anchor == f"term-{entry.key}"
        return block.section.startswith(location["section_prefix"])
    roles = data["role_policy"]
    def role(occurrence):
        if occurrence.canonical:
            return roles["canonical"]
        for prefix, value in roles["noncanonical_section_prefixes"].items():
            if occurrence.section.startswith(prefix):
                return value
        return roles["other_noncanonical"]
    return normative_terminology.TerminologyPolicy(
        registry_format, section, canonical, role, frozenset(data["allowed_roles"])
    )


def _terminology_error_categories(errors):
    categories = []
    for error in errors:
        if "more than once" in error:
            categories.append("duplicate_registered_anchor")
        elif "not registered" in error:
            categories.append("unregistered_canonical_anchor")
        elif "immediately followed" in error:
            categories.append("intervening_canonical_anchor")
        elif "non-empty canonical destination" in error:
            categories.append("empty_canonical_destination")
        elif "no canonical term" in error:
            categories.append("empty_canonical_term")
        elif "exactly one terminology registry" in error:
            categories.append("registry_marker_cardinality")
        else:
            categories.append("other_controlled_error")
    return categories


def _terminology(responsibility_id: str, arguments: dict[str, object]):
    operation = arguments["operation"]
    if operation == "normalize_text":
        return _success({"text": normative_terminology.normalize_text(arguments["text"])})
    if operation == "fingerprint_text":
        return _success({"sha256": normative_terminology.fingerprint_text(arguments["text"])})
    if operation == "definition_concepts":
        entries = [
            normative_terminology.RegistryEntry(key, expression, f"term-{key}", (), (), "fixture")
            for expression, key in arguments["expressions"].items()
        ]
        block = normative_terminology.MarkdownBlock(1, "", "", arguments["text"])
        concepts = normative_terminology_matching.definition_concepts(block, entries)
        return _success({"concepts": sorted(concepts)})
    registry_format = _terminology_format(arguments)
    if operation == "parse_registry":
        entries, errors = normative_terminology.parse_registry(arguments["markdown"], registry_format)
        if errors:
            return _rejection(responsibility_id)
        return _success({"entries": list(entries), "errors": []})
    if operation == "parse_registry_diagnostics":
        documents = arguments.get("documents", [arguments.get("markdown", "")])
        results = []
        for document in documents:
            entries, errors = normative_terminology.parse_registry(document, registry_format)
            results.append({"entries": len(entries), "categories": _terminology_error_categories(errors)})
        return _success({"variants": results})
    if operation == "occurrence_signatures":
        occurrences = [normative_terminology.Occurrence(tuple(item["concepts"]), item["section"], item["heading"], item["fingerprint"], item["line"], item["canonical"]) for item in arguments["occurrences"]]
        signatures = [item.signature for item in occurrences]
        return _success({"a_equals_b": signatures[0] == signatures[1], "a_equals_c": signatures[0] == signatures[2], "signatures": signatures})
    policy = _terminology_policy(registry_format, arguments["policy"])
    if operation == "parse_blocks":
        blocks, anchors = normative_terminology.parse_blocks(arguments["markdown"], policy)
        return _success({"blocks": list(blocks), "anchors": anchors})
    if operation.startswith("discover_"):
        entries, errors = normative_terminology.parse_registry(arguments["markdown"], registry_format)
        occurrences, discovery_errors = normative_terminology.discover_occurrences(arguments["markdown"], entries, policy)
        all_errors = errors + discovery_errors
        if operation == "discover_occurrences":
            return _success({"occurrences": list(occurrences), "errors": all_errors})
        if operation == "discover_error_categories":
            categories = _terminology_error_categories(all_errors)
            if "category_filter" in arguments:
                return _success({"category": arguments["category_filter"], "present": arguments["category_filter"] in categories})
            return _success({"categories": categories})
        if operation == "discover_location_summary":
            canonical = [item for item in occurrences if item.canonical]
            return _success({"errors": _terminology_error_categories(all_errors), "canonical_count": len(canonical), "canonical_sections": [item.section for item in canonical]})
        query = arguments["query"]
        matches = [item for item in occurrences if item.canonical == query["canonical"] and item.section == query["section"] and set(item.concepts) == set(query["concepts"])]
        selected = None if len(matches) != 1 else {
            "concepts": list(matches[0].concepts), "section": matches[0].section,
            "heading": matches[0].heading, "fingerprint": matches[0].fingerprint,
            "canonical": matches[0].canonical,
        }
        return _success({"count": len(matches), "occurrence": selected, "errors": _terminology_error_categories(all_errors)})
    if operation == "reconcile_inventory":
        entries = [_terminology_entry(value) for value in arguments["entries"]]
        occurrences = [normative_terminology.Occurrence(
            tuple(value["concepts"]), value["section"], value["heading"],
            value["fingerprint"], value["line"], value["canonical"],
        ) for value in arguments["occurrences"]]
        errors = normative_terminology_inventory.reconcile_inventory(
            entries, occurrences, arguments["inventory"], policy
        )
        return _success({"status": "PASS" if not errors else "NON_PASS"})
    raise ValueError(f"unknown terminology operation for {responsibility_id}: {operation}")


def _dispatch(responsibility_id: str, root: Path | None, arguments: dict[str, object]):
    try:
        if responsibility_id == "structured-data.document":
            return _success(structured_data.parse_document_bytes(bytes.fromhex(arguments["bytes_hex"])))
        if responsibility_id == "structured-data.frontmatter":
            data = bytes.fromhex(arguments["bytes_hex"])
            if arguments["operation"] == "immediate_close_boundaries":
                try:
                    structured_data.parse_frontmatter_bytes(data)
                except structured_data.StructuredDataError as error:
                    structured = error.args == ("structured frontmatter must not be empty",)
                else:
                    structured = False
                try:
                    governance_bootstrap.load(root)
                except governance_bootstrap.GovernanceBootstrapError as error:
                    bootstrap = isinstance(error.__cause__, structured_data.StructuredDataError)
                else:
                    bootstrap = False
                return _success({
                    "immediate_close_recognized": structured,
                    "frontmatter_payload": "EMPTY",
                    "structured_data_boundary": "CONTROLLED_REJECTION" if structured else "NOT_REJECTED",
                    "governance_bootstrap_boundary": "CONTROLLED_REJECTION" if bootstrap else "NOT_REJECTED",
                    "bootstrap_cause_boundary": "STRUCTURED_DATA" if bootstrap else "NONE",
                    "governance_body_interpreted": False,
                })
            parsed = structured_data.parse_frontmatter_bytes(data)
            return _success({"metadata": parsed.metadata, "body_hex": parsed.body.hex()})
        if responsibility_id == "governance-bootstrap.root":
            loaded = governance_bootstrap.load(root)
            return _success({"metadata": loaded.metadata, "repository_governance": loaded.repository_governance})
        if responsibility_id == "governance-routing.resolve":
            resolved = _route(arguments, root)
            return _success({"declared_path": resolved.declared_path, "route": list(resolved.route), "target_relative": resolved.target.relative_to(root.resolve()).as_posix()})
        if responsibility_id == "repository-governance-model.load":
            return _success(_model_value(repository_governance_model.load(root)))
        if responsibility_id == "repository-governance-model.binding-compatibility":
            model = repository_governance_model.load(root)
            registry = _binding_registry(root, arguments["semantic_inputs"]["binding_registry"])
            repository_governance_model.validate_binding_capabilities(model, registry)
            return _success({"compatible": True})
        if responsibility_id == "adr-metadata.parse":
            return _adr_parse(arguments)
        if responsibility_id == "adr-metadata.validation":
            return _adr_validation(arguments)
        if responsibility_id == "adr-metadata.repository-path":
            path = adr_metadata.repository_path(root, arguments["relative"])
            return _success({"relative": path.relative_to(root.resolve()).as_posix()})
        if responsibility_id == "portable-pattern.full-match":
            return _portable(arguments)
        if responsibility_id == "canonical-adr.resolve":
            value = canonical_adr.resolve(root, arguments["adr_id"])
            result = {"adr_id": value.id, "relative_path": value.path.resolve().relative_to(root.resolve()).as_posix()}
            if arguments.get("include_decision_body"):
                result["decision_body"] = adr_metadata.parse_adr(value.path)[1]
            return _success(result)
        if responsibility_id == "accepted-adr-body.immutability":
            errors = accepted_adr_body.check(root)
            category = None
            if errors:
                text = "\n".join(errors)
                category = "body_changed" if "body changed" in text else "historical_state_invalid"
            return _success({"status": "PASS" if not errors else "NON_PASS", "category": category})
        if responsibility_id.startswith("normative-terminology."):
            return _terminology(responsibility_id, arguments)
    except (ValueError, OSError) as error:
        return _rejection(responsibility_id)
    raise ValueError(f"unsupported responsibility: {responsibility_id}")


def execute(responsibility_id, fixture):
    if responsibility_id not in OWNED_RESPONSIBILITIES:
        raise ValueError(f"responsibility is not owned by adapter 63/64: {responsibility_id}")
    return fixture.run(lambda root, arguments: _dispatch(responsibility_id, root, arguments))
