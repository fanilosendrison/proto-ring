from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path

from conformance_corpus_test_support import decode_transport
from conformance_issue66_metadata_audit import (
    ISSUE66_RESPONSIBILITIES,
    audit_review3_metadata,
)


def _exact_keys(value: object, required: set[str], optional: set[str] = set()) -> None:
    if not isinstance(value, dict):
        raise AssertionError("fixed record must decode to a mapping")
    actual = set(value)
    if not required <= actual or actual - required - optional:
        raise AssertionError(
            f"fixed record keys differ: required={sorted(required)} actual={sorted(actual)}"
        )


def _strings(value: object) -> None:
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise AssertionError("expected a sequence of strings")


def _raw_transport(value: object) -> None:
    if not isinstance(value, dict) or set(value) != {"type", "value"}:
        raise AssertionError("TransportValue must have exact type/value fields")
    kind = value["type"]
    payload = value["value"]
    if kind == "record":
        if not isinstance(payload, list):
            raise AssertionError("record payload must be a sequence")
        names = []
        for entry in payload:
            if not isinstance(entry, dict) or set(entry) != {"name", "value"}:
                raise AssertionError("RecordEntry must have exact name/value fields")
            if not isinstance(entry["name"], str):
                raise AssertionError("RecordEntry name must be a string")
            names.append(entry["name"])
            _raw_transport(entry["value"])
        if len(names) != len(set(names)):
            raise AssertionError("duplicate RecordEntry name")
    elif kind == "sequence":
        if not isinstance(payload, list):
            raise AssertionError("sequence payload must be a sequence")
        for item in payload:
            _raw_transport(item)
    elif kind == "string":
        if not isinstance(payload, str):
            raise AssertionError("string payload has incorrect kind")
    elif kind == "integer":
        if not isinstance(payload, str):
            raise AssertionError("integer payload has incorrect kind")
    elif kind == "boolean":
        if type(payload) is not bool:
            raise AssertionError("boolean payload has incorrect kind")
    elif kind == "bytes":
        if not isinstance(payload, str):
            raise AssertionError("bytes payload has incorrect kind")
    elif kind == "null":
        if payload is not None:
            raise AssertionError("null payload must be null")
    else:
        raise AssertionError(f"unknown TransportValue kind: {kind}")


def _audit_authority(value: object) -> None:
    _exact_keys(value, {"carrier", "sources", "responsibilities"})
    if not isinstance(value["carrier"], str):
        raise AssertionError("authority carrier must be a string")
    if not isinstance(value["sources"], dict):
        raise AssertionError("sources must be a dynamic record")
    for source_id, source in value["sources"].items():
        if not isinstance(source_id, str):
            raise AssertionError("source ID must be a string")
        _exact_keys(source, {"repository_target"})
        if source["repository_target"] is not None and not isinstance(
            source["repository_target"], str
        ):
            raise AssertionError("repository_target must be string or null")
    if not isinstance(value["responsibilities"], dict):
        raise AssertionError("responsibilities must be a dynamic record")
    for responsibility_id, responsibility in value["responsibilities"].items():
        if not isinstance(responsibility_id, str):
            raise AssertionError("responsibility ID must be a string")
        _exact_keys(responsibility, {"roles", "precedence"})
        if not isinstance(responsibility["roles"], dict):
            raise AssertionError("roles must be a dynamic record")
        if any(not isinstance(key, str) or not isinstance(role, str)
               for key, role in responsibility["roles"].items()):
            raise AssertionError("roles must map exact source IDs to strings")
        if not isinstance(responsibility["precedence"], list):
            raise AssertionError("precedence must be a sequence")
        for edge in responsibility["precedence"]:
            _exact_keys(edge, {"higher_source", "lower_source"})
            if not all(isinstance(edge[key], str) for key in edge):
                raise AssertionError("precedence endpoints must be strings")


def _audit_catalog(value: object) -> None:
    _exact_keys(value, {"carrier", "interfaces"})
    if not isinstance(value["carrier"], str) or not isinstance(value["interfaces"], dict):
        raise AssertionError("catalog support shape is malformed")
    for interface_id, objects in value["interfaces"].items():
        if not isinstance(interface_id, str) or not isinstance(objects, dict):
            raise AssertionError("catalog interfaces are dynamic records")
        for object_id, governed_object in objects.items():
            if not isinstance(object_id, str):
                raise AssertionError("object ID must be a string")
            _exact_keys(governed_object, {"responsibilities"})
            _strings(governed_object["responsibilities"])


