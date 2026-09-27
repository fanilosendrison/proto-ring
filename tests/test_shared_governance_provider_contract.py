from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
CONTRACT = Path("docs/contracts/shared-governance-provider.md")
README = Path("README.md")


class SharedGovernanceProviderContractTests(unittest.TestCase):
    def read(self, path: Path) -> str:
        return (REPOSITORY_ROOT / path).read_text(encoding="utf-8")

    def test_contract_preserves_required_generic_guarantees(self) -> None:
        contract = " ".join(self.read(CONTRACT).split())
        requirements = (
            "For generic or reusable repository-governance mechanisms, "
            "proto-ring is the mandatory provider.",
            "The consumer MUST NOT maintain a competing local implementation "
            "of the same generic governance responsibility.",
            "The absence of a required generic capability from proto-ring "
            "does not create a permanent consumer-local ownership exception.",
            "Mutable proto-ring state, including an unpinned branch such as "
            "`main`, MUST NOT become ambient consumer authority.",
            "proto-ring is the mandatory provider of applicable generic "
            "governance mechanisms; it is not the owner of the consumer "
            "authority those mechanisms govern.",
        )
        for requirement in requirements:
            with self.subTest(requirement=requirement):
                self.assertIn(requirement, contract)

    def test_readme_references_the_contract(self) -> None:
        self.assertIn(
            "docs/contracts/shared-governance-provider.md",
            self.read(README),
        )


if __name__ == "__main__":
    unittest.main()
