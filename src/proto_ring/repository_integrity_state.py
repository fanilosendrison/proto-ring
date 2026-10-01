"""Content-sensitive exact-state capture for Repository Integrity."""

from __future__ import annotations

import hashlib
import os
import stat as stat_module
import subprocess
from dataclasses import dataclass
from pathlib import Path

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


def _stat_guard(st: os.stat_result) -> tuple[int, int, int, int]:
    return (st.st_mode, st.st_size, st.st_mtime_ns, st.st_ctime_ns)


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
    return _StructuralCapture(
        symbolic=symbolic,
        head_oid=head_oid,
        index_raw=index_result.stdout,
        paths=tuple(sorted(set(entries) | set(additional_paths))),
    )


def _capture_path_records(
    root: Path, relative_paths: tuple[bytes, ...]
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
        raise StateCaptureError("governed path has an unsupported filesystem object kind")
    return tuple(records)


def capture_state(root: Path, governed_paths: tuple[str, ...] = ()) -> str:
    additional_paths = tuple(os.fsencode(path) for path in governed_paths)
    structural = _capture_structural(root, additional_paths)
    records = _capture_path_records(root, structural.paths)
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
    return digest.hexdigest()
