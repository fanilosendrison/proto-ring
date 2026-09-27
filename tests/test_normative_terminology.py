from __future__ import annotations

import re
import unittest

from proto_ring.normative_terminology import (
    RegistryFormat,
    TerminologyPolicy,
    discover_occurrences,
    parse_blocks,
    parse_registry,
)
FORMAT = RegistryFormat(
    start_marker="<!-- terminology:start -->",
    end_marker="<!-- terminology:end -->",
    headers=(
        "Concept key",
        "Canonical term",
        "Canonical anchor",
        "Accepted aliases",
        "Deprecated wording",
        "Term structure",
    ),
    key_pattern=r"^[a-z][a-z0-9-]*$",
    anchor_link_pattern=r"\[`[^`]+`\]\(#(term-[a-z0-9-]+)\)",
    anchor_pattern=r'<a id="(term-[a-z0-9-]+)"></a>',
)


def numbered_section(heading: str) -> str:
    match = re.match(r"^(\d+(?:\.\d+)*[A-Z]?)\b", heading)
    return match.group(1) if match else ""


def canonical_section_two(_entry, block) -> bool:
    return block.section.startswith("2")


def role_for(occurrence) -> str:
    if occurrence.canonical:
        return "definition"
    return "summary" if occurrence.section.startswith("8") else "reference"


POLICY = TerminologyPolicy(
    registry=FORMAT,
    section_from_heading=numbered_section,
    canonical_location=canonical_section_two,
    role_for=role_for,
    allowed_roles=frozenset({"definition", "reference", "summary"}),
)

SPEC = """# Example specification

# 0. Intent

Alpha supports the product promise without defining it.

# 2. Core model

<!-- terminology:start -->
| Concept key | Canonical term | Canonical anchor | Accepted aliases | Deprecated wording | Term structure |
| ----------- | -------------- | ---------------- | ---------------- | ------------------ | -------------- |
| `alpha` | `alpha` | [`term-alpha`](#term-alpha) | `first concept` | `old alpha` | base |
| `beta` | `beta region` | [`term-beta`](#term-beta) | `bounded region` | — | compound |
<!-- terminology:end -->

## 2.1 Definitions

<a id="term-alpha"></a>

An **alpha** is the first canonical concept.

<a id="term-beta"></a>

A **beta region** is a separate compound concept.

# 8. Summary

Alpha refers to the canonical concept.
"""


