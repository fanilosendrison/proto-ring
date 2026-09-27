from __future__ import annotations

from dataclasses import replace
import unittest

from proto_ring.normative_terminology import (
    discover_occurrences,
    fingerprint_text,
    normalize_text,
    parse_registry,
)
from test_normative_terminology import FORMAT, POLICY, SPEC


class RegistryTests(unittest.TestCase):
    def test_valid_registry_preserves_canonical_alias_deprecated_and_structure(self) -> None:
        entries, errors = parse_registry(SPEC, FORMAT)

        self.assertEqual([], errors)
        self.assertEqual(["alpha", "beta"], [entry.key for entry in entries])
        self.assertEqual("alpha", entries[0].canonical_term)
        self.assertEqual(("first concept",), entries[0].aliases)
        self.assertEqual(("old alpha",), entries[0].deprecated)
        self.assertEqual("base", entries[0].term_structure)
        self.assertEqual(
            ("alpha", "first concept", "old alpha"), entries[0].expressions
        )

    def test_registry_requires_exactly_one_boundary_pair(self) -> None:
        for changed in (
            SPEC.replace(FORMAT.start_marker, "", 1),
            SPEC.replace(FORMAT.end_marker, "", 1),
            SPEC.replace(FORMAT.start_marker, f"{FORMAT.start_marker}\n{FORMAT.start_marker}", 1),
            SPEC.replace(FORMAT.end_marker, f"{FORMAT.end_marker}\n{FORMAT.end_marker}", 1),
        ):
            with self.subTest(changed=changed.count(FORMAT.start_marker)):
                entries, errors = parse_registry(changed, FORMAT)
                self.assertEqual([], entries)
                self.assertIn(
                    "document must contain exactly one terminology registry",
                    errors,
                )

    def test_registry_boundaries_must_be_standalone_and_ordered(self) -> None:
        reversed_markers = SPEC.replace(FORMAT.start_marker, "TEMP", 1)
        reversed_markers = reversed_markers.replace(FORMAT.end_marker, FORMAT.start_marker, 1)
        reversed_markers = reversed_markers.replace("TEMP", FORMAT.end_marker, 1)
        _, errors = parse_registry(reversed_markers, FORMAT)
        self.assertTrue(any("end marker must follow" in error for error in errors))

        embedded = SPEC.replace(
            FORMAT.start_marker,
            f"prefix {FORMAT.start_marker} suffix",
            1,
        )
        entries, errors = parse_registry(embedded, FORMAT)
        self.assertEqual([], entries)
        self.assertIn("document must contain exactly one terminology registry", errors)

    def test_registry_rejects_unexpected_headers_and_malformed_separator(self) -> None:
        changed_header = SPEC.replace("Concept key", "Identifier", 1)
        _, header_errors = parse_registry(changed_header, FORMAT)
        self.assertTrue(any("headers" in error for error in header_errors))

        changed_separator = SPEC.replace("-----------", "--", 1)
        _, separator_errors = parse_registry(changed_separator, FORMAT)
        self.assertTrue(any("separator" in error for error in separator_errors))

    def test_registry_requires_exactly_one_outer_pipe_at_each_trimmed_edge(self) -> None:
        valid_row = "| `alpha` | `alpha` | [`term-alpha`](#term-alpha) | `first concept` | `old alpha` | base |"
        for malformed in (valid_row[:-1], f"|{valid_row}|"):
            changed = SPEC.replace(valid_row, malformed, 1)
            entries, errors = parse_registry(changed, FORMAT)
            self.assertEqual([], entries)
            self.assertTrue(any("outer pipe" in error for error in errors))

        entries, errors = parse_registry(
            SPEC.replace(valid_row, f"  {valid_row}  ", 1), FORMAT
        )
        self.assertEqual([], errors)
        self.assertEqual("alpha", entries[0].key)

    def test_anchor_patterns_must_capture_nonempty_destinations(self) -> None:
        link_patterns = (
            r"\[`[^`]+`\]\(#term-[a-z0-9-]+()\)",
            r"\[`[^`]+`\]\(#term-[a-z0-9-]+(?:-(missing))?\)",
        )
        for pattern in link_patterns:
            entries, errors = parse_registry(
                SPEC, replace(FORMAT, anchor_link_pattern=pattern)
            )
            self.assertEqual([], entries)
            self.assertTrue(any("non-empty canonical destination" in error for error in errors))

        anchor_format = replace(
            FORMAT,
            anchor_pattern=r'<a id="term-[a-z0-9-]+()"></a>',
        )
        entries, errors = parse_registry(SPEC, anchor_format)
        self.assertEqual([], errors)
        occurrences, discovery_errors = discover_occurrences(
            SPEC,
            entries,
            replace(POLICY, registry=anchor_format),
        )
        self.assertEqual([], occurrences)
        self.assertTrue(any("empty canonical destination" in error for error in discovery_errors))

    def test_duplicate_concept_keys_and_destinations_are_rejected(self) -> None:
        duplicate_key = SPEC.replace("| `beta` |", "| `alpha` |")
        _, errors = parse_registry(duplicate_key, FORMAT)
        self.assertIn("terminology registry contains duplicate concept keys", errors)

        duplicate_anchor = SPEC.replace(
            "[`term-beta`](#term-beta)", "[`term-alpha`](#term-alpha)"
        )
        _, errors = parse_registry(duplicate_anchor, FORMAT)
        self.assertIn(
            "terminology registry contains duplicate canonical destinations",
            errors,
        )

    def test_intra_concept_expression_duplication_is_case_insensitive(self) -> None:
        changed = SPEC.replace("`first concept`", "`ALPHA`", 1)
        _, errors = parse_registry(changed, FORMAT)
        self.assertTrue(any("repeats" in error for error in errors))

    def test_registry_rejects_malformed_backtick_delimiters(self) -> None:
        for malformed in ("`first concept", "first concept`", "``first concept``"):
            changed = SPEC.replace("`first concept`", malformed, 1)
            entries, errors = parse_registry(changed, FORMAT)
            self.assertTrue(any("backtick delimiters" in error for error in errors))
            occurrences, discovery_errors = discover_occurrences(changed, entries, POLICY)
            self.assertEqual([], occurrences)
            self.assertTrue(discovery_errors)

    def test_non_sentinel_list_cells_must_decode_an_expression(self) -> None:
        for column_value in (";", ";;", "`one`;;`two`", "`one`;"):
            for original in ("`first concept`", "`old alpha`"):
                changed = SPEC.replace(original, column_value, 1)
                entries, errors = parse_registry(changed, FORMAT)
                self.assertTrue(any("empty alias" in error or "empty deprecated" in error for error in errors))
                occurrences, discovery_errors = discover_occurrences(changed, entries, POLICY)
                self.assertEqual([], occurrences)
                self.assertTrue(discovery_errors)

    def test_em_dash_is_only_an_undecorated_empty_list_sentinel(self) -> None:
        invalid_values = (
            "`first concept`; —",
            "`—`",
            "*—*",
            "_—_",
            "~~—~~",
            '"—"',
            "‘—’",
            "`“—”`",
        )
        for value in invalid_values:
            for original in ("`first concept`", "`old alpha`"):
                changed = SPEC.replace(original, value, 1)
                entries, errors = parse_registry(changed, FORMAT)
                self.assertTrue(any("em dash" in error for error in errors))
                self.assertTrue(
                    all("—" not in expression for expression in entries[0].expressions)
                )
                occurrences, discovery_errors = discover_occurrences(changed, entries, POLICY)
                self.assertEqual([], occurrences)
                self.assertTrue(discovery_errors)

    def test_decoded_registry_values_must_remain_nonempty(self) -> None:
        empty_term = SPEC.replace("| `alpha` | `alpha` |", "| `alpha` | ` ` |")
        entries, errors = parse_registry(empty_term, FORMAT)
        self.assertTrue(any("no canonical term" in error for error in errors))
        self.assertNotIn("", entries[0].expressions)

        empty_alias = SPEC.replace("`first concept`", "``", 1)
        entries, errors = parse_registry(empty_alias, FORMAT)
        self.assertTrue(any("empty alias" in error for error in errors))
        self.assertNotIn("", entries[0].expressions)

        empty_deprecated = SPEC.replace("`old alpha`", "``", 1)
        entries, errors = parse_registry(empty_deprecated, FORMAT)
        self.assertTrue(any("empty deprecated" in error for error in errors))
        self.assertNotIn("", entries[0].expressions)

        empty_structure = SPEC.replace("| base |", "| `` |", 1)
        entries, errors = parse_registry(empty_structure, FORMAT)
        self.assertTrue(any("term-structure" in error for error in errors))
        self.assertNotIn("alpha", [entry.key for entry in entries])

    def test_same_alias_may_belong_to_distinct_compound_concepts(self) -> None:
        changed = SPEC.replace("`bounded region` | —", "`first concept` | —")
        entries, errors = parse_registry(changed, FORMAT)
        self.assertEqual([], errors)
        self.assertEqual(
            2, sum("first concept" in entry.expressions for entry in entries)
        )


class FingerprintTests(unittest.TestCase):
    def test_normalization_collapses_unicode_whitespace(self) -> None:
        self.assertEqual("alpha beta gamma", normalize_text("  alpha\n beta\tgamma  "))

    def test_sha256_vector_is_independently_fixed(self) -> None:
        self.assertEqual(
            "64989ccbf3efa9c84e2afe7cee9bc5828bf0fcb91e44f8c1e591638a2c2e90e3",
            fingerprint_text("  alpha\n beta\tgamma  "),
        )


if __name__ == "__main__":
    unittest.main()
