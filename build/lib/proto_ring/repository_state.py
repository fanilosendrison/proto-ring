"""Capture one content-sensitive exact Git repository state."""

from __future__ import annotations

import hashlib
import os
import stat as stat_module
import subprocess
from dataclasses import dataclass
from pathlib import Path

__all__ = [
    "RepositoryState",
    "StateCaptureError",
    "resolve_worktree_root",
    "discover_repository_paths",
    "capture",
]

# Preserve the established identity domain for every already-supported state.
_STATE_FORMAT = b"proto-ring:repository-integrity:state:v1"
_GIT_INTERNAL_ENV_REMOVE = frozenset(
    {
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CONFIG",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_GLOBAL",
        "GIT_CONFIG_SYSTEM",
        "GIT_CONFIG_NOSYSTEM",
        "GIT_OBJECT_DIRECTORY",
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_IMPLICIT_WORK_TREE",
        "GIT_GRAFT_FILE",
        "GIT_INDEX_FILE",
        "GIT_NO_REPLACE_OBJECTS",
        "GIT_REPLACE_REF_BASE",
        "GIT_PREFIX",
        "GIT_SHALLOW_FILE",
        "GIT_COMMON_DIR",
        "GIT_NAMESPACE",
        "GIT_CEILING_DIRECTORIES",
        "GIT_DISCOVERY_ACROSS_FILESYSTEM",
    }
)
_GIT_CONFIG_INDEXED_PREFIXES = ("GIT_CONFIG_KEY_", "GIT_CONFIG_VALUE_")


class StateCaptureError(Exception):
    """Raised when an exact repository state identity cannot be determined."""


@dataclass(frozen=True)
class RepositoryState:
    repository: Path
    identity: str
    scope_paths: tuple[str, ...]


@dataclass(frozen=True)
class _StructuralCapture:
    symbolic: bytes | None
    head_oid: bytes | None
    index_raw: bytes
    paths: tuple[bytes, ...]


@dataclass(frozen=True)
class _PathRecord:
    relative: bytes
    kind: str
    mode: int | None
    content: bytes | None


def _frame(digest, data: bytes) -> None:
    digest.update(len(data).to_bytes(8, "big"))
    digest.update(data)


def _stat_guard(
    st: os.stat_result,
) -> tuple[int, int, int, int, int | None, int | None]:
    return (
        st.st_mode,
        st.st_size,
        st.st_mtime_ns,
        st.st_ctime_ns,
        getattr(st, "st_dev", None),
        getattr(st, "st_ino", None),
    )


def _internal_git_environment() -> dict[str, str]:
    environment = dict(os.environ)
    for key in tuple(environment):
        if key in _GIT_INTERNAL_ENV_REMOVE or key.startswith(
            _GIT_CONFIG_INDEXED_PREFIXES
        ):
            del environment[key]
    return environment


def _run_git(root: Path, args: list[str]) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        ["git", "-C", os.fspath(root), *args],
        env=_internal_git_environment(),
        capture_output=True,
        check=False,
    )


def resolve_worktree_root(repository: Path) -> Path:
    try:
        requested = Path(os.path.realpath(repository))
    except OSError as error:
        raise StateCaptureError(
            f"repository path cannot be resolved: {error}"
        ) from error
    if not requested.is_dir():
        raise StateCaptureError("repository path is not an existing directory")

    result = _run_git(requested, ["rev-parse", "--show-toplevel"])
    if result.returncode != 0:
        raise StateCaptureError("repository is not a Git worktree root")
    reported_text = result.stdout.decode("utf-8", errors="surrogateescape")
    if reported_text.endswith("\n"):
        reported_text = reported_text[:-1]
    if not reported_text:
        raise StateCaptureError("Git did not report a worktree root")
    reported = Path(os.path.realpath(reported_text))
    if reported != requested:
        raise StateCaptureError("repository path is not the exact Git worktree root")
    return reported