def _audit_binding_support(value: object) -> None:
    _exact_keys(value, {"carrier", "source", "bindings"})
    if not isinstance(value["carrier"], str) or not isinstance(value["source"], str):
        raise AssertionError("binding support carrier/source must be strings")
    if not isinstance(value["bindings"], list):
        raise AssertionError("binding support declarations must be a sequence")
    for binding in value["bindings"]:
        required = {
            "id", "kind", "scope_kind", "repository", "commit",
            "responsibility", "source",
        }
        optional = {"capability", "path"}
        _exact_keys(binding, required, optional)
        if any(not isinstance(binding[key], str) for key in binding):
            raise AssertionError("binding support fields must be strings")


def _audit_integrity_support(value: object) -> None:
    _exact_keys(
        value,
        {
            "authority", "carrier", "continue_after_non_satisfied",
            "environments", "identity", "order", "validations",
        },
    )
    _exact_keys(value["authority"], {"responsibility", "source"})
    _strings(value["environments"])
    _strings(value["order"])
    if type(value["continue_after_non_satisfied"]) is not bool:
        raise AssertionError("continuation flag must be boolean")
    if not isinstance(value["validations"], list):
        raise AssertionError("support validations must be a sequence")
    for validation in value["validations"]:
        _exact_keys(
            validation,
            {
                "arguments", "environment", "id", "prerequisites",
                "responsibility", "undetermined_exit_codes",
            },
        )
        _strings(validation["arguments"])
        _strings(validation["prerequisites"])
        if not isinstance(validation["undetermined_exit_codes"], list) or any(
            type(code) is not int for code in validation["undetermined_exit_codes"]
        ):
            raise AssertionError("undetermined exit codes must be integers")


def _audit_semantic_inputs(value: object) -> None:
    _exact_keys(value, {"authority", "bindings", "catalog", "integrity_profile"})
    _audit_authority(value["authority"])
    _audit_binding_support(value["bindings"])
    _audit_catalog(value["catalog"])
    _audit_integrity_support(value["integrity_profile"])


def _audit_requirement(value: object) -> None:
    _exact_keys(
        value,
        {"admitted_classes", "subject_hex", "context_hex", "context_required"},
    )
    _strings(value["admitted_classes"])
    if value["subject_hex"] is not None and not isinstance(value["subject_hex"], str):
        raise AssertionError("subject token must be string or null")
    if value["context_hex"] is not None and not isinstance(value["context_hex"], str):
        raise AssertionError("context token must be string or null")
    if type(value["context_required"]) is not bool:
        raise AssertionError("context_required must be boolean")


def _audit_binding(value: object) -> None:
    if value is None:
        return
    _exact_keys(value, {"evidence_class", "subject_hex", "context_hex"})
    for key in value:
        if value[key] is not None and not isinstance(value[key], str):
            raise AssertionError("candidate binding fields must be string or null")


def _audit_consumer_resolution(value: object) -> None:
    _exact_keys(value, {"subject", "context", "candidate"})
    _exact_keys(value["subject"], {"known", "identity_hex"})
    _exact_keys(value["context"], {"known", "identity_hex", "source_id"})
    _exact_keys(value["candidate"], {"evidence_class", "subject_hex", "context_hex"})


