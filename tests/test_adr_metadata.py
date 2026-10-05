from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from proto_ring.adr_metadata import (
    AdrMetadataError,
    decision_body_bytes,
    h1_text,
    load_json,
    load_yaml,
    parse_adr,
    parse_adr_bytes,
    preserved_payload_bytes,
    relation_target_errors,
    repository_path,
    require_mapping,
    require_string,
    require_string_list,
    schema_errors,
    sha256_hex,
)


class AdrMetadataPrimitiveTests(unittest.TestCase):
    def test_sha256_hex_matches_independent_hashlib_oracle(self) -> None:
        payload = b"proto-ring\x00adr-metadata\n"
        expected = hashlib.sha256(payload).hexdigest()
        self.assertEqual(
            expected,
            "6d32221790c32421c6c5feadf46e527492b83fe55969c3c1cab938c1c842e5e4",
        )
        self.assertEqual(sha256_hex(payload), expected)

    def test_yaml_and_json_loaders_accept_utf8_data(self) -> None:
        with tempfile.TemporaryDirectory(prefix="proto-ring-load-") as temporary:
            root = Path(temporary)
            yaml_path = root / "metadata.yaml"
            json_path = root / "metadata.json"
            yaml_path.write_text("name: Décision\nitems:\n  - one\n", encoding="utf-8")
            json_path.write_text(
                json.dumps({"name": "Décision", "items": ["one"]}),
                encoding="utf-8",
            )
            self.assertEqual(
                load_yaml(yaml_path),
                {"name": "Décision", "items": ["one"]},
            )
            self.assertEqual(
                load_json(json_path),
                {"name": "Décision", "items": ["one"]},
            )

    def test_yaml_loader_rejects_unsafe_constructor(self) -> None:
        with tempfile.TemporaryDirectory(prefix="proto-ring-yaml-") as temporary:
            root = Path(temporary)
            marker = root / "unsafe-marker"
            source = root / "unsafe.yaml"
            source.write_text(
                "!!python/object/apply:pathlib.Path.touch\n"
                f"- {marker.as_posix()}\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(AdrMetadataError, "cannot read YAML"):
                load_yaml(source)
            self.assertFalse(marker.exists())

    def test_loaders_convert_parse_and_io_failures_to_domain_error(self) -> None:
        with tempfile.TemporaryDirectory(prefix="proto-ring-load-errors-") as temporary:
            root = Path(temporary)
            malformed_yaml = root / "malformed.yaml"
            malformed_json = root / "malformed.json"
            malformed_yaml.write_text("value: [\n", encoding="utf-8")
            malformed_json.write_text("{\n", encoding="utf-8")
            with self.assertRaises(AdrMetadataError):
                load_yaml(malformed_yaml)
            with self.assertRaises(AdrMetadataError):
                load_json(malformed_json)
            with self.assertRaises(AdrMetadataError):
                load_json(root / "missing.json")
            with self.assertRaises(AdrMetadataError):
                load_yaml(root / "missing.yaml")

    def test_yaml_loader_uses_canonical_scalar_semantics(self) -> None:
        payload = """legacy_yes: yes
legacy_on: on
legacy_null: Null
leading_zero: 01
floating: 1.0
date_like: 2026-09-29
truth: true
nothing: null
count: 3
"""
        expected = {
            "legacy_yes": "yes", "legacy_on": "on", "legacy_null": "Null",
            "leading_zero": "01", "floating": "1.0",
            "date_like": "2026-09-29", "truth": True, "nothing": None,
            "count": 3,
        }
        with tempfile.TemporaryDirectory(prefix="proto-ring-yaml-scalars-") as temporary:
            path = Path(temporary) / "metadata.yaml"
            path.write_text(payload, encoding="utf-8")
            self.assertEqual(load_yaml(path), expected)

    def test_yaml_loader_rejects_duplicate_and_forbidden_representations(self) -> None:
        cases = {
            "duplicate": b"foo: one\nfoo: two\n",
            "anchor alias": b"first: &x value\nsecond: *x\n",
            "merge": b"base: &base\n  one: two\nvalue:\n  <<: *base\n",
            "block scalar": b"value: |\n  text\n",
            "omitted value": b"value:\n",
        }
        with tempfile.TemporaryDirectory(prefix="proto-ring-yaml-forbidden-") as temporary:
            path = Path(temporary) / "metadata.yaml"
            for label, data in cases.items():
                with self.subTest(label=label):
                    path.write_bytes(data)
                    with self.assertRaises(AdrMetadataError) as raised:
                        load_yaml(path)
                    self.assertIsNotNone(raised.exception.__cause__)

    def test_yaml_loader_document_markers_and_byte_restrictions(self) -> None:
        invalid = {
            "multiple documents": b"---\nvalue: one\n---\nvalue: two\n",
            "BOM": b"\xef\xbb\xbfvalue: text\n",
            "CR": b"value: text\r\n",
            "invalid UTF-8": b"value: \xff\n",
        }
        with tempfile.TemporaryDirectory(prefix="proto-ring-yaml-bytes-") as temporary:
            path = Path(temporary) / "metadata.yaml"
            path.write_bytes(b"---\nvalue: text\n...\n# comment\n")
            self.assertEqual(load_yaml(path), {"value": "text"})
            for label, data in invalid.items():
                with self.subTest(label=label):
                    path.write_bytes(data)
                    with self.assertRaises(AdrMetadataError):
                        load_yaml(path)

    def test_yaml_loader_leaves_mapping_shape_to_caller(self) -> None:
        with tempfile.TemporaryDirectory(prefix="proto-ring-yaml-root-") as temporary:
            path = Path(temporary) / "metadata.yaml"
            path.write_bytes(b"- one\n- two\n")
            loaded = load_yaml(path)
            self.assertEqual(loaded, ["one", "two"])
            with self.assertRaises(AdrMetadataError):
                require_mapping(loaded, "profile")

    def test_yaml_loader_preserves_demonstrated_profile_shape(self) -> None:
        profile = r"""profile_version: 0.1.0
repository:
  adr_directory: docs/adr
  filename_pattern: '^adr-(?P<number>[0-9]{3})-[a-z0-9-]+\.md$'
  id_pattern: '^ADR-[0-9]{3}$'
  id_width: 3
  require_contiguous_ids: true
legacy:
  null_dates: []
generated_index:
  required: true
"""
        with tempfile.TemporaryDirectory(prefix="proto-ring-profile-shape-") as temporary:
            path = Path(temporary) / "profile.yaml"
            path.write_text(profile, encoding="utf-8")
            loaded = require_mapping(load_yaml(path), "profile")
        repository = require_mapping(loaded["repository"], "repository")
        self.assertEqual(loaded["profile_version"], "0.1.0")
        self.assertEqual(repository["adr_directory"], "docs/adr")
        self.assertEqual(repository["id_width"], 3)
        self.assertIs(repository["require_contiguous_ids"], True)
        self.assertEqual(require_mapping(loaded["legacy"], "legacy")["null_dates"], [])
        self.assertIs(require_mapping(loaded["generated_index"], "index")["required"], True)
        self.assertIsInstance(repository["filename_pattern"], str)
        self.assertIsInstance(repository["id_pattern"], str)

    def test_decision_body_is_exact_context_suffix(self) -> None:
        source = (
            b"---\nid: ADR-001\n---\n\n# ADR-001: Example\n\n"
            b"## Context\n\nExact bytes.\n\n## Decision\n\nKeep them.\n"
        )
        expected = b"## Context\n\nExact bytes.\n\n## Decision\n\nKeep them.\n"
        self.assertEqual(decision_body_bytes(source), expected)

    def test_decision_body_rejects_non_exact_text_encodings(self) -> None:
        invalid_sources = (
            b"\xef\xbb\xbf## Context\nbody\n",
            b"# ADR\r\n\r\n## Context\r\nbody\r\n",
            b"# ADR\n\n## Context\n\xff\n",
            b"# ADR\n\n## Decision\nbody\n",
            b"# ADR\n\n## Context\none\n## Context\ntwo\n",
        )
        for source in invalid_sources:
            with self.subTest(source=source):
                with self.assertRaises(AdrMetadataError):
                    decision_body_bytes(source)

    def test_contextual_heading_is_not_a_decision_body_boundary(self) -> None:
        with self.assertRaisesRegex(AdrMetadataError, "exactly one"):
            decision_body_bytes(b"# ADR\n\n## Contextual notes\nbody\n")

    def test_h1_and_preserved_payload_use_exact_authored_bytes(self) -> None:
        source = (
            b"---\nid: ADR-001\n---\n\n# ADR-001: Exact title\n\n"
            b"## Context\nbody\n"
        )
        expected = b"# ADR-001: Exact title\n\n## Context\nbody\n"
        self.assertEqual(h1_text(source), "ADR-001: Exact title")
        self.assertEqual(preserved_payload_bytes(source), expected)
        with self.assertRaisesRegex(AdrMetadataError, "exactly one H1"):
            h1_text(source + b"# Duplicate\n")

    def test_parse_adr_returns_safe_frontmatter_and_exact_body(self) -> None:
        with tempfile.TemporaryDirectory(prefix="proto-ring-adr-") as temporary:
            path = Path(temporary) / "adr.md"
            body = b"## Context\nbody\n"
            path.write_bytes(b"---\nid: ADR-001\nname: Example\n---\n\n# ADR\n\n" + body)
            metadata, parsed_body = parse_adr(path)
            byte_metadata, byte_body = parse_adr_bytes(path.read_bytes())
            self.assertEqual(metadata, {"id": "ADR-001", "name": "Example"})
            self.assertEqual(parsed_body, body)
            self.assertEqual((byte_metadata, byte_body), (metadata, parsed_body))

    def test_parse_adr_rejects_duplicate_constructed_keys(self) -> None:
        source = (
            b"---\nid: ADR-001\nname: Example\n\"name\": Other\n---\n\n"
            b"# ADR\n\n## Context\nbody\n"
        )
        with self.assertRaises(AdrMetadataError):
            parse_adr_bytes(source)

    def test_parse_adr_rejects_forbidden_yaml_representation(self) -> None:
        source = (
            b"---\nid: ADR-001\nname: &identity Example\n---\n\n"
            b"# ADR\n\n## Context\nbody\n"
        )
        with self.assertRaises(AdrMetadataError):
            parse_adr_bytes(source)

    def test_parse_adr_uses_canonical_scalars_and_adr_body_boundary(self) -> None:
        body = b"## Context\nbody\n"
        source = (
            b"---\nid: ADR-001\nlegacy_word: yes\ndate_like: 2026-09-29\n"
            b"explicit_null: null\n---\n\n# ADR\n\n" + body
        )
        metadata, parsed_body = parse_adr_bytes(source)
        self.assertEqual(metadata["legacy_word"], "yes")
        self.assertEqual(metadata["date_like"], "2026-09-29")
        self.assertIsNone(metadata["explicit_null"])
        self.assertEqual(parsed_body, body)

    def test_parse_adr_rejects_missing_or_non_mapping_frontmatter(self) -> None:
        with tempfile.TemporaryDirectory(prefix="proto-ring-adr-errors-") as temporary:
            path = Path(temporary) / "adr.md"
            cases = (
                b"# ADR\n\n## Context\nbody\n",
                b"---\n- value\n---\n\n# ADR\n\n## Context\nbody\n",
                b"---\nid: ADR-001\n\n# ADR\n\n## Context\nbody\n",
            )
            for source in cases:
                with self.subTest(source=source):
                    path.write_bytes(source)
                    with self.assertRaises(AdrMetadataError):
                        parse_adr(path)

    def test_schema_errors_validate_schema_and_calendar_format(self) -> None:
        base = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "required": ["date"],
            "properties": {"date": {"type": "string", "format": "date"}},
        }
        overlay = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "required": ["domain"],
            "properties": {"domain": {"const": "example"}},
        }
        errors = schema_errors(
            {"date": "2025-02-29", "domain": "wrong"},
            base,
            overlay,
        )
        self.assertTrue(any("base schema $.date" in error for error in errors))
        self.assertTrue(any("overlay schema $.domain" in error for error in errors))
        invalid_schema = {"type": "not-a-json-schema-type"}
        self.assertTrue(
            any(
                error.startswith("base schema is invalid:")
                for error in schema_errors({}, invalid_schema, overlay)
            )
        )

    def test_repository_path_contains_relative_paths_and_symlinks(self) -> None:
        with tempfile.TemporaryDirectory(prefix="proto-ring-path-") as temporary:
            root = Path(temporary) / "repository"
            outside = Path(temporary) / "outside"
            root.mkdir()
            outside.mkdir()
            self.assertEqual(
                repository_path(root, "docs/adr"), root.resolve() / "docs/adr"
            )
            with self.assertRaisesRegex(AdrMetadataError, "must be relative"):
                repository_path(root, str(outside))
            with self.assertRaisesRegex(AdrMetadataError, "escapes root"):
                repository_path(root, "../outside")
            (root / "alias").symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(AdrMetadataError, "escapes root"):
                repository_path(root, "alias/record.md")

    def test_structure_requirements_fail_closed(self) -> None:
        self.assertEqual(require_mapping({"key": "value"}, "profile"), {"key": "value"})
        self.assertEqual(require_string("value", "field"), "value")
        self.assertEqual(require_string_list(["one", "two"], "items"), ["one", "two"])
        invalid_calls = (
            lambda: require_mapping([], "profile"),
            lambda: require_string("", "field"),
            lambda: require_string_list(["one", "one"], "items"),
            lambda: require_string_list(["one", 2], "items"),
        )
        for call in invalid_calls:
            with self.subTest(call=call):
                with self.assertRaises(AdrMetadataError):
                    call()

    def test_relation_target_errors_do_not_own_relation_vocabulary(self) -> None:
        known = frozenset({"ADR-001", "ADR-002"})
        self.assertEqual(
            relation_target_errors(
                source_id="ADR-001",
                relation_type="amends",
                targets=("ADR-001", "ADR-003"),
                known_ids=known,
            ),
            [
                "amends cannot reference itself",
                "amends references missing ADR-003",
            ],
        )
        self.assertEqual(
            relation_target_errors(
                source_id="ADR-001",
                relation_type="consumer-defined",
                targets=("ADR-002",),
                known_ids=known,
            ),
            [],
        )


if __name__ == "__main__":
    unittest.main()
