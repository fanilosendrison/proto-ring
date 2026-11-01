"""Fail-closed absence control for the retired Python production package."""
from __future__ import annotations

import importlib.machinery
import importlib.metadata
from pathlib import Path
import re
import sys
from typing import Iterable


class PythonPackageAbsenceError(RuntimeError):
    """The retired package remains visible to the active Python process."""


def _search_paths(repository: Path, effective_paths: Iterable[str]) -> tuple[str, ...]:
    explicit = (
        repository,
        repository / "src",
        repository / "tests",
        repository / "tools",
    )
    ordered: list[str] = []
    for raw in (*map(str, explicit), *effective_paths):
        resolved = str(Path.cwd() if raw == "" else Path(raw).resolve(strict=False))
        if resolved not in ordered:
            ordered.append(resolved)
    return tuple(ordered)


def _distribution_name(value: str) -> str:
    return re.sub(r"[-_.]+", "-", value).lower()


def assert_python_package_absent(
    repository: Path,
    effective_paths: Iterable[str] | None = None,
) -> None:
    """Require no loaded, importable, installed, ambient, or editable package."""
    loaded = sorted(
        name for name in sys.modules if name == "proto_ring" or name.startswith("proto_ring.")
    )
    if loaded:
        raise PythonPackageAbsenceError(f"proto_ring is already loaded: {loaded}")

    paths = _search_paths(repository.resolve(), sys.path if effective_paths is None else effective_paths)
    for finder in sys.meta_path:
        search_path = paths if finder is importlib.machinery.PathFinder else None
        try:
            specification = finder.find_spec("proto_ring", search_path, None)
        except (ImportError, AttributeError, TypeError, ValueError) as error:
            raise PythonPackageAbsenceError(
                f"proto_ring import resolution failed closed via {type(finder).__name__}: {error}"
            ) from error
        if specification is not None:
            location = specification.origin or specification.submodule_search_locations
            raise PythonPackageAbsenceError(
                f"proto_ring remains importable via {type(finder).__name__} from {location}"
            )

    for distribution in importlib.metadata.distributions(path=paths):
        name = distribution.metadata.get("Name")
        if name is not None and _distribution_name(name) == "proto-ring":
            raise PythonPackageAbsenceError(
                f"proto-ring distribution remains installed at {distribution.locate_file('')}"
            )
