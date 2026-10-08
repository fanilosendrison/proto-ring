from __future__ import annotations

import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

import conformance_python_adapter_66_evidence as evidence_adapter
from conformance_corpus_test_support import decode_transport, load_corpus, transport


class Issue90AdapterBoundaryTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, _, cases, _ = load_corpus()
        cls.vector = next(
            vector
            for vector in cases["exact-evidence-binding.evaluate"]["vectors"]
            if vector["vector_id"]
            == "exact-evidence-binding.evaluate.match-no-context"
        )

    def _assert_boundary(self, field: str, invalid_value: object) -> None:
        fixture = copy.deepcopy(self.vector["fixture"])
        input_key = "invocation" if "invocation" in fixture else "input"
        arguments = decode_transport(fixture[input_key])
        arguments["requirement"][field] = invalid_value
        fixture[input_key] = transport(arguments)
        request = [
            {
                "fixture": fixture,
                "responsibility_id": "exact-evidence-binding.evaluate",
                "vector_id": f"issue90.transport-boundary.{field}",
            }
        ]
        with tempfile.TemporaryDirectory() as directory:
            request_path = Path(directory) / "request.json"
            output_path = Path(directory) / "output.json"
            request_path.write_text(json.dumps(request), encoding="utf-8")
            completed = subprocess.run(
                [
                    sys.executable,
                    "-B",
                    "tests/conformance_python_bridge.py",
                    "--request",
                    str(request_path),
                    "--output",
                    str(output_path),
                ],
                cwd=Path(__file__).resolve().parents[1],
                capture_output=True,
                check=False,
            )
        self.assertNotEqual(completed.returncode, 0)
        with mock.patch.object(evidence_adapter, "evaluate") as evaluator:
            with self.assertRaises(TypeError):
                evidence_adapter.binding_status(
                    arguments["requirement"], arguments["binding"]
                )
        evaluator.assert_not_called()

    def test_admitted_classes_wrong_kind_fails_before_evaluation(self) -> None:
        self._assert_boundary("admitted_classes", {"class": "check"})

    def test_subject_wrong_kind_fails_before_evaluation(self) -> None:
        self._assert_boundary("subject_hex", 1)

    def test_context_required_wrong_kind_fails_before_evaluation(self) -> None:
        self._assert_boundary("context_required", 1)

    def test_context_wrong_kind_fails_before_evaluation(self) -> None:
        self._assert_boundary("context_hex", 1)


if __name__ == "__main__":
    unittest.main()
