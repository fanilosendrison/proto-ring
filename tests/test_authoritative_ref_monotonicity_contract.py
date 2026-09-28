from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "authoritative-ref-monotonicity.md"
EXTRACTION = (
    ROOT / "docs" / "bootstrap" / "authoritative-ref-monotonicity-extraction.md"
)


class AuthoritativeRefMonotonicityContractTests(unittest.TestCase):
    def test_contract_is_consumer_and_provider_neutral(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        forbidden = (
            "Turnlock",
            "Ruu",
            "fanilosendrison",
            "github.com",
            "api.github.com",
            "GitHub ruleset",
            "refs/heads/main",
            "ADR-051",
            "ADR-085",
        )
        for value in forbidden:
            with self.subTest(value=value):
                self.assertNotIn(value, contract)

    def test_contract_states_required_semantics(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        required = (
            "authoritative ref",
            "ancestor-or-equal",
            "non-fast-forward",
            "deletion",
            "ordinary repository writer",
            "external enforcement",
            "local binding",
            "protection-control-plane administrator",
            "Authoritative Ref Monotonicity is not Authoritative State Admission",
        )
        for value in required:
            with self.subTest(value=value):
                self.assertIn(value, contract)

    def test_extraction_records_required_evidence(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")
        required = (
            "Turnlock",
            "Ruu",
            "d8b8c9f48fae43006a0764ac8542160671ede474",
            "ed187c0c58b626f82404ab475d9dc4c4b36330fd",
            "fast-forward",
            "force-push",
            "Accepted ADR Body Immutability",
            "not derived as an intersection",
        )
        for value in required:
            with self.subTest(value=value):
                self.assertIn(value, extraction)

    def test_contract_rejects_unnecessary_mechanism_requirements(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        for non_goal in (
            "pull request",
            "status checks",
            "linear history",
            "signed commits",
        ):
            with self.subTest(non_goal=non_goal):
                self.assertIn(non_goal, contract)


if __name__ == "__main__":
    unittest.main()
