from __future__ import annotations

import io
import json
import os
from pathlib import Path
import shutil
import socket
import ssl
import subprocess
import sys
import tempfile
from typing import Callable
import urllib.error

from conformance_corpus_test_support import decode_transport


class FixtureError(ValueError):
    pass


class _HttpResponse:
    def __init__(self, status: int, body: bytes) -> None:
        self.status = status
        self._body = body

    def __enter__(self):
        return self

    def __exit__(self, *_args: object) -> None:
        return None

    def read(self) -> bytes:
        return self._body


class ProviderSequence:
    def __init__(self, interactions: list[dict[str, object]]) -> None:
        self._interactions = interactions
        self._position = 0

    @property
    def consumed(self) -> int:
        return self._position

    def urlopen(self, *_args: object, **_kwargs: object):
        if self._position >= len(self._interactions):
            raise FixtureError("implementation requested an unsupplied interaction")
        interaction = self._interactions[self._position]
        self._position += 1
        kind = interaction["kind"]
        if kind == "http_response":
            return _HttpResponse(
                int(interaction["status"]), bytes.fromhex(str(interaction["body_hex"]))
            )
        if kind == "http_error":
            status = int(interaction["status"])
            raise urllib.error.HTTPError(
                "https://example.invalid", status, "fixture HTTP error", {}, None
            )
        if kind == "network_error":
            raise urllib.error.URLError(str(interaction["message"]))
        if kind == "tls_verification_error":
            raise ssl.SSLCertVerificationError(1, str(interaction["message"]))
        if kind == "timeout":
            raise socket.timeout(str(interaction["message"]))
        if kind == "io_error":
            raise OSError(str(interaction["message"]))
        raise FixtureError(f"unsupported provider interaction: {kind}")

    def require_consumed(self) -> None:
        if self._position != len(self._interactions):
            raise FixtureError(
                f"unused provider interactions: {len(self._interactions) - self._position}"
            )


def interpolate(value: object, captures: dict[str, str]) -> object:
    if isinstance(value, str):
        result = value
        while "${" in result:
            start = result.index("${")
            end = result.find("}", start)
            if end < 0:
                raise FixtureError("unterminated capture interpolation")
            name = result[start + 2 : end]
            if name not in captures:
                raise FixtureError(f"unbound capture: {name}")
            result = result[:start] + captures[name] + result[end + 1 :]
        return result
    if isinstance(value, list):
        return [interpolate(item, captures) for item in value]
    if isinstance(value, dict):
        return {key: interpolate(item, captures) for key, item in value.items()}
    return value


