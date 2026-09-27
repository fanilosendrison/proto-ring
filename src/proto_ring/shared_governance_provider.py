"""Verify consumer bindings to the Shared Governance Provider contract."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re

from proto_ring.adr_metadata import AdrMetadataError, repository_path

__all__ = [
    "SharedGovernanceProviderProfile",
    "check",
]

_PROVIDER_REPOSITORY = "fanilosendrison/proto-ring"
_CONTRACT_PATH = "docs/contracts/shared-governance-provider.md"
_COMMIT_PATTERN = re.compile(r"[0-9a-f]{40}")
_IDENTITY_PATTERN = re.compile(
    rf"(?m)^{re.escape(_PROVIDER_REPOSITORY)}\n([^\n]*)\n"
    rf"{re.escape(_CONTRACT_PATH)}$"
)


@dataclass(frozen=True)
class SharedGovernanceProviderProfile:
    agent_directives_path: str
    binding_path: str
    contract_commit: str
    required_agent_directive: str
    required_binding_directive: str


def _profile_paths(
    repository: Path,
    profile: SharedGovernanceProviderProfile,
) -> tuple[Path | None, Path | None, list[str]]:
    errors: list[str] = []
    resolved: list[Path | None] = []
    for label, relative in (
        ("agent_directives_path", profile.agent_directives_path),
        ("binding_path", profile.binding_path),
    ):
        try:
            resolved.append(repository_path(repository, relative))
        except AdrMetadataError as error:
            errors.append(f"{label}: {error}")
            resolved.append(None)
    return resolved[0], resolved[1], errors


def _read_strict(path: Path, label: str) -> tuple[str | None, list[str]]:
    try:
        data = path.read_bytes()
    except OSError as error:
        return None, [f"cannot read {label}: {error}"]
    if data.startswith(b"\xef\xbb\xbf"):
        return None, [f"{label} contains a UTF-8 BOM"]
    if b"\r" in data:
        return None, [f"{label} contains CR or CRLF line endings"]
    try:
        return data.decode("utf-8"), []
    except UnicodeDecodeError as error:
        return None, [f"{label} is not valid UTF-8: {error}"]


def check(
    repository: Path,
    profile: SharedGovernanceProviderProfile,
) -> list[str]:
    """Return controlled errors for an invalid consumer-owned binding."""

    agent_path, binding_path, errors = _profile_paths(repository, profile)

    if not isinstance(profile.contract_commit, str) or not _COMMIT_PATTERN.fullmatch(
        profile.contract_commit
    ):
        errors.append("contract_commit must be exactly 40 lowercase hexadecimal characters")
    if not isinstance(profile.required_agent_directive, str) or not profile.required_agent_directive:
        errors.append("required_agent_directive must be a non-empty string")
    elif not isinstance(profile.binding_path, str) or profile.binding_path not in profile.required_agent_directive:
        errors.append("required_agent_directive must contain binding_path literally")
    if not isinstance(profile.required_binding_directive, str) or not profile.required_binding_directive:
        errors.append("required_binding_directive must be a non-empty string")

    agent_text: str | None = None
    binding_text: str | None = None
    if agent_path is not None:
        agent_text, read_errors = _read_strict(agent_path, "agent directives")
        errors.extend(read_errors)
    if binding_path is not None:
        binding_text, read_errors = _read_strict(binding_path, "binding")
        errors.extend(read_errors)

    if binding_text is not None:
        identities = _IDENTITY_PATTERN.findall(binding_text)
        if not identities:
            errors.append(
                "binding does not contain the canonical Shared Governance Provider identity"
            )
        elif len(identities) > 1:
            errors.append("binding contains multiple Shared Governance Provider identities")
        else:
            identity = identities[0]
            if not _COMMIT_PATTERN.fullmatch(identity):
                errors.append(
                    "binding contract identity must be exactly 40 lowercase hexadecimal characters"
                )
            elif identity != profile.contract_commit:
                errors.append("binding contract identity does not match contract_commit")

        if (
            isinstance(profile.required_binding_directive, str)
            and profile.required_binding_directive
            and profile.required_binding_directive not in binding_text
        ):
            errors.append("binding is missing the required mandatory-provider directive")

    if (
        agent_text is not None
        and isinstance(profile.required_agent_directive, str)
        and profile.required_agent_directive
        and profile.required_agent_directive not in agent_text
    ):
        errors.append(
            "agent directives do not route repository-governance work through the binding"
        )

    return errors
