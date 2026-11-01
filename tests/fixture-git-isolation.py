"""Defensive integration oracle for the controlled fixture Git environment."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from unittest import mock

from conformance_fixture_runner import _execute_receipt_bound_action, materialize


def verify() -> None:
    fixture = {
        "kind": "repository_plan",
        "steps": [],
        "invocation": {"type": "null", "value": None},
    }
    with materialize(fixture) as realized:
        assert realized.root is not None
        with tempfile.TemporaryDirectory(prefix="proto-ring-git-canary-") as directory:
            control = Path(directory)
            template_hooks = control / "template/hooks"
            configured_hooks = control / "configured-hooks"
            template_hooks.mkdir(parents=True)
            configured_hooks.mkdir()
            canary = "#!/bin/sh\nprintf executed > hook-executed\nexit 99\n"
            for path in (template_hooks / "pre-commit", configured_hooks / "pre-commit"):
                path.write_text(canary, encoding="utf-8")
                path.chmod(0o755)
            global_config = control / "global.gitconfig"
            global_config.write_text(
                f"[init]\n\ttemplateDir = {json.dumps(str(control / 'template'))}\n",
                encoding="utf-8",
            )
            untrusted = {
                "GIT_CONFIG_GLOBAL": str(global_config),
                "GIT_CONFIG_COUNT": "1",
                "GIT_CONFIG_KEY_0": "core.hooksPath",
                "GIT_CONFIG_VALUE_0": str(configured_hooks),
                "GIT_TEMPLATE_DIR": str(control / "template"),
            }
            with mock.patch.dict(os.environ, untrusted, clear=False):
                _step(realized, ["init", "--quiet"])
                (realized.root / "tracked.txt").write_text("tracked\n", encoding="utf-8")
                _step(realized, ["add", "tracked.txt"])
                subprocess.run(
                    ["git", "-C", str(realized.root), "config", "core.hooksPath", str(configured_hooks)],
                    check=True,
                )
                _step(realized, ["commit", "-q", "-m", "accepted"])
            assert not (realized.root / "hook-executed").exists()
            subprocess.run(
                ["git", "-C", str(realized.root), "config", "filter.canary.clean", "false"],
                check=True,
            )
            try:
                _step(realized, ["add", "clean.txt"])
            except ValueError as error:
                assert "outside the fixture surface" in str(error)
            else:
                raise AssertionError("executable local Git configuration must be rejected")
            subprocess.run(
                ["git", "-C", str(realized.root), "config", "--unset-all", "filter.canary.clean"],
                check=True,
            )

            (realized.root / "tracked.txt").write_text("changed\n", encoding="utf-8")
            subprocess.run(["git", "-C", str(realized.root), "add", "tracked.txt"], check=True)
            receipts = control / "receipts"
            receipts.mkdir()
            git_executable = shutil.which("git")
            assert git_executable is not None
            trusted_environment = {
                name: os.environ[name]
                for name in ("PATH", "SYSTEMROOT", "WINDIR", "PATHEXT", "COMSPEC")
                if name in os.environ
            }
            action = {
                "action": "git",
                "argv": ["commit", "-q", "-m", "changed B"],
                "_fixture_git_executable": str(Path(git_executable).resolve()),
                "_fixture_git_environment": trusted_environment,
            }
            payload = json.dumps(action, separators=(",", ":")).encode()
            digest = hashlib.sha256(payload).hexdigest()
            previous = Path.cwd()
            try:
                os.chdir(realized.root)
                with mock.patch.dict(os.environ, untrusted, clear=False):
                    code = _execute_receipt_bound_action([
                        "--command-action", payload.hex(),
                        "--receipt-directory", str(receipts),
                        "--request-identity", "request",
                        "--action-identity", "action",
                        "--action-digest", digest,
                        "--",
                    ])
            finally:
                os.chdir(previous)
            assert code == 0
            assert not (realized.root / "hook-executed").exists()
            records = list(receipts.glob("*.json"))
            assert len(records) == 1
            assert json.loads(records[0].read_text(encoding="utf-8"))["return_code"] == 0


def _step(realized, argv: list[str]) -> None:
    realized._step({
        "op": "git", "argv": argv, "stdin_hex": None,
        "environment": {}, "capture_stdout_as": None,
    })
