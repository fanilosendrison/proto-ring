#!/usr/bin/env python3
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "repository-integrity.md"
MODEL = ROOT / "src" / "proto_ring" / "repository_integrity.py"
EVALUATION = ROOT / "src" / "proto_ring" / "repository_integrity_evaluation.py"


class RepositoryIntegrityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contract = CONTRACT.read_text(encoding="utf-8")
        cls.model = MODEL.read_text(encoding="utf-8")
        cls.evaluation = EVALUATION.read_text(encoding="utf-8")

    def test_contract_defines_one_persistent_model_version_one_profile(self) -> None:
        required = (
            "## Persistent consumer profile",
            "repository_integrity:",
            "model_version: 1",
            "authority:",
            "environments:",
            "continue_after_non_satisfied:",
            "validations:",
            "order:",
            "A `ValidationId` is an opaque, non-empty consumer-owned identity",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.contract)

    def test_contract_defines_exact_instantiation_and_selector_boundary(self) -> None:
        required = (
            "exactly `single` and `repository_paths`",
            "exactly `append_all` and `for_each`",
            "A literal `path` remains selected when absent",
            "only `*` as syntax",
            "Duplicate paths across selectors fail resolution",
            "Zero glob matches are valid",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.contract)

    def test_contract_owns_zero_instance_aggregation_and_prerequisite_gating(self) -> None:
        required = (
            "zero concrete instances\n→ SATISFIED",
            "`VIOLATED` has precedence over `UNDETERMINED`",
            "zero-instance `for_each` validation is deliberately",
            "does not prove that a\npath exists",
            "Prerequisites refer to aggregate `ValidationId` status",
            "zero-instance satisfied\nprerequisite therefore permits its dependent",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.contract)
        for forbidden in ("NOT_APPLICABLE =", "SKIPPED =", "EMPTY ="):
            self.assertNotIn(forbidden, self.contract)

    def test_append_all_zero_selection_remains_one_obligation(self) -> None:
        self.assertIn(
            "`append_all` always resolves to one obligation", self.contract
        )
        self.assertIn(
            "With zero selected paths, that one obligation contains only\n"
            "the base arguments",
            self.contract,
        )

    def test_profile_and_evaluation_context_identity_boundaries_are_explicit(self) -> None:
        required = (
            "Profile identity derives from canonical persistent profile semantics",
            "Evaluation-context identity\ndepends only on the opaque identities",
            "repository state identity\nprofile identity\nevaluation context identity",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.contract)
        self.assertNotIn("sys.executable", self.model)
        self.assertIn("_context_identity", self.evaluation)

    def test_exact_state_requires_unambiguous_binding_without_hiding_obligation_drift(self) -> None:
        normalized = " ".join(self.contract.split())
        required = (
            "MUST NOT be ambiguous because of unaccounted contract-relevant mutation",
            "requires a coherence basis sufficient for the claims made by the evaluation",
            "does not require total repository immobility",
            "through isolation, versioning, snapshots, or another mechanism",
            "identity equality remains mandatory drift detection; it is not proof that a coherence basis existed",
            "currently evaluated obligation remains observable as a possible mutator",
            "state change MUST continue to participate in the existing before/after RepositoryState comparison, state relation",
            "`SATISFIED`, `VIOLATED`, `UNDETERMINED`, `PASS`, and `NON_PASS` semantics",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, normalized)
        self.assertNotIn(
            "inherits RepositoryState's mutation-isolation requirement",
            normalized,
        )

    def test_implementation_exposes_persistent_profile_without_new_statuses(self) -> None:
        required = (
            "class ConsumerIntegrityProfile:",
            "class ValidationEnvironmentRealization:",
            "class EvaluationContext:",
            "def load(",
            "def evaluate_consumer_profile(",
            "evaluation_context_identity: str",
        )
        combined = self.model + self.evaluation
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, combined)
        for forbidden in ("NOT_APPLICABLE", "SKIPPED", "EMPTY"):
            self.assertNotIn(forbidden, combined)

    def test_implementation_keeps_profile_loading_separate_from_execution(self) -> None:
        load_region = self.model[self.model.index("def load(") :]
        self.assertNotIn("subprocess.run", load_region)
        self.assertNotIn("subprocess", self.model)
        self.assertIn("subprocess.run", self.evaluation)


if __name__ == "__main__":
    unittest.main()