class RealizedFixture:
    def __init__(self, fixture: dict[str, object]) -> None:
        self.fixture = fixture
        self._temporary: tempfile.TemporaryDirectory[str] | None = None
        self.root: Path | None = None
        self.captures: dict[str, str] = {}
        self.provider: ProviderSequence | None = None

    def __enter__(self):
        kind = self.fixture["kind"]
        if kind == "inline":
            return self
        if kind == "procedural_requirement":
            base_kind = self.fixture["base"]["kind"]
            if base_kind == "inline":
                return self
            if base_kind != "repository_plan":
                raise FixtureError("procedural base must be inline or repository_plan")
        elif kind not in {"provider_observation", "repository_plan"}:
            raise FixtureError(f"unsupported fixture kind: {kind}")
        self._temporary = tempfile.TemporaryDirectory(prefix="proto-ring-corpus-")
        self.root = Path(self._temporary.name) / "repository"
        self.root.mkdir()
        if kind == "provider_observation":
            self._write_binding(self.fixture["binding"])
            self.provider = ProviderSequence(self.fixture["interactions"])
        return self

    def __exit__(self, *_args: object) -> None:
        if self._temporary is not None:
            self._temporary.cleanup()

    def _write_binding(self, binding: dict[str, object]) -> None:
        assert self.root is not None
        path = self.root / str(binding["path"])
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(bytes.fromhex(str(binding["bytes_hex"])))

    @staticmethod
    def _git_environment(extra: dict[str, str]) -> dict[str, str]:
        return {
            **os.environ,
            "GIT_AUTHOR_NAME": "Corpus Author",
            "GIT_AUTHOR_EMAIL": "corpus@example.invalid",
            "GIT_COMMITTER_NAME": "Corpus Committer",
            "GIT_COMMITTER_EMAIL": "corpus@example.invalid",
            "GIT_AUTHOR_DATE": "2001-02-03T04:05:06+0000",
            "GIT_COMMITTER_DATE": "2001-02-03T04:05:06+0000",
            **extra,
        }

    def _path(self, relative: str) -> Path:
        assert self.root is not None
        path = self.root / relative
        try:
            path.resolve(strict=False).relative_to(self.root.resolve())
        except ValueError as error:
            raise FixtureError(f"fixture path escapes root: {relative}") from error
        return path

    def _step(self, step: dict[str, object]) -> None:
        operation = step["op"]
        interpolated = interpolate(step, self.captures)
        if operation == "mkdir":
            self._path(str(interpolated["path"])).mkdir(parents=True, exist_ok=True)
        elif operation == "write_utf8":
            path = self._path(str(interpolated["path"]))
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(str(interpolated["text"]), encoding="utf-8", newline="")
        elif operation == "write_hex":
            path = self._path(str(interpolated["path"]))
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(bytes.fromhex(str(interpolated["hex"])))
        elif operation == "remove":
            path = self._path(str(interpolated["path"]))
            if path.is_dir() and not path.is_symlink():
                shutil.rmtree(path)
            elif path.exists() or path.is_symlink():
                path.unlink()
        elif operation == "symlink":
            path = self._path(str(interpolated["path"]))
            path.parent.mkdir(parents=True, exist_ok=True)
            path.symlink_to(str(interpolated["target"]), target_is_directory=bool(interpolated["target_is_directory"]))
        elif operation == "chmod":
            self._path(str(interpolated["path"])).chmod(int(str(interpolated["mode"]), 8))
        elif operation == "git":
            self._git(interpolated)
        else:
            raise FixtureError(f"step requires callback handling: {operation}")

    def _git(self, step: dict[str, object]) -> None:
        assert self.root is not None
        stdin_hex = step["stdin_hex"]
        completed = subprocess.run(
            ["git", *step["argv"]],
            cwd=self.root,
            input=None if stdin_hex is None else bytes.fromhex(str(stdin_hex)),
            env=self._git_environment(step["environment"]),
            capture_output=True,
            check=False,
        )
        if completed.returncode:
            raise FixtureError(completed.stderr.decode("utf-8", errors="replace"))
        capture = step["capture_stdout_as"]
        if capture is not None:
            try:
                value = completed.stdout.decode("ascii").strip(" \t\r\n\v\f")
            except UnicodeDecodeError as error:
                raise FixtureError("captured Git stdout is not ASCII") from error
            self.captures[str(capture)] = value

    def run(self, callback: Callable[[Path | None, dict[str, object]], object]):
        kind = self.fixture["kind"]
        if kind == "inline":
            return callback(None, decode_transport(self.fixture["input"]))
        if kind == "provider_observation":
            assert self.root is not None
            result = callback(
                self.root,
                {"operation": "provider_observation", "binding_path": self.fixture["binding"]["path"]},
            )
            assert self.provider is not None
            self.provider.require_consumed()
            return result
        if kind == "procedural_requirement":
            base = self.fixture["base"]
            if base["kind"] == "inline":
                arguments = decode_transport(base["input"])
                return callback(None, {**arguments, "fault": self.fixture["condition"]})
            return self._run_plan(base, callback, self.fixture["condition"])
        return self._run_plan(self.fixture, callback, None)

    def _run_plan(self, plan: dict[str, object], callback, fault):
        outputs: dict[str, object] = {}
        for raw_step in plan["steps"]:
            step = interpolate(raw_step, self.captures)
            if step["op"] == "invoke":
                arguments = dict(decode_transport(step["arguments"]))
                if fault is not None:
                    arguments["fault"] = fault
                outputs[str(step["invocation_id"])] = callback(self.root, arguments)
            else:
                self._step(step)
        if outputs:
            return {"invocations": outputs}
        invocation = dict(decode_transport(interpolate(plan["invocation"], self.captures)))
        if fault is not None:
            invocation["fault"] = fault
        return callback(self.root, invocation)


