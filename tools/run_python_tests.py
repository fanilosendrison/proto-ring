#!/usr/bin/env python3
"""Run the complete Python test inventory in isolated atomic subprocesses."""
from __future__ import annotations

import argparse
import ast
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, replace
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

REPO_ROOT = Path(__file__).resolve().parents[1]
TESTS_DIR = REPO_ROOT / "tests"
SRC_DIR = REPO_ROOT / "src"
CONFORMANCE_MODULE = "test_conformance_corpus"
AUTO_WORKER_CAP = 4
STATEFUL_METHODS = (
    "test_08_every_vector_executes_and_current_python_conforms",
    "test_09_historical_resolution_transport_and_projection_audits",
    "test_10_execution_evidence_and_final_metrics",
)
STATEFUL_TEST_IDS = tuple(
    f"{CONFORMANCE_MODULE}.ConformanceCorpusTest.{method}"
    for method in STATEFUL_METHODS
)
SHARED_ARTIFACTS = (
    Path("/tmp/proto-ring-61-actual-observations.json"),
    Path("/tmp/proto-ring-61-execution-metrics.txt"),
)
FAILURE_KINDS = frozenset({"NONE", "TEST_FAILURE", "INVENTORY_FAILURE", "PROCESS_FAILURE", "RUNNER_FAILURE"})
EXIT_CODES = {"PASS": 0, "TEST_FAILURE": 1, "INVENTORY_FAILURE": 2, "PROCESS_FAILURE": 3, "RUNNER_FAILURE": 3}

class InventoryError(Exception):
    """The discovered and scheduled test inventories are not equivalent."""

@dataclass(frozen=True)
class AtomicTask:
    task_id: str
    test_ids: tuple[str, ...]

