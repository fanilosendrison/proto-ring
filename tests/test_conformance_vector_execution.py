from __future__ import annotations

from concurrent.futures.process import BrokenProcessPool
from dataclasses import fields
import inspect
import os
from typing import Callable
import unittest
from unittest import mock

import conformance_vector_execution as vector_execution
from conformance_corpus_test_support import OWNER_BY_ID, load_corpus


RESPONSIBILITY_ID = "structured-data.document"


class _PlannedFuture:
    def __init__(self, outcome: object, result_calls: list[int], ordinal: int):
        self._outcome = outcome
        self._result_calls = result_calls
        self._ordinal = ordinal
        self._ready = False

    def mark_ready(self, completion_order: list[int]) -> None:
        self._ready = True
        completion_order.append(self._ordinal)

    def result(self) -> object:
        if not self._ready:
            raise AssertionError("result consumed before deterministic completion")
        self._result_calls.append(self._ordinal)
        if isinstance(self._outcome, BaseException):
            raise self._outcome
        return self._outcome


class _PlannedExecutor:
    outcomes: list[object] = []
    completion_order: list[int] = []
    constructed_worker_counts: list[int] = []
    result_calls: list[int] = []
    submitted_jobs: list[vector_execution.VectorExecutionJob] = []

    def __init__(self, max_workers: int):
        type(self).constructed_worker_counts.append(max_workers)
        self._futures: list[_PlannedFuture] = []

    def __enter__(self) -> _PlannedExecutor:
        return self

    def __exit__(self, *_exception: object) -> None:
        return None

    def submit(
        self,
        function: Callable[
            [vector_execution.VectorExecutionJob],
            vector_execution.VectorExecutionResult,
        ],
        job: vector_execution.VectorExecutionJob,
    ) -> _PlannedFuture:
        del function
        type(self).submitted_jobs.append(job)
        future = _PlannedFuture(
            type(self).outcomes[job.canonical_ordinal],
            type(self).result_calls,
            job.canonical_ordinal,
        )
        self._futures.append(future)
        if len(self._futures) == len(type(self).outcomes):
            for completed in reversed(self._futures):
                completed.mark_ready(type(self).completion_order)
        return future

    @classmethod
    def reset(cls, outcomes: list[object]) -> None:
        cls.outcomes = outcomes
        cls.completion_order = []
        cls.constructed_worker_counts = []
        cls.result_calls = []
        cls.submitted_jobs = []