COMMAND_ACTIONS = frozenset({
    "exit", "write_utf8", "append_utf8", "remove", "touch", "chmod", "git",
    "record_arguments_utf8", "assert_environment",
    "exit_code_from_utf8_integer_file_argument",
})


def command_argv(action: dict[str, object]) -> tuple[str, ...]:
    """Return the checked-in CLI realization of one language-neutral action."""
    if action.get("action") not in COMMAND_ACTIONS:
        raise FixtureError(f"unsupported command action: {action.get('action')}")
    payload = json.dumps(action, ensure_ascii=False, sort_keys=True).encode("utf-8").hex()
    return (sys.executable, "-B", os.fspath(Path(__file__).resolve()), "--command-action", payload)


def _action_path(value: object) -> Path:
    root = Path.cwd().resolve()
    path = (root / str(value)).resolve(strict=False)
    try:
        path.relative_to(root)
    except ValueError as error:
        raise FixtureError(f"command action path escapes root: {value}") from error
    return path


def _run_command_action(action: dict[str, object], arguments: list[str]) -> int:
    kind = str(action["action"])
    if kind == "exit":
        return int(action["code"])
    if kind == "git":
        return subprocess.run(
            ["git", *(str(item) for item in action["argv"]), *arguments],
            capture_output=True,
            check=False,
        ).returncode
    if kind == "assert_environment":
        required = action["required"]
        forbidden = action["forbidden"]
        valid = all(os.environ.get(str(key)) == str(value) for key, value in required.items())
        valid = valid and all(str(name) not in os.environ for name in forbidden)
        return 0 if valid else int(action["failure_exit_code"])
    if kind == "exit_code_from_utf8_integer_file_argument":
        index = int(action["argument_index"])
        try:
            raw = _action_path(arguments[index]).read_text(encoding="utf-8").strip()
        except (IndexError, OSError, UnicodeError):
            return 125
        if not raw.isascii() or not raw.isdecimal() or (len(raw) > 1 and raw.startswith("0")):
            return 125
        code = int(raw)
        return code if 0 <= code <= 255 else 125
    path = _action_path(action["path"])
    if kind == "record_arguments_utf8":
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("" if not arguments else "\n".join(arguments) + "\n", encoding="utf-8")
    elif kind == "write_utf8":
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(str(action["text"]), encoding="utf-8")
    elif kind == "append_utf8":
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a", encoding="utf-8") as stream:
            stream.write(str(action["text"]))
    elif kind == "remove":
        if path.exists() or path.is_symlink():
            path.unlink()
    elif kind == "touch":
        path.parent.mkdir(parents=True, exist_ok=True)
        path.touch()
    elif kind == "chmod":
        path.chmod(int(str(action["mode"]), 8))
    else:
        raise FixtureError(f"unsupported command action: {kind}")
    return 0


def materialize(fixture: dict[str, object]) -> RealizedFixture:
    return RealizedFixture(fixture)


if __name__ == "__main__":
    if len(sys.argv) < 3 or sys.argv[1] != "--command-action":
        raise SystemExit(125)
    try:
        decoded = bytes.fromhex(sys.argv[2]).decode("utf-8")
        action = json.loads(decoded)
        if not isinstance(action, dict):
            raise FixtureError("command action must be an object")
        raise SystemExit(_run_command_action(action, sys.argv[3:]))
    except (FixtureError, ValueError, KeyError, TypeError):
        raise SystemExit(125)
