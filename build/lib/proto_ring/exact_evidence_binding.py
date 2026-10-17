"""Evaluate opaque evidence bindings against consumer-supplied requirements."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

__all__ = [
    "BindingStatus",
    "EvidenceRequirement",
    "EvidenceBinding",
    "evaluate",
]


class BindingStatus(str, Enum):
    MATCH = "MATCH"
    MISMATCH = "MISMATCH"
    UNDETERMINED = "UNDETERMINED"


@dataclass(frozen=True)
class EvidenceRequirement:
    admitted_classes: frozenset[str]
    subject_identity: bytes | None
    context_identity: bytes | None = None
    context_required: bool = False

    def __post_init__(self) -> None:
        if not isinstance(self.admitted_classes, frozenset):
            raise ValueError("admitted_classes must be a frozenset")
        if not self.admitted_classes:
            raise ValueError("admitted_classes must not be empty")
        if any(
            not isinstance(evidence_class, str)
            for evidence_class in self.admitted_classes
        ):
            raise ValueError("admitted_classes members must be strings")
        if any(evidence_class == "" for evidence_class in self.admitted_classes):
            raise ValueError("admitted_classes members must not be empty")

        if self.subject_identity is not None and (
            not isinstance(self.subject_identity, bytes)
            or self.subject_identity == b""
        ):
            raise ValueError("subject_identity must be non-empty bytes or None")

        if not isinstance(self.context_required, bool):
            raise ValueError("context_required must be a bool")
        if self.context_identity is not None and (
            not isinstance(self.context_identity, bytes)
            or self.context_identity == b""
        ):
            raise ValueError("context_identity must be non-empty bytes or None")
        if not self.context_required and self.context_identity is not None:
            raise ValueError(
                "context_identity must be None when context_required is False"
            )


@dataclass(frozen=True)
class EvidenceBinding:
    evidence_class: str | None
    subject_identity: bytes | None
    context_identity: bytes | None = None


def evaluate(
    requirement: EvidenceRequirement,
    evidence: EvidenceBinding | None,
) -> BindingStatus:
    if requirement.subject_identity is None:
        return BindingStatus.UNDETERMINED
    if requirement.context_required and requirement.context_identity is None:
        return BindingStatus.UNDETERMINED

    if evidence is None:
        return BindingStatus.UNDETERMINED
    if not isinstance(evidence.evidence_class, str) or evidence.evidence_class == "":
        return BindingStatus.UNDETERMINED
    if (
        not isinstance(evidence.subject_identity, bytes)
        or evidence.subject_identity == b""
    ):
        return BindingStatus.UNDETERMINED
    if requirement.context_required and (
        not isinstance(evidence.context_identity, bytes)
        or evidence.context_identity == b""
    ):
        return BindingStatus.UNDETERMINED

    if evidence.evidence_class not in requirement.admitted_classes:
        return BindingStatus.MISMATCH
    if evidence.subject_identity != requirement.subject_identity:
        return BindingStatus.MISMATCH
    if (
        requirement.context_required
        and evidence.context_identity != requirement.context_identity
    ):
        return BindingStatus.MISMATCH

    return BindingStatus.MATCH
