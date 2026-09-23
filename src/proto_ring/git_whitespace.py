from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Mapping

ZERO_SHA = "0" * 40


def _git_check(root: Path, args: list[str], label: str) -> list[str]:
    result = subprocess.run(
        ["git", *args],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if result.returncode == 0:
        return []

    details: list[str] = []
    for stream in (result.stdout, result.stderr):
        text = stream.decode("utf-8", errors="replace").strip()
        if text:
            details.append(text)

    if details:
        return [f"{label}: " + "\n".join(details)]

    return [f"{label}: git exited with {result.returncode}"]


def _empty_tree_hash(root: Path) -> str | None:
    result = subprocess.run(
        ["git", "hash-object", "-t", "tree", "--stdin"],
        cwd=root,
        input=b"",
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        return None

    return result.stdout.decode("ascii", errors="replace").strip() or None


def _event_payload(
    environment: Mapping[str, str],
    event_name: str,
) -> tuple[dict | None, list[str]]:
    prefix = f"cannot determine GitHub {event_name} whitespace range"
    event_path = environment.get("GITHUB_EVENT_PATH")

    if not event_path:
        return None, [f"{prefix}: GITHUB_EVENT_PATH is missing"]

    try:
        payload = json.loads(Path(event_path).read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        return None, [f"{prefix}: {error}"]

    if not isinstance(payload, dict):
        return None, [f"{prefix}: event JSON must be an object"]

    return payload, []


def _push_range_errors(
    root: Path,
    environment: Mapping[str, str],
) -> list[str]:
    payload, errors = _event_payload(environment, "push")

    if payload is None:
        return errors

    before = payload.get("before")
    after = payload.get("after")

    if (
        not isinstance(before, str)
        or not isinstance(after, str)
        or not before
        or not after
    ):
        return [
            "cannot determine GitHub push whitespace range: "
            "event.before/event.after missing"
        ]

    label = "GitHub push committed-range whitespace check"

    if before == ZERO_SHA:
        empty_tree = _empty_tree_hash(root)
        if empty_tree is None:
            return [
                "cannot determine GitHub push whitespace range: "
                "cannot resolve the Git empty tree"
            ]

        return _git_check(
            root,
            ["diff", "--check", empty_tree, after],
            label,
        )

    return _git_check(
        root,
        ["diff", "--check", f"{before}..{after}"],
        label,
    )


def _pull_request_range_errors(
    root: Path,
    environment: Mapping[str, str],
) -> list[str]:
    payload, errors = _event_payload(environment, "pull_request")

    if payload is None:
        return errors

    pull_request = payload.get("pull_request")
    base = pull_request.get("base") if isinstance(pull_request, dict) else None
    head = pull_request.get("head") if isinstance(pull_request, dict) else None
    base_sha = base.get("sha") if isinstance(base, dict) else None
    head_sha = head.get("sha") if isinstance(head, dict) else None

    if (
        not isinstance(base_sha, str)
        or not isinstance(head_sha, str)
        or not base_sha
        or not head_sha
    ):
        return [
            "cannot determine GitHub pull-request whitespace range: "
            "event.pull_request.base.sha/head.sha missing"
        ]

    return _git_check(
        root,
        ["diff", "--check", f"{base_sha}...{head_sha}"],
        "GitHub pull-request committed-range whitespace check",
    )


def check(
    repository: Path,
    *,
    env: Mapping[str, str],
) -> list[str]:
    errors: list[str] = []

    errors.extend(
        _git_check(
            repository,
            ["diff", "--check"],
            "unstaged whitespace check",
        )
    )

    errors.extend(
        _git_check(
            repository,
            ["diff", "--cached", "--check"],
            "staged whitespace check",
        )
    )

    event_name = env.get("GITHUB_EVENT_NAME")

    if event_name == "push":
        errors.extend(_push_range_errors(repository, env))
    elif event_name == "pull_request":
        errors.extend(_pull_request_range_errors(repository, env))

    return errors
