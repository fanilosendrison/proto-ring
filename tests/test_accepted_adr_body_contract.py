from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/accepted-adr-body-immutability.md"
EXTRACTION = ROOT / "docs/bootstrap/accepted-adr-body-immutability-extraction.md"
IMPLEMENTATION = ROOT / "src/proto_ring/accepted_adr_body.py"


class AcceptedAdrBodyContractTests(unittest.TestCase):
    def test_contract_and_implementation_contain_no_consumer_bindings(self) -> None:
        forbidden = (
            "Turnlock",
            "Ruu",
            "ADR-051",
            "ADR-085",
            "scripts/adr-metadata.py",
            "tools/adr-metadata.py",
        )
        for path in (CONTRACT, IMPLEMENTATION):
            content = path.read_text(encoding="utf-8")
            for value in forbidden:
                with self.subTest(path=path, value=value):
                    self.assertNotIn(value, content)

    def test_extraction_records_required_consumer_evidence(self) -> None:
        extraction = EXTRACTION.read_text(encoding="utf-8")
        for value in (
            "Turnlock",
            "Ruu",
            "metadata-migration-evidence",
            "ADR-031",
            "first-parent",
            "decision body",
        ):
            with self.subTest(value=value):
                self.assertIn(value, extraction)

    def test_contract_states_history_configuration_and_threat_boundaries(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        for value in (
            "non-shallow",
            "first-parent",
            "GIT_NO_REPLACE_OBJECTS",
            "current Canonical ADR Identity profile",
            "profile routing",
            "outside this contract",
            "force-push",
            "history rewrite",
        ):
            with self.subTest(value=value):
                self.assertIn(value, contract)


if __name__ == "__main__":
    unittest.main()
