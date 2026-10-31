#!/usr/bin/env python3
"""Run the installed distribution's Python tests in isolated module subprocesses."""
from __future__ import annotations

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import traceback
import unittest

from python_test_runner_protocol import (
    EXIT_CODES,
    FAILURE_KINDS,
    TaskProcessResult,
    classify_results,
    worker_payload as _worker_payload,
)

REPO_ROOT = Path(__file__).resolve().parents[1]
TESTS_DIR = REPO_ROOT / "tests"
AUTO_WORKER_CAP = 4
class InventoryError(Exception):
    """The discovered and scheduled test inventories are not equivalent."""


@dataclass(frozen=True)
class AtomicTask:
    task_id: str
    test_ids: tuple[str, ...]


@dataclass(frozen=True)
class RunSummary:
    workers: int
    atomic_tasks: int
    discovered_tests: int
    executed_tests: int
    wall_seconds: float
    status: str
    failed_tasks: tuple[str, ...]


def _flatten_suite(suite: unittest.TestSuite):
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            yield from _flatten_suite(item)
        else:
            yield item


def discover_test_ids() -> tuple[str, ...]:
    previous_path = list(sys.path)
    try:
        sys.path.insert(0, str(TESTS_DIR))
        suite = unittest.defaultTestLoader.discover(
            start_dir=str(TESTS_DIR),
            pattern="test_*.py",
            top_level_dir=str(TESTS_DIR),
        )
    finally:
        sys.path[:] = previous_path
    tests = list(_flatten_suite(suite))
    failed = sorted(test.id() for test in tests if test.__class__.__name__ == "_FailedTest")
    if failed:
        raise InventoryError(f"discovery failures: {failed}")
    identifiers = [test.id() for test in tests]
    duplicates = sorted(name for name, count in Counter(identifiers).items() if count > 1)
    if duplicates:
        raise InventoryError(f"duplicate discovered IDs: {duplicates}")
    if not identifiers:
        raise InventoryError("discovery produced zero tests")
    return tuple(sorted(identifiers))


def build_tasks(discovered_test_ids: tuple[str, ...]) -> tuple[AtomicTask, ...]:
    grouped: dict[str, list[str]] = {}
    for test_id in discovered_test_ids:
        module = test_id.split(".", 1)[0]
        grouped.setdefault(module, []).append(test_id)
    return tuple(
        AtomicTask(f"module:{module}", tuple(sorted(grouped[module])))
        for module in sorted(grouped)
    )


def validate_partition(
    discovered_test_ids: tuple[str, ...], tasks: tuple[AtomicTask, ...]
) -> None:
    scheduled = [test_id for task in tasks for test_id in task.test_ids]
    counts = Counter(scheduled)
    discovered = set(discovered_test_ids)
    problems = []
    duplicate = sorted(test_id for test_id, count in counts.items() if count > 1)
    missing = sorted(discovered - set(scheduled))
    unknown = sorted(set(scheduled) - discovered)
    empty = sorted(task.task_id for task in tasks if not task.test_ids)
    if duplicate:
        problems.append(f"duplicate scheduled IDs: {duplicate}")
    if missing:
        problems.append(f"missing discovered IDs: {missing}")
    if unknown:
        problems.append(f"unknown scheduled IDs: {unknown}")
    if empty:
        problems.append(f"empty atomic tasks: {empty}")
    if problems:
        raise InventoryError("; ".join(problems))


def default_worker_count(logical_cpu_count: int | None = None) -> int:
    count = os.cpu_count() if logical_cpu_count is None else logical_cpu_count
    if count is None or count < 1:
        count = 1
    return min(count, AUTO_WORKER_CAP)


def validate_worker_count(value: int) -> int:
    if value < 1:
        raise ValueError("worker count must be at least 1")
    return value


