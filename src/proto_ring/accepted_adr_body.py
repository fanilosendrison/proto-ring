"""Verify accepted ADR decision-body immutability in first-parent Git history."""

from __future__ import annotations

import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

from proto_ring import adr_metadata, canonical_adr, portable_pattern

__all__ = ["check"]

_ALLOWED_GIT_OPERATIONS = frozenset(
    {"rev-parse", "rev-list", "diff", "diff-tree", "ls-tree", "cat-file"}
)
_REGULAR_FILE_MODES = frozenset({"100644", "100755"})


class _CheckFailure(Exception):
    """Report a controlled failure that prevents complete verification."""


@dataclass(frozen=True)
class _AcceptedBodyAnchor:
    id: str
    commit: str
    path: str
    decision_body: bytes


@dataclass(frozen=True)
class _DiscoveryConfig:
    adr_directory: str
    filename_pattern: portable_pattern.PortablePattern


def _git_environment() -> dict[str, str]:
    environment = os.environ.copy()
    environment["GIT_NO_REPLACE_OBJECTS"] = "1"
    return environment


def _run_git(repository: Path, *args: str) -> subprocess.CompletedProcess[bytes]:
    if not args or args[0] not in _ALLOWED_GIT_OPERATIONS:
        raise _CheckFailure("unsupported Git read operation")
    try:
        return subprocess.run(
            ["git", *args],
            cwd=repository,
            env=_git_environment(),
            capture_output=True,
            check=False,
        )
    except (OSError, ValueError) as error:
        raise _CheckFailure(f"Git command could not be started: {error}") from error


def _git_output(repository: Path, *args: str) -> bytes:
    result = _run_git(repository, *args)
    if result.returncode != 0:
        diagnostic = result.stderr.decode("utf-8", errors="replace").strip()
        suffix = f": {diagnostic}" if diagnostic else ""
        raise _CheckFailure(f"Git {' '.join(args)} failed{suffix}")
    return result.stdout


def _decode_git_text(data: bytes, label: str) -> str:
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise _CheckFailure(f"{label} is not valid UTF-8") from error


def _validate_repository(repository: Path) -> Path:
    try:
        root = repository.resolve()
    except OSError as error:
        raise _CheckFailure(f"repository path cannot be resolved: {error}") from error
    if not root.is_dir():
        raise _CheckFailure("repository must be an existing Git worktree root")

    top_result = _run_git(root, "rev-parse", "--show-toplevel")
    if top_result.returncode != 0:
        raise _CheckFailure("repository must be a Git worktree")
    top_text = _decode_git_text(top_result.stdout, "Git worktree root").strip()
    if not top_text or Path(top_text).resolve() != root:
        raise _CheckFailure("repository must be the exact Git worktree root")

    head_result = _run_git(root, "rev-parse", "--verify", "HEAD^{commit}")
    if head_result.returncode != 0 or not head_result.stdout.strip():
        raise _CheckFailure("repository must have a committed HEAD")

    shallow = _git_output(root, "rev-parse", "--is-shallow-repository").strip()
    if shallow == b"true":
        raise _CheckFailure(
            "accepted ADR body immutability requires complete non-shallow Git history"
        )
    if shallow != b"false":
        raise _CheckFailure("cannot determine whether Git history is shallow")
    return root


def _load_discovery_config(repository: Path) -> _DiscoveryConfig:
    profile_path = canonical_adr.configured_profile_path(repository)
    profile = adr_metadata.require_mapping(
        adr_metadata.load_yaml(profile_path), "ADR profile"
    )
    profile_repository = adr_metadata.require_mapping(
        profile.get("repository"), "repository"
    )
    directory_value = adr_metadata.require_string(
        profile_repository.get("adr_directory"), "repository.adr_directory"
    )
    filename_pattern = adr_metadata.require_string(
        profile_repository.get("filename_pattern"), "repository.filename_pattern"
    )
    directory = adr_metadata.repository_path(repository, directory_value)
    try:
        directory_relative = directory.relative_to(repository.resolve()).as_posix()
    except ValueError as error:
        raise adr_metadata.AdrMetadataError(
            "repository.adr_directory escapes repository"
        ) from error
    try:
        filename_matcher = portable_pattern.PortablePattern(filename_pattern)
    except portable_pattern.PortablePatternError as error:
        raise adr_metadata.AdrMetadataError(
            f"repository.filename_pattern is invalid: {error}"
        ) from error
    if "number" not in filename_matcher.capture_names:
        raise adr_metadata.AdrMetadataError(
            "repository.filename_pattern must define named group 'number'"
        )
    return _DiscoveryConfig(directory_relative, filename_matcher)


def _commits(repository: Path) -> list[str]:
    output = _git_output(repository, "rev-list", "--first-parent", "--reverse", "HEAD")
    commits = _decode_git_text(output, "first-parent commit list").splitlines()
    if not commits:
        raise _CheckFailure("committed HEAD has no first-parent history")
    return commits


def _first_parent(repository: Path, commit: str) -> str | None:
    output = _git_output(repository, "rev-list", "--parents", "-n", "1", commit)
    fields = _decode_git_text(output, "commit parent list").split()
    if not fields or fields[0] != commit:
        raise _CheckFailure(f"cannot determine first parent for commit {commit}")
    return fields[1] if len(fields) > 1 else None


def _changed_paths(repository: Path, commit: str) -> list[bytes]:
    parent = _first_parent(repository, commit)
    args = [
        "diff-tree",
        "--no-commit-id",
        "--name-only",
        "-r",
        "--no-renames",
        "-z",
    ]
    if parent is None:
        args.extend(["--root", commit])
    else:
        args.extend([parent, commit])
    raw_paths = _git_output(repository, *args).split(b"\x00")
    if raw_paths and raw_paths[-1] == b"":
        raw_paths.pop()
    return raw_paths


