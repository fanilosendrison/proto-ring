"""Generic Repository Integrity substrate.

This module implements the generic semantics of the canonical proto-ring
Repository Integrity contract in ``docs/contracts/repository-integrity.md``.

It owns generic obligation execution, tri-state classification, deterministic
profile identity, and a Git-visible content-sensitive exact-state identity.

It owns no consumer validation membership, no projection registry, no manifest,
no qualification machinery, and no repair, rendering, or regeneration behavior.
"""

from __future__ import annotations

import hashlib
import json
import os
import stat as stat_module
import subprocess
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Mapping

__all__ = [
    "CommandObligation",
    "IntegrityProfile",
    "IntegrityResult",
    "IntegrityVerdict",
    "ObligationResult",
    "ObligationStatus",
    "evaluate",
]

_STATE_FORMAT = b"proto-ring:repository-integrity:state:v1"


class ObligationStatus(str, Enum):
    SATISFIED = "SATISFIED"
    VIOLATED = "VIOLATED"
    UNDETERMINED = "UNDETERMINED"


class IntegrityVerdict(str, Enum):
    PASS = "PASS"
    NON_PASS = "NON_PASS"


@dataclass(frozen=True)
class CommandObligation:
    name: str
    argv: tuple[str, ...]
    undetermined_exit_codes: frozenset[int] = frozenset()

    def __post_init__(self) -> None:
        if 0 in self.undetermined_exit_codes:
            raise ValueError(
                "0 is not a valid undetermined exit code; exit code 0 means SATISFIED"
            )


@dataclass(frozen=True)
class IntegrityProfile:
    obligations: tuple[CommandObligation, ...]
    continue_after_non_satisfied: bool = True


@dataclass(frozen=True)
class ObligationResult:
    name: str
    status: ObligationStatus
    returncode: int | None
    detail: str | None


@dataclass(frozen=True)
class IntegrityResult:
    verdict: IntegrityVerdict
    profile_identity: str
    baseline_state_identity: str | None
    final_state_identity: str | None
    obligations: tuple[ObligationResult, ...]
    errors: tuple[str, ...]


class _StateCaptureError(Exception):
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


def _run_git(root: Path, args: list[str]) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        ["git", "-C", os.fspath(root), *args],
        capture_output=True,
        check=False,
    )


def _resolve_worktree_root(repository: Path) -> Path:
    try:
        requested = Path(os.path.realpath(repository))
    except OSError as error:
        raise _StateCaptureError(
            f"repository path cannot be resolved: {error}"
        ) from error

    if not requested.is_dir():
        raise _StateCaptureError("repository path is not an existing directory")

    result = _run_git(requested, ["rev-parse", "--show-toplevel"])
    if result.returncode != 0:
        raise _StateCaptureError("repository is not a Git worktree root")

    reported_text = result.stdout.decode("utf-8", errors="surrogateescape")
    if reported_text.endswith("\n"):
        reported_text = reported_text[:-1]
    if not reported_text:
        raise _StateCaptureError("Git did not report a worktree root")

    reported = Path(os.path.realpath(reported_text))
    if reported != requested:
        raise _StateCaptureError("repository path is not the exact Git worktree root")

    return reported


