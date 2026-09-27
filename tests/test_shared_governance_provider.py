from __future__ import annotations

from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

from proto_ring import canonical_adr
from proto_ring.adr_metadata import sha256_hex
from proto_ring.shared_governance_provider import check


CONTRACT_COMMIT = "974ca31ff12630a90da6371cc27c1f5ef0cc590e"
OTHER_COMMIT = "1" * 40
BINDING_PATH = "docs/repository-governance/consumer-binding.md"
PROFILE_PATH = "docs/adr/adr-profile.yaml"
ADR_PATH = "docs/adr/adr-001-provider.md"
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
        self.binding_path = self.repository / BINDING_PATH
        self.adr_path = self.repository / ADR_PATH
        self.write_agents()
        self.write_profile()
        self.write_binding()
        self.write_adr()

    def write_agents(
        self,
        *,
        required: str = "true",
        binding_path: str | None = BINDING_PATH,
        body: str = "# Consumer directives\n",
    ) -> None:
        binding_line = (
            f'    binding_path: "{binding_path}"\n'
            if binding_path is not None
            else ""
        )
        (self.repository / "AGENTS.md").write_text(
            "---\n"
            "repository_governance:\n"
            "  architecture_decisions:\n"
            f'    profile_path: "{PROFILE_PATH}"\n'
            "  shared_governance_provider:\n"
            f"    required: {required}\n"
            f"{binding_line}"
            "---\n"
            f"{body}",
            encoding="utf-8",
        )

    def write_profile(self) -> None:
        profile_path = self.repository / PROFILE_PATH
        profile_path.parent.mkdir(parents=True, exist_ok=True)
        profile_path.write_text(
            "repository:\n"
            '  adr_directory: "docs/adr"\n'
            "  filename_pattern: '^adr-(?P<number>[0-9]{3})-[a-z0-9]+(?:-[a-z0-9]+)*\\.md$'\n"
            "  id_pattern: '^ADR-[0-9]{3}$'\n"
            "  id_width: 3\n",
            encoding="utf-8",
        )

    def write_binding(
        self,
        *,
        mandatory: str = "true",
        authority_id: str | None = "ADR-001",
        authority_path: str | None = None,
        repository: str = "fanilosendrison/proto-ring",
        commit: str = CONTRACT_COMMIT,
        contract_path: str = "docs/contracts/shared-governance-provider.md",
        body: str | None = None,
    ) -> None:
        self.binding_path.parent.mkdir(parents=True, exist_ok=True)
        id_line = f'    id: "{authority_id}"\n' if authority_id is not None else ""
        path_line = (
            f'    path: "{authority_path}"\n'
            if authority_path is not None
            else ""
        )
        binding_body = body if body is not None else f"# Binding\n\n{IDENTITY}\n"
        self.binding_path.write_text(
            "---\n"
            "shared_governance_provider:\n"
            f"  mandatory: {mandatory}\n"
            "  authority_adr:\n"
            f"{id_line}"
            f"{path_line}"
            "  contract:\n"
            f'    repository: "{repository}"\n'
            f'    commit: "{commit}"\n'
            f'    path: "{contract_path}"\n'
            "---\n"
            f"{binding_body}",
            encoding="utf-8",
        )

    def write_adr(
        self,
        *,
        path: Path | None = None,
        adr_id: str = "ADR-001",
        status: str = "accepted",
        context: str | None = None,
    ) -> Path:
        adr_path = path if path is not None else self.adr_path
        adr_path.parent.mkdir(parents=True, exist_ok=True)
        decision_body = (
            context if context is not None else f"## Context\n\n{IDENTITY}\n"
        ).encode("utf-8")
        adr_path.write_bytes(
            (
                "---\n"
                f'id: "{adr_id}"\n'
                f'status: "{status}"\n'
                f'decision_body_sha256: "{sha256_hex(decision_body)}"\n'
                "---\n\n"
                f"# {adr_id}: Provider authority\n\n"
            ).encode("utf-8")
            + decision_body
        )
        return adr_path

    def assert_errors(self) -> None:
        self.assertTrue(check(self.repository))

    def test_valid_structured_chain_passes(self) -> None:
        self.assertEqual([], check(self.repository))

    def test_binding_authority_id_resolves_through_canonical_adr(self) -> None:
        with mock.patch.object(
            canonical_adr, "resolve", wraps=canonical_adr.resolve
        ) as resolver:
            self.assertEqual([], check(self.repository))
        resolver.assert_called_once_with(self.repository, "ADR-001")

    def test_same_id_adr_outside_configured_corpus_is_ignored(self) -> None:
        self.write_adr(
            path=self.repository / "outside" / "adr-001-impostor.md",
            context="## Context\n\nNo provider identity.\n",
        )
        self.assertEqual([], check(self.repository))

    def test_two_canonical_candidates_return_error(self) -> None:
        self.write_adr(path=self.repository / "docs/adr/adr-001-second.md")
        self.assert_errors()

    def test_declared_authority_path_returns_exact_error(self) -> None:
        self.write_binding(authority_path=ADR_PATH)
        self.assertIn(
            "authority_adr.path must not be declared; canonical ADR paths are derived",
            check(self.repository),
        )

    def test_missing_agents_returns_error(self) -> None:
        (self.repository / "AGENTS.md").unlink()
        self.assert_errors()

    def test_agents_without_frontmatter_returns_error(self) -> None:
        (self.repository / "AGENTS.md").write_text("# Directives\n", encoding="utf-8")
        self.assert_errors()

    def test_agents_malformed_yaml_returns_error(self) -> None:
        (self.repository / "AGENTS.md").write_text(
            "---\nrepository_governance: [\n---\n# Directives\n", encoding="utf-8"
        )
        self.assert_errors()

    def test_agents_without_repository_governance_returns_error(self) -> None:
        (self.repository / "AGENTS.md").write_text(
            "---\nkind: KnowledgeAsset\n---\n# Directives\n", encoding="utf-8"
        )
        self.assert_errors()

    def test_agents_without_shared_provider_returns_error(self) -> None:
        (self.repository / "AGENTS.md").write_text(
            "---\nrepository_governance: {}\n---\n# Directives\n", encoding="utf-8"
        )
        self.assert_errors()

    def test_required_false_returns_error(self) -> None:
        self.write_agents(required="false")
        self.assert_errors()

    def test_missing_binding_path_returns_error(self) -> None:
        self.write_agents(binding_path=None)
        self.assert_errors()

    def test_absolute_binding_path_returns_error(self) -> None:
        self.write_agents(binding_path="/binding.md")
        self.assert_errors()

    def test_escaping_binding_path_returns_error(self) -> None:
        self.write_agents(binding_path="../binding.md")
        self.assert_errors()

    def test_missing_binding_returns_error(self) -> None:
        self.binding_path.unlink()
        self.assert_errors()

    def test_binding_without_frontmatter_returns_error(self) -> None:
        self.binding_path.write_text("# Binding\n", encoding="utf-8")
        self.assert_errors()

    def test_binding_malformed_yaml_returns_error(self) -> None:
        self.binding_path.write_text(
            "---\nshared_governance_provider: [\n---\n# Binding\n", encoding="utf-8"
        )
        self.assert_errors()

    def test_mandatory_false_returns_error(self) -> None:
        self.write_binding(mandatory="false")
        self.assert_errors()

    def test_missing_authority_id_returns_error(self) -> None:
        self.write_binding(authority_id=None)
        self.assert_errors()

    def test_missing_authority_adr_returns_error(self) -> None:
        self.adr_path.unlink()
        self.assert_errors()

    def test_authority_id_mismatch_returns_error(self) -> None:
        self.write_adr(adr_id="ADR-002")
        self.assert_errors()

    def test_nonaccepted_authority_returns_error(self) -> None:
        self.write_adr(status="proposed")
        self.assert_errors()

    def test_malformed_decision_body_hash_returns_error(self) -> None:
        text = self.adr_path.read_text(encoding="utf-8")
        self.adr_path.write_text(
            text.replace(sha256_hex(f"## Context\n\n{IDENTITY}\n".encode()), "bad"),
            encoding="utf-8",
        )
        self.assert_errors()

    def test_decision_body_hash_mismatch_returns_error(self) -> None:
        self.adr_path.write_bytes(self.adr_path.read_bytes() + b"changed\n")
        self.assert_errors()

    def test_authority_with_zero_identity_blocks_returns_error(self) -> None:
        self.write_adr(context="## Context\n\nNo provider identity.\n")
        self.assert_errors()

    def test_authority_with_two_identity_blocks_returns_error(self) -> None:
        self.write_adr(context=f"## Context\n\n{IDENTITY}\n\n{IDENTITY}\n")
        self.assert_errors()

    def test_authority_with_mutable_identity_returns_error(self) -> None:
        self.write_adr(context=f"## Context\n\n{IDENTITY.replace(CONTRACT_COMMIT, 'main')}\n")
        self.assert_errors()

    def test_binding_contract_repository_mismatch_returns_error(self) -> None:
        self.write_binding(repository="other/provider")
        self.assert_errors()

    def test_binding_contract_path_mismatch_returns_error(self) -> None:
        self.write_binding(contract_path="docs/contracts/other.md")
        self.assert_errors()

    def test_binding_contract_commit_mismatch_returns_error(self) -> None:
        self.write_binding(commit=OTHER_COMMIT)
        self.assert_errors()

    def test_binding_body_identity_mismatch_returns_error(self) -> None:
        self.write_binding(
            body=f"# Binding\n\n{IDENTITY.replace(CONTRACT_COMMIT, OTHER_COMMIT)}\n"
        )
        self.assert_errors()

    def test_binding_body_with_zero_identity_blocks_returns_error(self) -> None:
        self.write_binding(body="# Binding\n\nNo identity.\n")
        self.assert_errors()

    def test_binding_body_with_two_identity_blocks_returns_error(self) -> None:
        self.write_binding(body=f"# Binding\n\n{IDENTITY}\n\n{IDENTITY}\n")
        self.assert_errors()

    def test_agents_with_utf8_bom_returns_error(self) -> None:
        path = self.repository / "AGENTS.md"
        path.write_bytes(b"\xef\xbb\xbf" + path.read_bytes())
        self.assert_errors()

    def test_agents_with_crlf_returns_error(self) -> None:
        path = self.repository / "AGENTS.md"
        path.write_bytes(path.read_bytes().replace(b"\n", b"\r\n"))
        self.assert_errors()

    def test_agents_with_invalid_utf8_returns_error(self) -> None:
        (self.repository / "AGENTS.md").write_bytes(b"\xff")
        self.assert_errors()

    def test_binding_with_utf8_bom_returns_error(self) -> None:
        self.binding_path.write_bytes(b"\xef\xbb\xbf" + self.binding_path.read_bytes())
        self.assert_errors()

    def test_binding_with_crlf_returns_error(self) -> None:
        self.binding_path.write_bytes(
            self.binding_path.read_bytes().replace(b"\n", b"\r\n")
        )
        self.assert_errors()

    def test_old_prose_directive_is_not_required(self) -> None:
        self.write_agents(body="# Directives\n\nCompletely different human guidance.\n")
        self.assertEqual([], check(self.repository))


if __name__ == "__main__":
    unittest.main()
