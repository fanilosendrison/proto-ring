from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = Path("docs/contracts/adr-metadata-primitives.md")
INVENTORY_PATH = Path("docs/bootstrap/adr-metadata-primitives-extraction.md")
IMPLEMENTATION_PATH = Path("src/proto_ring/adr_metadata.py")


class AdrMetadataContractTests(unittest.TestCase):
    def read(self, path: Path) -> str:
        return (REPOSITORY_ROOT / path).read_text(encoding="utf-8")

    def test_shared_layer_contains_no_consumer_binding(self) -> None:
        contract = self.read(CONTRACT_PATH).lower()
        implementation = self.read(IMPLEMENTATION_PATH).lower()
        forbidden = (
            "turnlock",
            "ruu",
            "scripts/adr-metadata.py",
            "tools/adr-metadata.py",
            "docs/adr",
            "adr-017",
            "adr-082",
        )
        for value in forbidden:
            with self.subTest(value=value):
                self.assertNotIn(value, contract)
                self.assertNotIn(value, implementation)

    def test_contract_excludes_consumer_authority(self) -> None:
        contract = self.read(CONTRACT_PATH)
        for boundary in (
            "an ADR corpus or accepted decision history",
            "a universal profile or profile version",
            "schema provenance or canonical schema ownership",
            "migration baselines or migration evidence",
            "qualification or formal evidence",
            "consumer validation membership and ordering",
        ):
            with self.subTest(boundary=boundary):
                self.assertIn(boundary, contract)

    def test_inventory_records_every_candidate_disposition(self) -> None:
        inventory = self.read(INVENTORY_PATH)
        extracted = (
            "Safe YAML loading",
            "Safe JSON loading",
            "Exact SHA-256 hashing",
            "Exact decision-body detection",
            "Exact preserved payload detection",
            "Safe ADR frontmatter parsing",
            "H1 text detection",
            "JSON Schema validation",
            "Repository-relative containment",
            "Structured configuration requirements",
            "Relation target checking",
        )
        retained = (
            "Profile constants and profile loading",
            "Canonical-schema and overlay provenance checks",
            "ADR discovery and repository identity policy",
            "Lifecycle and date policy",
            "Migration evidence",
            "Annotated history",
            "Rendering and generated-index policy",
            "Commands, orchestration, and diagnostics",
        )
        for candidate in (*extracted, *retained):
            with self.subTest(candidate=candidate):
                self.assertIn(f"### {candidate}", inventory)
        self.assertIn("not derived as an intersection", inventory)
        self.assertIn("absence was not the rejection reason", inventory)


if __name__ == "__main__":
    unittest.main()
