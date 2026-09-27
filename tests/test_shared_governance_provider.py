from __future__ import annotations

from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from proto_ring.shared_governance_provider import (
    SharedGovernanceProviderProfile,
    check,
)


CONTRACT_COMMIT = "974ca31ff12630a90da6371cc27c1f5ef0cc590e"
BINDING_PATH = "docs/repository-governance/consumer-binding.md"
AGENT_DIRECTIVE = (
    "Before changing repository governance, read and apply\n"
    f"  `{BINDING_PATH}`."
)
BINDING_DIRECTIVE = "Consumer uses proto-ring as the mandatory provider."
IDENTITY = (
    "fanilosendrison/proto-ring\n"
    f"{CONTRACT_COMMIT}\n"
    "docs/contracts/shared-governance-provider.md"
)


class SharedGovernanceProviderTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repository = Path(self.temporary.name)
        self.profile = SharedGovernanceProviderProfile(
            agent_directives_path="AGENTS.md",
            binding_path=BINDING_PATH,
            contract_commit=CONTRACT_COMMIT,
            required_agent_directive=AGENT_DIRECTIVE,
            required_binding_directive=BINDING_DIRECTIVE,
        )
        self.write_valid_files()

    @property
    def binding_path(self) -> Path:
        return self.repository / BINDING_PATH

    def write_valid_files(self) -> None:
        (self.repository / "AGENTS.md").write_text(
            f"# Directives\n\n{AGENT_DIRECTIVE}\n",
            encoding="utf-8",
        )
        self.binding_path.parent.mkdir(parents=True)
        self.binding_path.write_text(
            f"# Binding\n\n{IDENTITY}\n\n{BINDING_DIRECTIVE}\n",
            encoding="utf-8",
        )

    def test_valid_profile_and_files_pass(self) -> None:
        self.assertEqual([], check(self.repository, self.profile))

    def test_missing_binding_file_returns_errors(self) -> None:
        self.binding_path.unlink()
        self.assertTrue(check(self.repository, self.profile))

    def test_missing_agent_directives_file_returns_errors(self) -> None:
        (self.repository / "AGENTS.md").unlink()
        self.assertTrue(check(self.repository, self.profile))

    def test_absolute_binding_path_returns_error(self) -> None:
        profile = replace(self.profile, binding_path="/binding.md")
        self.assertTrue(check(self.repository, profile))

    def test_escaping_binding_path_returns_error(self) -> None:
        profile = replace(self.profile, binding_path="../binding.md")
        self.assertTrue(check(self.repository, profile))

    def test_contract_commit_must_be_lowercase_full_sha(self) -> None:
        profile = replace(self.profile, contract_commit="A" * 40)
        self.assertTrue(check(self.repository, profile))

    def test_mutable_binding_identity_returns_error(self) -> None:
        self.binding_path.write_text(
            self.binding_path.read_text(encoding="utf-8").replace(
                CONTRACT_COMMIT, "main"
            ),
            encoding="utf-8",
        )
        self.assertTrue(check(self.repository, self.profile))

    def test_stale_full_binding_identity_returns_error(self) -> None:
        self.binding_path.write_text(
            self.binding_path.read_text(encoding="utf-8").replace(
                CONTRACT_COMMIT, "1" * 40
            ),
            encoding="utf-8",
        )
        self.assertTrue(check(self.repository, self.profile))

    def test_zero_canonical_identity_blocks_returns_error(self) -> None:
        self.binding_path.write_text(
            f"# Binding\n\n{BINDING_DIRECTIVE}\n",
            encoding="utf-8",
        )
        errors = check(self.repository, self.profile)
        self.assertIn(
            "binding does not contain the canonical Shared Governance Provider identity",
            errors,
        )

    def test_two_canonical_identity_blocks_returns_error(self) -> None:
        self.binding_path.write_text(
            f"{IDENTITY}\n\n{IDENTITY}\n\n{BINDING_DIRECTIVE}\n",
            encoding="utf-8",
        )
        errors = check(self.repository, self.profile)
        self.assertIn(
            "binding contains multiple Shared Governance Provider identities",
            errors,
        )

    def test_missing_required_binding_directive_returns_error(self) -> None:
        self.binding_path.write_text(f"{IDENTITY}\n", encoding="utf-8")
        errors = check(self.repository, self.profile)
        self.assertIn(
            "binding is missing the required mandatory-provider directive",
            errors,
        )

    def test_navigation_reference_without_exact_agent_directive_returns_error(self) -> None:
        (self.repository / "AGENTS.md").write_text(
            f"Quick navigation: `{BINDING_PATH}`\n",
            encoding="utf-8",
        )
        errors = check(self.repository, self.profile)
        self.assertIn(
            "agent directives do not route repository-governance work through the binding",
            errors,
        )

    def test_exact_required_agent_directive_passes(self) -> None:
        self.assertNotIn(
            "agent directives do not route repository-governance work through the binding",
            check(self.repository, self.profile),
        )

    def test_binding_with_utf8_bom_returns_error(self) -> None:
        self.binding_path.write_bytes(b"\xef\xbb\xbf" + self.binding_path.read_bytes())
        self.assertTrue(check(self.repository, self.profile))

    def test_binding_with_crlf_returns_error(self) -> None:
        self.binding_path.write_bytes(self.binding_path.read_bytes().replace(b"\n", b"\r\n"))
        self.assertTrue(check(self.repository, self.profile))

    def test_invalid_utf8_in_agent_directives_returns_error(self) -> None:
        (self.repository / "AGENTS.md").write_bytes(b"\xff")
        self.assertTrue(check(self.repository, self.profile))


if __name__ == "__main__":
    unittest.main()
