import pickle
import unittest
from unittest import mock

from jsonschema import ValidationError

import conformance_corpus_test_support as support


class _Future:
    def __init__(self, value=None, error=None, events=None, label=None):
        self.value = value
        self.error = error
        self.events = events
        self.label = label

    def result(self):
        if self.events is not None:
            self.events.append(f"result:{self.label}")
        if self.error is not None:
            raise self.error
        return self.value


class _Executor:
    def __init__(self, futures, events=None, worker_counts=None):
        self.futures = iter(futures)
        self.events = events
        self.worker_counts = worker_counts
        self.submissions = []

    def factory(self, *, max_workers):
        if self.worker_counts is not None:
            self.worker_counts.append(max_workers)
        return self

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        return False

    def submit(self, function, *arguments):
        self.submissions.append((function, arguments))
        if self.events is not None:
            self.events.append(f"submit:{arguments[0]}")
        return next(self.futures)


def _synthetic_inputs():
    schemas = {
        "index.schema.json": object(),
        "responsibility.schema.json": object(),
        "case.schema.json": object(),
        "coverage.schema.json": object(),
    }
    index = {"kind": "index"}
    responsibilities = {
        "responsibility-b": {"kind": "responsibility-b"},
        "responsibility-a": {"kind": "responsibility-a"},
    }
    cases = {
        "case-b": {"kind": "case-b"},
        "case-a": {"kind": "case-a"},
    }
    coverage = [{"kind": "coverage-b"}, {"kind": "coverage-a"}]
    return schemas, index, responsibilities, cases, coverage


def _jobs():
    schemas, index, responsibilities, cases, coverage = _synthetic_inputs()
    return support._schema_validation_jobs(
        schemas, index, responsibilities, cases, coverage
    )


