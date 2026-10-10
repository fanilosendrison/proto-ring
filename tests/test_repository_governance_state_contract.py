from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "repository-governance-state.md"
EXTRACTION = (
    ROOT / "docs" / "bootstrap" / "repository-governance-state-extraction.md"
)
README = ROOT / "README.md"
PRODUCTION = ROOT / "src" / "proto_ring"


class RepositoryGovernanceStateContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contract = CONTRACT.read_text(encoding="utf-8")
        cls.normalized = " ".join(cls.contract.split())
        cls.extraction = EXTRACTION.read_text(encoding="utf-8")

    def test_contract_has_canonical_identity(self) -> None:
        expected = """---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-repository-governance-state"
severity: "strict"
name: "Canonical RepositoryGovernanceState contract"
---
"""
        self.assertTrue(self.contract.startswith(expected))
        self.assertIn("# Canonical RepositoryGovernanceState contract", self.contract)

    def test_v1_state_composes_only_rgm_v2_without_changing_rgm_v1(self) -> None:
        required = (
            "`RepositoryGovernanceState` version 1 composes Repository Governance Model version 2 only",
            "Repository Governance Model version 1 remains valid and frozen",
            "It is not composable into `RepositoryGovernanceState` version 1",
            "introduces neither a Repository Governance Model version nor a capability",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.normalized)

    def test_state_content_and_capability_presence_are_exact(self) -> None:
        for component in (
            "Canonical Governance Bootstrap result",
            "Repository Governance Model version-2 value",
            "Governance Authority profile",
            "Governed Objects catalog if and only if `governed_objects` is declared",
            "Governance Binding Registry",
            "Consumer Repository Integrity profile if and only if `repository_integrity` is declared",
            "Projection Registry if and only if `projection_integrity` is declared",
            "Evidence Requirement Registry if and only if `evidence_requirements` is declared",
        ):
            with self.subTest(component=component):
                self.assertIn(component, self.normalized)
        self.assertIn("A partial state must not be returned", self.normalized)

    def test_exact_observation_is_normative_and_provider_neutral(self) -> None:
        required = (
            "A `RepositoryGovernanceState` MUST NOT be returned unless every contained governance value was composed between mandatory initial and final RepositoryState captures for one explicit observation scope and those captures compare equal.",
            "Repository-state drift or observation-scope drift during construction fails closed.",
            "initial RepositoryState → authoritative composition → final RepositoryState → state/scope drift comparison",
            "identity equality is mandatory drift detection, not proof that a coherence basis existed",
            "only when a coherence basis covers the authoritative construction boundary",
            "isolation, a snapshot or version substrate, an immutable execution workspace, or an equivalent mechanism",
            "does not require a specific realization",
            "read-only, non-executing, and provider-neutral",
            "does not prescribe a programming language, digest algorithm, Git command, implementation structure, or exact number of observation passes",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.normalized)
        self.assertNotIn(
            "inherits RepositoryState's mutation-isolation interval",
            self.normalized,
        )

    def test_model_only_capabilities_do_not_trigger_execution(self) -> None:
        self.assertIn("must not eagerly enumerate an ADR corpus", self.contract)
        self.assertIn("must not perform live or provider-specific observation", self.contract)
        self.assertIn("must not perform live or provider-specific observation, inspect forge rulesets, call a provider API, or access the network", self.normalized)

    def test_construction_is_non_executing_and_preserves_semantic_states(self) -> None:
        for forbidden_operation in (
            "execute Repository Integrity validators",
            "execute Projection Integrity validators or generators",
            "resolve runtime evidence candidates",
            "evaluate Exact Evidence Binding",
            "perform qualification or historical replay",
            "modify repository state",
        ):
            with self.subTest(operation=forbidden_operation):
                self.assertIn(forbidden_operation, self.contract)
        for state in ("`UNKNOWN`", "`UNDECLARED`", "`UNDETERMINED`"):
            self.assertIn(state, self.contract)

    def test_extraction_uses_both_consumers_without_transferring_authority(self) -> None:
        required = (
            "Turnlock-Rust",
            "scripts/check-structured-governance.py",
            "Ruu",
            "tools/check-structured-governance.py",
            "does not make either observed consumer a reference implementation",
            "Those values remain consumer-owned inputs",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, self.extraction)

    def test_readme_indexes_contract_and_extraction(self) -> None:
        readme = README.read_text(encoding="utf-8")
        self.assertIn(
            "[RepositoryGovernanceState](docs/contracts/repository-governance-state.md)",
            readme,
        )
        self.assertIn(
            "[RepositoryGovernanceState extraction record](docs/bootstrap/repository-governance-state-extraction.md)",
            readme,
        )

if __name__ == "__main__":
    unittest.main()
