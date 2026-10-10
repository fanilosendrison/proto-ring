from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "repository-state.md"
README = ROOT / "README.md"


class RepositoryStateContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contract = CONTRACT.read_text(encoding="utf-8")
        cls.normalized = " ".join(cls.contract.split())

    def test_contract_preserves_required_repository_state_semantics(self) -> None:
        requirements = (
            "Canonical RepositoryState contract",
            "exact Git worktree-root establishment",
            "explicit observation-scope semantics",
            "An explicit scope entry MUST NOT cause RepositoryState to observe a filesystem object outside the exact worktree root.",
            "Explicit observation-scope entries are path identities, not Canonical Governance Routing declarations.",
            "symbolic HEAD identity",
            "detached-HEAD",
            "unborn-HEAD",
            "exact Git index stage entries",
            "exact regular-file bytes",
            "exact symbolic-link target",
            "Historical Python digest vectors or platform-specific mode integers do not become universal cross-language identity representations",
            "A coherence basis is a mechanism",
            "not a new serialized RepositoryState field or API primitive",
            "Successful ordinary capture alone MUST NOT be represented as proof",
            "only when a coherence basis has been established",
            "mutation isolation",
            "examples are non-exclusive and non-normative",
            "does not require a specific realization",
            "content-only change after an earlier governed-path observation",
            "Local per-path guards alone are insufficient",
            "does not prove that a coherence basis existed",
            "arbitrary mutation/restore or ABA scheduling",
            "RepositoryState is observational",
        )
        for requirement in requirements:
            with self.subTest(requirement=requirement):
                self.assertIn(requirement, self.normalized)
        self.assertNotIn(
            "global coherence requires an explicit mutation-isolation interval",
            self.normalized,
        )

    def test_readme_indexes_canonical_repository_state(self) -> None:
        readme = README.read_text(encoding="utf-8")
        self.assertIn(
            "[RepositoryState](docs/contracts/repository-state.md)",
            readme,
        )


if __name__ == "__main__":
    unittest.main()
