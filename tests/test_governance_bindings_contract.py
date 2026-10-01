from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
SHARED_PROVIDER = ROOT / "docs/contracts/shared-governance-provider.md"
ARM = ROOT / "docs/contracts/authoritative-ref-monotonicity.md"
IMPLEMENTATION = ROOT / "src/proto_ring/governance_bindings.py"


class GovernanceBindingsContractTests(unittest.TestCase):
    def test_shared_provider_contract_owns_exact_registry_shape(self) -> None:
        contract = SHARED_PROVIDER.read_text(encoding="utf-8")
        required = (
            "This contract owns the canonical structured Governance Binding Registry",
            "governance_bindings:",
            "model_version: 1",
            "source: <governed-source-id>",
            "bindings: {}",
            "kind\nscope\nidentity\nauthority",
            "executable_provider",
            "governance_contract",
            "logical_provider",
            "capability",
            "governed_object",
        )
        for value in required:
            with self.subTest(value=value):
                self.assertIn(value, contract)

    def test_contract_preserves_identity_and_authority_boundaries(self) -> None:
        contract = " ".join(SHARED_PROVIDER.read_text(encoding="utf-8").split())
        required = (
            "Executable-provider identity, governance-contract identity, and "
            "`BindingId` remain distinct",
            "Both values reference the consumer's Canonical Governance Authority profile.",
            "A governed-object scope resolves through the consumer's Canonical "
            "Governed Objects catalog.",
            "exactly one active `executable_provider` binding",
            "different commits for the same contract repository and path under "
            "the same effective scope",
        )
        for value in required:
            with self.subTest(value=value):
                self.assertIn(value, contract)

    def test_contract_excludes_installation_projection_and_mutation_semantics(self) -> None:
        contract = " ".join(SHARED_PROVIDER.read_text(encoding="utf-8").split())
        required = (
            "Registry loading is observational.",
            "performs no mutation, provisioning, repair, upgrade, imported-module "
            "discovery, or effective-provider discovery",
            "does not define `requirements.txt`, pip, Python packaging, containers, "
            "Nix, or another installation mechanism",
            "effective-realization relationship is a Projection Integrity and "
            "currentness responsibility, not binding authority",
        )
        for value in required:
            with self.subTest(value=value):
                self.assertIn(value, contract)

    def test_arm_local_binding_no_longer_duplicates_contract_pin(self) -> None:
        contract = " ".join(ARM.read_text(encoding="utf-8").split())
        self.assertIn(
            "The immutable proto-ring contract identity is owned by the consumer's "
            "canonical Governance Binding Registry.",
            contract,
        )
        self.assertIn(
            "The local Authoritative Ref Monotonicity binding MUST NOT duplicate "
            "that contract pin.",
            contract,
        )
        local_binding = contract.split("## Local binding", 1)[1].split("## Non-goals", 1)[0]
        self.assertNotIn("immutable proto-ring contract identity being adopted", local_binding)

    def test_implementation_contains_no_consumer_or_installation_vocabulary(self) -> None:
        implementation = IMPLEMENTATION.read_text(encoding="utf-8")
        forbidden = (
            "Turnlock",
            "Ruu",
            "requirements.txt",
            "pip",
            "site-packages",
            "sys.executable",
            "installed_provider",
            "effective_provider",
        )
        for value in forbidden:
            with self.subTest(value=value):
                self.assertNotIn(value, implementation)


if __name__ == "__main__":
    unittest.main()
