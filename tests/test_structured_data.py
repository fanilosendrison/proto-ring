from __future__ import annotations

import unittest

from proto_ring import structured_data
from proto_ring.structured_data import (
    StructuredDataError,
    parse_frontmatter_bytes,
)


def _carrier(payload: str, body: bytes = b"") -> bytes:
    return b"---\n" + payload.encode("utf-8") + b"\n---\n" + body


class StructuredDataTests(unittest.TestCase):
    def assert_parse_error(self, data: bytes) -> None:
        with self.assertRaises(StructuredDataError):
            parse_frontmatter_bytes(data)

    def test_public_api_is_exact(self) -> None:
        self.assertEqual(
            structured_data.__all__,
            [
                "ParsedFrontmatter",
                "StructuredDataError",
                "StructuredValue",
                "parse_frontmatter_bytes",
            ],
        )

    def test_complete_scalar_and_collection_model_is_constructed(self) -> None:
        parsed = parse_frontmatter_bytes(
            _carrier(
                """plain: text
quoted: "quoted text"
empty: ""
truth: true
falsehood: false
nothing: null
zero: 0
negative: -42
sequence:
  - item
  - false
mapping:
  nested: 3
flow_sequence: [one, true, null, 2]
flow_mapping: {left: value, right: false}
# admitted comment"""
            )
        )

        self.assertEqual(
            parsed.metadata,
            {
                "plain": "text",
                "quoted": "quoted text",
                "empty": "",
                "truth": True,
                "falsehood": False,
                "nothing": None,
                "zero": 0,
                "negative": -42,
                "sequence": ["item", False],
                "mapping": {"nested": 3},
                "flow_sequence": ["one", True, None, 2],
                "flow_mapping": {"left": "value", "right": False},
            },
        )

    def test_quoted_null_spellings_remain_strings(self) -> None:
        parsed = parse_frontmatter_bytes(
            _carrier("double: \"null\"\nsingle: 'null'")
        )

        self.assertEqual(parsed.metadata, {"double": "null", "single": "null"})

    def test_noncanonical_implicit_yaml_forms_remain_strings(self) -> None:
        values = [
            "True",
            "FALSE",
            "yes",
            "no",
            "on",
            "off",
            "Null",
            "NULL",
            "~",
            "01",
            "-0",
            "+1",
            "1_000",
            "0x10",
            "0o10",
            "0b10",
            "1.0",
            "1e3",
            "2026-09-29",
        ]
        payload = "\n".join(
            f"value_{index}: {value}" for index, value in enumerate(values)
        )

        parsed = parse_frontmatter_bytes(_carrier(payload))

        self.assertEqual(
            parsed.metadata,
            {f"value_{index}": value for index, value in enumerate(values)},
        )

    def test_body_bytes_are_preserved_without_interpretation(self) -> None:
        body = b"# prose\nrepository_governance:\n  malformed: [\n---\n"

        parsed = parse_frontmatter_bytes(_carrier("metadata: value", body))

        self.assertEqual(parsed.metadata, {"metadata": "value"})
        self.assertEqual(parsed.body, body)

    def test_invalid_utf8_anywhere_in_carrier_fails(self) -> None:
        self.assert_parse_error(b"---\nvalue: text\n---\nbody: \xff")

    def test_bom_at_start_fails(self) -> None:
        self.assert_parse_error(b"\xef\xbb\xbf" + _carrier("value: text"))

    def test_bom_later_in_carrier_fails(self) -> None:
        self.assert_parse_error(_carrier("value: text", b"body\xef\xbb\xbf"))

    def test_crlf_fails(self) -> None:
        self.assert_parse_error(b"---\r\nvalue: text\r\n---\r\n")

    def test_bare_cr_fails(self) -> None:
        self.assert_parse_error(_carrier("value: text", b"body\rtext"))

    def test_missing_opening_delimiter_fails(self) -> None:
        self.assert_parse_error(b"value: text\n---\n")

    def test_missing_closing_delimiter_fails(self) -> None:
        self.assert_parse_error(b"---\nvalue: text\n")

    def test_malformed_yaml_fails(self) -> None:
        self.assert_parse_error(_carrier("value: [one"))

    def test_non_mapping_top_levels_fail(self) -> None:
        for payload in ("- item", "", "scalar"):
            with self.subTest(payload=payload):
                self.assert_parse_error(_carrier(payload))

    def test_duplicate_root_key_fails(self) -> None:
        self.assert_parse_error(_carrier("foo: one\nfoo: two"))

    def test_duplicate_nested_key_fails(self) -> None:
        self.assert_parse_error(_carrier("outer:\n  foo: one\n  foo: two"))

    def test_plain_and_quoted_duplicate_key_fails(self) -> None:
        self.assert_parse_error(_carrier('foo: one\n"foo": two'))

    def test_anchor_fails(self) -> None:
        self.assert_parse_error(_carrier("value: &anchor text"))

    def test_alias_fails_independently(self) -> None:
        self.assert_parse_error(_carrier("value: *missing"))

    def test_unquoted_merge_key_fails(self) -> None:
        self.assert_parse_error(_carrier("<<: value"))

    def test_quoted_merge_key_fails(self) -> None:
        self.assert_parse_error(_carrier('"<<": value'))

    def test_explicit_standard_tag_fails(self) -> None:
        self.assert_parse_error(_carrier("value: !!str example"))

    def test_explicit_custom_tag_fails(self) -> None:
        self.assert_parse_error(_carrier("value: !custom example"))

    def test_yaml_directive_fails(self) -> None:
        self.assert_parse_error(_carrier("%YAML 1.2\n---\nvalue: text"))

    def test_document_start_marker_fails(self) -> None:
        self.assert_parse_error(
            _carrier("value: text\n--- # another document\nother: value")
        )

    def test_document_end_marker_fails(self) -> None:
        self.assert_parse_error(_carrier("value: text\n..."))

    def test_non_string_mapping_keys_fail(self) -> None:
        payloads = {
            "boolean": "true: value",
            "integer": "1: value",
            "null": "null: value",
            "sequence": "? [one, two]\n: value",
        }
        for label, payload in payloads.items():
            with self.subTest(label=label):
                self.assert_parse_error(_carrier(payload))

    def test_omitted_root_value_fails(self) -> None:
        self.assert_parse_error(_carrier("key:"))

    def test_omitted_nested_value_fails(self) -> None:
        self.assert_parse_error(_carrier("outer:\n  key:"))

    def test_literal_block_scalar_fails(self) -> None:
        self.assert_parse_error(_carrier("value: |\n  text"))

    def test_folded_block_scalar_fails(self) -> None:
        self.assert_parse_error(_carrier("value: >\n  text"))


if __name__ == "__main__":
    unittest.main()
