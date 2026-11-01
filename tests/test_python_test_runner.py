from __future__ import annotations

from collections import Counter
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import types
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


def temporary_runner_with_hidden_package(relative: Path, content: str):
    temporary = tempfile.TemporaryDirectory()
    root = Path(temporary.name)
    tools = root / "tools"
    tests = root / "tests"
    tools.mkdir()
    tests.mkdir()
    for name in (
        "run_python_tests.py",
        "python_package_absence.py",
        "python_test_inventory.py",
        "python_test_runner_protocol.py",
    ):
        shutil.copy2(REPOSITORY_ROOT / "tools" / name, tools / name)
    marker = root / "executed.txt"
    (tests / "test_control.py").write_text(
        "import pathlib\nimport unittest\n"
        "class Control(unittest.TestCase):\n"
        f"    def test_never_runs(self): pathlib.Path({str(marker)!r}).write_text('ran')\n",
        encoding="utf-8",
    )
    (tests / "retained-python-test-ids.txt").write_text(
        "test_control.Control.test_never_runs\n", encoding="utf-8"
    )
    hidden = root / relative
    hidden.parent.mkdir(parents=True, exist_ok=True)
    hidden.write_text(content, encoding="utf-8")
    environment = os.environ.copy()
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    if relative.parts[0] == "site":
        environment["PYTHONPATH"] = str(root / "site")
    return temporary, root, marker, environment


def process_result(
    task_id: str,
    *,
    returncode: int = 0,
    result_present: bool = True,
    tests_run: int = 1,
    failures: int = 0,
    errors: int = 0,
    skipped: int = 0,
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
        skipped=skipped,
        successful=successful,
        duration_seconds=0.01,
        output="output\n",
        failure_kind=failure_kind,
    )


class PythonTestRunnerTests(unittest.TestCase):
    def test_current_inventory_partitions_exactly_by_module(self) -> None:
        expected = runner.load_pinned_test_ids(runner.RETAINED_TEST_IDS)
        discovered = runner.discover_test_ids()
        runner.validate_exact_inventory(discovered, expected)
        self.assertEqual(len(expected), 176)
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

        expected = (
            "first.Case.test_one",
            "first.Case.test_two",
            "second.Case.test_one",
        )
        inventory_cases = (
            ("missing entire module", expected[:2], "missing pinned IDs"),
            ("missing individual ID", (expected[0], expected[2]), "missing pinned IDs"),
            ("duplicate ID", (*expected, expected[0]), "duplicate discovered IDs"),
            ("extra ID", (*expected, "third.Case.test_extra"), "extra discovered IDs"),
        )
        for label, actual, message in inventory_cases:
            with self.subTest(label=label):
                with self.assertRaisesRegex(runner.InventoryError, message):
                    runner.validate_exact_inventory(actual, expected)

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

        forged = process_result(task.task_id, skipped=1, successful=True)
        status, exit_code, _, _ = runner.classify_results(1, (task,), (forged,))
        self.assertEqual((status, exit_code), ("RUNNER_FAILURE", 3))

        module_name = "synthetic_skipped_runner_control"
        module = types.ModuleType(module_name)

        class SkippedCase(unittest.TestCase):
            @unittest.skip("synthetic skip must fail closed")
            def test_skipped(self) -> None:
                self.fail("skip decorator should prevent execution")

        SkippedCase.__module__ = module_name
        module.SkippedCase = SkippedCase
        sys.modules[module_name] = module
        try:
            with tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                result_path = directory / "result.json"
                spec_path = directory / "spec.json"
                spec_path.write_text(
                    json.dumps(
                        {
                            "task_id": "synthetic-skip",
                            "test_ids": [f"{module_name}.SkippedCase.test_skipped"],
                            "result_path": str(result_path),
                        }
                    ),
                    encoding="utf-8",
                )
                self.assertEqual(runner.execute_task_spec(spec_path), 1)
                payload = json.loads(result_path.read_text(encoding="utf-8"))
                self.assertEqual(payload["failure_kind"], "TEST_FAILURE")
                self.assertEqual(payload["skipped"], 1)
                self.assertFalse(payload["successful"])
        finally:
            del sys.modules[module_name]

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
        self.assertGreaterEqual(source.count("assert_python_package_absent"), 3)
        self.assertIn("load_pinned_test_ids(RETAINED_TEST_IDS)", source)

        hidden_cases = (
            (Path("tests/proto_ring.py"), ""),
            (Path("tests/proto_ring/__init__.py"), ""),
            (Path("tools/proto_ring.py"), ""),
            (
                Path("site/proto_ring-9.9.9.dist-info/METADATA"),
                "Metadata-Version: 2.1\nName: proto-ring\nVersion: 9.9.9\n",
            ),
        )
        for relative, content in hidden_cases:
            with self.subTest(relative=relative):
                temporary, root, marker, environment = temporary_runner_with_hidden_package(
                    relative, content
                )
                with temporary:
                    completed = subprocess.run(
                        [sys.executable, str(root / "tools/run_python_tests.py"), "--workers", "1"],
                        cwd=root,
                        env=environment,
                        capture_output=True,
                        text=True,
                    )
                    self.assertEqual(completed.returncode, 2, completed.stdout + completed.stderr)
                    self.assertIn("INVENTORY_ERROR=", completed.stdout)
                    self.assertFalse(marker.exists())

        temporary, root, marker, environment = temporary_runner_with_hidden_package(
            Path("tests/proto_ring.py"), ""
        )
        with temporary:
            result_path = root / "worker-result.json"
            spec_path = root / "worker-spec.json"
            spec_path.write_text(
                json.dumps(
                    {
                        "task_id": "module:test_control",
                        "test_ids": ["test_control.Control.test_never_runs"],
                        "result_path": str(result_path),
                    }
                ),
                encoding="utf-8",
            )
            completed = subprocess.run(
                [
                    sys.executable,
                    str(root / "tools/run_python_tests.py"),
                    "--_execute-task",
                    str(spec_path),
                ],
                cwd=root,
                env=environment,
                capture_output=True,
                text=True,
            )
            self.assertEqual(completed.returncode, 3, completed.stdout + completed.stderr)
            self.assertFalse(marker.exists())
            self.assertEqual(
                json.loads(result_path.read_text(encoding="utf-8"))["failure_kind"],
                "RUNNER_FAILURE",
            )


if __name__ == "__main__":
    unittest.main()
