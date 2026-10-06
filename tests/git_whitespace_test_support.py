from __future__ import annotations

import json
from pathlib import Path
import subprocess

ZERO_SHA = "0" * 40


def make_fixture(temporary: str) -> Path:
    fixture = Path(temporary) / "repo"
    fixture.mkdir()
    subprocess.run(["git", "init", "-q", str(fixture)], check=True)
    subprocess.run(
        ["git", "-C", str(fixture), "config", "user.name", "proto-ring Test"],
        check=True,
        capture_output=True,
    )
    subprocess.run(
        ["git", "-C", str(fixture), "config", "user.email", "proto-ring@example.invalid"],
        check=True,
        capture_output=True,
    )
    return fixture


def commit_all(fixture: Path, message: str) -> str:
    subprocess.run(
        ["git", "-C", str(fixture), "add", "-A"], check=True, capture_output=True
    )
    subprocess.run(
        ["git", "-C", str(fixture), "commit", "-q", "-m", message],
        check=True,
        capture_output=True,
    )
    result = subprocess.run(
        ["git", "-C", str(fixture), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def write_event(fixture: Path, name: str, payload: dict) -> Path:
    event_path = fixture.parent / name
    event_path.write_text(json.dumps(payload), encoding="utf-8")
    return event_path