class DiscoveryTests(unittest.TestCase):
    def entries(self, specification: str = SPEC):
        entries, errors = parse_registry(specification, FORMAT)
        self.assertEqual([], errors)
        return entries

    def occurrences(self, specification: str = SPEC, policy: TerminologyPolicy = POLICY):
        occurrences, errors = discover_occurrences(
            specification, self.entries(specification), policy
        )
        self.assertEqual([], errors)
        return occurrences

    def test_canonical_and_noncanonical_occurrences_are_distinct(self) -> None:
        occurrences = self.occurrences()

        self.assertEqual(3, len(occurrences))
        self.assertEqual(
            [("alpha",), ("beta",)],
            [item.concepts for item in occurrences if item.canonical],
        )
        summary = [item for item in occurrences if not item.canonical]
        self.assertEqual(1, len(summary))
        self.assertEqual(("alpha",), summary[0].concepts)
        self.assertEqual("8", summary[0].section)

    def test_missing_duplicate_and_unregistered_anchors_are_rejected(self) -> None:
        missing = SPEC.replace('<a id="term-alpha"></a>\n\n', "", 1)
        _, errors = discover_occurrences(missing, self.entries(missing), POLICY)
        self.assertIn("canonical anchor is missing: term-alpha", errors)

        anchor = '<a id="term-alpha"></a>'
        duplicated = SPEC.replace(anchor, f"{anchor}\n\n{anchor}", 1)
        _, errors = discover_occurrences(
            duplicated, self.entries(duplicated), POLICY
        )
        self.assertIn("canonical anchor occurs more than once: term-alpha", errors)

        unregistered = SPEC.replace(
            "# 8. Summary",
            '<a id="term-gamma"></a>\n\nGamma is another concept.\n\n# 8. Summary',
        )
        _, errors = discover_occurrences(
            unregistered, self.entries(unregistered), POLICY
        )
        self.assertIn(
            "canonical terminology anchor is not registered: term-gamma", errors
        )

    def test_atx_headings_accept_indentation_closers_and_empty_text(self) -> None:
        decorated = SPEC.replace(
            "## 2.1 Definitions", "   ## 2.1 Definitions ###"
        )
        occurrences = self.occurrences(decorated)
        self.assertTrue(
            all(item.heading == "2.1 Definitions" for item in occurrences if item.canonical)
        )

        empty = SPEC.replace(
            "# 8. Summary",
            "#\n\nAlpha means a detached candidate.\n\n# 8. Summary",
        )
        occurrences = self.occurrences(empty)
        detached = [item for item in occurrences if item.heading == ""]
        self.assertEqual(1, len(detached))
        self.assertEqual("", detached[0].section)

        blocks, _anchors = parse_blocks("## ###\n\nAlpha means one.\n", POLICY)
        self.assertEqual("", blocks[0].heading)
        self.assertEqual("", blocks[0].section)

    def test_anchor_must_immediately_bind_its_own_definition(self) -> None:
        non_definition = SPEC.replace(
            "An **alpha** is the first canonical concept.",
            "Alpha appears without a lexical assignment.",
        )
        _, errors = discover_occurrences(
            non_definition, self.entries(non_definition), POLICY
        )
        self.assertTrue(any("does not lead to a definition" in error for error in errors))

        consecutive = SPEC.replace(
            '<a id="term-alpha"></a>\n\nAn **alpha** is the first canonical concept.\n\n<a id="term-beta"></a>',
            '<a id="term-alpha"></a>\n\n<a id="term-beta"></a>\n\nAn **alpha** is the first canonical concept.',
        )
        _, errors = discover_occurrences(
            consecutive, self.entries(consecutive), POLICY
        )
        self.assertTrue(any("immediately followed" in error for error in errors))

    def test_duplicate_intervening_anchor_cannot_disappear_from_binding(self) -> None:
        changed = SPEC.replace(
            '<a id="term-alpha"></a>\n\nAn **alpha** is',
            '<a id="term-alpha"></a>\n\n<a id="term-gamma"></a>\n\n'
            '<a id="term-gamma"></a>\n\nAn **alpha** is',
        )
        _, errors = discover_occurrences(changed, self.entries(changed), POLICY)
        self.assertTrue(any("term-alpha" in error and "immediately followed" in error for error in errors))

    def test_one_block_can_report_multiple_concepts(self) -> None:
        changed = SPEC.replace(
            "# 8. Summary",
            "# 4. Combined\n\nAlpha means one. Beta region means two.\n\n# 8. Summary",
        )
        combined = [
            item
            for item in self.occurrences(changed)
            if not item.canonical and item.section == "4"
        ]
        self.assertEqual(1, len(combined))
        self.assertEqual(("alpha", "beta"), combined[0].concepts)

    def test_non_boolean_canonical_location_policy_fails_closed(self) -> None:
        invalid = TerminologyPolicy(
            registry=FORMAT,
            section_from_heading=numbered_section,
            canonical_location=lambda _entry, _block: "false",
            role_for=role_for,
            allowed_roles=POLICY.allowed_roles,
        )
        with self.assertRaises(ValueError):
            discover_occurrences(SPEC, self.entries(), invalid)

    def test_invalid_registry_prevents_occurrence_discovery(self) -> None:
        invalid_document = SPEC.replace("`first concept`", "``", 1)
        entries, registry_errors = parse_registry(invalid_document, FORMAT)
        self.assertTrue(registry_errors)
        occurrences, errors = discover_occurrences(invalid_document, entries, POLICY)
        self.assertEqual([], occurrences)
        self.assertTrue(any("empty alias" in error for error in errors))

    def test_consumer_canonical_location_policy_changes_interpretation(self) -> None:
        moved = SPEC.replace("## 2.1 Definitions", "## 7.1 Definitions")
        policy = TerminologyPolicy(
            registry=FORMAT,
            section_from_heading=numbered_section,
            canonical_location=lambda _entry, block: block.section.startswith("7"),
            role_for=role_for,
            allowed_roles=POLICY.allowed_roles,
        )
        occurrences, errors = discover_occurrences(
            moved, self.entries(moved), policy
        )
        self.assertEqual([], errors)
        self.assertEqual(2, sum(item.canonical for item in occurrences))
        self.assertTrue(
            all(item.section == "7.1" for item in occurrences if item.canonical)
        )


if __name__ == "__main__":
    unittest.main()