class ConformanceSchemaParallelTest(unittest.TestCase):
    def test_job_builder_preserves_exact_in_memory_inputs_and_order(self):
        schemas, index, responsibilities, cases, coverage = _synthetic_inputs()
        jobs = support._schema_validation_jobs(
            schemas, index, responsibilities, cases, coverage
        )

        self.assertEqual([job.index for job in jobs], list(range(7)))
        self.assertEqual(
            [job.category for job in jobs],
            [
                "index", "responsibility", "responsibility", "case", "case",
                "coverage", "coverage",
            ],
        )
        expected_schemas = [
            schemas["index.schema.json"],
            schemas["responsibility.schema.json"],
            schemas["responsibility.schema.json"],
            schemas["case.schema.json"],
            schemas["case.schema.json"],
            schemas["coverage.schema.json"],
            schemas["coverage.schema.json"],
        ]
        expected_values = [
            index, *responsibilities.values(), *cases.values(), *coverage,
        ]
        for job, schema, value in zip(jobs, expected_schemas, expected_values):
            self.assertIs(job.schema, schema)
            self.assertIs(job.value, value)

    def test_job_builder_covers_all_four_categories(self):
        jobs = _jobs()
        counts = {category: 0 for category in {job.category for job in jobs}}
        for job in jobs:
            counts[job.category] += 1
        self.assertEqual(
            counts, {"index": 1, "responsibility": 2, "case": 2, "coverage": 2}
        )

    def test_schema_worker_count_is_bounded_and_one_cpu_safe(self):
        for logical_cpus, expected in (
            (1, 1), (2, 2), (3, 3), (4, 3), (8, 3), (0, 1), (-1, 1)
        ):
            with self.subTest(logical_cpus=logical_cpus):
                self.assertEqual(
                    support.schema_validation_worker_count(logical_cpus), expected
                )
        for cpu_count, expected in ((None, 1), (1, 1), (4, 3)):
            with self.subTest(cpu_count=cpu_count):
                with mock.patch.object(support.os, "cpu_count", return_value=cpu_count):
                    self.assertEqual(support.schema_validation_worker_count(), expected)

    def test_valid_document_passes_through_real_process_pool(self):
        job = support.SchemaValidationJob(
            0, "index", {"type": "object", "required": ["value"]}, {"value": 1}
        )
        self.assertIsNone(
            support._run_schema_validation_jobs((job,), logical_cpu_count=1)
        )

    def test_invalid_document_fails_through_real_process_pool(self):
        schema = {"type": "object", "required": ["value"]}
        job = support.SchemaValidationJob(0, "index", schema, {})
        with self.assertRaises(ValidationError) as raised:
            support._run_schema_validation_jobs((job,), logical_cpu_count=1)
        error = raised.exception
        self.assertIn("'value' is a required property", error.message)
        self.assertEqual(error.validator, "required")
        self.assertEqual(error.validator_value, ["value"])
        self.assertEqual(error.instance, {})
        self.assertEqual(error.schema, schema)
        self.assertEqual(list(error.path), [])
        self.assertIn("required", error.schema_path)
        pickle.dumps(error)

        nested_schema = {
            "oneOf": [{"type": "string"}, {"type": "integer"}],
        }
        nested_job = support.SchemaValidationJob(0, "index", nested_schema, {})
        with self.assertRaises(ValidationError) as nested_raised:
            support._run_schema_validation_jobs((nested_job,), logical_cpu_count=1)
        nested_error = nested_raised.exception
        self.assertGreaterEqual(len(nested_error.context), 2)
        self.assertTrue(
            all(isinstance(child, ValidationError) for child in nested_error.context)
        )
        pickle.dumps(nested_error)

    def test_job_results_are_observed_in_submission_order(self):
        events = []
        executor = _Executor(
            [
                _Future(error=RuntimeError("first-submitted-failure"), events=events, label=0),
                _Future(error=RuntimeError("later-failure"), events=events, label=1),
                _Future(value=2, events=events, label=2),
            ],
            events=events,
        )
        with self.assertRaisesRegex(RuntimeError, "first-submitted-failure"):
            support._run_schema_validation_jobs(
                _jobs()[:3], logical_cpu_count=3, executor_factory=executor.factory
            )
        self.assertEqual(events, ["submit:0", "submit:1", "submit:2", "result:0"])

    def test_executor_initialization_failure_propagates(self):
        def failing_factory(*, max_workers):
            raise RuntimeError("executor-init")

        with self.assertRaisesRegex(RuntimeError, "executor-init"):
            support._run_schema_validation_jobs(
                _jobs()[:1], executor_factory=failing_factory
            )

    def test_worker_failure_propagates(self):
        executor = _Executor([_Future(error=RuntimeError("worker-failure"))])
        with self.assertRaisesRegex(RuntimeError, "worker-failure"):
            support._run_schema_validation_jobs(
                _jobs()[:1], executor_factory=executor.factory
            )

    def test_missing_or_mismatched_worker_result_fails_closed(self):
        for returned in (1, None):
            with self.subTest(returned=returned):
                executor = _Executor([_Future(value=returned)])
                with self.assertRaisesRegex(
                    RuntimeError, "schema validation worker result mismatch"
                ):
                    support._run_schema_validation_jobs(
                        _jobs()[:1], executor_factory=executor.factory
                    )

    def test_validate_schemas_checks_all_four_schemas_before_submission(self):
        schemas, index, responsibilities, cases, coverage = _synthetic_inputs()
        events = []
        with (
            mock.patch.object(support, "_load_conformance_schemas", return_value=schemas),
            mock.patch.object(
                support.Draft202012Validator,
                "check_schema",
                side_effect=lambda schema: events.append(("check", schema)),
            ),
            mock.patch.object(
                support,
                "_run_schema_validation_jobs",
                side_effect=lambda jobs: events.append(("run", jobs)),
            ),
        ):
            support.validate_schemas(index, responsibilities, cases, coverage)
        self.assertEqual([event[0] for event in events], ["check"] * 4 + ["run"])
        for event, schema in zip(events[:4], schemas.values()):
            self.assertIs(event[1], schema)

    def test_validate_schemas_submits_every_document_exactly_once(self):
        schemas, index, responsibilities, cases, coverage = _synthetic_inputs()
        captured = []
        with (
            mock.patch.object(support, "_load_conformance_schemas", return_value=schemas),
            mock.patch.object(support.Draft202012Validator, "check_schema"),
            mock.patch.object(
                support,
                "_run_schema_validation_jobs",
                side_effect=lambda jobs: captured.extend(jobs),
            ),
        ):
            support.validate_schemas(index, responsibilities, cases, coverage)
        expected = [index, *responsibilities.values(), *cases.values(), *coverage]
        self.assertEqual(len(captured), 7)
        self.assertEqual(len({id(job.value) for job in captured}), 7)
        for job, value in zip(captured, expected):
            self.assertIs(job.value, value)

    def test_one_cpu_fallback_uses_one_process_worker(self):
        jobs = _jobs()[:3]
        worker_counts = []
        executor = _Executor(
            [_Future(value=job.index) for job in jobs], worker_counts=worker_counts
        )
        support._run_schema_validation_jobs(
            jobs, logical_cpu_count=1, executor_factory=executor.factory
        )
        self.assertEqual(worker_counts, [1])
        self.assertEqual(
            [arguments[0] for _, arguments in executor.submissions], [0, 1, 2]
        )
        self.assertEqual(len(executor.submissions), 3)


if __name__ == "__main__":
    unittest.main()
