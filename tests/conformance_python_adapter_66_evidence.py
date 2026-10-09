from __future__ import annotations

from proto_ring.exact_evidence_binding import (
    EvidenceBinding,
    EvidenceRequirement,
    evaluate,
)

_REQUIREMENT_FIELDS = {
    "admitted_classes",
    "subject_hex",
    "context_hex",
    "context_required",
}
_BINDING_FIELDS = {"evidence_class", "subject_hex", "context_hex"}


def _exact_mapping(value: object, fields: set[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        raise TypeError(f"{label} must be an exact record")
    return value


def _optional_hex(value: object, label: str) -> bytes | None:
    if value is None:
        return None
    if not isinstance(value, str):
        raise TypeError(f"{label} must be a string or null")
    return bytes.fromhex(value)


def binding_status(
    requirement_value: object,
    binding_value: object,
) -> str | None:
    requirement_data = _exact_mapping(
        requirement_value, _REQUIREMENT_FIELDS, "requirement"
    )
    classes = requirement_data["admitted_classes"]
    if not isinstance(classes, list) or any(
        not isinstance(item, str) for item in classes
    ):
        raise TypeError("requirement.admitted_classes must be a sequence of strings")
    if len(set(classes)) != len(classes):
        return None
    context_required = requirement_data["context_required"]
    if type(context_required) is not bool:
        raise TypeError("requirement.context_required must be a boolean")
    subject_identity = _optional_hex(
        requirement_data["subject_hex"], "requirement.subject"
    )
    context_identity = _optional_hex(
        requirement_data["context_hex"], "requirement.context"
    )
    try:
        requirement = EvidenceRequirement(
            frozenset(classes),
            subject_identity,
            context_identity,
            context_required,
        )
    except ValueError:
        return None

    binding = None
    if binding_value is not None:
        binding_data = _exact_mapping(binding_value, _BINDING_FIELDS, "binding")
        evidence_class = binding_data["evidence_class"]
        if evidence_class is not None and not isinstance(evidence_class, str):
            raise TypeError("binding.evidence_class must be a string or null")
        binding = EvidenceBinding(
            evidence_class,
            _optional_hex(binding_data["subject_hex"], "binding.subject"),
            _optional_hex(binding_data["context_hex"], "binding.context"),
        )
    return evaluate(requirement, binding).value
