from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
CONTRACT = Path("docs/contracts/projection-integrity.md")
INVENTORY = Path("docs/bootstrap/projection-integrity-extraction.md")


class ProjectionIntegrityContractTests(unittest.TestCase):
    def read(self, path: Path) -> str:
        return (REPOSITORY_ROOT / path).read_text(encoding="utf-8")

    def test_contract_defines_one_owner_and_four_projection_modes(self) -> None:
        contract = self.read(CONTRACT)
        self.assertIn("one canonical owner", contract)
        for mode in (
            "Reference",
            "Generated projection",
            "Mechanically validated maintained projection",
            "Bounded historical snapshot",
        ):
            with self.subTest(mode=mode):
                self.assertIn(f"### {mode}", contract)

    def test_contract_preserves_required_generic_guarantees(self) -> None:
        contract = " ".join(self.read(CONTRACT).split())
        requirements = (
            "Projection chains are forbidden",
            "does not acquire semantic authority",
            "must have one canonical owner",
            "incomplete while any required projection is stale",
            "Agent memory, review attention, and ordinary diligence are not synchronization mechanisms",
            "mechanical guard must cover every duplicated derivable field",
            "must derive directly from the canonical owner",
            "must fail closed",
        )
        for requirement in requirements:
            with self.subTest(requirement=requirement):
                self.assertIn(requirement, contract)

    def test_contract_contains_no_consumer_owner_mapping(self) -> None:
        contract = self.read(CONTRACT).lower()
        forbidden = (
            "turnlock",
            "ruu",
            "docs/",
            "formal/",
            "qualification/",
            ".github/",
            "adr-",
            "tl-inv",
            "project #",
        )
        for value in forbidden:
            with self.subTest(value=value):
                self.assertNotIn(value, contract)

    def test_inventory_is_turnlock_first_and_ruu_confronted(self) -> None:
        inventory = self.read(INVENTORY)
        self.assertIn(
            "fanilosendrison/turnlock-rust\neb9d26d6f27a164272cd5a2ca734c19fc4459c90",
            inventory,
        )
        self.assertIn(
            "fanilosendrison/ruu\nfb76cc340a9167c73b455259114d3d3691a9f692",
            inventory,
        )
        self.assertIn("maximum justified Turnlock candidate", inventory)
        self.assertIn("Ruu confrontation", inventory)
        self.assertIn("absence alone was not used to reject", inventory)

    def test_inventory_keeps_governed_identity_findings_separate(self) -> None:
        inventory = self.read(INVENTORY)
        self.assertIn("fanilosendrison/turnlock-rust#35", inventory)
        self.assertIn("fanilosendrison/ruu#49", inventory)
        self.assertIn("remain open", inventory)
        self.assertIn("not incorporated", inventory)
        self.assertNotIn("minimum-sufficient", self.read(CONTRACT).lower())


if __name__ == "__main__":
    unittest.main()