@dataclass(frozen=True)
class TaskProcessResult:
    task_id: str
    returncode: int
    result_present: bool
    tests_run: int
    failures: int
    errors: int
    skipped: int
    successful: bool
    duration_seconds: float
    output: str
    failure_kind: str

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
        sys.path[:0] = [str(SRC_DIR), str(TESTS_DIR)]
        suite = unittest.defaultTestLoader.discover(
            start_dir=str(TESTS_DIR), pattern="test_*.py", top_level_dir=str(TESTS_DIR)
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
    discovered = set(discovered_test_ids)
    missing_stateful = sorted(set(STATEFUL_TEST_IDS) - discovered)
    if missing_stateful:
        raise InventoryError(f"missing stateful IDs: {missing_stateful}")
    prefix = f"{CONFORMANCE_MODULE}."
    static_ids = tuple(sorted(
        test_id for test_id in discovered_test_ids
        if test_id.startswith(prefix) and test_id not in STATEFUL_TEST_IDS
    ))
    grouped: dict[str, list[str]] = {}
    for test_id in discovered_test_ids:
        module = test_id.split(".", 1)[0]
        if module != CONFORMANCE_MODULE:
            grouped.setdefault(module, []).append(test_id)
    module_tasks = tuple(
        AtomicTask(f"module:{module}", tuple(sorted(grouped[module])))
        for module in sorted(grouped)
    )
    return (
        AtomicTask("corpus-stateful", STATEFUL_TEST_IDS),
        AtomicTask("corpus-static", static_ids),
        *module_tasks,
    )

def validate_partition(discovered_test_ids: tuple[str, ...], tasks: tuple[AtomicTask, ...]) -> None:
    scheduled = [test_id for task in tasks for test_id in task.test_ids]
    counts = Counter(scheduled)
    discovered = set(discovered_test_ids)
    duplicate = sorted(test_id for test_id, count in counts.items() if count > 1)
    missing = sorted(discovered - set(scheduled))
    unknown = sorted(set(scheduled) - discovered)
    empty = sorted(task.task_id for task in tasks if not task.test_ids)
    problems = []
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

def validate_shared_artifact_ownership() -> frozenset[str]:
    literals = tuple(str(path) for path in SHARED_ARTIFACTS)
    conformance_path = TESTS_DIR / f"{CONFORMANCE_MODULE}.py"
    for path in sorted(TESTS_DIR.glob("test_*.py")):
        if path != conformance_path:
            text = path.read_text(encoding="utf-8")
            found = sorted(literal for literal in literals if literal in text)
            if found:
                raise InventoryError(f"shared artifact literals in {path.name}: {found}")
    tree = ast.parse(conformance_path.read_text(encoding="utf-8"), filename=str(conformance_path))
    owners: set[str] = set()
    for node in tree.body:
        if isinstance(node, ast.ClassDef) and node.name == "ConformanceCorpusTest":
            for method in node.body:
                if not isinstance(method, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    continue
                references_name = any(
                    isinstance(child, ast.Name) and child.id == "ACTUAL_OBSERVATIONS_PATH"
                    for child in ast.walk(method)
                )
                references_metrics = any(
                    isinstance(child, ast.Constant)
                    and child.value == str(SHARED_ARTIFACTS[1])
                    for child in ast.walk(method)
                )
                if references_name or references_metrics:
                    owners.add(method.name)
    unexpected = sorted(owners - set(STATEFUL_METHODS))
    if unexpected:
        raise InventoryError(f"shared artifact ownership outside stateful methods: {unexpected}")
    return frozenset(owners)

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

def _worker_payload(task_id: str, expected: int, result: unittest.TestResult | None,
                    duration: float, failure_kind: str) -> dict[str, object]:
    return {
        "task_id": task_id, "expected_tests": expected,
        "tests_run": 0 if result is None else result.testsRun,
        "failures": 0 if result is None else len(result.failures),
        "errors": 0 if result is None else len(result.errors),
        "skipped": 0 if result is None else len(result.skipped),
        "expected_failures": 0 if result is None else len(result.expectedFailures),
        "unexpected_successes": 0 if result is None else len(result.unexpectedSuccesses),
        "successful": False if result is None else result.wasSuccessful(),
        "duration_seconds": duration, "failure_kind": failure_kind,
    }

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
            payload = _worker_payload(task_id, expected, None, time.perf_counter() - started,
                                      "INVENTORY_FAILURE")
            _atomic_write_json(result_path, payload)
            return 2
        stream = io.StringIO()
        result = unittest.TextTestRunner(stream=stream, verbosity=2).run(suite)
        output = stream.getvalue()
        sys.stdout.write(output)
        kind = "NONE" if result.wasSuccessful() else "TEST_FAILURE"
        _atomic_write_json(
            result_path, _worker_payload(task_id, expected, result,
                                         time.perf_counter() - started, kind)
        )
        return 0 if result.wasSuccessful() else 1
    except Exception:
        traceback.print_exc(file=sys.stderr)
        if result_path is not None:
            try:
                _atomic_write_json(
                    result_path, _worker_payload(task_id, expected, None,
                                                 time.perf_counter() - started, "RUNNER_FAILURE")
                )
            except Exception:
                traceback.print_exc(file=sys.stderr)
        return 3

def _execute_subprocess(task: AtomicTask, directory: Path) -> TaskProcessResult:
    try:
        return _execute_subprocess_inner(task, directory)
    except Exception:
        return TaskProcessResult(task.task_id, 3, False, 0, 0, 0, 0, False, 0.0,
                                 traceback.format_exc(), "RUNNER_FAILURE")

def _execute_subprocess_inner(task: AtomicTask, directory: Path) -> TaskProcessResult:
    safe_name = task.task_id.replace(":", "-")
    spec_path = directory / f"{safe_name}.spec.json"
    result_path = directory / f"{safe_name}.result.json"
    _atomic_write_json(spec_path, {
        "task_id": task.task_id, "test_ids": list(task.test_ids),
        "result_path": str(result_path),
    })
    environment = os.environ.copy()
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    environment["PYTHONPATH"] = os.pathsep.join((str(SRC_DIR), str(TESTS_DIR)))
    started = time.perf_counter()
    completed = subprocess.run(
        [sys.executable, "-B", str(Path(__file__).resolve()), "--_execute-task", str(spec_path)],
        cwd=REPO_ROOT, env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )
    duration = time.perf_counter() - started
    if completed.returncode < 0 or not result_path.is_file():
        return TaskProcessResult(task.task_id, completed.returncode, False, 0, 0, 0, 0,
                                 False, duration, completed.stdout, "PROCESS_FAILURE")
    try:
        payload = json.loads(result_path.read_text(encoding="utf-8"))
        kind = payload["failure_kind"]
        result_exit_codes = {
            "NONE": 0, "TEST_FAILURE": 1, "INVENTORY_FAILURE": 2,
            "PROCESS_FAILURE": 3, "RUNNER_FAILURE": 3,
        }
        if (payload["task_id"] != task.task_id or kind not in FAILURE_KINDS
                or int(payload["expected_tests"]) != len(task.test_ids)
                or completed.returncode != result_exit_codes[kind]
                or bool(payload["successful"]) != (kind == "NONE")):
            raise ValueError("inconsistent structured result")
        return TaskProcessResult(
            task.task_id, completed.returncode, True, int(payload["tests_run"]),
            int(payload["failures"]), int(payload["errors"]), int(payload["skipped"]),
            bool(payload["successful"]), duration, completed.stdout, kind,
        )
    except Exception:
        return TaskProcessResult(task.task_id, completed.returncode, True, 0, 0, 0, 0,
                                 False, duration, completed.stdout + traceback.format_exc(),
                                 "RUNNER_FAILURE")

def classify_results(discovered_count: int, tasks: tuple[AtomicTask, ...],
                     results: tuple[TaskProcessResult, ...]) -> tuple[str, int, int, tuple[str, ...]]:
    expected_ids = [task.task_id for task in tasks]
    result_ids = [result.task_id for result in results]
    duplicate_results = any(count > 1 for count in Counter(result_ids).values())
    missing_results = set(expected_ids) - set(result_ids)
    runner_problem = duplicate_results or bool(missing_results) or bool(set(result_ids) - set(expected_ids))
    kinds = {result.failure_kind for result in results}
    if any(not result.result_present or result.returncode < 0 for result in results):
        kinds.add("PROCESS_FAILURE")
    if not kinds.issubset(FAILURE_KINDS):
        runner_problem = True
    executed = sum(result.tests_run for result in results)
    if executed != discovered_count:
        runner_problem = True
    if "INVENTORY_FAILURE" in kinds:
        status = "INVENTORY_FAILURE"
    elif "PROCESS_FAILURE" in kinds:
        status = "PROCESS_FAILURE"
    elif runner_problem or "RUNNER_FAILURE" in kinds:
        status = "RUNNER_FAILURE"
    elif "TEST_FAILURE" in kinds:
        status = "TEST_FAILURE"
    else:
        status = "PASS"
    passed = {
        result.task_id for result in results
        if result.result_present and result.returncode == 0
        and result.failure_kind == "NONE" and result.successful
    }
    failed = tuple(sorted(set(expected_ids) - passed))
    return status, EXIT_CODES[status], executed, failed

def run_controller(workers: int) -> int:
    started = time.perf_counter()
    try:
        discovered = discover_test_ids()
        tasks = build_tasks(discovered)
        validate_partition(discovered, tasks)
        validate_shared_artifact_ownership()
    except InventoryError as error:
        print(f"INVENTORY_ERROR={error}")
        _print_summary(RunSummary(workers, 0, 0, 0, time.perf_counter() - started,
                                  "INVENTORY_FAILURE", ()))
        return 2
    for artifact in SHARED_ARTIFACTS:
        artifact.unlink(missing_ok=True)
    with tempfile.TemporaryDirectory(prefix="proto-ring-test-runner-") as temporary:
        with ThreadPoolExecutor(max_workers=min(workers, len(tasks))) as executor:
            futures = [executor.submit(_execute_subprocess, task, Path(temporary)) for task in tasks]
            results = tuple(future.result() for future in futures)
    for result in results:
        print(f"===== TASK {result.task_id} =====")
        print(result.output, end="" if result.output.endswith("\n") else "\n")
        print(f"===== END TASK {result.task_id} =====")
    stateful_index = next(index for index, task in enumerate(tasks) if task.task_id == "corpus-stateful")
    if results[stateful_index].failure_kind == "NONE" and not all(path.is_file() for path in SHARED_ARTIFACTS):
        results = tuple(
            replace(result, successful=False, failure_kind="RUNNER_FAILURE")
            if index == stateful_index else result
            for index, result in enumerate(results)
        )
    status, exit_code, executed, failed = classify_results(len(discovered), tasks, results)
    _print_summary(RunSummary(workers, len(tasks), len(discovered), executed,
                              time.perf_counter() - started, status, failed))
    return exit_code

def _print_summary(summary: RunSummary) -> None:
    print(f"PROTO_RING_TEST_RUNNER={summary.status}")
    print(f"WORKERS={summary.workers}")
    print(f"ATOMIC_TASKS={summary.atomic_tasks}")
    print(f"DISCOVERED_TESTS={summary.discovered_tests}")
    print(f"EXECUTED_TESTS={summary.executed_tests}")
    print(f"WALL_SECONDS={summary.wall_seconds:.6f}")
    print(f"FAILED_TASKS={','.join(summary.failed_tasks) if summary.failed_tasks else 'NONE'}")

def parse_arguments(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=("Process-isolated local full-suite runner with exact unittest inventory. "
                     "Sequential GitHub CI remains separate and authoritative."),
    )
    parser.add_argument(
        "--workers", type=int, default=None,
        help="child-process worker count (default: min(logical CPUs, 4))",
    )
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
