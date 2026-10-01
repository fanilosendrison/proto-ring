from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
CONTRACT = Path("docs/contracts/projection-integrity.md")
INVENTORY = Path("docs/bootstrap/projection-integrity-extraction.md")
IMPLEMENTATION = Path("src/proto_ring/projection_registry.py")


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

    def test_consumer_confrontation_adds_no_concrete_owner_or_install_mapping(self) -> None:
        contract = self.read(CONTRACT).lower()
        self.assertIn("turnlock-rust", contract)
        self.assertIn("ruu relations", contract)
        forbidden = (
            "docs/",
            "formal/",
            "qualification/",
            ".github/",
            "requirements.txt",
            "pip install",
            "virtualenv",
            "tl-inv",
            "project #",
        )
        for value in forbidden:
            with self.subTest(value=value):
                self.assertNotIn(value, contract)

    def test_contract_defines_persistent_registry_and_compositions(self) -> None:
        contract = self.read(CONTRACT)
        required = (
            "## Persistent Projection Registry",
            "projection_registry:",
            "model_version: 1",
            "canonical_source:",
            "secondary_source:",
            "validation: <ValidationId>",
            "generator_source: <GovernedSourceId>",
            "boundary_source: <GovernedSourceId>",
            "binding: <BindingId>",
            "projection_integrity declared",
            "repository_integrity must also be declared",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, contract)

    def test_registry_preserves_direct_source_modes_and_boundaries(self) -> None:
        contract = " ".join(self.read(CONTRACT).split())
        requirements = (
            "must have Governance Authority role `authority`",
            "must have role `secondary_representation`",
            "generator source must exist and have role `non_authoritative`",
            "Historical validation establishes only the declared boundary",
            "Both relations point directly to the authority",
            "must not contain conflicting canonical-source or mode declarations",
        )
        for requirement in requirements:
            with self.subTest(requirement=requirement):
                self.assertIn(requirement, contract)

    def test_registry_implementation_has_no_execution_or_install_mechanism(self) -> None:
        implementation = self.read(IMPLEMENTATION)
        for forbidden in (
            "subprocess",
            "sys.executable",
            "pip",
            "requirements.txt",
            "virtualenv",
            "sha256",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, implementation)

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
