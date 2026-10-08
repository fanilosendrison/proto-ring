from __future__ import annotations

import copy
import unittest

from conformance_corpus_test_support import load_corpus
from conformance_issue66_metadata_audit import (
    GB_READ_ONLY_AUTHORITY_KEY,
    GB_READ_ONLY_FINITE_WITNESS_VECTOR_IDS,
    ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS,
    context_only_runtime_overclaims,
    governance_bindings_read_only_unrelated_derivations,
    validate_governance_bindings_read_only_witness,
)


class Issue90ContextClassificationTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, _, cls.cases, cls.coverage = load_corpus()

    def test_a_valid_registry_is_canonical_witness(self) -> None:
        self.assertEqual(
            GB_READ_ONLY_FINITE_WITNESS_VECTOR_IDS,
            {"governance-bindings.registry.valid-registry"},
        )
        self.assertTrue(validate_governance_bindings_read_only_witness(self.cases))

    def test_b_classification_is_independent_of_prose_wording(self) -> None:
        cases = copy.deepcopy(self.cases)
        vector = next(
            item
            for item in cases["governance-bindings.registry"]["vectors"]
            if item["vector_id"] == "governance-bindings.registry.valid-registry"
        )
        vector["state_distinctions"] = ["arbitrary wording without classification tokens"]
        self.assertIn(vector["vector_id"], GB_READ_ONLY_FINITE_WITNESS_VECTOR_IDS)
        self.assertEqual(governance_bindings_read_only_unrelated_derivations(cases), [])

    def test_c_ordinary_vector_fanout_is_detected(self) -> None:
        cases = copy.deepcopy(self.cases)
        vector = next(
            item
            for item in cases["governance-bindings.registry"]["vectors"]
            if item["vector_id"] == "governance-bindings.registry.binding-kind-empty"
        )
        vector["expected_observation"]["derivation"]["authority_keys"].append(
            GB_READ_ONLY_AUTHORITY_KEY
        )
        self.assertEqual(
            governance_bindings_read_only_unrelated_derivations(cases),
            ["governance-bindings.registry.binding-kind-empty"],
        )

    def test_d_canonical_witness_is_not_unrelated(self) -> None:
        self.assertEqual(
            governance_bindings_read_only_unrelated_derivations(self.cases), []
        )

    def test_e_mixed_heading_remains_context(self) -> None:
        heading = next(
            heading
            for record in self.coverage
            if record["contract"]["path"]
            == "docs/contracts/shared-governance-provider.md"
            for heading in record["headings"]
            if heading["heading_path"]
            == "Governance Binding Registry / Read-only provider and installation boundary"
        )
        self.assertEqual(heading["disposition"], "context")
        self.assertNotIn(
            GB_READ_ONLY_AUTHORITY_KEY, ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS
        )

    def test_f_pure_context_derivation_is_rejected(self) -> None:
        cases = copy.deepcopy(self.cases)
        vector = cases["projection-registry.registry"]["vectors"][0]
        vector["expected_observation"]["derivation"]["authority_keys"].append(
            next(iter(ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS))
        )
        self.assertEqual(context_only_runtime_overclaims(self.coverage, cases), 1)

    def test_g_mode_fields_pure_context_authority_is_absent(self) -> None:
        vector = next(
            item
            for item in self.cases["projection-registry.registry"]["vectors"]
            if item["vector_id"] == "projection-registry.registry.mode-fields"
        )
        keys = vector["expected_observation"]["derivation"]["authority_keys"]
        self.assertTrue(ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS.isdisjoint(keys))
        print("PI_MODE_FIELDS_MECHANICALLY_SIGNIFICANT_ASSERTION_AUTHORITY=absent")


if __name__ == "__main__":
    unittest.main()
