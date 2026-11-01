"""Pinned retained-test inventory validation for the neutral Python suite."""
from __future__ import annotations

from collections import Counter
from pathlib import Path


class InventoryError(Exception):
    """The expected, discovered, and scheduled inventories are inconsistent."""


def load_pinned_test_ids(path: Path) -> tuple[str, ...]:
    identifiers = tuple(path.read_text(encoding="utf-8").splitlines())
    if not identifiers:
        raise InventoryError("pinned inventory is empty")
    if any(not identifier or identifier.strip() != identifier for identifier in identifiers):
        raise InventoryError("pinned inventory contains blank or noncanonical IDs")
    duplicates = sorted(
        identifier for identifier, count in Counter(identifiers).items() if count > 1
    )
    if duplicates:
        raise InventoryError(f"duplicate pinned IDs: {duplicates}")
    if identifiers != tuple(sorted(identifiers)):
        raise InventoryError("pinned inventory is not sorted")
    return identifiers


def validate_exact_inventory(
    discovered: tuple[str, ...], expected: tuple[str, ...]
) -> None:
    if not discovered:
        raise InventoryError("discovery produced zero tests")
    duplicate_discovered = sorted(
        identifier for identifier, count in Counter(discovered).items() if count > 1
    )
    if duplicate_discovered:
        raise InventoryError(f"duplicate discovered IDs: {duplicate_discovered}")
    missing = sorted(set(expected) - set(discovered))
    extra = sorted(set(discovered) - set(expected))
    if missing or extra or len(discovered) != len(expected):
        details = []
        if missing:
            details.append(f"missing pinned IDs: {missing}")
        if extra:
            details.append(f"extra discovered IDs: {extra}")
        raise InventoryError("; ".join(details))