def _audit_arguments(responsibility_id: str, arguments: object) -> None:
    if responsibility_id == "exact-evidence-binding.evaluate":
        operation = arguments.get("operation") if isinstance(arguments, dict) else None
        if operation == "evaluate_binding":
            _exact_keys(arguments, {"operation", "requirement", "binding"})
            _audit_requirement(arguments["requirement"])
            _audit_binding(arguments["binding"])
        elif operation == "evaluate_requirement_history":
            _exact_keys(arguments, {"operation", "comparisons"})
            if not isinstance(arguments["comparisons"], list):
                raise AssertionError("comparisons must be a sequence")
            for comparison in arguments["comparisons"]:
                _exact_keys(
                    comparison,
                    {"label", "historical_requirement", "current_requirement", "binding"},
                )
                _audit_requirement(comparison["historical_requirement"])
                _audit_requirement(comparison["current_requirement"])
                _audit_binding(comparison["binding"])
        else:
            raise AssertionError("unknown exact-binding operation")
        return
    common = {"operation", "path", "semantic_inputs", "with_catalog"}
    optional = {
        "identity_only", "observe_binding_id", "observe_execution_sentinel",
        "observe_projection_id", "observe_read_only", "observe_registry_authority",
        "observe_requirement_id", "observe_validation_id", "observe_validation_ids",
        "qualified_observation", "requirement_id",
        "consumer_resolution", "foreign_support_repository", "with_bindings",
    }
    required = set(common)
    if responsibility_id == "projection-registry.registry":
        required.add("with_bindings")
    _exact_keys(arguments, required, optional)
    if not isinstance(arguments["operation"], str) or not isinstance(arguments["path"], str):
        raise AssertionError("operation and path must be strings")
    if type(arguments["with_catalog"]) is not bool:
        raise AssertionError("with_catalog must be boolean")
    if "with_bindings" in arguments and type(arguments["with_bindings"]) is not bool:
        raise AssertionError("with_bindings must be boolean")
    if "foreign_support_repository" in arguments:
        selection = arguments["foreign_support_repository"]
        valid = (
            responsibility_id == "projection-registry.registry"
            and selection in {"integrity_profile", "catalog", "bindings"}
        ) or (
            responsibility_id in {
                "evidence-requirements.registry",
                "governance-bindings.registry",
                "repository-integrity.profile",
            }
            and selection == "catalog"
        )
        if not valid:
            raise AssertionError("invalid foreign support transport selection")
    for key in optional & set(arguments) - {
        "requirement_id", "consumer_resolution", "foreign_support_repository"
    }:
        if key == "observe_validation_ids":
            _strings(arguments[key])
        elif key.startswith("observe_") and key not in {
            "observe_read_only", "observe_execution_sentinel", "observe_registry_authority"
        }:
            if not isinstance(arguments[key], str):
                raise AssertionError(f"{key} must be a string")
        elif key in {
            "identity_only", "observe_read_only", "observe_execution_sentinel",
            "observe_registry_authority", "qualified_observation",
        }:
            if type(arguments[key]) is not bool:
                raise AssertionError(f"{key} must be boolean")
    _audit_semantic_inputs(arguments["semantic_inputs"])
    if "consumer_resolution" in arguments:
        if not isinstance(arguments.get("requirement_id"), str):
            raise AssertionError("requirement_id must be a string")
        _audit_consumer_resolution(arguments["consumer_resolution"])


def _raw_arguments(fixture: dict[str, object]) -> Iterable[dict[str, object]]:
    if "input" in fixture:
        yield fixture["input"]
    if "invocation" in fixture:
        yield fixture["invocation"]
    for step in fixture.get("steps", []):
        if step.get("op") == "invoke":
            yield step["arguments"]


def audit_raw_support_shapes(cases: dict[str, dict[str, object]]) -> tuple[int, list[str]]:
    audited = 0
    errors: list[str] = []
    for responsibility_id in sorted(ISSUE66_RESPONSIBILITIES):
        for vector in cases[responsibility_id]["vectors"]:
            try:
                for raw in _raw_arguments(vector["fixture"]):
                    _raw_transport(raw)
                    _audit_arguments(responsibility_id, decode_transport(raw))
                audited += 1
            except (AssertionError, KeyError, TypeError, ValueError) as error:
                errors.append(f"{vector['vector_id']}: {error}")
    return audited, errors


def require_raw_support_shapes(cases: dict[str, dict[str, object]]) -> None:
    audited, errors = audit_raw_support_shapes(cases)
    expected = issue66_vector_count(cases)
    if audited != expected or errors:
        raise AssertionError(
            "\n".join(errors) or f"audited only {audited}/{expected} vectors"
        )


def audit_issue66_metadata(
    responsibilities: dict[str, dict[str, object]],
    cases: dict[str, dict[str, object]],
    coverage: list[dict[str, object]],
) -> dict[str, int | str]:
    review3 = audit_review3_metadata(responsibilities, coverage, cases)
    uncovered = undeclared = 0
    for responsibility_id in ISSUE66_RESPONSIBILITIES:
        required = set(responsibilities[responsibility_id]["required_state_distinctions"])
        represented = {
            distinction
            for vector in cases[responsibility_id]["vectors"]
            for distinction in vector["state_distinctions"]
        }
        uncovered += len(required - represented)
        undeclared += len(represented - required)
    all_vector_ids = {
        vector["vector_id"]
        for matrix in cases.values()
        for vector in matrix["vectors"]
    }
    dangling = sum(
        vector_id not in all_vector_ids
        for record in coverage
        for heading in record["headings"]
        for vector_id in heading["vector_refs"]
    )
    return {
        **review3,
        "required_uncovered": uncovered,
        "vector_undeclared": undeclared,
        "coverage_dangling": dangling,
    }


def issue66_vector_count(cases: dict[str, dict[str, object]]) -> int:
    return sum(len(cases[item]["vectors"]) for item in ISSUE66_RESPONSIBILITIES)