def discover_repository_paths(root: Path) -> tuple[str, ...]:
    """Return Git-visible repository paths for deterministic selector expansion."""

    result = _run_git(
        root,
        ["ls-files", "-z", "--cached", "--others", "--exclude-per-directory=.gitignore"],
    )
    if result.returncode != 0:
        raise StateCaptureError("governed worktree paths could not be discovered")
    entries = result.stdout.split(b"\x00")
    if entries and entries[-1] == b"":
        entries = entries[:-1]
    if any(entry == b"" for entry in entries):
        raise StateCaptureError("governed worktree path discovery contained an empty path")
    try:
        return tuple(
            os.fsdecode(entry).replace(os.sep, "/") for entry in sorted(entries)
        )
    except UnicodeError as error:
        raise StateCaptureError("governed worktree path could not be decoded") from error


def _discovered_path_identity(raw_path: bytes) -> bytes:
    if len(raw_path) > 1 and raw_path.endswith(b"/"):
        return raw_path[:-1]
    return raw_path


def _explicit_scope_path_identity(
    repository: Path,
    value: object,
) -> str | None:
    if not isinstance(value, str) or value == "" or "\x00" in value:
        return None
    if os.path.splitdrive(value)[0] or os.path.isabs(value):
        return None

    separators = {os.sep}
    if os.altsep is not None:
        separators.add(os.altsep)
    components: list[str] = []
    current: list[str] = []
    for character in value:
        if character in separators:
            if not current:
                return None
            components.append("".join(current))
            current = []
        else:
            current.append(character)
    if not current:
        return None
    components.append("".join(current))
    if any(component in {".", ".."} for component in components):
        return None

    parent = repository.joinpath(*components[:-1])
    try:
        resolved_parent = parent.resolve(strict=False)
    except (OSError, RuntimeError, ValueError) as error:
        raise StateCaptureError(
            f"explicit scope parent cannot be resolved: {error}"
        ) from error
    try:
        resolved_parent.relative_to(repository)
    except ValueError:
        return None
    return value


def _capture_structural(
    root: Path, additional_paths: tuple[bytes, ...] = ()
) -> _StructuralCapture:
    symbolic_result = _run_git(root, ["symbolic-ref", "-q", "HEAD"])
    if symbolic_result.returncode == 0:
        symbolic: bytes | None = symbolic_result.stdout.rstrip(b"\n")
        if not symbolic:
            raise StateCaptureError("symbolic HEAD identity is empty")
    elif symbolic_result.returncode == 1:
        symbolic = None
    else:
        raise StateCaptureError("symbolic HEAD identity could not be determined")

    oid_result = _run_git(root, ["rev-parse", "--verify", "--quiet", "HEAD"])
    if oid_result.returncode == 0:
        head_oid: bytes | None = oid_result.stdout.rstrip(b"\n")
        if not head_oid:
            raise StateCaptureError("HEAD object identity is empty")
    elif oid_result.returncode == 1:
        head_oid = None
    else:
        raise StateCaptureError("HEAD object identity could not be determined")

    index_result = _run_git(root, ["ls-files", "--stage", "-z"])
    if index_result.returncode != 0:
        raise StateCaptureError("Git index identity could not be read")
    discovery_result = _run_git(
        root,
        ["ls-files", "-z", "--cached", "--others", "--exclude-per-directory=.gitignore"],
    )
    if discovery_result.returncode != 0:
        raise StateCaptureError("governed worktree paths could not be discovered")
    entries = discovery_result.stdout.split(b"\x00")
    if entries and entries[-1] == b"":
        entries = entries[:-1]
    if any(entry == b"" for entry in entries):
        raise StateCaptureError("governed worktree path discovery contained an empty path")
    discovered_paths = {
        _discovered_path_identity(entry) for entry in entries
    }
    return _StructuralCapture(
        symbolic=symbolic,
        head_oid=head_oid,
        index_raw=index_result.stdout,
        paths=tuple(sorted(discovered_paths | set(additional_paths))),
    )


