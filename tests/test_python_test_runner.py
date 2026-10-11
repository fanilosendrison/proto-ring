from __future__ import annotations

from collections import Counter
from contextlib import contextmanager, redirect_stdout
import importlib.util
import io
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

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
    def test_non_stateful_task_bypasses_shared_artifact_lock(self) -> None:
        task = runner.AtomicTask("module:synthetic", ("synthetic.Case.test_one",))
        expected = process_result(task.task_id)

        @contextmanager
        def forbidden_lock():
            raise AssertionError("non-stateful task entered artifact lock")
            yield

        lock_factory = mock.Mock(side_effect=forbidden_lock)
        with (
            mock.patch.object(runner, "_shared_artifact_lock", lock_factory),
            mock.patch.object(runner, "_execute_subprocess", return_value=expected) as execute,
        ):
            actual = runner._execute_atomic_task(task, Path("unused"))
        self.assertIs(actual, expected)
        execute.assert_called_once_with(task, Path("unused"))
        lock_factory.assert_not_called()
    def test_stateful_artifact_lifecycle_is_inside_lock(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = (Path(temporary) / "observations.json", Path(temporary) / "metrics.txt")
            for artifact in artifacts:
                artifact.write_text("stale", encoding="utf-8")
            events = []
            state = {"inside": False}
            expected = process_result("corpus-stateful", tests_run=3)

            @contextmanager
            def tracked_lock():
                events.append("lock-enter")
                state["inside"] = True
                try:
                    yield
                finally:
                    self.assertTrue(all(artifact.is_file() for artifact in artifacts))
                    events.append("lock-exit")
                    state["inside"] = False
                    for artifact in artifacts:
                        artifact.unlink()

            def execute(task, directory):
                self.assertTrue(state["inside"])
                self.assertFalse(any(artifact.exists() for artifact in artifacts))
                for artifact in artifacts:
                    artifact.write_text("fresh", encoding="utf-8")
                return expected

            task = runner.AtomicTask("corpus-stateful", runner.STATEFUL_TEST_IDS)
            with (
                mock.patch.object(runner, "SHARED_ARTIFACTS", artifacts),
                mock.patch.object(runner, "_shared_artifact_lock", tracked_lock),
                mock.patch.object(runner, "_execute_subprocess", side_effect=execute),
            ):
                actual = runner._execute_atomic_task(task, Path(temporary))
            self.assertIs(actual, expected)
            self.assertEqual(events, ["lock-enter", "lock-exit"])
    def test_stateful_missing_artifact_fails_closed_inside_lock(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = (Path(temporary) / "observations.json", Path(temporary) / "metrics.txt")
            state = {"inside": False}
            checks = []
            expected = process_result("corpus-stateful", tests_run=3)
            original_is_file = Path.is_file

            @contextmanager
            def tracked_lock():
                state["inside"] = True
                try:
                    yield
                finally:
                    state["inside"] = False

            def execute(task, directory):
                artifacts[0].write_text("fresh", encoding="utf-8")
                return expected

            def checked_is_file(path):
                self.assertTrue(state["inside"])
                checks.append(path)
                return original_is_file(path)

            task = runner.AtomicTask("corpus-stateful", runner.STATEFUL_TEST_IDS)
            with (
                mock.patch.object(runner, "SHARED_ARTIFACTS", artifacts),
                mock.patch.object(runner, "_shared_artifact_lock", tracked_lock),
                mock.patch.object(runner, "_execute_subprocess", side_effect=execute),
                mock.patch.object(Path, "is_file", autospec=True, side_effect=checked_is_file),
            ):
                actual = runner._execute_atomic_task(task, Path(temporary))
            self.assertFalse(actual.successful)
            self.assertEqual(actual.failure_kind, "RUNNER_FAILURE")
            self.assertEqual(checks, list(artifacts))
    def test_shared_artifact_lock_uses_exclusive_flock(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            lock_path = Path(temporary) / "shared.lock"
            calls = []

            def record_flock(file_descriptor, operation):
                os.fstat(file_descriptor)
                calls.append((file_descriptor, operation))

            with (
                mock.patch.object(runner, "SHARED_ARTIFACT_LOCK", lock_path),
                mock.patch.object(runner.fcntl, "flock", side_effect=record_flock),
            ):
                with runner._shared_artifact_lock():
                    self.assertTrue(lock_path.is_file())
            self.assertEqual([operation for _, operation in calls], [runner.fcntl.LOCK_EX, runner.fcntl.LOCK_UN])
            self.assertEqual(calls[0][0], calls[1][0])
            self.assertTrue(lock_path.is_file())
    def test_fixed_scratch_and_lock_paths_are_unchanged(self) -> None:
        expected = (
            str(Path("/tmp") / "proto-ring-61-actual-observations.json"),
            str(Path("/tmp") / "proto-ring-61-execution-metrics.txt"),
        )
        self.assertEqual(tuple(map(str, runner.SHARED_ARTIFACTS)), expected)
        self.assertEqual(
            str(runner.SHARED_ARTIFACT_LOCK),
            str(Path("/tmp") / "proto-ring-61-shared-artifacts.lock"),
        )
    def test_stateful_export_occurs_before_lock_release(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = (Path(temporary) / "observations.json", Path(temporary) / "metrics.txt")
            state = {"inside": False}
            expected = process_result("corpus-stateful", tests_run=3)
            evidence_run = mock.Mock()

            @contextmanager
            def tracked_lock():
                state["inside"] = True
                try:
                    yield
                finally:
                    state["inside"] = False

            def execute(task, directory):
                self.assertTrue(state["inside"])
                for artifact in artifacts:
                    artifact.write_bytes(b"evidence")
                return expected

            def export(actual_run, sources):
                self.assertTrue(state["inside"])
                self.assertIs(actual_run, evidence_run)
                self.assertEqual(tuple(sources.values()), artifacts)
                self.assertTrue(all(path.is_file() for path in artifacts))

            task = runner.AtomicTask("corpus-stateful", runner.STATEFUL_TEST_IDS)
            with (
                mock.patch.object(runner, "SHARED_ARTIFACTS", artifacts),
                mock.patch.object(runner, "_shared_artifact_lock", tracked_lock),
                mock.patch.object(runner, "_execute_subprocess", side_effect=execute),
                mock.patch.object(runner.evidence, "export_artifacts", side_effect=export) as exporter,
            ):
                actual = runner._execute_atomic_task(task, Path(temporary), evidence_run)
            self.assertIs(actual, expected)
            exporter.assert_called_once()
            self.assertFalse(state["inside"])
    def test_stateful_without_export_never_calls_export(self) -> None:
        expected = process_result("corpus-stateful", tests_run=3)
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = (Path(temporary) / "observations.json", Path(temporary) / "metrics.txt")

            def execute(task, directory):
                for artifact in artifacts:
                    artifact.write_bytes(b"evidence")
                return expected

            task = runner.AtomicTask("corpus-stateful", runner.STATEFUL_TEST_IDS)
            with (
                mock.patch.object(runner, "SHARED_ARTIFACTS", artifacts),
                mock.patch.object(runner, "_execute_subprocess", side_effect=execute),
                mock.patch.object(runner.evidence, "export_artifacts") as exporter,
            ):
                actual = runner._execute_atomic_task(task, Path(temporary))
            self.assertIs(actual, expected)
            exporter.assert_not_called()
    def test_export_failure_turns_successful_stateful_task_into_runner_failure(self) -> None:
        expected = process_result("corpus-stateful", tests_run=3)
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = (Path(temporary) / "observations.json", Path(temporary) / "metrics.txt")

            def execute(task, directory):
                for artifact in artifacts:
                    artifact.write_bytes(b"evidence")
                return expected

            task = runner.AtomicTask("corpus-stateful", runner.STATEFUL_TEST_IDS)
            with (
                mock.patch.object(runner, "SHARED_ARTIFACTS", artifacts),
                mock.patch.object(runner, "_execute_subprocess", side_effect=execute),
                mock.patch.object(
                    runner.evidence, "export_artifacts", side_effect=runner.evidence.EvidenceError("failed")
                ),
            ):
                actual = runner._execute_atomic_task(task, Path(temporary), mock.Mock())
            self.assertFalse(actual.successful)
            self.assertEqual(actual.failure_kind, "RUNNER_FAILURE")
    def test_evidence_option_defaults_disabled_and_parses_path(self) -> None:
        self.assertIsNone(runner.parse_arguments([]).evidence_dir)
        self.assertEqual(runner.parse_arguments(["--evidence-dir", "evidence"]).evidence_dir, Path("evidence"))
    def test_default_summary_exposes_no_run_evidence(self) -> None:
        summary = runner.RunSummary(4, 1, 1, 1, 0.1, "PASS", ())
        output = io.StringIO()
        with redirect_stdout(output):
            runner._print_summary(summary, None, None)
        self.assertIn("RUN_ID=NONE\n", output.getvalue())
        self.assertIn("EVIDENCE_HANDOFF=NONE\n", output.getvalue())
    def _controller_patches(self, execute):
        task = runner.AtomicTask("corpus-stateful", ("synthetic.Case.test_one",))
        return (
            mock.patch.object(runner, "discover_test_ids", return_value=task.test_ids),
            mock.patch.object(runner, "build_tasks", return_value=(task,)),
            mock.patch.object(runner, "validate_partition"),
            mock.patch.object(runner, "validate_shared_artifact_ownership"),
            mock.patch.object(runner, "_execute_atomic_task", side_effect=execute),
        )
    def test_successful_final_export_produces_verified_handoff(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            sources = (root / "source-observations", root / "source-metrics")
            for source in sources:
                source.write_bytes(source.name.encode())

            def execute(task, directory, evidence_run):
                runner.evidence.export_artifacts(
                    evidence_run,
                    {"actual_observations": sources[0], "execution_metrics": sources[1]},
                )
                return process_result(task.task_id)

            output = io.StringIO()
            patches = self._controller_patches(execute)
            with patches[0], patches[1], patches[2], patches[3], patches[4], redirect_stdout(output):
                exit_code = runner.run_controller(1, root / "evidence")
            self.assertEqual(exit_code, 0)
            handoff_line = next(line for line in output.getvalue().splitlines() if line.startswith("EVIDENCE_HANDOFF="))
            handoff = Path(handoff_line.split("=", 1)[1])
            payload = runner.evidence.verify_handoff(handoff)
            self.assertEqual(payload["runner"]["status"], "PASS")
    def test_handoff_finalization_failure_cannot_return_pass(self) -> None:
        def execute(task, directory, evidence_run):
            evidence_run.artifacts = {key: mock.Mock() for key in runner.evidence.ARTIFACT_FILENAMES}
            return process_result(task.task_id)

        with tempfile.TemporaryDirectory() as temporary:
            output = io.StringIO()
            patches = self._controller_patches(execute)
            with (
                patches[0], patches[1], patches[2], patches[3], patches[4],
                mock.patch.object(
                    runner.evidence, "finalize_handoff", side_effect=runner.evidence.EvidenceError("failed")
                ),
                redirect_stdout(output),
            ):
                exit_code = runner.run_controller(1, Path(temporary))
        self.assertEqual(exit_code, runner.EXIT_CODES["RUNNER_FAILURE"])
        self.assertIn("PROTO_RING_TEST_RUNNER=RUNNER_FAILURE", output.getvalue())
        self.assertIn("EVIDENCE_HANDOFF=NONE", output.getvalue())


if __name__ == "__main__":
    unittest.main()
