"""Run-bound evidence export and verification for the local Python test runner."""
from __future__ import annotations

from dataclasses import dataclass, field
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tempfile
from typing import Mapping
import uuid

HANDOFF_FILENAME = "handoff.json"
ARTIFACT_FILENAMES = {
    "actual_observations": "actual-observations.json",
    "execution_metrics": "execution-metrics.txt",
}
RUNNER_STATUSES = frozenset(
    {"PASS", "TEST_FAILURE", "INVENTORY_FAILURE", "PROCESS_FAILURE", "RUNNER_FAILURE"}
)
RUNNER_KEYS = frozenset(
    {"status", "workers", "atomic_tasks", "discovered_tests", "executed_tests", "failed_tasks"}
)
TOP_LEVEL_KEYS = frozenset({"schema_version", "run_id", "runner", "artifacts"})
ARTIFACT_KEYS = frozenset({"path", "size", "sha256"})
RUN_ID_PATTERN = re.compile(r"[0-9a-f]{32}\Z")
SHA256_PATTERN = re.compile(r"[0-9a-f]{64}\Z")


class EvidenceError(RuntimeError):
    """Evidence could not be constructed or verified exactly."""


@dataclass(frozen=True)
class ArtifactBinding:
    path: str
    size: int
    sha256: str


@dataclass
class EvidenceRun:
    run_id: str
    namespace: Path
    artifacts: dict[str, ArtifactBinding] = field(default_factory=dict)


@dataclass(frozen=True)
class RunnerRecord:
    status: str
    workers: int
    atomic_tasks: int
    discovered_tests: int
    executed_tests: int
    failed_tasks: tuple[str, ...]


