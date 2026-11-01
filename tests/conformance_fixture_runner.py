from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import secrets
import shutil
import sys
import tempfile
from typing import Callable

from conformance_corpus_test_support import decode_transport

_POLICY_SPEC = importlib.util.spec_from_file_location(
    "proto_ring_fixture_git_policy", Path(__file__).with_name("fixture-git-policy.py")
)
if _POLICY_SPEC is None or _POLICY_SPEC.loader is None:
    raise RuntimeError("fixture Git policy cannot be loaded")
_GIT_POLICY = importlib.util.module_from_spec(_POLICY_SPEC)
_POLICY_SPEC.loader.exec_module(_GIT_POLICY)
_PROVIDER_SPEC = importlib.util.spec_from_file_location(
    "proto_ring_provider_sequence", Path(__file__).with_name("provider-sequence.py")
)
if _PROVIDER_SPEC is None or _PROVIDER_SPEC.loader is None:
    raise RuntimeError("fixture provider sequence cannot be loaded")
_PROVIDER_MODULE = importlib.util.module_from_spec(_PROVIDER_SPEC)
_PROVIDER_SPEC.loader.exec_module(_PROVIDER_MODULE)
ProviderSequence = _PROVIDER_MODULE.ProviderSequence


class FixtureError(ValueError):
    pass


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
        argv = [str(value) for value in step["argv"]]
        _validate_git_arguments(argv)
        if not isinstance(step["environment"], dict) or not all(
            isinstance(name, str) and isinstance(value, str)
            for name, value in step["environment"].items()
        ):
            raise FixtureError("Git fixture environment must contain string pairs")
        # Fixture-supplied values are schema-checked but intentionally not inherited by Git.
        stdin_hex = step["stdin_hex"]
        completed = _GIT_POLICY.run(
            argv,
            cwd=self.root,
            input_bytes=None if stdin_hex is None else bytes.fromhex(str(stdin_hex)),
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


def _action_path(value: object) -> Path:
    root = Path.cwd().resolve()
    path = (root / str(value)).resolve(strict=False)
    try:
        path.relative_to(root)
    except ValueError as error:
        raise FixtureError(f"command action path escapes root: {value}") from error
    return path


def _validate_git_arguments(arguments: list[str]) -> None:
    if not _GIT_POLICY.allowed(arguments):
        raise FixtureError(
            f"Git fixture arguments are not allowed by the corpus surface: {tuple(arguments)!r}"
        )


def _run_command_action(action: dict[str, object], arguments: list[str]) -> int:
    kind = str(action["action"])
    if kind == "abnormal_termination":
        os.abort()
    if kind not in COMMAND_ACTIONS:
        raise FixtureError(f"unsupported command action: {kind}")
    if kind == "exit":
        code = int(action["code"])
        if not 0 <= code <= 255:
            raise FixtureError("exit code is outside the process range")
        return code
    if kind == "git":
        git_arguments = [*(str(item) for item in action["argv"]), *arguments]
        _validate_git_arguments(git_arguments)
        executable = Path(str(action["_fixture_git_executable"]))
        trusted_environment = action["_fixture_git_environment"]
        if (
            not executable.is_absolute()
            or not executable.is_file()
            or not isinstance(trusted_environment, dict)
            or not all(isinstance(key, str) and isinstance(value, str)
                       for key, value in trusted_environment.items())
        ):
            raise FixtureError("fixture Git execution identity is invalid")
        return _GIT_POLICY.run(
            git_arguments,
            cwd=Path.cwd(),
            git_executable=executable,
            trusted_environment=trusted_environment,
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
        except (IndexError, OSError, UnicodeError) as error:
            raise FixtureError("integer-file action arguments are invalid") from error
        if not raw.isascii() or not raw.isdecimal() or (len(raw) > 1 and raw.startswith("0")):
            raise FixtureError("integer-file action content is invalid")
        code = int(raw)
        if not 0 <= code <= 255:
            raise FixtureError("integer-file action code is outside the process range")
        return code
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


def _write_completion_receipt(
    directory: Path,
    request_identity: str,
    action_identity: str,
    action_digest: str,
    return_code: int,
) -> None:
    if not directory.is_dir() or directory.resolve().is_relative_to(Path.cwd().resolve()):
        raise FixtureError("receipt directory must be a private external directory")
    receipt = {
        "schema_version": 1,
        "completion": "completed",
        "request_identity": request_identity,
        "action_identity": action_identity,
        "action_digest": action_digest,
        "return_code": return_code,
    }
    suffix = secrets.token_hex(16)
    temporary = directory / f".{action_identity}.{suffix}.tmp"
    completed = directory / f"{action_identity}.{suffix}.json"
    try:
        with temporary.open("x", encoding="utf-8") as stream:
            json.dump(receipt, stream, sort_keys=True, separators=(",", ":"))
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, completed)
    finally:
        if temporary.exists():
            temporary.unlink()


def _execute_receipt_bound_action(arguments: list[str]) -> int:
    flags = (
        "--command-action",
        "--receipt-directory",
        "--request-identity",
        "--action-identity",
        "--action-digest",
    )
    if len(arguments) < 11 or tuple(arguments[0:10:2]) != flags or arguments[10] != "--":
        raise FixtureError("command action internal arguments are malformed")
    payload = bytes.fromhex(arguments[1])
    digest = hashlib.sha256(payload).hexdigest()
    if digest != arguments[9]:
        raise FixtureError("command action digest is invalid")
    action = json.loads(payload.decode("utf-8"))
    if not isinstance(action, dict):
        raise FixtureError("command action must be an object")
    if action.get("action") == "receipt_write_failure":
        return_code = 0
        shutil.rmtree(Path(arguments[3]))
    else:
        return_code = _run_command_action(action, arguments[11:])
    _write_completion_receipt(
        Path(arguments[3]), arguments[5], arguments[7], digest, return_code
    )
    return return_code


def materialize(fixture: dict[str, object]) -> RealizedFixture:
    return RealizedFixture(fixture)


if __name__ == "__main__":
    try:
        raise SystemExit(_execute_receipt_bound_action(sys.argv[1:]))
    except (FixtureError, OSError, UnicodeError, ValueError, KeyError, TypeError):
        raise SystemExit(125)
