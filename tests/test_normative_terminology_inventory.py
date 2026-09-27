from __future__ import annotations

import unittest

from proto_ring.normative_terminology import (
    Occurrence,
    TerminologyPolicy,
    discover_occurrences,
    occurrence_record,
    parse_registry,
    reconcile_inventory,
)
from test_normative_terminology import FORMAT, POLICY, SPEC, numbered_section


GOLDEN_RECORDS = [
    {
        "concepts": ["alpha"],
        "section": "2.1",
        "heading": "2.1 Definitions",
        "role": "definition",
        "fingerprint": "dd512041ce4865f1c234a7dfb3c29514888f360452003fe44238e8b5d49b6d28",
    },
    {
        "concepts": ["beta"],
        "section": "2.1",
        "heading": "2.1 Definitions",
        "role": "definition",
        "fingerprint": "fd9289cbf61c3e981e55efc0351db6191ea60d3dc9942e8fd704c410484f2644",
    },
    {
        "concepts": ["alpha"],
        "section": "8",
        "heading": "8. Summary",
        "role": "summary",
        "fingerprint": "5a52e666cb3d2f6f432d5e146d10160e30248bf6a2c3cf383981df52dad48801",
    },
]


class InventoryReconciliationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.entries, registry_errors = parse_registry(SPEC, FORMAT)
        self.assertEqual([], registry_errors)
        self.occurrences, occurrence_errors = discover_occurrences(
            SPEC, self.entries, POLICY
        )
        self.assertEqual([], occurrence_errors)

    def reconcile(self, records, policy: TerminologyPolicy = POLICY) -> list[str]:
        return reconcile_inventory(
            self.entries, self.occurrences, records, policy
        )

    def test_valid_inventory_reconciles(self) -> None:
        self.assertEqual([], self.reconcile(GOLDEN_RECORDS))

    def test_occurrence_record_has_consumer_role_and_shared_signature_fields(self) -> None:
        record = occurrence_record(self.occurrences[0], POLICY)
        self.assertEqual(GOLDEN_RECORDS[0], record)
        self.assertNotIn("line", record)
        self.assertNotIn("canonical", record)

    def test_unknown_inventory_concept_is_rejected(self) -> None:
        records = [dict(item) for item in GOLDEN_RECORDS]
        records[0] = {**records[0], "concepts": ["unknown"]}
        errors = self.reconcile(records)
        self.assertTrue(any("references unknown concepts: unknown" in error for error in errors))

    def test_invalid_and_allowed_but_incorrect_roles_are_rejected(self) -> None:
        records = [dict(item) for item in GOLDEN_RECORDS]
        records[0] = {**records[0], "role": "invented"}
        errors = self.reconcile(records)
        self.assertTrue(any("has invalid role" in error for error in errors))

        records = [dict(item) for item in GOLDEN_RECORDS]
        records[0] = {**records[0], "role": "reference"}
        errors = self.reconcile(records)
        self.assertTrue(any("must use role 'definition'" in error for error in errors))

    def test_invalid_consumer_role_policy_fails_closed(self) -> None:
        with self.assertRaises(ValueError):
            TerminologyPolicy(
                registry=FORMAT,
                section_from_heading=numbered_section,
                canonical_location=lambda _entry, _block: True,
                role_for=lambda _occurrence: "definition",
                allowed_roles="definition",  # type: ignore[arg-type]
            )

        with self.assertRaises(ValueError):
            TerminologyPolicy(
                registry=FORMAT,
                section_from_heading=numbered_section,
                canonical_location=lambda _entry, _block: True,
                role_for=lambda _occurrence: "",
                allowed_roles=frozenset(),
            )

        inconsistent = TerminologyPolicy(
            registry=FORMAT,
            section_from_heading=numbered_section,
            canonical_location=lambda _entry, _block: True,
            role_for=lambda _occurrence: "not-allowed",
            allowed_roles=frozenset({"allowed"}),
        )
        with self.assertRaises(ValueError):
            occurrence_record(self.occurrences[0], inconsistent)
        errors = self.reconcile(GOLDEN_RECORDS, inconsistent)
        self.assertTrue(any("role policy returned" in error for error in errors))

    def test_section_and_heading_are_required_strings(self) -> None:
        cases = (
            ("section", None),
            ("section", 42),
            ("heading", None),
            ("heading", 42),
        )
        for field, invalid in cases:
            records = [dict(item) for item in GOLDEN_RECORDS]
            records[0].pop(field, None)
            if invalid is not None:
                records[0][field] = invalid
            errors = self.reconcile(records)
            self.assertTrue(
                any(f"must contain a string {field}" in error for error in errors),
                errors,
            )

    def test_invalid_fingerprint_is_rejected(self) -> None:
        records = [dict(item) for item in GOLDEN_RECORDS]
        records[0] = {**records[0], "fingerprint": "0" * 63}
        errors = self.reconcile(records)
        self.assertTrue(any("invalid SHA-256 fingerprint" in error for error in errors))

    def test_unreviewed_occurrence_is_rejected(self) -> None:
        errors = self.reconcile(GOLDEN_RECORDS[:-1])
        self.assertTrue(any("unreviewed definition-like occurrence" in error for error in errors))

    def test_stale_occurrence_is_rejected(self) -> None:
        records = [dict(item) for item in GOLDEN_RECORDS]
        records[0] = {**records[0], "fingerprint": "0" * 64}
        errors = self.reconcile(records)
        self.assertTrue(any("stale terminology inventory entry" in error for error in errors))

    def test_duplicate_inventory_occurrence_is_rejected_explicitly(self) -> None:
        records = [*GOLDEN_RECORDS, dict(GOLDEN_RECORDS[0])]
        errors = self.reconcile(records)
        self.assertTrue(any("duplicates an earlier occurrence" in error for error in errors))
        self.assertTrue(any("unmatched copies: 1" in error for error in errors))

    def test_duplicate_discovered_occurrence_requires_another_reviewed_copy(self) -> None:
        duplicated_actual = [*self.occurrences, self.occurrences[0]]
        errors = reconcile_inventory(
            self.entries, duplicated_actual, GOLDEN_RECORDS, POLICY
        )
        self.assertTrue(any("unreviewed definition-like occurrence" in error for error in errors))

    def test_signature_collision_reconciles_roles_as_a_multiset(self) -> None:
        canonical = Occurrence(("alpha",), "4", "4. Terms", "0" * 64, 10, True)
        noncanonical = Occurrence(("alpha",), "4", "4. Terms", "0" * 64, 20, False)
        records = [
            occurrence_record(noncanonical, POLICY),
            occurrence_record(canonical, POLICY),
        ]
        self.assertEqual(
            [],
            reconcile_inventory(
                self.entries,
                [canonical, noncanonical],
                records,
                POLICY,
            ),
        )

    def test_role_multiset_preserves_repeated_multiplicity(self) -> None:
        canonical_one = Occurrence(("alpha",), "4", "4. Terms", "0" * 64, 10, True)
        canonical_two = Occurrence(("alpha",), "4", "4. Terms", "0" * 64, 20, True)
        noncanonical = Occurrence(("alpha",), "4", "4. Terms", "0" * 64, 30, False)
        records = [
            occurrence_record(canonical_one, POLICY),
            occurrence_record(noncanonical, POLICY),
            occurrence_record(noncanonical, POLICY),
        ]
        errors = reconcile_inventory(
            self.entries,
            [canonical_one, canonical_two, noncanonical],
            records,
            POLICY,
        )
        self.assertTrue(any("do not match policy" in error for error in errors), errors)

    def test_excess_signature_role_diagnostics_are_permutation_invariant(self) -> None:
        canonical = Occurrence(("alpha",), "4", "4. Terms", "0" * 64, 10, True)
        noncanonical = Occurrence(("alpha",), "4", "4. Terms", "0" * 64, 20, False)
        definition = occurrence_record(canonical, POLICY)
        reference = occurrence_record(noncanonical, POLICY)
        excess = {**reference, "role": "summary"}
        first = reconcile_inventory(
            self.entries,
            [canonical, noncanonical],
            [excess, definition, reference],
            POLICY,
        )
        second = reconcile_inventory(
            self.entries,
            [canonical, noncanonical],
            [definition, reference, excess],
            POLICY,
        )
        self.assertEqual(first, second)
        self.assertFalse(any("role" in error for error in first), first)
        self.assertTrue(any("duplicates an earlier occurrence" in error for error in first))

    def test_malformed_occurrence_collection_and_record_fail_closed(self) -> None:
        self.assertIn(
            "terminology inventory occurrences must be a list",
            self.reconcile(None),
        )
        errors = self.reconcile(["not-a-mapping"])
        self.assertIn("terminology inventory occurrence 1 must be a mapping", errors)

    def test_concepts_must_be_a_nonempty_string_list(self) -> None:
        records = [dict(item) for item in GOLDEN_RECORDS]
        records[0] = {**records[0], "concepts": []}
        errors = self.reconcile(records)
        self.assertTrue(any("non-empty concepts list" in error for error in errors))

    def test_consumer_role_policy_changes_record_and_reconciliation(self) -> None:
        alternate = TerminologyPolicy(
            registry=FORMAT,
            section_from_heading=numbered_section,
            canonical_location=POLICY.canonical_location,
            role_for=lambda occurrence: (
                "primary" if occurrence.canonical else "secondary"
            ),
            allowed_roles=frozenset({"primary", "secondary"}),
        )
        records = [occurrence_record(item, alternate) for item in self.occurrences]
        self.assertEqual([], self.reconcile(records, alternate))
        self.assertEqual(
            ["primary", "primary", "secondary"],
            [record["role"] for record in records],
        )

    def test_occurrence_signature_sorts_concepts_and_excludes_diagnostics(self) -> None:
        first = Occurrence(("beta", "alpha"), "4", "4. Terms", "0" * 64, 12, False)
        second = Occurrence(("alpha", "beta"), "4", "4. Terms", "0" * 64, 99, True)
        self.assertEqual(("alpha", "beta"), first.concepts)
        self.assertEqual(first.signature, second.signature)
        self.assertEqual(["alpha", "beta"], occurrence_record(first, POLICY)["concepts"])


if __name__ == "__main__":
    unittest.main()