def _atomic_write_json(path: Path, payload: dict[str, object]) -> None:
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temporary.write_text(json.dumps(payload, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temporary, path)


def execute_task_spec(spec_path: Path) -> int:
    started = time.perf_counter()
    result_path: Path | None = None
    task_id = "UNKNOWN"
    expected = 0
    try:
        spec = json.loads(spec_path.read_text(encoding="utf-8"))
        task_id = spec["task_id"]
        test_ids = tuple(spec["test_ids"])
        result_path = Path(spec["result_path"])
        expected = len(test_ids)
        suite = unittest.defaultTestLoader.loadTestsFromNames(test_ids)
        if suite.countTestCases() != expected:
            _atomic_write_json(
                result_path,
                _worker_payload(
                    task_id,
                    expected,
                    None,
                    time.perf_counter() - started,
                    "INVENTORY_FAILURE",
                ),
            )
            return 2
        stream = io.StringIO()
        result = unittest.TextTestRunner(stream=stream, verbosity=2).run(suite)
        sys.stdout.write(stream.getvalue())
        kind = "NONE" if result.wasSuccessful() else "TEST_FAILURE"
        _atomic_write_json(
            result_path,
            _worker_payload(task_id, expected, result, time.perf_counter() - started, kind),
        )
        return 0 if result.wasSuccessful() else 1
    except Exception:
        traceback.print_exc(file=sys.stderr)
        if result_path is not None:
            _atomic_write_json(
                result_path,
                _worker_payload(
                    task_id,
                    expected,
                    None,
                    time.perf_counter() - started,
                    "RUNNER_FAILURE",
                ),
            )
        return 3


def _execute_subprocess(task: AtomicTask, directory: Path) -> TaskProcessResult:
    safe_name = task.task_id.replace(":", "-")
    spec_path = directory / f"{safe_name}.spec.json"
    result_path = directory / f"{safe_name}.result.json"
    _atomic_write_json(
        spec_path,
        {
            "task_id": task.task_id,
            "test_ids": list(task.test_ids),
            "result_path": str(result_path),
        },
    )
    environment = os.environ.copy()
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    environment["PYTHONPATH"] = str(TESTS_DIR)
    started = time.perf_counter()
    completed = subprocess.run(
        [sys.executable, "-B", str(Path(__file__).resolve()), "--_execute-task", str(spec_path)],
        cwd=REPO_ROOT,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    duration = time.perf_counter() - started
    if completed.returncode < 0 or not result_path.is_file():
        return TaskProcessResult(
            task.task_id,
            completed.returncode,
            False,
            0,
            0,
            0,
            0,
            False,
            duration,
            completed.stdout,
            "PROCESS_FAILURE",
        )
    try:
        payload = json.loads(result_path.read_text(encoding="utf-8"))
        kind = payload["failure_kind"]
        expected_exit_codes = {
            "NONE": 0,
            "TEST_FAILURE": 1,
            "INVENTORY_FAILURE": 2,
            "PROCESS_FAILURE": 3,
            "RUNNER_FAILURE": 3,
        }
        if (
            payload["task_id"] != task.task_id
            or kind not in FAILURE_KINDS
            or int(payload["expected_tests"]) != len(task.test_ids)
            or completed.returncode != expected_exit_codes[kind]
            or bool(payload["successful"]) != (kind == "NONE")
        ):
            raise ValueError("inconsistent structured result")
        return TaskProcessResult(
            task.task_id,
            completed.returncode,
            True,
            int(payload["tests_run"]),
            int(payload["failures"]),
            int(payload["errors"]),
            int(payload["skipped"]),
            bool(payload["successful"]),
            duration,
            completed.stdout,
            kind,
        )
    except Exception:
        return TaskProcessResult(
            task.task_id,
            completed.returncode,
            True,
            0,
            0,
            0,
            0,
            False,
            duration,
            completed.stdout + traceback.format_exc(),
            "RUNNER_FAILURE",
        )


def run_controller(workers: int) -> int:
    started = time.perf_counter()
    try:
        discovered = discover_test_ids()
        tasks = build_tasks(discovered)
        validate_partition(discovered, tasks)
    except InventoryError as error:
        print(f"INVENTORY_ERROR={error}")
        return EXIT_CODES["INVENTORY_FAILURE"]
    with tempfile.TemporaryDirectory(prefix="proto-ring-test-runner-") as temporary:
        with ThreadPoolExecutor(max_workers=min(workers, len(tasks))) as executor:
            results = tuple(
                future.result()
                for future in [
                    executor.submit(_execute_subprocess, task, Path(temporary))
                    for task in tasks
                ]
            )
    for result in results:
        print(f"===== TASK {result.task_id} =====")
        print(result.output, end="" if result.output.endswith("\n") else "\n")
        print(f"===== END TASK {result.task_id} =====")
    status, exit_code, executed, failed = classify_results(len(discovered), tasks, results)
    summary = RunSummary(
        workers,
        len(tasks),
        len(discovered),
        executed,
        time.perf_counter() - started,
        status,
        failed,
    )
    print(f"PROTO_RING_TEST_RUNNER={summary.status}")
    print(f"WORKERS={summary.workers}")
    print(f"ATOMIC_TASKS={summary.atomic_tasks}")
    print(f"DISCOVERED_TESTS={summary.discovered_tests}")
    print(f"EXECUTED_TESTS={summary.executed_tests}")
    print(f"WALL_SECONDS={summary.wall_seconds:.6f}")
    print(f"FAILED_TASKS={','.join(summary.failed_tasks) if summary.failed_tasks else 'NONE'}")
    return exit_code


def parse_arguments(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Process-isolated full-suite runner using the installed distribution.",
    )
    parser.add_argument("--workers", type=int, default=None)
    parser.add_argument("--_execute-task", type=Path, help=argparse.SUPPRESS)
    arguments = parser.parse_args(argv)
    if arguments.workers is not None:
        try:
            validate_worker_count(arguments.workers)
        except ValueError as error:
            parser.error(str(error))
    return arguments


def main(argv: list[str] | None = None) -> int:
    arguments = parse_arguments(argv)
    if arguments._execute_task is not None:
        return execute_task_spec(arguments._execute_task)
    workers = default_worker_count() if arguments.workers is None else arguments.workers
    return run_controller(workers)


if __name__ == "__main__":
    raise SystemExit(main())
