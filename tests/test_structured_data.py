from __future__ import annotations

import subprocess
import sys
import textwrap
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

    def test_ordinary_canonical_integers_remain_exact(self) -> None:
        parsed = parse_frontmatter_bytes(_carrier("zero: 0\npositive: 42\nnegative: -42"))

        self.assertEqual(
            parsed.metadata,
            {"zero": 0, "positive": 42, "negative": -42},
        )

    def test_canonical_integers_beyond_fixed_width_remain_exact(self) -> None:
        values = (2**64, 2**127)
        payload = "\n".join(
            f"value_{index}: {value}" for index, value in enumerate(values)
        )

        parsed = parse_frontmatter_bytes(_carrier(payload))

        self.assertEqual(
            parsed.metadata,
            {f"value_{index}": value for index, value in enumerate(values)},
        )

    def test_canonical_integers_ignore_cpython_digit_limit(self) -> None:
        if not hasattr(sys, "set_int_max_str_digits"):
            self.skipTest("CPython integer-string digit limits are unavailable")

        cases = (
            (640, 640, False),
            (4300, 4300, False),
            (640, 640, True),
        )
        for limit, zero_count, negative in cases:
            with self.subTest(
                limit=limit,
                zero_count=zero_count,
                negative=negative,
            ):
                script = textwrap.dedent(
                    f"""
                    import sys
                    from proto_ring.structured_data import parse_frontmatter_bytes

                    sys.set_int_max_str_digits({limit})
                    before = sys.get_int_max_str_digits()
                    digits = "1" + "0" * {zero_count}
                    sign = "-" if {negative!r} else ""
                    carrier = (
                        b"---\\nvalue: "
                        + (sign + digits).encode("ascii")
                        + b"\\n---\\n"
                    )
                    observed = parse_frontmatter_bytes(carrier).metadata["value"]
                    expected = 10 ** {zero_count}
                    if {negative!r}:
                        expected = -expected
                    if observed != expected:
                        raise AssertionError("canonical integer value differs")
                    after = sys.get_int_max_str_digits()
                    print(before)
                    print(after)
                    """
                )
                completed = subprocess.run(
                    [sys.executable, "-B", "-c", script],
                    check=False,
                    capture_output=True,
                    text=True,
                )

                self.assertEqual(completed.returncode, 0, completed.stderr)
                self.assertEqual(
                    completed.stdout.splitlines(),
                    [str(limit), str(limit)],
                )

    def test_quoted_giant_integer_remains_a_string(self) -> None:
        digits = "1" + "0" * 640

        parsed = parse_frontmatter_bytes(_carrier(f'value: "{digits}"'))

        self.assertEqual(parsed.metadata["value"], digits)
        self.assertIsInstance(parsed.metadata["value"], str)

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

    def test_immediate_exact_close_is_empty_frontmatter(self) -> None:
        self.assert_parse_error(b"---\n---\n")

    def test_first_exact_close_wins_over_body_delimiters_and_yaml(self) -> None:
        body = (
            b"# body\n---\nrepository_governance:\n  replacement: true\n---\n"
        )

        parsed = parse_frontmatter_bytes(_carrier("value: frontmatter", body))

        self.assertEqual(parsed.metadata, {"value": "frontmatter"})
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

    def test_ordinary_single_document_is_accepted(self) -> None:
        parsed = parse_frontmatter_bytes(_carrier("value: text"))

        self.assertEqual(parsed.metadata, {"value": "text"})

    def test_explicit_document_end_marker_is_accepted(self) -> None:
        parsed = parse_frontmatter_bytes(_carrier("value: text\n..."))

        self.assertEqual(parsed.metadata, {"value": "text"})

    def test_non_closing_document_start_and_end_markers_are_accepted(self) -> None:
        parsed = parse_frontmatter_bytes(
            _carrier("--- # document\nvalue: text\n...")
        )

        self.assertEqual(parsed.metadata, {"value": "text"})

    def test_document_markers_followed_by_comment_are_accepted(self) -> None:
        parsed = parse_frontmatter_bytes(
            _carrier("--- # document\nvalue: text\n...\n# comment")
        )

        self.assertEqual(parsed.metadata, {"value": "text"})

    def test_two_actual_documents_fail(self) -> None:
        self.assert_parse_error(
            _carrier(
                "--- # first document\n"
                "value: first\n"
                "--- # second document\n"
                "value: second"
            )
        )

    def test_residual_content_after_explicit_document_end_fails(self) -> None:
        self.assert_parse_error(_carrier("value: text\n...\nother: value"))

    def test_quoted_document_marker_strings_remain_strings(self) -> None:
        parsed = parse_frontmatter_bytes(_carrier('a: "---"\nb: "..."'))

        self.assertEqual(parsed.metadata, {"a": "---", "b": "..."})

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