class ConformanceVectorExecutionTest(unittest.TestCase):
    def _job(
        self,
        ordinal: int,
        vector_id: str | None = None,
        responsibility_id: str = RESPONSIBILITY_ID,
    ) -> vector_execution.VectorExecutionJob:
        return vector_execution.VectorExecutionJob(
            canonical_ordinal=ordinal,
            vector_id=(
                f"vector-{ordinal}" if vector_id is None else vector_id
            ),
            responsibility_id=responsibility_id,
            fixture={"kind": "inline"},
        )

    def _result(
        self,
        job: vector_execution.VectorExecutionJob,
        *,
        ordinal: int | None = None,
        vector_id: str | None = None,
        responsibility_id: str | None = None,
    ) -> vector_execution.VectorExecutionResult:
        return vector_execution.VectorExecutionResult(
            canonical_ordinal=(
                job.canonical_ordinal if ordinal is None else ordinal
            ),
            vector_id=job.vector_id if vector_id is None else vector_id,
            responsibility_id=(
                job.responsibility_id
                if responsibility_id is None
                else responsibility_id
            ),
            actual_observation={"kind": "result", "value": job.vector_id},
        )

    def _execute_with_planned_results(
        self,
        jobs: tuple[vector_execution.VectorExecutionJob, ...],
        outcomes: list[object],
        logical_cpu_count: int = 3,
    ) -> tuple[vector_execution.VectorExecutionResult, ...]:
        _PlannedExecutor.reset(outcomes)
        with mock.patch.object(
            vector_execution, "ProcessPoolExecutor", _PlannedExecutor
        ):
            return vector_execution.execute_vector_jobs(
                jobs, logical_cpu_count=logical_cpu_count
            )

    def test_vector_worker_cap_is_three(self) -> None:
        self.assertEqual(vector_execution.VECTOR_WORKER_CAP, 3)

    def test_worker_count_is_bounded_by_cpu_and_cap(self) -> None:
        expected = {1: 1, 2: 2, 3: 3, 4: 3, 8: 3}
        for logical_cpu_count, worker_count in expected.items():
            with self.subTest(logical_cpu_count=logical_cpu_count):
                self.assertEqual(
                    vector_execution.default_vector_worker_count(
                        logical_cpu_count
                    ),
                    worker_count,
                )

    def test_worker_count_handles_unavailable_or_invalid_cpu_count(self) -> None:
        for logical_cpu_count in (None, 0, -1):
            with self.subTest(logical_cpu_count=logical_cpu_count):
                with mock.patch.object(os, "cpu_count", return_value=None):
                    worker_count = vector_execution.default_vector_worker_count(
                        logical_cpu_count
                    )
                self.assertGreaterEqual(worker_count, 1)
                self.assertLessEqual(worker_count, 3)

    def test_job_fields_are_exact(self) -> None:
        self.assertEqual(
            tuple(field.name for field in fields(vector_execution.VectorExecutionJob)),
            ("canonical_ordinal", "vector_id", "responsibility_id", "fixture"),
        )

    def test_result_fields_are_exact(self) -> None:
        self.assertEqual(
            tuple(
                field.name for field in fields(
                    vector_execution.VectorExecutionResult
                )
            ),
            (
                "canonical_ordinal",
                "vector_id",
                "responsibility_id",
                "actual_observation",
            ),
        )

    def test_transport_fields_contain_no_oracle_concept(self) -> None:
        forbidden = {
            "expected_observation",
            "comparison",
            "behavior_classification",
            "authorities",
            "resolution",
            "frozen_python_observation",
            "later_python_observation",
            "expected_value",
            "expected_kind",
            "match",
            "mismatch",
        }
        for value_type in (
            vector_execution.VectorExecutionJob,
            vector_execution.VectorExecutionResult,
        ):
            with self.subTest(value_type=value_type.__name__):
                field_names = {field.name.lower() for field in fields(value_type)}
                self.assertTrue(forbidden.isdisjoint(field_names))

    def test_duplicate_vector_ids_fail_inventory_validation(self) -> None:
        jobs = (self._job(0, "duplicate"), self._job(1, "duplicate"))
        with self.assertRaises(vector_execution.VectorExecutionError):
            vector_execution.validate_vector_jobs(jobs)

    def test_empty_vector_id_fails_inventory_validation(self) -> None:
        with self.assertRaises(vector_execution.VectorExecutionError):
            vector_execution.validate_vector_jobs((self._job(0, ""),))

    def test_noncanonical_duplicate_or_missing_ordinals_fail(self) -> None:
        invalid_ordinals = ((1,), (0, 0), (0, 2))
        for ordinals in invalid_ordinals:
            jobs = tuple(
                self._job(ordinal, f"vector-{index}")
                for index, ordinal in enumerate(ordinals)
            )
            with self.subTest(ordinals=ordinals):
                with self.assertRaises(vector_execution.VectorExecutionError):
                    vector_execution.validate_vector_jobs(jobs)

    def test_unknown_responsibility_fails_before_executor_construction(self) -> None:
        jobs = (self._job(0, responsibility_id="unknown.responsibility"),)
        with mock.patch.object(vector_execution, "ProcessPoolExecutor") as executor:
            with self.assertRaises(vector_execution.VectorExecutionError):
                vector_execution.execute_vector_jobs(jobs)
        executor.assert_not_called()

    def test_wrong_ordinal_fails_closed(self) -> None:
        job = self._job(0)
        with self.assertRaises(vector_execution.VectorExecutionError):
            self._execute_with_planned_results(
                (job,), [self._result(job, ordinal=1)]
            )

    def test_wrong_vector_id_fails_closed(self) -> None:
        job = self._job(0)
        with self.assertRaises(vector_execution.VectorExecutionError):
            self._execute_with_planned_results(
                (job,), [self._result(job, vector_id="unknown-vector")]
            )

    def test_wrong_responsibility_id_fails_closed(self) -> None:
        job = self._job(0)
        other = next(value for value in OWNER_BY_ID if value != job.responsibility_id)
        with self.assertRaises(vector_execution.VectorExecutionError):
            self._execute_with_planned_results(
                (job,), [self._result(job, responsibility_id=other)]
            )

    def test_missing_logical_result_fails_closed(self) -> None:
        job = self._job(0)
        with self.assertRaises(vector_execution.VectorExecutionError):
            self._execute_with_planned_results((job,), [None])

    def test_duplicate_logical_result_fails_closed(self) -> None:
        jobs = (self._job(0), self._job(1))
        duplicate = self._result(jobs[0])
        with self.assertRaises(vector_execution.VectorExecutionError):
            self._execute_with_planned_results(jobs, [duplicate, duplicate])

    def test_worker_exception_propagates_as_controlled_failure(self) -> None:
        job = self._job(0)
        with self.assertRaises(vector_execution.VectorExecutionError) as context:
            self._execute_with_planned_results(
                (job,), [RuntimeError("worker failed")]
            )
        self.assertIsInstance(context.exception.__cause__, RuntimeError)

    def test_broken_process_pool_propagates_as_controlled_failure(self) -> None:
        job = self._job(0)
        with self.assertRaises(vector_execution.VectorExecutionError) as context:
            self._execute_with_planned_results(
                (job,), [BrokenProcessPool("pool failed")]
            )
        self.assertIsInstance(context.exception.__cause__, BrokenProcessPool)

    def test_completion_order_cannot_reorder_semantic_results(self) -> None:
        jobs = tuple(self._job(ordinal) for ordinal in range(3))
        results = self._execute_with_planned_results(
            jobs, [self._result(job) for job in jobs]
        )
        self.assertEqual(_PlannedExecutor.completion_order, [2, 1, 0])
        self.assertEqual(_PlannedExecutor.result_calls, [0, 1, 2])
        self.assertEqual(
            [result.canonical_ordinal for result in results], [0, 1, 2]
        )

    def test_executor_receives_maximum_three_workers(self) -> None:
        job = self._job(0)
        self._execute_with_planned_results(
            (job,), [self._result(job)], logical_cpu_count=8
        )
        self.assertEqual(_PlannedExecutor.constructed_worker_counts, [3])

    def test_worker_source_has_execution_only_call_boundary(self) -> None:
        signature = inspect.signature(vector_execution.execute_vector_job)
        self.assertEqual(tuple(signature.parameters), ("job",))
        source = inspect.getsource(vector_execution.execute_vector_job)
        self.assertNotIn("expected_observation", source)
        self.assertNotIn("comparison", source)
        self.assertNotIn("compare(", source)
        self.assertNotIn('"MATCH"', source)
        self.assertNotIn('"MISMATCH"', source)

    def test_real_processes_execute_lightweight_vectors_in_canonical_order(
        self,
    ) -> None:
        _, _, cases, _ = load_corpus()
        selected = [
            (responsibility_id, vector)
            for responsibility_id, matrix in sorted(cases.items())
            for vector in matrix["vectors"]
            if vector["fixture"]["kind"] == "inline"
        ][:4]
        jobs = tuple(
            vector_execution.VectorExecutionJob(
                canonical_ordinal=ordinal,
                vector_id=vector["vector_id"],
                responsibility_id=responsibility_id,
                fixture=vector["fixture"],
            )
            for ordinal, (responsibility_id, vector) in enumerate(selected)
        )
        results = vector_execution.execute_vector_jobs(jobs, logical_cpu_count=3)
        self.assertEqual(len(results), len(jobs))
        self.assertEqual(
            [result.canonical_ordinal for result in results],
            list(range(len(jobs))),
        )
        self.assertEqual(
            [result.vector_id for result in results],
            [job.vector_id for job in jobs],
        )
        self.assertTrue(
            all(result.actual_observation is not None for result in results)
        )


if __name__ == "__main__":
    unittest.main()
