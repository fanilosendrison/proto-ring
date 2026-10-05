from __future__ import annotations

from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from proto_ring import governance_bootstrap
from proto_ring.governance_bootstrap import GovernanceBootstrapError, load
from proto_ring.structured_data import StructuredDataError


def _agents(payload: str, body: str = "# directives\n") -> bytes:
    return b"---\n" + payload.encode("utf-8") + b"\n---\n" + body.encode("utf-8")


TURNLOCK_PAYLOAD = '''okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "agent-directives"
domain: "turnlock-rust"
severity: "strict"
name: "Turnlock-Rust repository agent directives"
repository_governance:
  architecture_decisions:
    profile_path: "docs/adr/adr-profile.yaml"
  shared_governance_provider:
    required: true
    binding_path: "docs/repository-governance/turnlock-rust-shared-governance-provider.md"'''

RUU_PAYLOAD = '''okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "agent-directives"
domain: "ruu"
severity: "strict"
name: "Ruu repository agent directives"
repository_governance:
  architecture_decisions:
    profile_path: "docs/adr/adr-profile.yaml"
  shared_governance_provider:
    required: true
    binding_path: "docs/repository-governance/ruu-shared-governance-provider.md"'''


class GovernanceBootstrapTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repository = Path(self.temporary.name) / "repository"
        self.repository.mkdir()

    def write_agents(self, payload: str, body: str = "# directives\n") -> Path:
        carrier = self.repository / "AGENTS.md"
        carrier.write_bytes(_agents(payload, body))
        return carrier

    def test_public_api_is_exact(self) -> None:
        self.assertEqual(
            governance_bootstrap.__all__,
            ["GovernanceBootstrap", "GovernanceBootstrapError", "load"],
        )

    def test_turnlock_style_bootstrap_is_loaded_exactly(self) -> None:
        carrier = self.write_agents(TURNLOCK_PAYLOAD)

        bootstrap = load(self.repository)

        self.assertEqual(bootstrap.repository, self.repository)
        self.assertEqual(bootstrap.carrier, carrier)
        self.assertEqual(bootstrap.metadata["domain"], "turnlock-rust")
        self.assertEqual(
            bootstrap.repository_governance,
            {
                "architecture_decisions": {
                    "profile_path": "docs/adr/adr-profile.yaml"
                },
                "shared_governance_provider": {
                    "required": True,
                    "binding_path": (
                        "docs/repository-governance/"
                        "turnlock-rust-shared-governance-provider.md"
                    ),
                },
            },
        )

    def test_ruu_style_bootstrap_is_loaded_exactly(self) -> None:
        self.write_agents(RUU_PAYLOAD)

        bootstrap = load(self.repository)

        self.assertEqual(bootstrap.metadata["domain"], "ruu")
        self.assertEqual(
            bootstrap.repository_governance["shared_governance_provider"],
            {
                "required": True,
                "binding_path": (
                    "docs/repository-governance/"
                    "ruu-shared-governance-provider.md"
                ),
            },
        )

    def test_unknown_valid_descendants_are_preserved(self) -> None:
        self.write_agents(
            '''repository_governance:
  future_capability:
    enabled: true
    count: 2
    optional: null
    opaque: "keep-me"'''
        )

        bootstrap = load(self.repository)

        self.assertEqual(
            bootstrap.repository_governance,
            {
                "future_capability": {
                    "enabled": True,
                    "count": 2,
                    "optional": None,
                    "opaque": "keep-me",
                }
            },
        )

    def test_other_top_level_metadata_is_retained(self) -> None:
        self.write_agents(
            'okf_version: "1.0"\nother: [one, two]\nrepository_governance: {}'
        )

        bootstrap = load(self.repository)

        self.assertEqual(
            bootstrap.metadata,
            {
                "okf_version": "1.0",
                "other": ["one", "two"],
                "repository_governance": {},
            },
        )

    def test_missing_carrier_fails(self) -> None:
        with self.assertRaises(GovernanceBootstrapError):
            load(self.repository)

    def test_nested_carrier_is_not_discovered(self) -> None:
        nested = self.repository / "nested"
        nested.mkdir()
        (nested / "AGENTS.md").write_bytes(_agents("repository_governance: {}"))

        with self.assertRaises(GovernanceBootstrapError):
            load(self.repository)

    def test_parent_carrier_is_not_discovered(self) -> None:
        self.write_agents("repository_governance: {}")
        nested_repository = self.repository / "nested"
        nested_repository.mkdir()

        with self.assertRaises(GovernanceBootstrapError):
            load(nested_repository)

    def test_lowercase_filename_is_not_a_fallback(self) -> None:
        (self.repository / "agents.md").write_bytes(
            _agents("repository_governance: {}")
        )

        with self.assertRaises(GovernanceBootstrapError):
            load(self.repository)

    def test_unreadable_carrier_is_a_controlled_failure(self) -> None:
        with patch.object(Path, "read_bytes", side_effect=PermissionError("denied")):
            with self.assertRaises(GovernanceBootstrapError) as raised:
                load(self.repository)

        self.assertIsInstance(raised.exception.__cause__, PermissionError)

    def test_structured_parser_failures_are_wrapped_with_cause(self) -> None:
        cases = {
            "invalid UTF-8": b"---\nrepository_governance: {}\n---\n\xff",
            "malformed frontmatter": _agents("repository_governance: ["),
        }
        for label, data in cases.items():
            with self.subTest(label=label):
                (self.repository / "AGENTS.md").write_bytes(data)

                with self.assertRaises(GovernanceBootstrapError) as raised:
                    load(self.repository)

                self.assertIsInstance(
                    raised.exception.__cause__, StructuredDataError
                )

    def test_missing_governance_is_not_reconstructed_from_prose(self) -> None:
        self.write_agents(
            'okf_version: "1.0"',
            "# prose\nrepository_governance:\n  inferred: true\n",
        )

        with self.assertRaises(GovernanceBootstrapError):
            load(self.repository)

    def test_governance_looking_body_cannot_repair_immediate_close(self) -> None:
        (self.repository / "AGENTS.md").write_bytes(
            b"---\n"
            b"---\n"
            b"repository_governance:\n"
            b"  model_version: 2\n"
            b"...\n"
        )

        with self.assertRaises(GovernanceBootstrapError) as raised:
            load(self.repository)

        self.assertIsInstance(raised.exception.__cause__, StructuredDataError)

    def test_first_exact_close_preserves_only_first_governance_mapping(self) -> None:
        body = (
            "# body\n"
            "---\n"
            "repository_governance:\n"
            "  source: replacement\n"
            "---\n"
        )
        self.write_agents("repository_governance:\n  source: frontmatter", body)

        bootstrap = load(self.repository)

        self.assertEqual(
            bootstrap.repository_governance,
            {"source": "frontmatter"},
        )

    def test_non_mapping_governance_fails(self) -> None:
        for value in ("true", "[item]"):
            with self.subTest(value=value):
                self.write_agents(f"repository_governance: {value}")

                with self.assertRaises(GovernanceBootstrapError):
                    load(self.repository)

    def test_exact_carrier_identity_is_returned(self) -> None:
        carrier = self.write_agents("repository_governance: {}")

        bootstrap = load(self.repository)

        self.assertEqual(bootstrap.repository, self.repository)
        self.assertEqual(bootstrap.carrier, carrier)


if __name__ == "__main__":
    unittest.main()
