from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "canonical-governance-routing.md"
EXTRACTION = (
    ROOT / "docs" / "bootstrap" / "canonical-governance-routing-extraction.md"
)
IMPLEMENTATION = ROOT / "src" / "proto_ring" / "governance_routing.py"


class GovernanceRoutingContractTests(unittest.TestCase):
    def test_contract_exists(self) -> None:
        self.assertTrue(CONTRACT.is_file())

    def test_extraction_record_exists(self) -> None:
        self.assertTrue(EXTRACTION.is_file())

    def test_extraction_records_exact_evidence_baselines(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")

        for sha in (
            "c298343efa07d71790b53493ff15494f2fcac671",
            "0c2fd68974466df37105682d9aa8d034ed22dad2",
            "e27adc9290b4781396cb5c9745014ca5427e98ac",
        ):
            with self.subTest(sha=sha):
                self.assertIn(sha, extraction)

    def test_extraction_contains_every_disposition_class(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")

        for disposition in ("`EXTRACT`", "`NARROW`", "`CONSUMER`"):
            with self.subTest(disposition=disposition):
                self.assertIn(disposition, extraction)
        self.assertIn("No candidate remains unresolved", extraction)

    def test_extraction_keeps_carrier_and_vocabulary_consumer_owned(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")

        self.assertIn("carrier serialization/parsing", extraction)
        self.assertIn("routing vocabulary", extraction)
        self.assertIn("remain consumer-owned", extraction)

    def test_contract_requires_consumer_supplied_route_selection(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")

        self.assertIn("one exact route supplied by the consumer", contract)
        self.assertIn("route selection", contract)

    def test_contract_prohibits_fallback_and_discovery(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")

        self.assertIn("performs no sibling search", contract)
        self.assertIn("does not trigger discovery", contract)
        self.assertIn("convention", contract)
        self.assertIn("fallback", contract)

    def test_implementation_contains_no_consumer_specific_vocabulary(self) -> None:
        implementation = IMPLEMENTATION.read_text(encoding="utf-8")
        forbidden = (
            "Turnlock",
            "Ruu",
            "AGENTS.md",
            "repository_governance",
            "architecture_decisions",
            "profile_path",
            "shared_governance_provider",
            "binding_path",
        )

        for value in forbidden:
            with self.subTest(value=value):
                self.assertNotIn(value, implementation)

    def test_readme_links_contract_and_extraction_record(self) -> None:
        readme = (ROOT / "README.md").read_text(encoding="utf-8")

        self.assertIn(
            "[Canonical Governance Routing]"
            "(docs/contracts/canonical-governance-routing.md)",
            readme,
        )
        self.assertIn(
            "[Canonical Governance Routing extraction record]"
            "(docs/bootstrap/canonical-governance-routing-extraction.md)",
            readme,
        )

    def test_package_version_is_0_10_1(self) -> None:
        project = (ROOT / "pyproject.toml").read_text(encoding="utf-8")

        self.assertIn('version = "0.10.1"', project)


if __name__ == "__main__":
    unittest.main()
