from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/git-whitespace-validation.md"
EXTRACTION = ROOT / "docs/bootstrap/git-whitespace-validation-extraction.md"
README = ROOT / "README.md"
PROJECT = ROOT / "pyproject.toml"


class GitWhitespaceContractTests(unittest.TestCase):
    def test_readme_links_contract_and_extraction(self) -> None:
        readme = README.read_text(encoding="utf-8")
        self.assertIn(
            "[Git Whitespace Validation]"
            "(docs/contracts/git-whitespace-validation.md)",
            readme,
        )
        self.assertIn(
            "[Git Whitespace Validation extraction record]"
            "(docs/bootstrap/git-whitespace-validation-extraction.md)",
            readme,
        )

    def test_extraction_is_non_normative_and_records_exact_sources(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")
        for required in (
            "Status: non-normative extraction analysis.",
            "fanilosendrison/turnlock-rust",
            "7534a4667ec9190aec4021f90bfae2fa8a046dae",
            "7ccde07f4d9801383d6a2410e3d12ce6ee716f34",
            "`PROTO-RING`",
        ):
            self.assertIn(required, extraction)

    def test_contract_owns_exact_local_and_event_domains(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        normalized_contract = " ".join(contract.split())
        for required in (
            "git diff --check",
            "git diff --cached --check",
            "before..after",
            "0000000000000000000000000000000000000000",
            "base_sha...head_sha",
        ):
            self.assertIn(required, contract)
        for required in (
            "Git empty tree",
            "triple-dot",
            "distinct from two-dot",
            "No hard-coded SHA-1 empty-tree hash is normative",
        ):
            self.assertIn(required, normalized_contract)

    def test_contract_has_fail_closed_event_and_aggregate_boundary(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        for required in (
            "anything other than exactly `push` or",
            "no committed-range whitespace check is required",
            "invalid UTF-8",
            "malformed JSON",
            "PASS",
            "NON_PASS",
            "`SATISFIED`, `VIOLATED`, and `UNDETERMINED` are not standalone",
        ):
            self.assertIn(required, contract)

    def test_contract_keeps_git_and_diagnostics_in_their_boundaries(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        self.assertIn("delegated to Git `diff --check` semantics", contract)
        self.assertIn("diagnostic order are non-normative", contract)
        self.assertNotIn("Turnlock", contract)
        self.assertNotIn("Ruu", contract)

    def test_package_version_is_0_17_0(self) -> None:
        self.assertIn('version = "0.17.0"', PROJECT.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
