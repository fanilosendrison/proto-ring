"""Schedule conformance vector execution without owning conformance truth."""
from __future__ import annotations

from concurrent.futures import Future, ProcessPoolExecutor
from dataclasses import dataclass
import os
from typing import Sequence

import conformance_python_adapter_63_64 as adapter_63_64
import conformance_python_adapter_65_66 as adapter_65_66
import conformance_python_adapter_67_69 as adapter_67_69
from conformance_corpus_test_support import OWNER_BY_ID
from conformance_fixture_runner import materialize


VECTOR_WORKER_CAP = 3
ADAPTERS = {
    63: adapter_63_64,
    64: adapter_63_64,
    65: adapter_65_66,
    66: adapter_65_66,
    67: adapter_67_69,
    68: adapter_67_69,
    69: adapter_67_69,
}


class VectorExecutionError(RuntimeError):
    """Vector scheduling or result accounting failed closed."""


@dataclass(frozen=True)
class VectorExecutionJob:
    canonical_ordinal: int
    vector_id: str
    responsibility_id: str
    fixture: dict[str, object]


@dataclass(frozen=True)
class VectorExecutionResult:
    canonical_ordinal: int
    vector_id: str
    responsibility_id: str
    actual_observation: dict[str, object]


def default_vector_worker_count(logical_cpu_count: int | None = None) -> int:
    count = os.cpu_count() if logical_cpu_count is None else logical_cpu_count
    if count is None or count < 1:
        count = 1
    return min(count, VECTOR_WORKER_CAP)


def validate_vector_jobs(jobs: Sequence[VectorExecutionJob]) -> None:
    ordinals = [job.canonical_ordinal for job in jobs]
    if ordinals != list(range(len(jobs))):
        raise VectorExecutionError(
            "vector job ordinals must be the canonical range 0..N-1"
        )
    vector_ids = [job.vector_id for job in jobs]
    if any(not vector_id for vector_id in vector_ids):
        raise VectorExecutionError("vector job IDs must be non-empty")
    if len(set(vector_ids)) != len(vector_ids):
        raise VectorExecutionError("vector job IDs must be unique")
    for job in jobs:
        owner_issue = OWNER_BY_ID.get(job.responsibility_id)
        if owner_issue is None or owner_issue not in ADAPTERS:
            raise VectorExecutionError(
                f"unknown vector responsibility: {job.responsibility_id}"
            )


def execute_vector_job(job: VectorExecutionJob) -> VectorExecutionResult:
    adapter = ADAPTERS[OWNER_BY_ID[job.responsibility_id]]
    with materialize(job.fixture) as realized_fixture:
        actual_observation = adapter.execute(
            job.responsibility_id, realized_fixture
        )
    return VectorExecutionResult(
        canonical_ordinal=job.canonical_ordinal,
        vector_id=job.vector_id,
        responsibility_id=job.responsibility_id,
        actual_observation=actual_observation,
    )


def _validate_result_binding(
    job: VectorExecutionJob,
    result: object,
    observed_bindings: set[tuple[int, str, str]],
) -> VectorExecutionResult:
    if not isinstance(result, VectorExecutionResult):
        raise VectorExecutionError(
            f"missing or invalid result for vector job {job.vector_id}"
        )
    expected_binding = (
        job.canonical_ordinal,
        job.vector_id,
        job.responsibility_id,
    )
    actual_binding = (
        result.canonical_ordinal,
        result.vector_id,
        result.responsibility_id,
    )
    if actual_binding != expected_binding:
        raise VectorExecutionError(
            "vector result binding does not match its canonical job: "
            f"expected {expected_binding!r}, received {actual_binding!r}"
        )
    if actual_binding in observed_bindings:
        raise VectorExecutionError(
            f"duplicate logical vector result: {actual_binding!r}"
        )
    observed_bindings.add(actual_binding)
    return result


def execute_vector_jobs(
    jobs: Sequence[VectorExecutionJob],
    *,
    logical_cpu_count: int | None = None,
) -> tuple[VectorExecutionResult, ...]:
    canonical_jobs = tuple(jobs)
    validate_vector_jobs(canonical_jobs)
    if not canonical_jobs:
        return ()
    observed_bindings: set[tuple[int, str, str]] = set()
    canonical_results: list[VectorExecutionResult] = []
    try:
        with ProcessPoolExecutor(
            max_workers=default_vector_worker_count(logical_cpu_count)
        ) as executor:
            futures: tuple[Future[VectorExecutionResult], ...] = tuple(
                executor.submit(execute_vector_job, job)
                for job in canonical_jobs
            )
            for job, future in zip(canonical_jobs, futures, strict=True):
                canonical_results.append(
                    _validate_result_binding(
                        job, future.result(), observed_bindings
                    )
                )
    except VectorExecutionError:
        raise
    except Exception as error:
        raise VectorExecutionError("vector process execution failed") from error
    if len(canonical_results) != len(canonical_jobs):
        raise VectorExecutionError("missing logical vector result")
    return tuple(canonical_results)
