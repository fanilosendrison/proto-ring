from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "governed-objects.md"
IMPLEMENTATION = ROOT / "src" / "proto_ring" / "governed_objects.py"


class GovernedObjectsContractTests(unittest.TestCase):
    def test_contract_and_implementation_exist(self) -> None:
        self.assertTrue(CONTRACT.is_file())
        self.assertTrue(IMPLEMENTATION.is_file())

    def test_contract_preserves_core_identity_and_shape_boundary(self) -> None:
        contract = " ".join(CONTRACT.read_text(encoding="utf-8").split())
        required = (
            "GovernedObjectRef = (interface_id, object_id)",
            "model_version: 1",
            "consumer-owned canonical identity",
            "does not canonicalize it",
            "Unknown structural keys",
            "interfaces: {}",
            "interface.objects: {}",
            "object.relations: []",
            "object.responsibilities: []",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, contract)

    def test_contract_preserves_authority_and_relation_boundary(self) -> None:
        contract = " ".join(CONTRACT.read_text(encoding="utf-8").split())
        required = (
            "roles: {}",
            "same catalog",
            "cross-catalog and cross-repository targets",
            "Forward references, self-relations, and cycles",
            "Exact duplicate responsibility identities are invalid",
            "An exact duplicate relation is invalid",
            "Relation identities are consumer-owned",
            "no universal `SourceRole`",
        )
        for text in required:
            with self.subTest(text=text):
                self.assertIn(text, contract)

    def test_contract_keeps_semantic_and_registry_work_out_of_scope(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        for excluded in (
            "universal object taxonomy or object kind",
            "semantic object bodies or locators",
            "validation registry",
            "evidence registry",
            "projection or catalog-currentness registry",
            "exact RepositoryGovernanceState",
        ):
            with self.subTest(excluded=excluded):
                self.assertIn(excluded, contract)

    def test_implementation_contains_no_consumer_vocabulary(self) -> None:
        implementation = IMPLEMENTATION.read_text(encoding="utf-8")
        for forbidden in ("Turnlock", "Ruu", "TL-INV", "TL-CLAIM", "ADR-"):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, implementation)


if __name__ == "__main__":
    unittest.main()
