"""Role binding and review-inventory reconciliation for terminology governance."""

from __future__ import annotations

from collections import Counter
from collections.abc import Mapping
import re

from .normative_terminology import Occurrence, RegistryEntry, TerminologyPolicy


def _resolved_role(occurrence: Occurrence, policy: TerminologyPolicy) -> str:
    role = policy.role_for(occurrence)
    if not isinstance(role, str) or role not in policy.allowed_roles:
        raise ValueError(f"consumer role policy returned an unallowed role: {role!r}")
    return role


def occurrence_record(
    occurrence: Occurrence, policy: TerminologyPolicy
) -> dict[str, object]:
    return {
        "concepts": list(occurrence.concepts),
        "section": occurrence.section,
        "heading": occurrence.heading,
        "role": _resolved_role(occurrence, policy),
        "fingerprint": occurrence.fingerprint,
    }


def reconcile_inventory(
    entries: list[RegistryEntry],
    occurrences: list[Occurrence],
    inventory_occurrences: object,
    policy: TerminologyPolicy,
) -> list[str]:
    if not isinstance(inventory_occurrences, list):
        return ["terminology inventory occurrences must be a list"]

    errors: list[str] = []
    known_keys = {entry.key for entry in entries}
    actual = Counter(occurrence.signature for occurrence in occurrences)
    recorded: Counter[tuple[tuple[str, ...], str, str, str]] = Counter()
    expected_roles: dict[
        tuple[tuple[str, ...], str, str, str], Counter[str]
    ] = {}
    recorded_roles: dict[
        tuple[tuple[str, ...], str, str, str], Counter[str]
    ] = {}
    for occurrence in occurrences:
        try:
            role = _resolved_role(occurrence, policy)
        except ValueError as error:
            errors.append(str(error))
        else:
            expected_roles.setdefault(occurrence.signature, Counter())[role] += 1

    for index, item in enumerate(inventory_occurrences, start=1):
        label = f"terminology inventory occurrence {index}"
        if not isinstance(item, Mapping):
            errors.append(f"{label} must be a mapping")
            continue
        concepts = item.get("concepts")
        valid_concepts = (
            isinstance(concepts, list)
            and bool(concepts)
            and all(isinstance(value, str) for value in concepts)
        )
        if not valid_concepts:
            errors.append(f"{label} must contain a non-empty concepts list")
            continue
        assert isinstance(concepts, list)
        concepts_tuple = tuple(sorted(concepts))
        unknown = sorted(set(concepts_tuple) - known_keys)
        if unknown:
            errors.append(f"{label} references unknown concepts: {', '.join(unknown)}")

        section, heading = item.get("section"), item.get("heading")
        if not isinstance(section, str):
            errors.append(f"{label} must contain a string section")
            continue
        if not isinstance(heading, str):
            errors.append(f"{label} must contain a string heading")
            continue
        role = item.get("role")
        role_valid = isinstance(role, str) and role in policy.allowed_roles
        if not role_valid:
            errors.append(f"{label} has invalid role: {role!r}")
        fingerprint = item.get("fingerprint")
        if not isinstance(fingerprint, str) or not re.fullmatch(
            r"[0-9a-f]{64}", fingerprint
        ):
            errors.append(f"{label} has an invalid SHA-256 fingerprint")
            continue

        signature = (concepts_tuple, section, heading, fingerprint)
        recorded[signature] += 1
        if recorded[signature] > max(actual[signature], 1):
            errors.append(f"{label} duplicates an earlier occurrence")
        if role_valid:
            assert isinstance(role, str)
            recorded_roles.setdefault(signature, Counter())[role] += 1

    for signature in sorted(actual.keys() & recorded.keys()):
        if actual[signature] != recorded[signature]:
            continue
        expected = expected_roles.get(signature, Counter())
        observed = recorded_roles.get(signature, Counter())
        if expected == observed:
            continue
        expected_values = sorted(expected.elements())
        observed_values = sorted(observed.elements())
        concepts, section, heading, _fingerprint = signature
        location = f"{', '.join(concepts)} in section {section} ({heading})"
        if len(expected_values) == 1 and len(observed_values) == 1:
            errors.append(
                f"terminology inventory occurrence for {location} must use role "
                f"{expected_values[0]!r}, not {observed_values[0]!r}"
            )
        else:
            errors.append(
                f"terminology inventory roles for {location} do not match policy: "
                f"expected {expected_values}, recorded {observed_values}"
            )

    for signature, count in (actual - recorded).items():
        concepts, section, heading, _fingerprint = signature
        lines = [str(item.line) for item in occurrences if item.signature == signature]
        errors.append(
            f"unreviewed definition-like occurrence for {', '.join(concepts)} "
            f"in section {section} ({heading}) near candidate lines {', '.join(lines)}; "
            f"unmatched copies: {count}"
        )
    for signature, count in (recorded - actual).items():
        concepts, section, heading, _fingerprint = signature
        errors.append(
            f"stale terminology inventory entry for {', '.join(concepts)} "
            f"in section {section} ({heading}); unmatched copies: {count}"
        )
    return errors
