"""Shared Git-worktree fixtures for Repository Integrity tests."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from proto_ring.repository_integrity import (
    CommandObligation,
    IntegrityProfile,
    evaluate,
)


class RepositoryFixtureTestCase(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        subprocess.run(
            ["git", "init", "-q", os.fspath(self.repo)],
            check=True,
            capture_output=True,
            text=True,
        )
        self.git("config", "user.name", "Proto Ring Test")
        self.git("config", "user.email", "proto-ring@example.invalid")
        self.git("config", "commit.gpgsign", "false")
        self.write("tracked.txt", "original\n")
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "initial")

    def git(self, *args: str) -> subprocess.CompletedProcess[str]:
        return self.git_in(self.repo, *args)

    def git_in(
        self, repository: Path, *args: str
    ) -> subprocess.CompletedProcess[str]:
        completed = subprocess.run(
            ["git", "-C", os.fspath(repository), *args],
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            raise AssertionError(
                f"git {' '.join(args)} failed: {completed.stderr.strip()}"
            )
        return completed

    def write(self, relative: str, content: str) -> Path:
        path = self.repo / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def environment(self) -> dict[str, str]:
        return dict(os.environ)

    def run_integrity(
        self,
        obligations: list[CommandObligation],
        *,
        continue_after: bool = True,
        env: dict[str, str] | None = None,
        repository: Path | None = None,
    ):
        return evaluate(
            repository if repository is not None else self.repo,
            IntegrityProfile(tuple(obligations), continue_after),
            env=self.environment() if env is None else env,
            evaluation_context_identity="test-runtime-context",
        )

    def python(self, code: str) -> tuple[str, ...]:
        return (sys.executable, "-c", code)

    def exit_code(self, code: int) -> tuple[str, ...]:
        return self.python(f"import sys\nsys.exit({code})")

    def write_code(self, relative: str, content: str) -> tuple[str, ...]:
        target = self.repo / relative
        return self.python(
            "from pathlib import Path\n"
            f"Path({os.fspath(target)!r}).write_text({content!r}, encoding='utf-8')"
        )

    def delete_code(self, relative: str) -> tuple[str, ...]:
        target = self.repo / relative
        return self.python(
            "from pathlib import Path\n" f"Path({os.fspath(target)!r}).unlink()"
        )

    def marker_code(self, marker: Path, content: str = "ran") -> tuple[str, ...]:
        return self.python(
            "from pathlib import Path\n"
            f"Path({os.fspath(marker)!r}).write_text({content!r})"
        )

    def append_code(self, log: Path, token: str) -> tuple[str, ...]:
        return self.python(
            "from pathlib import Path\n"
            f"path = Path({os.fspath(log)!r})\n"
            "path.write_text((path.read_text() if path.exists() else '') + "
            f"{token!r})"
        )

    def status_porcelain(self) -> str:
        return self.git("status", "--porcelain").stdout
