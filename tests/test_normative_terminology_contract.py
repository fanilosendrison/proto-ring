from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "normative-terminology.md"
EXTRACTION = ROOT / "docs" / "bootstrap" / "normative-terminology-extraction.md"
IMPLEMENTATIONS = (
    ROOT / "src" / "proto_ring" / "normative_terminology.py",
    ROOT / "src" / "proto_ring" / "normative_terminology_matching.py",
    ROOT / "src" / "proto_ring" / "normative_terminology_inventory.py",
)


class NormativeTerminologyContractTests(unittest.TestCase):
    def test_contract_preserves_non_proof_and_consumer_authority_boundaries(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")

        self.assertIn("heuristic and structural analysis, not semantic proof", contract)
        self.assertIn("semantic equivalence", contract)
        self.assertIn("completeness of a terminology corpus", contract)
        self.assertIn("consumer supplies", contract)
        self.assertIn("Section labels are opaque consumer values", contract)
        self.assertIn("Actual and maintained signatures are compared as multisets", contract)
        self.assertIn("restricted GFM-style table", contract)
        self.assertIn("no escaped-pipe interpretation", contract)
        self.assertNotIn("Turnlock", contract)
        self.assertNotIn("Ruu", contract)
        self.assertNotIn("Section 2", contract)
        self.assertNotIn("turnlock-spec.md", contract)
        self.assertNotIn("terminology-inventory.yaml", contract)

    def test_extraction_records_exact_evidence_and_all_disposition_classes(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")

        self.assertIn("ddb6b766bd9ca779ec95ee214996831cd86f68ef", extraction)
        self.assertIn("8ce798de15ee67cfd17cf55128eccd61f94ef0fa", extraction)
        self.assertIn("4b139e2858fcb26b694503386096eb3fcea80d31", extraction)
        self.assertIn("absence of a Ruu Normative Terminology implementation", extraction)
        for disposition in ("`EXTRACT`", "`TURNLOCK`", "`EXTERNAL`", "`UNRESOLVED`"):
            self.assertIn(disposition, extraction)
        self.assertIn("No candidate remains `UNRESOLVED`", extraction)

    def test_shared_implementation_contains_no_consumer_policy(self) -> None:
        implementation = "\n".join(
            path.read_text(encoding="utf-8") for path in IMPLEMENTATIONS
        )
        forbidden = (
            "Turnlock",
            "Ruu",
            "Section 2",
            "turnlock-spec.md",
            "terminology-inventory.yaml",
            "TL-INV",
            '"intent"',
            '"obligation"',
            '"implication"',
            '"synopsis"',
        )
        for value in forbidden:
            with self.subTest(value=value):
                self.assertNotIn(value, implementation)

    def test_readme_links_contract_and_package_has_feature_version(self) -> None:
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        project = (ROOT / "pyproject.toml").read_text(encoding="utf-8")

        self.assertIn(
            "[Normative Terminology](docs/contracts/normative-terminology.md)",
            readme,
        )
        self.assertIn('version = "0.4.0"', project)


if __name__ == "__main__":
    unittest.main()
