from __future__ import annotations

import ast
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "exact-evidence-binding.md"
EXTRACTION = ROOT / "docs" / "bootstrap" / "exact-evidence-binding-extraction.md"
IMPLEMENTATION = ROOT / "src" / "proto_ring" / "exact_evidence_binding.py"
README = ROOT / "README.md"
PROJECT = ROOT / "pyproject.toml"

EXPECTED_BASELINES = (
    "1b767eb4e81a83e778c90649c632694c22131595",
    "0c2fd68974466df37105682d9aa8d034ed22dad2",
    "e27adc9290b4781396cb5c9745014ca5427e98ac",
)
FORBIDDEN_CONSUMER_TERMS = (
    "Turnlock",
    "Ruu",
    "Gate A",
    "TLC",
    "qualification",
    "review packet",
    "Git OID",
    "ADR-030",
    "ADR-043",
)


class ExactEvidenceBindingContractTests(unittest.TestCase):
    def test_contract_and_extraction_record_exist(self) -> None:
        self.assertTrue(CONTRACT.is_file())
        self.assertTrue(EXTRACTION.is_file())

    def test_contract_has_exact_frontmatter(self) -> None:
        expected = """---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-exact-evidence-binding"
severity: "strict"
name: "Exact Evidence Binding contract"
---
"""
        self.assertTrue(CONTRACT.read_text(encoding="utf-8").startswith(expected))

    def test_extraction_record_has_exact_frontmatter(self) -> None:
        expected = """---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Exact Evidence Binding extraction evidence"
---
"""
        self.assertTrue(EXTRACTION.read_text(encoding="utf-8").startswith(expected))

    def test_extraction_records_exact_baselines_and_issues(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")

        for baseline in EXPECTED_BASELINES:
            self.assertIn(baseline, extraction)
        self.assertIn("fanilosendrison/proto-ring#19", extraction)
        self.assertIn("fanilosendrison/proto-ring#20", extraction)

    def test_extraction_records_all_final_disposition_sections(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")

        self.assertIn("## EXTRACT", extraction)
        self.assertIn("## CONSUMER", extraction)
        self.assertIn("## Rejected over-generalization", extraction)
        self.assertIn("No candidate remains unresolved", extraction)

    def test_contract_denies_success_proof_truth_and_satisfaction_meanings(
        self,
    ) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")

        for statement in (
            "MATCH is not evidence success",
            "MATCH is not proof validity",
            "MATCH is not claim truth",
            "MATCH is not obligation satisfaction",
        ):
            self.assertIn(statement, contract)

    def test_contract_distinguishes_mismatch_from_undetermined(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")

        self.assertIn("MISMATCH and UNDETERMINED are distinct", contract)
        self.assertIn(
            "all binding information required to compare the relevant dimension is known",
            contract,
        )
        self.assertIn(
            "at least one mandatory current or observed binding identity required for",
            contract,
        )
        self.assertIn("evaluation is unavailable or malformed", contract)

    def test_contract_preserves_historical_and_current_requirements(self) -> None:
        contract = " ".join(CONTRACT.read_text(encoding="utf-8").split())

        self.assertIn(
            "historically bound evidence may remain historically inspectable even when it "
            "does not match a later current requirement",
            contract,
        )

    def test_contract_names_consumer_authority_and_class_boundary(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")

        self.assertIn("## Consumer authority boundary", contract)
        self.assertIn("No implicit evidence substitution", contract)
        self.assertIn("Evidence-class vocabulary remains consumer-owned", contract)

    def test_contract_defines_repository_integrity_relationship(self) -> None:
        contract = " ".join(CONTRACT.read_text(encoding="utf-8").split())

        self.assertIn(
            "determines only whether one candidate evidence binding matches one",
            contract,
        )
        self.assertIn(
            "may execute a consumer-owned obligation that uses Exact Evidence Binding",
            contract,
        )
        self.assertIn(
            "Repository Integrity does not become the owner of evidence semantics",
            contract,
        )

    def test_generic_module_contains_no_consumer_vocabulary(self) -> None:
        implementation = IMPLEMENTATION.read_text(encoding="utf-8")

        for term in FORBIDDEN_CONSUMER_TERMS:
            with self.subTest(term=term):
                self.assertNotIn(term, implementation)

    def test_generic_module_has_no_io_or_domain_identity_dependency(self) -> None:
        tree = ast.parse(IMPLEMENTATION.read_text(encoding="utf-8"))
        imported_roots: set[str] = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                imported_roots.update(alias.name.split(".")[0] for alias in node.names)
            elif isinstance(node, ast.ImportFrom) and node.module:
                imported_roots.add(node.module.split(".")[0])

        self.assertEqual({"__future__", "dataclasses", "enum"}, imported_roots)
        forbidden_calls = {
            "open",
            "exec",
            "eval",
            "compile",
            "input",
            "print",
        }
        called_names = {
            node.func.id
            for node in ast.walk(tree)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        }
        self.assertTrue(forbidden_calls.isdisjoint(called_names))

    def test_readme_links_contract_and_extraction_record(self) -> None:
        readme = README.read_text(encoding="utf-8")

        self.assertIn(
            "[Exact Evidence Binding](docs/contracts/exact-evidence-binding.md)",
            readme,
        )
        self.assertIn(
            "[Exact Evidence Binding extraction record]"
            "(docs/bootstrap/exact-evidence-binding-extraction.md)",
            readme,
        )

    def test_package_version_is_0_10_0(self) -> None:
        project = PROJECT.read_text(encoding="utf-8")

        self.assertIn('version = "0.10.0"', project)
        self.assertNotIn('version = "0.9.0"', project)


if __name__ == "__main__":
    unittest.main()