def _tree_entry(
    repository: Path, commit: str, path: str
) -> tuple[str, str, str] | None:
    output = _git_output(repository, "ls-tree", "-z", commit, "--", path)
    if not output:
        return None
    records = output.rstrip(b"\x00").split(b"\x00")
    if len(records) != 1:
        raise _CheckFailure(f"historical path has multiple tree entries: {path}")
    try:
        header, reported_path = records[0].split(b"\t", 1)
        mode, object_type, object_id = header.split()
    except ValueError as error:
        raise _CheckFailure(f"historical tree entry is malformed: {path}") from error
    if _decode_git_text(reported_path, "historical tree path") != path:
        raise _CheckFailure(f"historical tree entry path mismatch: {path}")
    return mode.decode("ascii"), object_type.decode("ascii"), object_id.decode("ascii")


def _matching_slot(
    raw_path: bytes, config: _DiscoveryConfig
) -> tuple[str, str] | None:
    directory_bytes = config.adr_directory.encode("utf-8")
    if config.adr_directory == ".":
        if not raw_path or b"/" in raw_path:
            return None
        basename_bytes = raw_path
    else:
        prefix = directory_bytes + b"/"
        if not raw_path.startswith(prefix):
            return None
        basename_bytes = raw_path[len(prefix) :]
        if not basename_bytes or b"/" in basename_bytes:
            return None
    try:
        basename = basename_bytes.decode("utf-8")
    except UnicodeDecodeError:
        return None
    match = config.filename_pattern.full_match(basename)
    if match is None:
        return None
    number = match.group("number")
    if not number:
        raise _CheckFailure(
            "repository.filename_pattern matched an empty number group: "
            f"{basename}"
        )
    try:
        path = raw_path.decode("utf-8")
    except UnicodeDecodeError as error:
        raise _CheckFailure(
            "participating historical Git path is not valid UTF-8"
        ) from error
    return f"ADR-{number}", path


def _body_change_error(
    anchor: _AcceptedBodyAnchor, commit: str, path: str
) -> str:
    return (
        f"accepted ADR body changed: {anchor.id}; offending commit {commit}; "
        f"offending historical path {path}; acceptance anchor commit "
        f"{anchor.commit}; acceptance anchor path {anchor.path}"
    )


def _inspect_history(
    repository: Path,
    config: _DiscoveryConfig,
) -> tuple[dict[str, _AcceptedBodyAnchor], list[str]]:
    seals: dict[str, _AcceptedBodyAnchor] = {}
    errors: list[str] = []
    for commit in _commits(repository):
        for raw_path in _changed_paths(repository, commit):
            participating = _matching_slot(raw_path, config)
            if participating is None:
                continue
            expected_id, path = participating
            entry = _tree_entry(repository, commit, path)
            if entry is None:
                continue
            mode, object_type, object_id = entry
            if mode not in _REGULAR_FILE_MODES or object_type != "blob":
                if expected_id in seals:
                    errors.append(
                        f"historical ADR slot {expected_id} is not a regular file "
                        f"after acceptance: commit {commit}; path {path}"
                    )
                continue
            blob = _git_output(repository, "cat-file", "blob", object_id)
            try:
                metadata, decision_body = adr_metadata.parse_adr_bytes(blob)
            except adr_metadata.AdrMetadataError as error:
                if expected_id in seals:
                    errors.append(
                        f"historical ADR {expected_id} is not parseable after "
                        f"acceptance: commit {commit}; path {path}; {error}"
                    )
                continue

            metadata_id = metadata.get("id")
            if metadata_id != expected_id:
                metadata_id_is_sealed = (
                    isinstance(metadata_id, str) and metadata_id in seals
                )
                if expected_id in seals or metadata_id_is_sealed:
                    errors.append(
                        "historical ADR identity changed after acceptance: "
                        f"slot {expected_id}; metadata.id {metadata_id!r}; "
                        f"commit {commit}; path {path}"
                    )
                continue

            anchor = seals.get(expected_id)
            if anchor is None:
                if metadata.get("status") == "accepted":
                    seals[expected_id] = _AcceptedBodyAnchor(
                        expected_id, commit, path, decision_body
                    )
                continue
            if decision_body != anchor.decision_body:
                errors.append(_body_change_error(anchor, commit, path))
    return seals, errors


def _current_errors(
    repository: Path, seals: dict[str, _AcceptedBodyAnchor]
) -> list[str]:
    errors: list[str] = []
    for adr_id in sorted(seals):
        anchor = seals[adr_id]
        try:
            current = canonical_adr.resolve(repository, adr_id)
        except adr_metadata.AdrMetadataError as error:
            errors.append(f"cannot resolve current sealed ADR {adr_id}: {error}")
            continue
        if current.decision_body != anchor.decision_body:
            errors.append(
                f"current accepted ADR body changed: {adr_id}; acceptance anchor "
                f"commit {anchor.commit}; acceptance anchor path {anchor.path}; "
                f"current path {current.path}"
            )
    return errors


def check(repository: Path) -> list[str]:
    """Return controlled accepted-body immutability errors."""

    try:
        root = _validate_repository(repository)
        config = _load_discovery_config(root)
        seals, errors = _inspect_history(root, config)
        errors.extend(_current_errors(root, seals))
        return errors
    except (adr_metadata.AdrMetadataError, _CheckFailure, OSError) as error:
        return [str(error)]


def main() -> int:
    errors = check(Path.cwd())
    if errors:
        print("accepted ADR body immutability: FAILED")
        for error in errors:
            print(error)
        return 1
    print("accepted ADR body immutability: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
