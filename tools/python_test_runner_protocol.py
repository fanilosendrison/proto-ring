"""Structured child-process results for the Python test runner."""
from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
from typing import Protocol
import unittest

FAILURE_KINDS = frozenset(
    {"NONE", "TEST_FAILURE", "INVENTORY_FAILURE", "PROCESS_FAILURE", "RUNNER_FAILURE"}
)
EXIT_CODES = {
    "PASS": 0,
    "TEST_FAILURE": 1,
    "INVENTORY_FAILURE": 2,
    "PROCESS_FAILURE": 3,
    "RUNNER_FAILURE": 3,
}


class TaskLike(Protocol):
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


def worker_payload(
    task_id: str,
    expected: int,
    result: unittest.TestResult | None,
    duration: float,
    failure_kind: str,
) -> dict[str, object]:
    return {
        "task_id": task_id,
        "expected_tests": expected,
        "tests_run": 0 if result is None else result.testsRun,
        "failures": 0 if result is None else len(result.failures),
        "errors": 0 if result is None else len(result.errors),
        "skipped": 0 if result is None else len(result.skipped),
        "successful": False if result is None else result.wasSuccessful(),
        "duration_seconds": duration,
        "failure_kind": failure_kind,
    }


def classify_results(
    discovered_count: int,
    tasks: tuple[TaskLike, ...],
    results: tuple[TaskProcessResult, ...],
) -> tuple[str, int, int, tuple[str, ...]]:
    expected_ids = [task.task_id for task in tasks]
    result_ids = [result.task_id for result in results]
    runner_problem = (
        any(count > 1 for count in Counter(result_ids).values())
        or bool(set(expected_ids) - set(result_ids))
        or bool(set(result_ids) - set(expected_ids))
    )
    kinds = {result.failure_kind for result in results}
    if any(not result.result_present or result.returncode < 0 for result in results):
        kinds.add("PROCESS_FAILURE")
    executed = sum(result.tests_run for result in results)
    runner_problem = runner_problem or executed != discovered_count
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
        result.task_id
        for result in results
        if result.result_present
        and result.returncode == 0
        and result.failure_kind == "NONE"
        and result.successful
    }
    failed = tuple(sorted(set(expected_ids) - passed))
    return status, EXIT_CODES[status], executed, failed
