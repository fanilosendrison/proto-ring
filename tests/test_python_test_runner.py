from __future__ import annotations

from collections import Counter
import importlib.util
from pathlib import Path
import sys
import unittest

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
RUNNER_PATH = REPOSITORY_ROOT / "tools" / "run_python_tests.py"
sys.path.insert(0, str(RUNNER_PATH.parent))
SPEC_NAME = "proto_ring_python_test_runner"
SPEC = importlib.util.spec_from_file_location(SPEC_NAME, RUNNER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load runner from {RUNNER_PATH}")
runner = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC_NAME] = runner
SPEC.loader.exec_module(runner)


def process_result(
    task_id: str,
    *,
    returncode: int = 0,
    result_present: bool = True,
    tests_run: int = 1,
    failures: int = 0,
    successful: bool = True,
    failure_kind: str = "NONE",
):
    return runner.TaskProcessResult(
        task_id=task_id,
        returncode=returncode,
        result_present=result_present,
        tests_run=tests_run,
        failures=failures,
        errors=0,
        skipped=0,
        successful=successful,
        duration_seconds=0.01,
        output="output\n",
        failure_kind=failure_kind,
    )


class PythonTestRunnerTests(unittest.TestCase):
    def test_current_inventory_partitions_exactly_by_module(self) -> None:
        discovered = runner.discover_test_ids()
        tasks = runner.build_tasks(discovered)
        runner.validate_partition(discovered, tasks)
        scheduled = [test_id for task in tasks for test_id in task.test_ids]
        self.assertEqual(Counter(scheduled), Counter(discovered))
        self.assertEqual(
            [task.task_id for task in tasks],
            sorted(task.task_id for task in tasks),
        )
        self.assertTrue(all(task.task_id.startswith("module:") for task in tasks))
        self.assertIn("module:test_python_test_runner", {task.task_id for task in tasks})

    def test_partition_rejects_duplicate_missing_and_unknown_ids(self) -> None:
        discovered = ("module.Case.test_one", "module.Case.test_two")
        invalid = (
            (
                "duplicate scheduled IDs",
                (
                    runner.AtomicTask("first", (discovered[0],)),
                    runner.AtomicTask("second", discovered),
                ),
            ),
            ("missing discovered IDs", (runner.AtomicTask("only", (discovered[0],)),)),
            (
                "unknown scheduled IDs",
                (runner.AtomicTask("only", (*discovered, "module.Case.test_unknown")),),
            ),
        )
        for message, tasks in invalid:
            with self.subTest(message=message):
                with self.assertRaisesRegex(runner.InventoryError, message):
                    runner.validate_partition(discovered, tasks)

    def test_worker_count_default_and_validation(self) -> None:
        self.assertEqual(runner.default_worker_count(1), 1)
        self.assertEqual(runner.default_worker_count(8), 4)
        self.assertGreaterEqual(runner.default_worker_count(None), 1)
        for count in (1, 2, 3, 4, 5):
            self.assertEqual(runner.validate_worker_count(count), count)
        for count in (0, -1):
            with self.assertRaises(ValueError):
                runner.validate_worker_count(count)

    def test_result_classification_fails_closed(self) -> None:
        task = runner.AtomicTask("task", ("module.Case.test_one",))
        cases = (
            (
                process_result(
                    task.task_id,
                    returncode=1,
                    failures=1,
                    successful=False,
                    failure_kind="TEST_FAILURE",
                ),
                ("TEST_FAILURE", 1),
            ),
            (
                process_result(
                    task.task_id,
                    returncode=-9,
                    result_present=False,
                    tests_run=0,
                    successful=False,
                ),
                ("PROCESS_FAILURE", 3),
            ),
        )
        for result, expected in cases:
            with self.subTest(expected=expected):
                status, exit_code, _, _ = runner.classify_results(1, (task,), (result,))
                self.assertEqual((status, exit_code), expected)

    def test_aggregate_accounting_mismatch_is_runner_failure(self) -> None:
        task = runner.AtomicTask(
            "task", ("module.Case.test_one", "module.Case.test_two")
        )
        result = process_result(task.task_id, tests_run=1)
        status, exit_code, executed, _ = runner.classify_results(2, (task,), (result,))
        self.assertEqual((status, exit_code, executed), ("RUNNER_FAILURE", 3, 1))

    def test_runner_never_injects_repository_src_or_fixed_tmp_artifacts(self) -> None:
        source = RUNNER_PATH.read_text(encoding="utf-8")
        self.assertNotIn("SRC_DIR", source)
        self.assertNotIn("proto-ring-61-actual-observations", source)
        self.assertNotIn("proto-ring-61-execution-metrics", source)
        self.assertIn('environment["PYTHONPATH"] = str(TESTS_DIR)', source)


if __name__ == "__main__":
    unittest.main()
