from __future__ import annotations

import subprocess
import sys
import textwrap
import unittest

from proto_ring import structured_data
from proto_ring.structured_data import (
    StructuredDataError,
    parse_document_bytes,
    parse_frontmatter_bytes,
)


def _carrier(payload: str, body: bytes = b"") -> bytes:
    return b"---\n" + payload.encode("utf-8") + b"\n---\n" + body


class StructuredDataTests(unittest.TestCase):
    def assert_frontmatter_error(self, data: bytes) -> None:
        with self.assertRaises(StructuredDataError):
            parse_frontmatter_bytes(data)

    def assert_document_error(self, data: bytes) -> None:
        with self.assertRaises(StructuredDataError):
            parse_document_bytes(data)

    def assert_large_integer_ignores_limit(
        self,
        *,
        entrypoint: str,
        limit: int,
        zero_count: int,
        negative: bool,
    ) -> None:
        script = textwrap.dedent(
            f"""
            import sys
            from proto_ring.structured_data import {entrypoint}

            sys.set_int_max_str_digits({limit})
            before = sys.get_int_max_str_digits()
            digits = "1" + "0" * {zero_count}
            sign = "-" if {negative!r} else ""
            number = (sign + digits).encode("ascii")
            if {entrypoint!r} == "parse_frontmatter_bytes":
                observed = {entrypoint}(
                    b"---\\nvalue: " + number + b"\\n---\\n"
                ).metadata["value"]
            else:
                observed = {entrypoint}(number)
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
        self.assertEqual(completed.stdout.splitlines(), [str(limit), str(limit)])

    def test_public_api_is_exact(self) -> None:
        self.assertEqual(
            structured_data.__all__,
            [
                "ParsedFrontmatter",
                "StructuredDataError",
                "StructuredValue",
                "parse_document_bytes",
                "parse_frontmatter_bytes",
            ],
        )

    def test_frontmatter_complete_scalar_and_collection_model(self) -> None:
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
sequence: [item, false]
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

    def test_frontmatter_canonical_integer_semantics(self) -> None:
        values = (0, 42, -42, 2**64, 2**127)
        payload = "\n".join(f"value_{i}: {value}" for i, value in enumerate(values))
        parsed = parse_frontmatter_bytes(_carrier(payload))
        self.assertEqual(parsed.metadata, {f"value_{i}": v for i, v in enumerate(values)})
        if not hasattr(sys, "set_int_max_str_digits"):
            self.skipTest("CPython integer-string digit limits are unavailable")
        for limit, zero_count, negative in ((640, 640, False), (4300, 4300, False), (640, 640, True)):
            with self.subTest(limit=limit, negative=negative):
                self.assert_large_integer_ignores_limit(
                    entrypoint="parse_frontmatter_bytes",
                    limit=limit,
                    zero_count=zero_count,
                    negative=negative,
                )

    def test_frontmatter_quoted_and_noncanonical_scalars_remain_strings(self) -> None:
        giant = "1" + "0" * 640
        values = [
            "True", "FALSE", "yes", "no", "on", "off", "Null", "NULL", "~",
            "01", "-0", "+1", "1_000", "0x10", "0o10", "0b10", "1.0",
            "1e3", "2026-09-29",
        ]
        payload = "\n".join(f"value_{i}: {value}" for i, value in enumerate(values))
        payload += f'\nquoted_giant: "{giant}"\ndouble: "null"\nsingle: \'null\''
        parsed = parse_frontmatter_bytes(_carrier(payload))
        self.assertEqual(
            parsed.metadata,
            {
                **{f"value_{i}": value for i, value in enumerate(values)},
                "quoted_giant": giant,
                "double": "null",
                "single": "null",
            },
        )

    def test_frontmatter_body_and_first_exact_close_are_preserved(self) -> None:
        body = b"# body\n---\nrepository_governance:\n  replacement: true\n---\n"
        parsed = parse_frontmatter_bytes(_carrier("value: frontmatter", body))
        self.assertEqual(parsed.metadata, {"value": "frontmatter"})
        self.assertEqual(parsed.body, body)
        self.assert_frontmatter_error(b"---\n---\n")

    def test_frontmatter_complete_carrier_byte_rules(self) -> None:
        cases = (
            b"---\nvalue: text\n---\nbody: \xff",
            b"\xef\xbb\xbf" + _carrier("value: text"),
            _carrier("value: text", b"body\xef\xbb\xbf"),
            b"---\r\nvalue: text\r\n---\r\n",
            _carrier("value: text", b"body\rtext"),
        )
        for data in cases:
            with self.subTest(data=data):
                self.assert_frontmatter_error(data)

    def test_frontmatter_envelope_and_root_failures(self) -> None:
        cases = (
            b"value: text\n---\n",
            b"---\nvalue: text\n",
            _carrier("value: [one"),
            _carrier("- item"),
            _carrier(""),
            _carrier("scalar"),
        )
        for data in cases:
            with self.subTest(data=data):
                self.assert_frontmatter_error(data)

    def test_frontmatter_forbidden_yaml_representations(self) -> None:
        payloads = {
            "duplicate root": "foo: one\nfoo: two",
            "duplicate nested": "outer:\n  foo: one\n  foo: two",
            "constructed duplicate": 'foo: one\n"foo": two',
            "anchor": "value: &anchor text",
            "alias": "value: *missing",
            "merge": "<<: value",
            "quoted merge": '"<<": value',
            "standard tag": "value: !!str example",
            "custom tag": "value: !custom example",
            "directive": "%YAML 1.2\n---\nvalue: text",
            "boolean key": "true: value",
            "integer key": "1: value",
            "null key": "null: value",
            "sequence key": "? [one, two]\n: value",
            "omitted root": "key:",
            "omitted nested": "outer:\n  key:",
            "literal block": "value: |\n  text",
            "folded block": "value: >\n  text",
        }
        for label, payload in payloads.items():
            with self.subTest(label=label):
                self.assert_frontmatter_error(_carrier(payload))

    def test_frontmatter_single_document_markers_and_strings(self) -> None:
        accepted = (
            "value: text",
            "value: text\n...",
            "--- # document\nvalue: text\n...",
            "--- # document\nvalue: text\n...\n# comment",
        )
        for payload in accepted:
            with self.subTest(payload=payload):
                self.assertEqual(parse_frontmatter_bytes(_carrier(payload)).metadata, {"value": "text"})
        quoted = parse_frontmatter_bytes(_carrier('a: "---"\nb: "..."'))
        self.assertEqual(quoted.metadata, {"a": "---", "b": "..."})
        rejected = (
            "--- # first\nvalue: first\n--- # second\nvalue: second",
            "value: text\n...\nother: value",
        )
        for payload in rejected:
            with self.subTest(payload=payload):
                self.assert_frontmatter_error(_carrier(payload))

    def test_standalone_complete_root_domain_without_frontmatter(self) -> None:
        cases = (
            (b"mapping:\n  value: one\n", {"mapping": {"value": "one"}}),
            (b"- one\n- false\n", ["one", False]),
            (b"plain-scalar\n", "plain-scalar"),
            (b"true\n", True),
            (b"42\n", 42),
            (b"null\n", None),
        )
        for data, expected in cases:
            with self.subTest(data=data):
                observed = parse_document_bytes(data)
                self.assertEqual(observed, expected)
                self.assertIs(type(observed), type(expected))

    def test_standalone_exact_scalar_matrix_and_quoted_forms(self) -> None:
        strings = (
            "True", "FALSE", "yes", "no", "on", "off", "Null", "NULL", "~",
            "01", "-0", "+1", "1_000", "0x10", "0o10", "0b10", "1.0",
            "1e3", "2026-09-29", "0.1.0",
        )
        cases: tuple[tuple[bytes, object], ...] = (
            (b"true", True), (b"false", False), (b"null", None),
            (b"0", 0), (b"-42", -42),
            *((value.encode("ascii"), value) for value in strings),
            (b'"true"', "true"), (b"'null'", "null"),
            (b'"01"', "01"), (b"'2026-09-29'", "2026-09-29"),
        )
        for data, expected in cases:
            with self.subTest(data=data):
                observed = parse_document_bytes(data)
                self.assertEqual(observed, expected)
                self.assertIs(type(observed), type(expected))

    def test_standalone_arbitrary_precision_integers(self) -> None:
        for value in (2**64, 2**127):
            with self.subTest(value=value):
                self.assertEqual(parse_document_bytes(str(value).encode("ascii")), value)
        if not hasattr(sys, "set_int_max_str_digits"):
            self.skipTest("CPython integer-string digit limits are unavailable")
        self.assert_large_integer_ignores_limit(
            entrypoint="parse_document_bytes",
            limit=640,
            zero_count=640,
            negative=False,
        )

    def test_standalone_forbidden_yaml_representations(self) -> None:
        documents = {
            "duplicate root": b"foo: one\nfoo: two\n",
            "duplicate nested": b"outer:\n  foo: one\n  foo: two\n",
            "constructed duplicate": b'foo: one\n"foo": two\n',
            "anchor": b"value: &a one\n",
            "alias": b"value: *a\n",
            "merge form": b"base: &base\n  one: two\nvalue:\n  <<: *base\n",
            "merge key": b'value:\n  "<<": merged\n',
            "standard tag": b"value: !!str example\n",
            "custom tag": b"value: !custom example\n",
            "directive": b"%YAML 1.2\n---\nvalue: text\n",
            "literal block": b"value: |\n  text\n",
            "folded block": b"value: >\n  text\n",
            "omitted root": b"key:\n",
            "omitted nested": b"outer:\n  key:\n",
            "non-string key": b"1: value\n",
            "malformed": b"value: [one\n",
            "empty": b"",
        }
        for label, data in documents.items():
            with self.subTest(label=label):
                self.assert_document_error(data)

    def test_standalone_document_markers(self) -> None:
        accepted = (
            b"---\nvalue: text\n...\n",
            b"--- # document\nvalue: text\n...\n# comment\n",
        )
        for data in accepted:
            with self.subTest(data=data):
                self.assertEqual(parse_document_bytes(data), {"value": "text"})
        rejected = (
            b"---\nvalue: first\n---\nvalue: second\n",
            b"---\nvalue: text\n...\nother: value\n",
        )
        for data in rejected:
            with self.subTest(data=data):
                self.assert_document_error(data)

    def test_standalone_exact_byte_failures(self) -> None:
        cases = (
            b"\xef\xbb\xbfvalue: text\n",
            b"value: text\n# \xef\xbb\xbf\n",
            b"value: text\r\n",
            b"value: text\rcomment\n",
            b"value: \xff\n",
        )
        for data in cases:
            with self.subTest(data=data):
                self.assert_document_error(data)


if __name__ == "__main__":
    unittest.main()
