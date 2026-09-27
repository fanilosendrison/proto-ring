from __future__ import annotations

from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

import yaml

from proto_ring.adr_metadata import AdrMetadataError
from proto_ring.canonical_adr import configured_profile_path, resolve


PROFILE_PATH = "config/adr-profile.yaml"
ADR_DIRECTORY = "records"
FILENAME_PATTERN = r"^adr-(?P<number>[0-9]{3})-[a-z]+(?:-[a-z]+)*\.md$"
ID_PATTERN = r"^ADR-[0-9]{3}$"


class CanonicalAdrTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repository = Path(self.temporary.name)
        self.write_agents()
        self.write_profile()

    def write_agents(self, architecture: object = PROFILE_PATH) -> None:
        governance: dict[str, object] = {}
        if architecture is not None:
            governance["architecture_decisions"] = (
                {"profile_path": architecture}
                if isinstance(architecture, str)
                else architecture
            )
        payload = {"repository_governance": governance}
        text = yaml.safe_dump(payload, sort_keys=False)
        (self.repository / "AGENTS.md").write_text(
            f"---\n{text}---\n# Directives\n", encoding="utf-8"
        )

    def write_profile(self, **repository_overrides: object) -> None:
        repository: dict[str, object] = {
            "adr_directory": ADR_DIRECTORY,
            "filename_pattern": FILENAME_PATTERN,
            "id_pattern": ID_PATTERN,
            "id_width": 3,
        }
        repository.update(repository_overrides)
        path = self.repository / PROFILE_PATH
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            yaml.safe_dump({"repository": repository}, sort_keys=False),
            encoding="utf-8",
        )

    def write_adr(
        self,
        name: str = "adr-001-authority.md",
        *,
        adr_id: str = "ADR-001",
        body: bytes = b"## Context\n\nExact decision body.\n",
        directory: str = ADR_DIRECTORY,
        structured: bool = True,
    ) -> Path:
        path = self.repository / directory / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if structured:
            path.write_bytes(
                b"---\n"
                + f'id: "{adr_id}"\n'.encode()
                + b"---\n\n# Decision\n\n"
                + body
            )
        else:
            path.write_bytes(b"# Decision\n\n" + body)
        return path

    def assert_resolution_error(self, adr_id: str = "ADR-001") -> None:
        with self.assertRaises(AdrMetadataError):
            resolve(self.repository, adr_id)

    def test_valid_configured_profile(self) -> None:
        self.assertEqual(
            configured_profile_path(self.repository),
            (self.repository / PROFILE_PATH).resolve(),
        )

    def test_missing_agents_fails(self) -> None:
        (self.repository / "AGENTS.md").unlink()
        with self.assertRaises(AdrMetadataError):
            configured_profile_path(self.repository)

    def test_missing_architecture_decisions_fails(self) -> None:
        self.write_agents(None)
        with self.assertRaises(AdrMetadataError):
            configured_profile_path(self.repository)

    def test_missing_profile_path_fails(self) -> None:
        self.write_agents({})
        with self.assertRaises(AdrMetadataError):
            configured_profile_path(self.repository)

    def test_absolute_profile_path_fails(self) -> None:
        self.write_agents("/profile.yaml")
        with self.assertRaises(AdrMetadataError):
            configured_profile_path(self.repository)

    def test_escaping_profile_path_fails(self) -> None:
        self.write_agents("../profile.yaml")
        with self.assertRaises(AdrMetadataError):
            configured_profile_path(self.repository)

    def test_missing_profile_fails(self) -> None:
        (self.repository / PROFILE_PATH).unlink()
        self.assert_resolution_error()

    def test_malformed_profile_yaml_fails(self) -> None:
        (self.repository / PROFILE_PATH).write_text("repository: [\n", encoding="utf-8")
        self.assert_resolution_error()

    def test_missing_repository_mapping_fails(self) -> None:
        (self.repository / PROFILE_PATH).write_text("profile_version: 1\n", encoding="utf-8")
        self.assert_resolution_error()

    def test_missing_adr_directory_field_fails(self) -> None:
        self.write_profile(adr_directory=None)
        self.assert_resolution_error()

    def test_missing_filename_pattern_fails(self) -> None:
        self.write_profile(filename_pattern=None)
        self.assert_resolution_error()

    def test_missing_id_pattern_fails(self) -> None:
        self.write_profile(id_pattern=None)
        self.assert_resolution_error()

    def test_missing_id_width_fails(self) -> None:
        self.write_profile(id_width=None)
        self.assert_resolution_error()

    def test_invalid_filename_regex_fails(self) -> None:
        self.write_profile(filename_pattern="[")
        self.assert_resolution_error()

    def test_invalid_id_regex_fails(self) -> None:
        self.write_profile(id_pattern="[")
        self.assert_resolution_error()

    def test_requested_id_rejected_by_profile_fails(self) -> None:
        self.write_profile(id_pattern=r"^ADR-002$")
        self.assert_resolution_error("ADR-001")

    def test_requested_id_without_exact_adr_shape_fails(self) -> None:
        self.write_profile(id_pattern=r"^DEC-[0-9]{3}$")
        self.assert_resolution_error("DEC-001")

    def test_requested_id_with_wrong_width_fails(self) -> None:
        self.write_profile(id_pattern=r"^ADR-[0-9]+$", id_width=3)
        self.assert_resolution_error("ADR-01")

    def test_missing_adr_directory_fails(self) -> None:
        self.assert_resolution_error()

    def test_zero_canonical_candidates_fails(self) -> None:
        (self.repository / ADR_DIRECTORY).mkdir()
        self.assert_resolution_error()

    def test_exactly_one_candidate_resolves(self) -> None:
        expected = self.write_adr()
        result = resolve(self.repository, "ADR-001")
        self.assertEqual(result.id, "ADR-001")
        self.assertEqual(result.path, expected.resolve())
        self.assertEqual(result.metadata["id"], "ADR-001")

    def test_two_candidates_for_requested_number_fail_ambiguous(self) -> None:
        self.write_adr("adr-001-first.md")
        self.write_adr("adr-001-second.md")
        self.assert_resolution_error()

    def test_candidate_metadata_id_mismatch_fails(self) -> None:
        self.write_adr(adr_id="ADR-002")
        self.assert_resolution_error()

    def test_unstructured_candidate_fails(self) -> None:
        self.write_adr(structured=False)
        self.assert_resolution_error()

    def test_same_id_file_outside_corpus_is_ignored(self) -> None:
        expected = self.write_adr()
        self.write_adr(
            "adr-001-impostor.md",
            directory="outside",
            body=b"## Context\n\nImpostor.\n",
        )
        self.assertEqual(resolve(self.repository, "ADR-001").path, expected.resolve())

    def test_matching_symlink_to_file_outside_adr_directory_fails(self) -> None:
        self.write_profile(adr_directory="docs/adr")
        self.write_adr("authority.md", directory="outside")
        symlink = self.repository / "docs/adr/adr-001-symlink.md"
        symlink.parent.mkdir(parents=True, exist_ok=True)
        symlink.symlink_to("../../outside/authority.md")

        with self.assertRaisesRegex(
            AdrMetadataError,
            "canonical ADR candidate must be a direct non-symlink file",
        ):
            resolve(self.repository, "ADR-001")

    def test_matching_symlink_to_nested_adr_file_fails(self) -> None:
        self.write_profile(adr_directory="docs/adr")
        self.write_adr("authority.md", directory="docs/adr/nested")
        symlink = self.repository / "docs/adr/adr-001-symlink.md"
        symlink.symlink_to("nested/authority.md")

        with self.assertRaisesRegex(
            AdrMetadataError,
            "canonical ADR candidate must be a direct non-symlink file",
        ):
            resolve(self.repository, "ADR-001")

    def test_matching_symlink_to_same_directory_target_fails(self) -> None:
        self.write_profile(adr_directory="docs/adr")
        self.write_adr("authority-target.md", directory="docs/adr")
        symlink = self.repository / "docs/adr/adr-001-symlink.md"
        symlink.symlink_to("authority-target.md")

        with self.assertRaisesRegex(
            AdrMetadataError,
            "canonical ADR candidate must be a direct non-symlink file",
        ):
            resolve(self.repository, "ADR-001")

    def test_unrelated_symlink_does_not_affect_valid_resolution(self) -> None:
        self.write_profile(adr_directory="docs/adr")
        expected = self.write_adr(directory="docs/adr")
        self.write_adr("authority.md", adr_id="ADR-002", directory="outside")
        symlink = self.repository / "docs/adr/adr-002-unrelated.md"
        symlink.symlink_to("../../outside/authority.md")

        self.assertEqual(resolve(self.repository, "ADR-001").path, expected.resolve())

    def test_generated_index_claim_has_no_effect(self) -> None:
        expected = self.write_adr()
        (self.repository / ADR_DIRECTORY / "index.md").write_text(
            "ADR-001: ../outside/adr-001-impostor.md\n", encoding="utf-8"
        )
        self.assertEqual(resolve(self.repository, "ADR-001").path, expected.resolve())

    def test_annotated_history_claim_has_no_effect(self) -> None:
        expected = self.write_adr()
        (self.repository / ADR_DIRECTORY / "README.md").write_text(
            "ADR-001: ../outside/adr-001-impostor.md\n", encoding="utf-8"
        )
        self.assertEqual(resolve(self.repository, "ADR-001").path, expected.resolve())

    def test_exact_decision_body_is_returned(self) -> None:
        body = b"## Context\n\nExact bytes.\n\n"
        self.write_adr(body=body)
        self.assertEqual(resolve(self.repository, "ADR-001").decision_body, body)

    def test_filename_pattern_requires_number_group(self) -> None:
        self.write_profile(filename_pattern=r"^adr-[0-9]{3}-[a-z]+\.md$")
        self.write_adr()
        self.assert_resolution_error()


if __name__ == "__main__":
    unittest.main()
