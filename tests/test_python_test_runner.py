from __future__ import annotations

from collections import Counter
import importlib.util
from pathlib import Path
import sys
import unittest

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
RUNNER_PATH = REPOSITORY_ROOT / "tools" / "run_python_tests.py"
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
    errors: int = 0,
    successful: bool = True,
    failure_kind: str = "NONE",
):
    return runner.TaskProcessResult(
        task_id=task_id,
        returncode=returncode,
        result_present=result_present,
        tests_run=tests_run,
        failures=failures,
        errors=errors,
        skipped=0,
        successful=successful,
        duration_seconds=0.01,
        output="output\n",
        failure_kind=failure_kind,
    )


class PythonTestRunnerTests(unittest.TestCase):
    def test_current_inventory_partitions_exactly(self) -> None:
        discovered = runner.discover_test_ids()
        tasks = runner.build_tasks(discovered)
        runner.validate_partition(discovered, tasks)
        scheduled = [test_id for task in tasks for test_id in task.test_ids]
        self.assertEqual(Counter(scheduled), Counter(discovered))
        self.assertEqual(tasks[0].task_id, "corpus-stateful")
        self.assertEqual(tasks[1].task_id, "corpus-static")
        module_task_ids = [task.task_id for task in tasks[2:]]
        self.assertEqual(module_task_ids, sorted(module_task_ids))
        self.assertTrue(all(task_id.startswith("module:") for task_id in module_task_ids))
        self.assertIn("module:test_python_test_runner", module_task_ids)

    def test_partition_rejects_duplicate_test_id(self) -> None:
        discovered = ("module.Case.test_one", "module.Case.test_two")
        tasks = (
            runner.AtomicTask("first", (discovered[0],)),
            runner.AtomicTask("second", discovered),
        )
        with self.assertRaisesRegex(runner.InventoryError, "duplicate scheduled IDs"):
            runner.validate_partition(discovered, tasks)

    def test_partition_rejects_missing_test_id(self) -> None:
        discovered = ("module.Case.test_one", "module.Case.test_two")
        tasks = (runner.AtomicTask("only", (discovered[0],)),)
        with self.assertRaisesRegex(runner.InventoryError, "missing discovered IDs"):
            runner.validate_partition(discovered, tasks)

    def test_partition_rejects_unknown_test_id(self) -> None:
        discovered = ("module.Case.test_one",)
        tasks = (runner.AtomicTask("only", (discovered[0], "module.Case.test_unknown")),)
        with self.assertRaisesRegex(runner.InventoryError, "unknown scheduled IDs"):
            runner.validate_partition(discovered, tasks)

    def test_stateful_group_is_exact_and_ordered(self) -> None:
        tasks = runner.build_tasks(runner.discover_test_ids())
        stateful = next(task for task in tasks if task.task_id == "corpus-stateful")
        static = next(task for task in tasks if task.task_id == "corpus-static")
        self.assertEqual(stateful.test_ids, runner.STATEFUL_TEST_IDS)
        self.assertTrue(set(stateful.test_ids).isdisjoint(static.test_ids))

    def test_worker_count_default_and_validation(self) -> None:
        self.assertEqual(runner.default_worker_count(1), 1)
        self.assertEqual(runner.default_worker_count(2), 2)
        self.assertEqual(runner.default_worker_count(4), 4)
        self.assertEqual(runner.default_worker_count(8), 4)
        self.assertGreaterEqual(runner.default_worker_count(None), 1)
        for count in (1, 2, 3, 4, 5):
            self.assertEqual(runner.validate_worker_count(count), count)
        for count in (0, -1):
            with self.assertRaises(ValueError):
                runner.validate_worker_count(count)

    def test_test_failure_propagates(self) -> None:
        task = runner.AtomicTask("task", ("module.Case.test_one",))
        result = process_result(
            task.task_id,
            returncode=1,
            failures=1,
            successful=False,
            failure_kind="TEST_FAILURE",
        )
        status, exit_code, executed, failed = runner.classify_results(1, (task,), (result,))
        self.assertEqual((status, exit_code, executed, failed), ("TEST_FAILURE", 1, 1, ("task",)))

    def test_process_failure_propagates(self) -> None:
        task = runner.AtomicTask("task", ("module.Case.test_one",))
        result = process_result(
            task.task_id,
            returncode=-9,
            result_present=False,
            tests_run=0,
            successful=False,
            failure_kind="NONE",
        )
        status, exit_code, _, failed = runner.classify_results(1, (task,), (result,))
        self.assertEqual((status, exit_code, failed), ("PROCESS_FAILURE", 3, ("task",)))

    def test_aggregate_accounting_mismatch_fails_closed(self) -> None:
        task = runner.AtomicTask("task", ("module.Case.test_one", "module.Case.test_two"))
        result = process_result(task.task_id, tests_run=1)
        status, exit_code, executed, failed = runner.classify_results(2, (task,), (result,))
        self.assertEqual((status, exit_code, executed, failed), ("RUNNER_FAILURE", 3, 1, ()))

    def test_shared_artifact_ownership_is_stateful_only(self) -> None:
        owners = runner.validate_shared_artifact_ownership()
        self.assertEqual(
            owners,
            {
                "test_08_every_vector_executes_and_current_python_conforms",
                "test_10_execution_evidence_and_final_metrics",
            },
        )
        self.assertTrue(owners.issubset(runner.STATEFUL_METHODS))


if __name__ == "__main__":
    unittest.main()
