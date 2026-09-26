"""Consumer-independent Architecture Decision Record metadata primitives.

This module owns byte-exact parsing, safe data loading, schema-validation,
repository-containment, and relation-reference mechanisms. It owns no consumer
profile, ADR corpus, migration policy, history, rendering policy, or generated
artifact.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
from typing import Collection, Iterable, Mapping, cast

import yaml
from jsonschema import Draft202012Validator, FormatChecker
from jsonschema.exceptions import SchemaError

__all__ = [
    "AdrMetadataError",
    "decision_body_bytes",
    "h1_text",
    "load_json",
    "load_yaml",
    "parse_adr",
    "preserved_payload_bytes",
    "relation_target_errors",
    "repository_path",
    "require_mapping",
    "require_string",
    "require_string_list",
    "schema_errors",
    "sha256_hex",
]

_BODY_BOUNDARY = re.compile(br"(?m)^## Context(?: |$)")
_H1_BOUNDARY = re.compile(br"(?m)^# .+$")


class AdrMetadataError(ValueError):
    """Report a controlled failure to determine valid ADR metadata."""


def load_yaml(path: Path) -> object:
    """Load one UTF-8 YAML document without enabling object constructors."""

    try:
        return yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, yaml.YAMLError) as error:
        raise AdrMetadataError(f"cannot read YAML {path}: {error}") from error


def load_json(path: Path) -> object:
    """Load one UTF-8 JSON document with controlled parse and I/O failures."""

    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise AdrMetadataError(f"cannot read JSON {path}: {error}") from error


def sha256_hex(data: bytes) -> str:
    """Return lowercase SHA-256 for exact input bytes."""

    return hashlib.sha256(data).hexdigest()


def _decode_exact_utf8_lf(data: bytes) -> str:
    if data.startswith(b"\xef\xbb\xbf"):
        raise AdrMetadataError("UTF-8 BOM is forbidden")
    if b"\r" in data:
        raise AdrMetadataError("CRLF or bare CR is forbidden; ADRs must use LF")
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise AdrMetadataError(f"ADR is not valid UTF-8: {error}") from error


def decision_body_bytes(data: bytes) -> bytes:
    """Return exact bytes from the unique ``## Context`` boundary through EOF."""

    _decode_exact_utf8_lf(data)
    boundaries = list(_BODY_BOUNDARY.finditer(data))
    if len(boundaries) != 1:
        raise AdrMetadataError(
            "ADR must contain exactly one line beginning with '## Context'"
        )
    return data[boundaries[0].start() :]


def h1_text(data: bytes) -> str:
    """Return the text of the unique level-one heading in exact UTF-8/LF data."""

    _decode_exact_utf8_lf(data)
    headings = list(_H1_BOUNDARY.finditer(data))
    if len(headings) != 1:
        raise AdrMetadataError("ADR must contain exactly one H1 heading")
    return headings[0].group()[2:].decode("utf-8")


def preserved_payload_bytes(data: bytes) -> bytes:
    """Return exact bytes from the unique H1 through EOF after ADR validation."""

    decision_body_bytes(data)
    headings = list(_H1_BOUNDARY.finditer(data))
    if len(headings) != 1:
        raise AdrMetadataError("ADR must contain exactly one H1 heading")
    return data[headings[0].start() :]


def parse_adr(path: Path) -> tuple[dict[str, object], bytes]:
    """Parse safe YAML frontmatter and return the exact decision body bytes."""

    try:
        data = path.read_bytes()
    except OSError as error:
        raise AdrMetadataError(f"cannot read {path}: {error}") from error

    body = decision_body_bytes(data)
    if not data.startswith(b"---\n"):
        raise AdrMetadataError("ADR has no YAML frontmatter")

    closing = data.find(b"\n---\n", 4)
    if closing < 0:
        raise AdrMetadataError("ADR frontmatter has no closing delimiter")
    try:
        metadata = yaml.safe_load(data[4:closing].decode("utf-8"))
    except (UnicodeDecodeError, yaml.YAMLError) as error:
        raise AdrMetadataError(f"invalid ADR frontmatter: {error}") from error
    if not isinstance(metadata, dict):
        raise AdrMetadataError("ADR frontmatter must be a mapping")
    return cast(dict[str, object], metadata), body


def schema_errors(
    metadata: Mapping[str, object],
    base_schema: Mapping[str, object],
    overlay_schema: Mapping[str, object],
) -> list[str]:
    """Validate metadata independently against labeled base and overlay schemas."""

    errors: list[str] = []
    checker = FormatChecker()
    for label, schema in (("base", base_schema), ("overlay", overlay_schema)):
        try:
            Draft202012Validator.check_schema(schema)
            validator = Draft202012Validator(schema, format_checker=checker)
        except SchemaError as error:
            errors.append(f"{label} schema is invalid: {error.message}")
            continue
        for error in sorted(
            validator.iter_errors(metadata),
            key=lambda item: item.json_path,
        ):
            errors.append(f"{label} schema {error.json_path}: {error.message}")
    return errors


def repository_path(root: Path, relative: object) -> Path:
    """Resolve a non-empty relative path and require containment in ``root``."""

    if not isinstance(relative, str) or not relative:
        raise AdrMetadataError("repository path must be a non-empty string")
    candidate = Path(relative)
    if candidate.is_absolute():
        raise AdrMetadataError(f"repository path must be relative: {relative}")
    resolved_root = root.resolve()
    resolved = (resolved_root / candidate).resolve()
    if not resolved.is_relative_to(resolved_root):
        raise AdrMetadataError(f"repository path escapes root: {relative}")
    return resolved


def require_mapping(value: object, label: str) -> dict[str, object]:
    """Require a mapping for consumer-owned structured configuration."""

    if not isinstance(value, dict):
        raise AdrMetadataError(f"{label} must be a mapping")
    return cast(dict[str, object], value)


def require_string(value: object, label: str) -> str:
    """Require a non-empty string for consumer-owned configuration."""

    if not isinstance(value, str) or not value:
        raise AdrMetadataError(f"{label} must be a non-empty string")
    return value


def require_string_list(value: object, label: str) -> list[str]:
    """Require a duplicate-free list of non-empty strings."""

    if not isinstance(value, list) or any(
        not isinstance(item, str) or not item for item in value
    ):
        raise AdrMetadataError(f"{label} must be a list of non-empty strings")
    strings = cast(list[str], value)
    if len(strings) != len(set(strings)):
        raise AdrMetadataError(f"{label} contains duplicates")
    return strings


def relation_target_errors(
    *,
    source_id: str,
    relation_type: str,
    targets: Iterable[str],
    known_ids: Collection[str],
) -> list[str]:
    """Check self- and missing-target references without owning relation types."""

    errors: list[str] = []
    for target in targets:
        if target == source_id:
            errors.append(f"{relation_type} cannot reference itself")
        elif target not in known_ids:
            errors.append(f"{relation_type} references missing {target}")
    return errors
