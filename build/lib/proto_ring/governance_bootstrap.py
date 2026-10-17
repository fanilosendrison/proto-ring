"""Load canonical repository-governance bootstrap configuration."""

from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path
from typing import cast

from proto_ring import structured_data
from proto_ring.structured_data import StructuredValue


__all__ = [
    "GovernanceBootstrap",
    "GovernanceBootstrapError",
    "load",
]


class GovernanceBootstrapError(ValueError):
    """Report controlled failure to obtain canonical repository governance."""


@dataclass(frozen=True)
class GovernanceBootstrap:
    repository: Path
    carrier: Path
    metadata: dict[str, StructuredValue]
    repository_governance: dict[str, StructuredValue]


def load(repository: Path) -> GovernanceBootstrap:
    """Load structured governance from exactly the repository's root AGENTS.md."""

    carrier = repository / "AGENTS.md"
    try:
        data = carrier.read_bytes()
    except OSError as error:
        raise GovernanceBootstrapError(
            f"cannot read root AGENTS.md: {error}"
        ) from error
    try:
        with os.scandir(repository) as entries:
            exact_carrier_exists = any(entry.name == "AGENTS.md" for entry in entries)
    except OSError as error:
        raise GovernanceBootstrapError(
            f"cannot verify root AGENTS.md identity: {error}"
        ) from error
    if not exact_carrier_exists:
        raise GovernanceBootstrapError("root carrier must be named exactly AGENTS.md")

    try:
        parsed = structured_data.parse_frontmatter_bytes(data)
    except structured_data.StructuredDataError as error:
        raise GovernanceBootstrapError(
            f"cannot parse root AGENTS.md: {error}"
        ) from error

    if "repository_governance" not in parsed.metadata:
        raise GovernanceBootstrapError("repository_governance is required")
    repository_governance = parsed.metadata["repository_governance"]
    if not isinstance(repository_governance, dict):
        raise GovernanceBootstrapError("repository_governance must be a mapping")

    return GovernanceBootstrap(
        repository=repository,
        carrier=carrier,
        metadata=parsed.metadata,
        repository_governance=cast(
            dict[str, StructuredValue], repository_governance
        ),
    )
