"""Verify a consumer's structured Shared Governance Provider authority chain."""

from __future__ import annotations

from pathlib import Path
import re
from typing import cast

import yaml

from proto_ring import adr_metadata, canonical_adr

__all__ = [
    "check",
]

_AGENT_DIRECTIVES_PATH = "AGENTS.md"
_PROVIDER_REPOSITORY = "fanilosendrison/proto-ring"
_CONTRACT_PATH = "docs/contracts/shared-governance-provider.md"
_COMMIT_PATTERN = re.compile(r"[0-9a-f]{40}")
_SHA256_PATTERN = re.compile(r"[0-9a-f]{64}")
_IDENTITY_PATTERN = re.compile(
    rf"(?m)^{re.escape(_PROVIDER_REPOSITORY)}\n([^\n]*)\n"
    rf"{re.escape(_CONTRACT_PATH)}$"
)


def _load_frontmatter(
    path: Path,
    label: str,
) -> tuple[dict[str, object] | None, str | None, list[str]]:
    try:
        data = path.read_bytes()
    except OSError as error:
        return None, None, [f"cannot read {label}: {error}"]
    if data.startswith(b"\xef\xbb\xbf"):
        return None, None, [f"{label} contains a UTF-8 BOM"]
    if b"\r" in data:
        return None, None, [f"{label} contains CR or CRLF line endings"]
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        return None, None, [f"{label} is not valid UTF-8: {error}"]
    if not text.startswith("---\n"):
        return None, None, [f"{label} has no YAML frontmatter"]
    closing = text.find("\n---\n", 4)
    if closing < 0:
        return None, None, [f"{label} frontmatter has no closing delimiter"]
    try:
        metadata = yaml.safe_load(text[4:closing])
    except yaml.YAMLError as error:
        return None, None, [f"{label} frontmatter is invalid YAML: {error}"]
    if not isinstance(metadata, dict):
        return None, None, [f"{label} frontmatter must be a mapping"]
    return cast(dict[str, object], metadata), text[closing + 5 :], []


def _mapping(
    parent: dict[str, object],
    key: str,
    label: str,
    errors: list[str],
) -> dict[str, object] | None:
    value = parent.get(key)
    if not isinstance(value, dict):
        errors.append(f"{label} must be a mapping")
        return None
    return cast(dict[str, object], value)


def _repository_path(
    repository: Path,
    relative: object,
    label: str,
    errors: list[str],
) -> Path | None:
    try:
        return adr_metadata.repository_path(repository, relative)
    except (adr_metadata.AdrMetadataError, OSError, RuntimeError) as error:
        errors.append(f"{label}: {error}")
        return None


def _identity(
    body: str,
    label: str,
    errors: list[str],
) -> str | None:
    identities = _IDENTITY_PATTERN.findall(body)
    if not identities:
        errors.append(f"{label} contains no Shared Governance Provider identity block")
        return None
    if len(identities) > 1:
        errors.append(f"{label} contains multiple Shared Governance Provider identities")
        return None
    identity = identities[0]
    if not _COMMIT_PATTERN.fullmatch(identity):
        errors.append(
            f"{label} contract identity must be exactly 40 lowercase hexadecimal characters"
        )
        return None
    return identity


def check(repository: Path) -> list[str]:
    """Return controlled errors for an invalid structured authority chain."""

    errors: list[str] = []
    agent_path = _repository_path(
        repository, _AGENT_DIRECTIVES_PATH, "agent directives path", errors
    )
    if agent_path is None:
        return errors
    agent_metadata, _agent_body, agent_errors = _load_frontmatter(
        agent_path, "agent directives"
    )
    errors.extend(agent_errors)
    if agent_metadata is None:
        return errors

    governance = _mapping(
        agent_metadata, "repository_governance", "repository_governance", errors
    )
    if governance is None:
        return errors
    routing = _mapping(
        governance,
        "shared_governance_provider",
        "repository_governance.shared_governance_provider",
        errors,
    )
    if routing is None:
        return errors
    if routing.get("required") is not True:
        errors.append("shared governance provider routing required must be exactly true")
    binding_path = _repository_path(
        repository, routing.get("binding_path"), "binding_path", errors
    )
    if binding_path is None:
        return errors

    binding_metadata, binding_body, binding_errors = _load_frontmatter(
        binding_path, "binding"
    )
    errors.extend(binding_errors)
    if binding_metadata is None or binding_body is None:
        return errors

    provider = _mapping(
        binding_metadata,
        "shared_governance_provider",
        "shared_governance_provider",
        errors,
    )
    if provider is None:
        return errors
    if provider.get("mandatory") is not True:
        errors.append("shared_governance_provider.mandatory must be exactly true")

    authority = _mapping(
        provider, "authority_adr", "shared_governance_provider.authority_adr", errors
    )
    contract = _mapping(
        provider, "contract", "shared_governance_provider.contract", errors
    )
    if authority is None or contract is None:
        return errors

    authority_id = authority.get("id")
    if not isinstance(authority_id, str) or not authority_id:
        errors.append("authority_adr.id must be a non-empty string")
    if "path" in authority:
        errors.append(
            "authority_adr.path must not be declared; canonical ADR paths are derived"
        )

    contract_repository = contract.get("repository")
    contract_commit = contract.get("commit")
    contract_path = contract.get("path")
    if contract_repository != _PROVIDER_REPOSITORY:
        errors.append("binding contract repository is not canonical")
    if not isinstance(contract_commit, str) or not _COMMIT_PATTERN.fullmatch(
        contract_commit
    ):
        errors.append("binding contract commit must be 40 lowercase hexadecimal characters")
    if contract_path != _CONTRACT_PATH:
        errors.append("binding contract path is not canonical")

    authoritative_commit: str | None = None
    if isinstance(authority_id, str) and authority_id:
        try:
            authority_record = canonical_adr.resolve(repository, authority_id)
        except adr_metadata.AdrMetadataError as error:
            errors.append(f"cannot resolve authority ADR: {error}")
        else:
            if authority_record.metadata.get("status") != "accepted":
                errors.append("authority ADR status must be accepted")
            body_hash = authority_record.metadata.get("decision_body_sha256")
            if not isinstance(body_hash, str) or not _SHA256_PATTERN.fullmatch(body_hash):
                errors.append("authority ADR decision_body_sha256 must be 64 lowercase hex")
            elif body_hash != adr_metadata.sha256_hex(authority_record.decision_body):
                errors.append("authority ADR decision body hash does not match")
            authoritative_commit = _identity(
                authority_record.decision_body.decode("utf-8"),
                "authority ADR",
                errors,
            )

    if authoritative_commit is not None and contract_commit != authoritative_commit:
        errors.append("binding contract commit does not match authority ADR")

    binding_identity = _identity(binding_body, "binding body", errors)
    if binding_identity is not None and binding_identity != contract_commit:
        errors.append("binding body identity does not match binding frontmatter")

    return errors


def main() -> int:
    errors = check(Path.cwd())

    if errors:
        print("shared governance provider binding: FAILED")
        for error in errors:
            print(error)
        return 1

    print("shared governance provider binding: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