def sha256_bytes(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def generate_run_id() -> str:
    run_id = uuid.uuid4().hex
    if RUN_ID_PATTERN.fullmatch(run_id) is None:
        raise EvidenceError("generated run ID is not 32 lowercase hexadecimal characters")
    return run_id


def prepare_evidence_run(evidence_root: Path | None) -> EvidenceRun | None:
    if evidence_root is None:
        return None
    run_id = generate_run_id()
    root = Path(os.path.abspath(evidence_root))
    try:
        root.mkdir(parents=True, exist_ok=True)
        namespace = root / run_id
        namespace.mkdir(exist_ok=False)
    except OSError as error:
        raise EvidenceError(f"cannot create unique evidence namespace for {run_id}") from error
    return EvidenceRun(run_id, namespace)


def _require_exact_keys(value: object, expected: frozenset[str], subject: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != expected:
        raise EvidenceError(f"{subject} does not have the exact required key set")
    return value


def _require_nonnegative_integer(value: object, subject: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise EvidenceError(f"{subject} must be a non-negative integer")
    return value


def _require_regular_file(path: Path, subject: str) -> bytes:
    try:
        if not stat.S_ISREG(path.lstat().st_mode):
            raise EvidenceError(f"{subject} is not a regular file")
        return path.read_bytes()
    except FileNotFoundError as error:
        raise EvidenceError(f"{subject} is missing") from error
    except OSError as error:
        raise EvidenceError(f"{subject} cannot be read") from error


def _validate_run(run: EvidenceRun) -> None:
    if RUN_ID_PATTERN.fullmatch(run.run_id) is None:
        raise EvidenceError("run ID is invalid")
    if run.namespace.name != run.run_id:
        raise EvidenceError("evidence namespace basename does not match run ID")
    try:
        if not stat.S_ISDIR(run.namespace.lstat().st_mode):
            raise EvidenceError("evidence namespace is not a directory")
    except OSError as error:
        raise EvidenceError("evidence namespace is unavailable") from error


def _validate_relative_filename(filename: object, expected: str) -> str:
    if not isinstance(filename, str) or filename != expected:
        raise EvidenceError("artifact path is not the exact permitted filename")
    path = Path(filename)
    if path.is_absolute() or path.parts != (filename,) or filename in {".", ".."}:
        raise EvidenceError("artifact path escapes its run namespace")
    return filename


def _atomic_write_bytes(destination: Path, content: bytes) -> None:
    temporary_path: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="wb",
            prefix=f".{destination.name}.",
            suffix=".tmp",
            dir=destination.parent,
            delete=False,
        ) as stream:
            temporary_path = Path(stream.name)
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_path, destination)
    except OSError as error:
        raise EvidenceError(f"cannot atomically write {destination.name}") from error
    finally:
        if temporary_path is not None:
            try:
                temporary_path.unlink(missing_ok=True)
            except OSError:
                pass


def export_artifacts(run: EvidenceRun, source_artifacts: Mapping[str, Path]) -> None:
    _validate_run(run)
    if set(source_artifacts) != set(ARTIFACT_FILENAMES):
        raise EvidenceError("source artifact mapping does not have the exact required keys")
    bindings: dict[str, ArtifactBinding] = {}
    for key, expected_filename in ARTIFACT_FILENAMES.items():
        filename = _validate_relative_filename(expected_filename, expected_filename)
        destination = run.namespace / filename
        if destination.parent != run.namespace:
            raise EvidenceError("artifact destination escapes its run namespace")
        source_bytes = _require_regular_file(Path(source_artifacts[key]), f"source artifact {key}")
        source_size = len(source_bytes)
        source_sha256 = sha256_bytes(source_bytes)
        _atomic_write_bytes(destination, source_bytes)
        destination_bytes = _require_regular_file(destination, f"exported artifact {key}")
        if destination_bytes != source_bytes:
            raise EvidenceError(f"exported artifact {key} does not preserve exact bytes")
        if len(destination_bytes) != source_size:
            raise EvidenceError(f"exported artifact {key} has the wrong size")
        if sha256_bytes(destination_bytes) != source_sha256:
            raise EvidenceError(f"exported artifact {key} has the wrong SHA-256")
        bindings[key] = ArtifactBinding(filename, source_size, source_sha256)
    run.artifacts = bindings


def _canonical_json_bytes(payload: dict[str, object]) -> bytes:
    return (
        json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode("utf-8")


def _handoff_payload(run: EvidenceRun, runner: RunnerRecord) -> dict[str, object]:
    _validate_run(run)
    if set(run.artifacts) != set(ARTIFACT_FILENAMES):
        raise EvidenceError("complete exported artifacts are unavailable")
    return {
        "schema_version": 1,
        "run_id": run.run_id,
        "runner": {
            "status": runner.status,
            "workers": runner.workers,
            "atomic_tasks": runner.atomic_tasks,
            "discovered_tests": runner.discovered_tests,
            "executed_tests": runner.executed_tests,
            "failed_tasks": list(runner.failed_tasks),
        },
        "artifacts": {
            key: {
                "path": run.artifacts[key].path,
                "size": run.artifacts[key].size,
                "sha256": run.artifacts[key].sha256,
            }
            for key in ARTIFACT_FILENAMES
        },
    }


def finalize_handoff(run: EvidenceRun, runner: RunnerRecord) -> Path:
    handoff = run.namespace / HANDOFF_FILENAME
    _atomic_write_bytes(handoff, _canonical_json_bytes(_handoff_payload(run, runner)))
    verify_handoff(handoff)
    return handoff.absolute()


def verify_handoff(handoff_path: Path) -> dict[str, object]:
    handoff = Path(handoff_path)
    if handoff.name != HANDOFF_FILENAME:
        raise EvidenceError("handoff filename is invalid")
    raw = _require_regular_file(handoff, "handoff")
    try:
        text = raw.decode("utf-8", errors="strict")
        parsed = json.loads(text)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise EvidenceError("handoff is not valid UTF-8 JSON") from error
    payload = _require_exact_keys(parsed, TOP_LEVEL_KEYS, "handoff")
    if type(payload["schema_version"]) is not int or payload["schema_version"] != 1:
        raise EvidenceError("handoff schema version is unsupported")
    run_id = payload["run_id"]
    if not isinstance(run_id, str) or RUN_ID_PATTERN.fullmatch(run_id) is None:
        raise EvidenceError("handoff run ID is invalid")
    if handoff.parent.name != run_id:
        raise EvidenceError("handoff parent does not match run ID")
    runner = _require_exact_keys(payload["runner"], RUNNER_KEYS, "runner record")
    if not isinstance(runner["status"], str) or runner["status"] not in RUNNER_STATUSES:
        raise EvidenceError("runner status is unknown")
    for key in ("workers", "atomic_tasks", "discovered_tests", "executed_tests"):
        _require_nonnegative_integer(runner[key], f"runner {key}")
    failed_tasks = runner["failed_tasks"]
    if not isinstance(failed_tasks, list) or not all(isinstance(item, str) for item in failed_tasks):
        raise EvidenceError("runner failed_tasks must be an array of strings")
    artifacts = _require_exact_keys(
        payload["artifacts"], frozenset(ARTIFACT_FILENAMES), "artifacts record"
    )
    for key, expected_filename in ARTIFACT_FILENAMES.items():
        record = _require_exact_keys(artifacts[key], ARTIFACT_KEYS, f"artifact {key}")
        filename = _validate_relative_filename(record["path"], expected_filename)
        size = _require_nonnegative_integer(record["size"], f"artifact {key} size")
        digest = record["sha256"]
        if not isinstance(digest, str) or SHA256_PATTERN.fullmatch(digest) is None:
            raise EvidenceError(f"artifact {key} SHA-256 is invalid")
        content = _require_regular_file(handoff.parent / filename, f"artifact {key}")
        if len(content) != size:
            raise EvidenceError(f"artifact {key} size does not match")
        if sha256_bytes(content) != digest:
            raise EvidenceError(f"artifact {key} SHA-256 does not match")
    if raw != _canonical_json_bytes(payload):
        raise EvidenceError("handoff is not canonical UTF-8 JSON")
    return payload