def _capture_structural(root: Path) -> _StructuralCapture:
    symbolic_result = _run_git(root, ["symbolic-ref", "-q", "HEAD"])
    if symbolic_result.returncode == 0:
        symbolic: bytes | None = symbolic_result.stdout.rstrip(b"\n")
        if not symbolic:
            raise _StateCaptureError("symbolic HEAD identity is empty")
    elif symbolic_result.returncode == 1:
        symbolic = None
    else:
        raise _StateCaptureError("symbolic HEAD identity could not be determined")

    oid_result = _run_git(root, ["rev-parse", "--verify", "--quiet", "HEAD"])
    if oid_result.returncode == 0:
        head_oid: bytes | None = oid_result.stdout.rstrip(b"\n")
        if not head_oid:
            raise _StateCaptureError("HEAD object identity is empty")
    elif oid_result.returncode == 1:
        head_oid = None
    else:
        raise _StateCaptureError("HEAD object identity could not be determined")

    index_result = _run_git(root, ["ls-files", "--stage", "-z"])
    if index_result.returncode != 0:
        raise _StateCaptureError("Git index identity could not be read")
    index_raw = index_result.stdout

    discovery_result = _run_git(
        root,
        [
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-per-directory=.gitignore",
        ],
    )
    if discovery_result.returncode != 0:
        raise _StateCaptureError("governed worktree paths could not be discovered")

    entries = discovery_result.stdout.split(b"\x00")
    if entries and entries[-1] == b"":
        entries = entries[:-1]
    if any(entry == b"" for entry in entries):
        raise _StateCaptureError(
            "governed worktree path discovery contained an empty path"
        )

    return _StructuralCapture(
        symbolic=symbolic,
        head_oid=head_oid,
        index_raw=index_raw,
        paths=tuple(sorted(entries)),
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
            raise _StateCaptureError(
                f"governed path could not be inspected: {error}"
            ) from error

        if stat_module.S_ISREG(before.st_mode):
            try:
                with open(full, "rb") as handle:
                    content = handle.read()
            except OSError as error:
                raise _StateCaptureError(
                    f"governed file could not be read: {error}"
                ) from error

            try:
                after = os.lstat(full)
            except OSError as error:
                raise _StateCaptureError(
                    f"governed path changed during content read: {error}"
                ) from error
            if _stat_guard(before) != _stat_guard(after):
                raise _StateCaptureError(
                    "governed file changed while its content was read"
                )

            records.append(
                _PathRecord(
                    relative,
                    "regular",
                    before.st_mode,
                    hashlib.sha256(content).digest(),
                )
            )
            continue

        if stat_module.S_ISLNK(before.st_mode):
            try:
                target = os.readlink(full)
            except OSError as error:
                raise _StateCaptureError(
                    f"governed symlink could not be read: {error}"
                ) from error

            try:
                after = os.lstat(full)
            except OSError as error:
                raise _StateCaptureError(
                    f"governed path changed during symlink read: {error}"
                ) from error
            if _stat_guard(before) != _stat_guard(after):
                raise _StateCaptureError(
                    "governed symlink changed while its target was read"
                )

            records.append(
                _PathRecord(relative, "symlink", before.st_mode, os.fsencode(target))
            )
            continue

        raise _StateCaptureError(
            "governed path has an unsupported filesystem object kind"
        )

    return tuple(records)


def _capture_state(root: Path) -> str:
    structural = _capture_structural(root)
    records = _capture_path_records(root, structural.paths)
    final_structural = _capture_structural(root)

    if final_structural != structural:
        raise _StateCaptureError(
            "repository structure changed during state capture"
        )

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
        if record.mode is None:
            continue
        _frame(digest, record.mode.to_bytes(8, "big"))
        _frame(digest, record.content or b"")

    return digest.hexdigest()


def _profile_identity(profile: IntegrityProfile) -> str:
    payload = {
        "continue_after_non_satisfied": profile.continue_after_non_satisfied,
        "obligations": [
            {
                "name": obligation.name,
                "argv": list(obligation.argv),
                "undetermined_exit_codes": sorted(
                    obligation.undetermined_exit_codes
                ),
            }
            for obligation in profile.obligations
        ],
    }
    canonical = json.dumps(
        payload,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    )
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def _execute_obligation(
    *,
    root: Path,
    obligation: CommandObligation,
    env: Mapping[str, str],
) -> tuple[ObligationStatus, int | None, str | None]:
    try:
        completed = subprocess.run(
            list(obligation.argv),
            cwd=os.fspath(root),
            env=dict(env),
            capture_output=True,
            check=False,
        )
    except (OSError, ValueError, IndexError) as error:
        return (
            ObligationStatus.UNDETERMINED,
            None,
            f"command could not be started: {error}",
        )

    returncode = completed.returncode
    if returncode == 0:
        return ObligationStatus.SATISFIED, 0, None

    if returncode in obligation.undetermined_exit_codes:
        return (
            ObligationStatus.UNDETERMINED,
            returncode,
            f"command exited with {returncode}",
        )

    return (
        ObligationStatus.VIOLATED,
        returncode,
        f"command exited with {returncode}",
    )


def evaluate(
    repository: Path,
    profile: IntegrityProfile,
    *,
    env: Mapping[str, str],
) -> IntegrityResult:
    profile_identity = _profile_identity(profile)
    repository_path = Path(repository)

    baseline_identity: str | None = None
    baseline_error: str | None = None
    root: Path | None = None

    try:
        root = _resolve_worktree_root(repository_path)
        baseline_identity = _capture_state(root)
    except _StateCaptureError as error:
        baseline_error = str(error)

    if baseline_identity is None:
        detail = (
            "baseline repository state could not be determined: "
            f"{baseline_error or 'unknown error'}"
        )
        return IntegrityResult(
            verdict=IntegrityVerdict.NON_PASS,
            profile_identity=profile_identity,
            baseline_state_identity=None,
            final_state_identity=None,
            obligations=tuple(
                ObligationResult(
                    name=obligation.name,
                    status=ObligationStatus.UNDETERMINED,
                    returncode=None,
                    detail=detail,
                )
                for obligation in profile.obligations
            ),
            errors=(detail,),
        )

    assert root is not None

    errors: list[str] = []
    results: list[ObligationResult] = []
    final_identity: str | None = baseline_identity
    executed = 0

    for index, obligation in enumerate(profile.obligations):
        try:
            before_identity = _capture_state(root)
        except _StateCaptureError as error:
            detail = (
                f"repository state could not be determined before obligation "
                f"{obligation.name}: {error}"
            )
            results.append(
                ObligationResult(
                    name=obligation.name,
                    status=ObligationStatus.UNDETERMINED,
                    returncode=None,
                    detail=detail,
                )
            )
            errors.append(detail)
            executed = index + 1
            break

        final_identity = before_identity

        if before_identity != baseline_identity:
            errors.append(
                f"repository state changed before obligation {obligation.name}"
            )
            results.append(
                ObligationResult(
                    name=obligation.name,
                    status=ObligationStatus.UNDETERMINED,
                    returncode=None,
                    detail=(
                        "obligation not executed because repository state no "
                        "longer matches baseline"
                    ),
                )
            )
            executed = index + 1
            break

        status, returncode, detail = _execute_obligation(
            root=root,
            obligation=obligation,
            env=env,
        )

        try:
            after_identity = _capture_state(root)
        except _StateCaptureError as error:
            detail = (
                f"repository state could not be determined after obligation "
                f"{obligation.name}: {error}"
            )
            results.append(
                ObligationResult(
                    name=obligation.name,
                    status=ObligationStatus.UNDETERMINED,
                    returncode=returncode,
                    detail=detail,
                )
            )
            errors.append(detail)
            executed = index + 1
            break

        final_identity = after_identity

        if after_identity != baseline_identity:
            detail = f"repository state changed during obligation {obligation.name}"
            results.append(
                ObligationResult(
                    name=obligation.name,
                    status=ObligationStatus.VIOLATED,
                    returncode=returncode,
                    detail=detail,
                )
            )
            errors.append(detail)
            executed = index + 1
            break

        results.append(
            ObligationResult(
                name=obligation.name,
                status=status,
                returncode=returncode,
                detail=detail,
            )
        )
        executed = index + 1

        if (
            status is not ObligationStatus.SATISFIED
            and not profile.continue_after_non_satisfied
        ):
            break

    for obligation in profile.obligations[executed:]:
        results.append(
            ObligationResult(
                name=obligation.name,
                status=ObligationStatus.UNDETERMINED,
                returncode=None,
                detail="obligation skipped after previous non-satisfied result",
            )
        )

    complete = len(results) == len(profile.obligations)
    all_satisfied = complete and all(
        result.status is ObligationStatus.SATISFIED for result in results
    )

    verdict = (
        IntegrityVerdict.PASS
        if final_identity == baseline_identity
        and all_satisfied
        and not errors
        else IntegrityVerdict.NON_PASS
    )

    return IntegrityResult(
        verdict=verdict,
        profile_identity=profile_identity,
        baseline_state_identity=baseline_identity,
        final_state_identity=final_identity,
        obligations=tuple(results),
        errors=tuple(errors),
    )
