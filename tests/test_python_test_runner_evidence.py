from __future__ import annotations

import copy
import json
from pathlib import Path
import re
import shutil
import sys
import tempfile
import unittest
from unittest import mock

TOOLS_DIRECTORY = Path(__file__).resolve().parents[1] / "tools"
sys.path.insert(0, str(TOOLS_DIRECTORY))
import python_test_runner_evidence as evidence


class FixedUuid:
    def __init__(self, value: str) -> None:
        self.hex = value


def canonical(payload: dict[str, object]) -> bytes:
    return (json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


class PythonTestRunnerEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def sources(self, *, same_bytes: bool = False) -> dict[str, Path]:
        source = self.root / f"sources-{len(list(self.root.glob('sources-*')))}"
        source.mkdir()
        observations = source / "observations.json"
        metrics = source / "metrics.txt"
        observations.write_bytes(b"same\x00bytes" if same_bytes else b'{"value":1}\n')
        metrics.write_bytes(b"same\x00bytes" if same_bytes else b"STATUS=PASS\n")
        return {"actual_observations": observations, "execution_metrics": metrics}

    @staticmethod
    def runner_record(status: str = "PASS") -> evidence.RunnerRecord:
        return evidence.RunnerRecord(status, 4, 68, 856, 856, ())

    def valid_run(self, name: str = "evidence") -> tuple[evidence.EvidenceRun, Path]:
        run = evidence.prepare_evidence_run(self.root / name)
        self.assertIsNotNone(run)
        assert run is not None
        evidence.export_artifacts(run, self.sources())
        handoff = evidence.finalize_handoff(run, self.runner_record())
        return run, handoff

    def rewrite(self, handoff: Path, mutate) -> None:
        payload = json.loads(handoff.read_text(encoding="utf-8"))
        mutate(payload)
        handoff.write_bytes(canonical(payload))

    def assert_rejected(self, handoff: Path) -> None:
        with self.assertRaises(evidence.EvidenceError):
            evidence.verify_handoff(handoff)

    def test_run_identity_and_namespace_are_unique_and_exact(self) -> None:
        values = ["0" * 32, "abcdef0123456789" * 2]
        with mock.patch.object(evidence.uuid, "uuid4", side_effect=[FixedUuid(v) for v in values]) as uuid4:
            first = evidence.prepare_evidence_run(self.root / "root")
            second = evidence.prepare_evidence_run(self.root / "root")
        assert first is not None and second is not None
        self.assertEqual((first.run_id, second.run_id), tuple(values))
        self.assertNotEqual(first.run_id, second.run_id)
        self.assertNotEqual(first.namespace, second.namespace)
        self.assertTrue(all(re.fullmatch(r"[0-9a-f]{32}", run.run_id) for run in (first, second)))
        self.assertEqual(uuid4.call_count, 2)

    def test_disabled_export_creates_nothing(self) -> None:
        root = self.root / "disabled"
        self.assertIsNone(evidence.prepare_evidence_run(None))
        self.assertFalse(root.exists())

    def test_preexisting_namespace_collision_fails_without_regeneration(self) -> None:
        run_id = "a" * 32
        root = self.root / "collision"
        (root / run_id).mkdir(parents=True)
        with mock.patch.object(evidence.uuid, "uuid4", return_value=FixedUuid(run_id)) as uuid4:
            with self.assertRaises(evidence.EvidenceError):
                evidence.prepare_evidence_run(root)
        uuid4.assert_called_once_with()

    def test_export_preserves_exact_bytes_size_and_sha256(self) -> None:
        run = evidence.prepare_evidence_run(self.root / "export")
        assert run is not None
        sources = self.sources()
        evidence.export_artifacts(run, sources)
        self.assertEqual(set(run.artifacts), set(evidence.ARTIFACT_FILENAMES))
        for key, source in sources.items():
            binding = run.artifacts[key]
            destination = run.namespace / binding.path
            self.assertEqual(destination.read_bytes(), source.read_bytes())
            self.assertEqual(binding.size, len(source.read_bytes()))
            self.assertEqual(binding.sha256, evidence.sha256_bytes(source.read_bytes()))

    def test_export_missing_source_and_destination_failure_fail_closed(self) -> None:
        run = evidence.prepare_evidence_run(self.root / "missing")
        assert run is not None
        sources = self.sources()
        sources["actual_observations"].unlink()
        with self.assertRaises(evidence.EvidenceError):
            evidence.export_artifacts(run, sources)
        run = evidence.prepare_evidence_run(self.root / "write-failure")
        assert run is not None
        sources = self.sources()
        (run.namespace / "actual-observations.json").mkdir()
        with self.assertRaises(evidence.EvidenceError):
            evidence.export_artifacts(run, sources)

    def test_export_destination_cannot_escape_namespace(self) -> None:
        run = evidence.prepare_evidence_run(self.root / "escape")
        assert run is not None
        with mock.patch.dict(
            evidence.ARTIFACT_FILENAMES,
            {"actual_observations": "../actual-observations.json"},
        ):
            with self.assertRaises(evidence.EvidenceError):
                evidence.export_artifacts(run, self.sources())
        self.assertFalse((run.namespace.parent / "actual-observations.json").exists())

    def test_valid_handoff_has_exact_schema_and_verifies(self) -> None:
        run, handoff = self.valid_run()
        payload = evidence.verify_handoff(handoff)
        self.assertEqual(set(payload), {"schema_version", "run_id", "runner", "artifacts"})
        self.assertEqual(payload["schema_version"], 1)
        self.assertEqual(payload["run_id"], run.run_id)
        self.assertEqual(
            set(payload["runner"]),
            {"status", "workers", "atomic_tasks", "discovered_tests", "executed_tests", "failed_tasks"},
        )
        self.assertEqual(set(payload["artifacts"]), set(evidence.ARTIFACT_FILENAMES))
        for key, filename in evidence.ARTIFACT_FILENAMES.items():
            self.assertEqual(payload["artifacts"][key]["path"], filename)
            self.assertEqual(set(payload["artifacts"][key]), {"path", "size", "sha256"})
        self.assertEqual(handoff.read_bytes(), canonical(payload))

    def test_missing_invalid_utf8_and_malformed_json_are_rejected(self) -> None:
        missing = self.root / "missing" / "handoff.json"
        self.assert_rejected(missing)
        parent = self.root / ("b" * 32)
        parent.mkdir()
        handoff = parent / "handoff.json"
        for content in (b"\xff", b"{"):
            handoff.write_bytes(content)
            self.assert_rejected(handoff)

    def test_handoff_identity_and_top_level_contract_fail_closed(self) -> None:
        mutations = (
            lambda p: p.update(extra=True),
            lambda p: p.update(schema_version=2),
            lambda p: p.update(schema_version=1.0),
            lambda p: p.update(run_id="A" * 32),
            lambda p: p.update(run_id="a" * 31),
        )
        for index, mutation in enumerate(mutations):
            with self.subTest(index=index):
                _, handoff = self.valid_run(f"identity-{index}")
                self.rewrite(handoff, mutation)
                self.assert_rejected(handoff)
        run, handoff = self.valid_run("parent-mismatch")
        wrong_parent = self.root / ("c" * 32)
        shutil.copytree(run.namespace, wrong_parent)
        self.assert_rejected(wrong_parent / handoff.name)

    def test_runner_record_validation_fails_closed(self) -> None:
        def bad_keys(payload):
            payload["runner"]["extra"] = 1

        def unknown_status(payload):
            payload["runner"]["status"] = "UNKNOWN"

        def bad_accounting(payload):
            payload["runner"]["workers"] = True

        def negative_accounting(payload):
            payload["runner"]["executed_tests"] = -1

        def bad_failed_tasks(payload):
            payload["runner"]["failed_tasks"] = "task"

        def bad_failed_task_item(payload):
            payload["runner"]["failed_tasks"] = [1]

        for index, mutation in enumerate((bad_keys, unknown_status, bad_accounting,
                                          negative_accounting, bad_failed_tasks, bad_failed_task_item)):
            with self.subTest(index=index):
                _, handoff = self.valid_run(f"runner-{index}")
                self.rewrite(handoff, mutation)
                self.assert_rejected(handoff)

    def test_artifact_contract_and_path_confinement_fail_closed(self) -> None:
        def artifact(payload):
            return payload["artifacts"]["actual_observations"]

        mutations = (
            lambda p: p["artifacts"].update(extra={}),
            lambda p: artifact(p).update(extra=1),
            lambda p: artifact(p).update(path="/tmp/actual-observations.json"),
            lambda p: artifact(p).update(path="../actual-observations.json"),
            lambda p: artifact(p).update(path="execution-metrics.txt"),
            lambda p: artifact(p).update(size=-1),
            lambda p: artifact(p).update(sha256="A" * 64),
            lambda p: artifact(p).update(sha256="a" * 63),
        )
        for index, mutation in enumerate(mutations):
            with self.subTest(index=index):
                _, handoff = self.valid_run(f"artifact-{index}")
                self.rewrite(handoff, mutation)
                self.assert_rejected(handoff)

    def test_missing_wrong_size_modified_and_missing_artifact_are_rejected(self) -> None:
        run, handoff = self.valid_run("wrong-size")
        self.rewrite(handoff, lambda p: p["artifacts"]["actual_observations"].update(size=0))
        self.assert_rejected(handoff)
        run, handoff = self.valid_run("wrong-digest")
        self.rewrite(handoff, lambda p: p["artifacts"]["actual_observations"].update(sha256="0" * 64))
        self.assert_rejected(handoff)
        for name, action in (
            ("modified", lambda path: path.write_bytes(path.read_bytes() + b"changed")),
            ("deleted", lambda path: path.unlink()),
        ):
            with self.subTest(name=name):
                run, handoff = self.valid_run(name)
                action(run.namespace / "actual-observations.json")
                self.assert_rejected(handoff)

    def test_cross_run_handoffs_bind_to_exact_parent_even_for_identical_bytes(self) -> None:
        sources = self.sources(same_bytes=True)
        run_a = evidence.prepare_evidence_run(self.root / "shared")
        run_b = evidence.prepare_evidence_run(self.root / "shared")
        assert run_a is not None and run_b is not None
        evidence.export_artifacts(run_a, sources)
        evidence.export_artifacts(run_b, sources)
        handoff_a = evidence.finalize_handoff(run_a, self.runner_record())
        handoff_b = evidence.finalize_handoff(run_b, self.runner_record())
        bytes_a, bytes_b = handoff_a.read_bytes(), handoff_b.read_bytes()
        self.assertNotEqual(run_a.run_id, run_b.run_id)
        evidence.verify_handoff(handoff_a)
        evidence.verify_handoff(handoff_b)
        handoff_a.write_bytes(bytes_b)
        self.assert_rejected(handoff_a)
        handoff_a.write_bytes(bytes_a)
        handoff_b.write_bytes(bytes_a)
        self.assert_rejected(handoff_b)
        self.assertEqual(
            (run_a.namespace / "actual-observations.json").read_bytes(),
            (run_b.namespace / "actual-observations.json").read_bytes(),
        )
        self.assertNotEqual(run_a.run_id, run_b.run_id)


if __name__ == "__main__":
    unittest.main()
