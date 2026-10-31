from __future__ import annotations

import copy
import json
import unittest

from conformance_corpus_test_support import (
    decode_transport,
    load_corpus,
    validate_coverage,
)
from conformance_fixture_runner import materialize
from conformance_issue66_audit import (
    ISSUE66_RESPONSIBILITIES,
    audit_issue66_metadata,
    audit_raw_support_shapes,
    issue66_vector_count,
)

EXPECTED_COUNTS = {
    "exact-evidence-binding.evaluate": 44,
    "evidence-requirements.registry": 120,
    "governance-bindings.registry": 109,
    "projection-registry.registry": 109,
    "repository-integrity.profile": 173,
}


class Issue66CorpusClosureTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.index, cls.responsibilities, cls.cases, cls.coverage = load_corpus()

    def test_01_exact_inventory_is_mechanical(self) -> None:
        counts = {
            responsibility_id: len(self.cases[responsibility_id]["vectors"])
            for responsibility_id in ISSUE66_RESPONSIBILITIES
        }
        self.assertEqual(counts, EXPECTED_COUNTS)
        self.assertEqual(issue66_vector_count(self.cases), 555)
        total = sum(len(matrix["vectors"]) for matrix in self.cases.values())
        self.assertEqual(total, 1153)

    def test_02_every_raw_support_shape_is_strict(self) -> None:
        audited, errors = audit_raw_support_shapes(self.cases)
        self.assertEqual(audited, 555, "\n".join(errors))
        self.assertEqual(errors, [])

    def test_03_raw_duplicate_names_are_detected_before_decode(self) -> None:
        synthetic = copy.deepcopy(self.cases)
        vector = synthetic["governance-bindings.registry"]["vectors"][0]
        invocation = vector["fixture"]["invocation"]
        invocation["value"].append(copy.deepcopy(invocation["value"][0]))
        audited, errors = audit_raw_support_shapes(synthetic)
        self.assertEqual(
            audited,
            issue66_vector_count(self.cases) - 1,
        )
        self.assertEqual(len(errors), 1)
        self.assertIn("duplicate RecordEntry name", errors[0])

    def test_07_every_issue66_fixture_materializes_without_engine_execution(self) -> None:
        materialized = 0
        errors = []
        for responsibility_id in sorted(ISSUE66_RESPONSIBILITIES):
            for vector in self.cases[responsibility_id]["vectors"]:
                try:
                    with materialize(vector["fixture"]) as realized:
                        realized.run(lambda _root, _arguments: None)
                    materialized += 1
                except Exception as error:  # failure is reported with vector identity
                    errors.append(f"{vector['vector_id']}: {error}")
        self.assertEqual(materialized, 555, "\n".join(errors))

    def test_09_metadata_and_coverage_are_exact(self) -> None:
        metrics = audit_issue66_metadata(
            self.responsibilities, self.cases, self.coverage
        )
        self.assertEqual(
            metrics,
            {
                "derivation_mismatches": 0,
                "blocking_gaps_nonempty": 0,
                "context_runtime_overclaims": 0,
                "gb_read_only_unrelated_derivation_violations": 0,
                "gb_read_only_finite_witness_count": 1,
                "gb_read_only_witness_classification": "PASS",
                "gb_read_only_heading_classification": "MIXED_CONTEXT",
                "coverage_authority_violations": 0,
                "coverage_authority": "PASS",
                "coverage_full_vector_map_size": 1153,
                "coverage_relevant_headings": 43,
                "open_domain_source_audit_required": "yes",
                "required_uncovered": 0,
                "vector_undeclared": 0,
                "coverage_dangling": 0,
            },
        )
        type(self).metadata_metrics = metrics
        validate_coverage(self.coverage, self.responsibilities, self.cases)

    def test_10_loaded_state_observations_are_complete(self) -> None:
        required = {
            "governance-bindings.registry.loaded-provider": {
                "authority", "id", "identity", "kind", "scope"
            },
            "projection-registry.registry.loaded-generated": {
                "binding", "boundary_source", "canonical_source", "generator_source",
                "id", "mode", "responsibility", "secondary_source", "target", "validation",
            },
            "repository-integrity.profile.loaded-command": {
                "command", "id", "instances", "prerequisites", "responsibility", "target",
            },
            "evidence-requirements.registry.loaded-source-context-target": {
                "candidate_source", "context", "evidence_classes", "id", "instances",
                "responsibility", "subject_source", "target",
            },
        }
        vectors = {
            vector["vector_id"]: vector
            for responsibility_id in ISSUE66_RESPONSIBILITIES
            for vector in self.cases[responsibility_id]["vectors"]
        }
        field = {
            "governance-bindings.registry.loaded-provider": "loaded_binding",
            "projection-registry.registry.loaded-generated": "loaded_projection",
            "repository-integrity.profile.loaded-command": "loaded_validation",
            "evidence-requirements.registry.loaded-source-context-target": "loaded_requirement",
        }
        for vector_id, expected_fields in required.items():
            value = decode_transport(vectors[vector_id]["expected_observation"]["value"])
            self.assertEqual(set(value[field[vector_id]]), expected_fields)

    def test_11_identity_relations_cover_every_persistent_family(self) -> None:
        suffixes = {
            "authority", "environments", "continuation", "validation-membership",
            "total-order", "validation-responsibility", "prerequisites", "instances",
            "selectors", "target", "command-environment", "command-arguments",
            "undetermined-exit-codes",
        }
        vectors = {
            vector["vector_id"]: vector
            for vector in self.cases["repository-integrity.profile"]["vectors"]
        }
        for suffix in suffixes:
            vector_id = f"repository-integrity.profile.identity-{suffix}"
            value = decode_transport(vectors[vector_id]["expected_observation"]["value"])
            self.assertEqual(value["identity_relation"]["relation"], "NOT_EQUAL")
        for vector_id in {
            "repository-integrity.profile.profile-order-identity",
            "repository-integrity.profile.profile-glob-identity",
            "repository-integrity.profile.profile-body-identity",
            "repository-integrity.profile.identity-environment-order",
            "repository-integrity.profile.identity-prerequisite-order",
            "repository-integrity.profile.identity-exit-code-order",
        }:
            value = decode_transport(vectors[vector_id]["expected_observation"]["value"])
            self.assertEqual(value["identity_relation"]["relation"], "EQUAL")
        for vector_id in {
            "repository-integrity.profile.identity-selector-order",
            "repository-integrity.profile.identity-command-argument-order",
            "repository-integrity.profile.identity-environment-value",
            "repository-integrity.profile.identity-validation-id",
            "repository-integrity.profile.identity-prerequisite-value",
            "repository-integrity.profile.identity-target-value",
            "repository-integrity.profile.identity-exit-code-value",
            "repository-integrity.profile.identity-authority-responsibility-value",
            "repository-integrity.profile.identity-instance-kind-value",
        }:
            value = decode_transport(vectors[vector_id]["expected_observation"]["value"])
            self.assertEqual(value["identity_relation"]["relation"], "NOT_EQUAL")
        encoded = json.dumps(
            [vectors[vector_id] for vector_id in vectors], sort_keys=True
        ).lower()
        self.assertNotIn("sha-256", encoded)
        self.assertNotIn("python framing", encoded)

    def test_12_closure_metrics_remain_exact_without_global_artifacts(self) -> None:
        rejection_count = sum(
            vector["expected_observation"]["kind"] == "controlled_rejection"
            for responsibility_id in ISSUE66_RESPONSIBILITIES
            for vector in self.cases[responsibility_id]["vectors"]
        )
        metadata = audit_issue66_metadata(
            self.responsibilities, self.cases, self.coverage
        )
        self.assertEqual(issue66_vector_count(self.cases), 555)
        self.assertGreater(rejection_count, 0)
        self.assertEqual(metadata["derivation_mismatches"], 0)
        self.assertEqual(metadata["blocking_gaps_nonempty"], 0)
        self.assertEqual(metadata["context_runtime_overclaims"], 0)
        self.assertEqual(metadata["coverage_authority_violations"], 0)
        self.assertEqual(metadata["required_uncovered"], 0)
        self.assertEqual(metadata["vector_undeclared"], 0)
        self.assertEqual(metadata["coverage_dangling"], 0)


if __name__ == "__main__":
    unittest.main()
