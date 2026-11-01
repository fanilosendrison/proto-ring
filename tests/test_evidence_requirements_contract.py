from __future__ import annotations

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "evidence-requirements.md"
EXACT_CONTRACT = ROOT / "docs" / "contracts" / "exact-evidence-binding.md"
README = ROOT / "README.md"


class EvidenceRequirementsContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contract = CONTRACT.read_text(encoding="utf-8")

    def test_contract_has_exact_identity_and_frontmatter(self) -> None:
        expected = """---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-evidence-requirements"
severity: "strict"
name: "Canonical Evidence Requirements contract"
---
"""
        self.assertTrue(self.contract.startswith(expected))
        self.assertIn("# Canonical Evidence Requirements contract", self.contract)

    def test_contract_defines_exact_registry_and_forms(self) -> None:
        required = (
            "evidence_requirements:",
            "model_version: 1",
            "<EvidenceRequirementId>",
            "kind: single",
            "kind: source",
            "kind: explicit",
            "required: false",
            "required: true",
            "candidates:",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.contract)

    def test_contract_keeps_sources_and_object_identities_consumer_owned(self) -> None:
        normalized = " ".join(self.contract.split())
        required = (
            "Instance, class, subject, context, and candidate sources have no universal",
            "GovernedObjectRef != Exact Evidence Binding subject identity",
            "does not encode `interface_id/object_id` into subject or context tokens",
            "Concrete instance identity remains consumer-owned",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, normalized)

    def test_contract_composes_without_absorbing_exact_binding_or_decisions(self) -> None:
        required = (
            "concrete EvidenceRequirement + candidate EvidenceBinding values",
            "MATCH | MISMATCH | UNDETERMINED",
            "Exact Evidence Binding remains the sole lower-level comparator",
            "defines no any-match, all-match, latest-wins, ranking, majority",
            "Loading the registry is read-only",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.contract)

    def test_contract_confronts_both_consumers(self) -> None:
        self.assertIn("Turnlock Gate A", self.contract)
        self.assertIn("Ruu post-baseline", self.contract)

    def test_exact_evidence_binding_contract_remains_separate(self) -> None:
        self.assertTrue(EXACT_CONTRACT.is_file())
        self.assertNotIn(
            "Evidence Requirement Registry",
            EXACT_CONTRACT.read_text(encoding="utf-8"),
        )

    def test_readme_links_new_shared_contract(self) -> None:
        self.assertIn(
            "[Evidence Requirements](docs/contracts/evidence-requirements.md)",
            README.read_text(encoding="utf-8"),
        )


if __name__ == "__main__":
    unittest.main()
