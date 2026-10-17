"""Resolve logical ADR identities within consumer-owned canonical corpora."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
from typing import Mapping, cast

from proto_ring import adr_metadata, portable_pattern, repository_governance_model


__all__ = [
    "CanonicalAdr",
    "configured_profile_path",
    "resolve",
]


@dataclass(frozen=True)
class CanonicalAdr:
    id: str
    path: Path
    metadata: Mapping[str, object]
    decision_body: bytes


def configured_profile_path(repository: Path) -> Path:
    """Return the contained ADR profile configured by root ``AGENTS.md``."""

    try:
        model = repository_governance_model.load(repository)
    except repository_governance_model.RepositoryGovernanceModelError as error:
        raise adr_metadata.AdrMetadataError(str(error)) from error
    capability = model.capabilities.get("architecture_decisions")
    if capability is None:
        raise adr_metadata.AdrMetadataError(
            "architecture_decisions capability is required"
        )
    profile = capability.routes.get("profile")
    if profile is None:
        raise adr_metadata.AdrMetadataError(
            "architecture_decisions profile route is required"
        )
    return profile.target


def _profile_repository(repository: Path) -> dict[str, object]:
    profile_path = configured_profile_path(repository)
    profile = adr_metadata.require_mapping(
        adr_metadata.load_yaml(profile_path), "ADR profile"
    )
    return adr_metadata.require_mapping(profile.get("repository"), "repository")


def _portable_pattern(
    source: str, label: str
) -> portable_pattern.PortablePattern:
    try:
        return portable_pattern.PortablePattern(source)
    except portable_pattern.PortablePatternError as error:
        raise adr_metadata.AdrMetadataError(f"{label} is invalid: {error}") from error


def resolve(repository: Path, adr_id: str) -> CanonicalAdr:
    """Resolve one logical ADR ID to its unique canonical structured artifact."""

    profile_repository = _profile_repository(repository)
    adr_directory_value = adr_metadata.require_string(
        profile_repository.get("adr_directory"), "repository.adr_directory"
    )
    filename_pattern = adr_metadata.require_string(
        profile_repository.get("filename_pattern"), "repository.filename_pattern"
    )
    id_pattern = adr_metadata.require_string(
        profile_repository.get("id_pattern"), "repository.id_pattern"
    )
    id_width = profile_repository.get("id_width")
    if isinstance(id_width, bool) or not isinstance(id_width, int) or id_width < 1:
        raise adr_metadata.AdrMetadataError(
            "repository.id_width must be a positive integer"
        )

    filename_matcher = _portable_pattern(
        filename_pattern, "repository.filename_pattern"
    )
    id_matcher = _portable_pattern(id_pattern, "repository.id_pattern")
    if "number" not in filename_matcher.capture_names:
        raise adr_metadata.AdrMetadataError(
            "repository.filename_pattern must define named group 'number'"
        )
    if not isinstance(adr_id, str) or id_matcher.full_match(adr_id) is None:
        raise adr_metadata.AdrMetadataError(
            "requested ADR ID does not match repository.id_pattern"
        )
    identity = re.fullmatch(r"ADR-([0-9]+)", adr_id)
    if identity is None:
        raise adr_metadata.AdrMetadataError(
            "requested ADR ID must have exact form ADR-<digits>"
        )
    number = identity.group(1)
    if len(number) != id_width:
        raise adr_metadata.AdrMetadataError(
            "requested ADR ID digit width does not match repository.id_width"
        )

    adr_directory = adr_metadata.repository_path(repository, adr_directory_value)
    if not adr_directory.is_dir():
        raise adr_metadata.AdrMetadataError(
            f"repository.adr_directory is not an existing directory: {adr_directory_value}"
        )
    candidates: list[Path] = []
    try:
        for entry in adr_directory.iterdir():
            match = filename_matcher.full_match(entry.name)
            if match is None:
                continue
            if match.group("number") != number:
                continue
            if entry.is_symlink():
                raise adr_metadata.AdrMetadataError(
                    "canonical ADR candidate must be a direct non-symlink file: "
                    f"{entry.name}"
                )
            if not entry.is_file():
                continue
            candidates.append(entry)
    except OSError as error:
        raise adr_metadata.AdrMetadataError(
            f"cannot inspect repository.adr_directory: {error}"
        ) from error
    if not candidates:
        raise adr_metadata.AdrMetadataError(
            f"no canonical ADR candidate exists for {adr_id}"
        )
    if len(candidates) > 1:
        raise adr_metadata.AdrMetadataError(
            f"canonical ADR identity {adr_id} is ambiguous: {len(candidates)} candidates"
        )

    resolved_root = repository.resolve()
    try:
        candidate_relative = candidates[0].relative_to(resolved_root)
    except ValueError as error:
        raise adr_metadata.AdrMetadataError(
            f"canonical ADR candidate escapes repository: {candidates[0]}"
        ) from error
    candidate = adr_metadata.repository_path(repository, candidate_relative.as_posix())
    metadata, decision_body = adr_metadata.parse_adr(candidate)
    if metadata.get("id") != adr_id:
        raise adr_metadata.AdrMetadataError(
            "canonical ADR metadata.id does not match requested ADR ID"
        )
    return CanonicalAdr(
        id=adr_id,
        path=candidate,
        metadata=cast(Mapping[str, object], metadata),
        decision_body=decision_body,
    )