def _capture_path_records(
    root: Path,
    relative_paths: tuple[bytes, ...],
    explicit_paths: frozenset[bytes],
) -> tuple[_PathRecord, ...]:
    root_bytes = os.fsencode(os.fspath(root))
    records: list[_PathRecord] = []
    for relative in relative_paths:
        full = os.path.join(root_bytes, relative)
        try:
            before = os.lstat(full)
        except FileNotFoundError:
            records.append(_PathRecord(relative, "absent", None, None))
            continue
        except OSError as error:
            raise StateCaptureError(f"governed path could not be inspected: {error}") from error

        if stat_module.S_ISREG(before.st_mode):
            try:
                with open(full, "rb") as handle:
                    content = handle.read()
                after = os.lstat(full)
            except OSError as error:
                raise StateCaptureError(f"governed file could not be read: {error}") from error
            if _stat_guard(before) != _stat_guard(after):
                raise StateCaptureError("governed file changed while its content was read")
            records.append(
                _PathRecord(relative, "regular", before.st_mode, hashlib.sha256(content).digest())
            )
            continue

        if stat_module.S_ISLNK(before.st_mode):
            try:
                target = os.readlink(full)
                after = os.lstat(full)
            except OSError as error:
                raise StateCaptureError(f"governed symlink could not be read: {error}") from error
            if _stat_guard(before) != _stat_guard(after):
                raise StateCaptureError("governed symlink changed while its target was read")
            records.append(
                _PathRecord(relative, "symlink", before.st_mode, os.fsencode(target))
            )
            continue

        if stat_module.S_ISDIR(before.st_mode) and relative in explicit_paths:
            records.append(_PathRecord(relative, "directory", before.st_mode, None))
            continue

        raise StateCaptureError("governed path has an unsupported filesystem object kind")
    return tuple(records)


def capture(repository: Path, scope_paths: tuple[str, ...] = ()) -> RepositoryState:
    """Return one immutable state bound to the exact Git worktree root."""

    root = resolve_worktree_root(repository)
    validated_scope: list[str] = []
    for value in scope_paths:
        identity = _explicit_scope_path_identity(root, value)
        if identity is None:
            raise StateCaptureError("explicit scope path is not admissible")
        validated_scope.append(identity)
    canonical_scope = tuple(sorted(set(validated_scope)))
    additional_paths = tuple(os.fsencode(path) for path in canonical_scope)
    structural = _capture_structural(root, additional_paths)
    explicit_paths = frozenset(additional_paths)
    records = _capture_path_records(root, structural.paths, explicit_paths)
    validation_records = _capture_path_records(
        root, structural.paths, explicit_paths
    )
    if validation_records != records:
        raise StateCaptureError("governed path state changed during state capture")
    final_structural = _capture_structural(root, additional_paths)
    if final_structural != structural:
        raise StateCaptureError("repository structure changed during state capture")

    digest = hashlib.sha256()
    _frame(digest, _STATE_FORMAT)
    if structural.symbolic is None:
        _frame(digest, b"HEAD-DETACHED")
    else:
        _frame(digest, b"HEAD-SYMBOLIC")
        _frame(digest, structural.symbolic)
    if structural.head_oid is None:
        _frame(digest, b"HEAD-UNBORN")
    else:
        _frame(digest, b"HEAD-OID")
        _frame(digest, structural.head_oid)
    _frame(digest, b"INDEX")
    _frame(digest, structural.index_raw)
    for record in records:
        _frame(digest, b"PATH")
        _frame(digest, record.relative)
        _frame(digest, record.kind.encode("ascii"))
        if record.mode is not None:
            _frame(digest, record.mode.to_bytes(8, "big"))
            _frame(digest, record.content or b"")
    return RepositoryState(root, digest.hexdigest(), canonical_scope)
