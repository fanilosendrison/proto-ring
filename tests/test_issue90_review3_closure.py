from __future__ import annotations

import json
from pathlib import Path
import unittest

import yaml

from conformance_corpus_test_support import authority_key, decode_transport, load_corpus
from conformance_issue66_audit import ISSUE66_RESPONSIBILITIES, audit_issue66_metadata
from conformance_issue66_metadata_audit import (
    ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS,
    context_only_runtime_overclaims,
    coverage_authority_binding_violations,
    heading_is_issue66_relevant,
)


class Issue90Review3ClosureTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, cls.responsibilities, cls.cases, cls.coverage = load_corpus()
        cls.vectors = {
            vector["vector_id"]: vector
            for matrix in cls.cases.values()
            for vector in matrix["vectors"]
        }

    @staticmethod
    def _arguments(vector: dict[str, object]) -> dict[str, object]:
        fixture = vector["fixture"]
        return decode_transport(fixture.get("invocation", fixture.get("input")))

    @classmethod
    def _carrier(cls, vector_id: str) -> dict[str, object]:
        vector = cls.vectors[vector_id]
        arguments = cls._arguments(vector)
        step = next(
            item
            for item in vector["fixture"]["steps"]
            if item.get("op") == "write_utf8"
            and item.get("path") == arguments["path"]
        )
        return yaml.safe_load(step["text"].split("---", 2)[1])

    def test_governance_binding_scalar_partitions_are_complete(self) -> None:
        expected = {
            f"governance-bindings.registry.{field}-{kind}"
            for field in {
                "registry-source", "binding-kind", "scope-kind", "capability",
                "scope-interface", "scope-object", "authority-responsibility",
                "authority-source",
            }
            for kind in {"empty", "non-string"}
        }
        self.assertTrue(expected <= set(self.vectors))
        self.assertTrue(
            all(
                self.vectors[item]["expected_observation"]["kind"]
                == "controlled_rejection"
                for item in expected
            )
        )
        print("GB_SCALAR_PARTITIONS_COMPLETE=PASS")

    def test_projection_cardinality_scalars_and_ambiguity_are_closed(self) -> None:
        empty = self.vectors["projection-registry.registry.empty-registry"]
        value = decode_transport(empty["expected_observation"]["value"])
        self.assertEqual(value["projections"], [])
        self.assertEqual(value["modes"], {})
        scalar_ids = {
            f"projection-registry.registry.{field}-{kind}"
            for field in {
                "authority-responsibility", "authority-source", "responsibility",
                "canonical-source", "secondary-source", "mode", "validation",
                "generator-source", "boundary-source", "binding",
                "target-interface", "target-object",
            }
            for kind in {"empty", "non-string"}
        }
        self.assertTrue(scalar_ids <= set(self.vectors))
        for suffix, kind in {
            "ambiguity-distinct-targets": "result",
            "ambiguity-same-target": "controlled_rejection",
            "ambiguity-distinct-bindings": "result",
            "ambiguity-same-binding": "controlled_rejection",
        }.items():
            vector = self.vectors[f"projection-registry.registry.{suffix}"]
            self.assertEqual(vector["expected_observation"]["kind"], kind)
        print("PROJECTION_AMBIGUITY_TARGET_DIMENSION=PASS")
        print("PROJECTION_AMBIGUITY_BINDING_DIMENSION=PASS")

    def test_repository_integrity_exit_codes_and_identity_are_closed(self) -> None:
        negative = decode_transport(
            self.vectors["repository-integrity.profile.exit-code-negative"]
            ["expected_observation"]["value"]
        )
        arbitrary = decode_transport(
            self.vectors["repository-integrity.profile.exit-code-arbitrary-precision"]
            ["expected_observation"]["value"]
        )
        self.assertEqual(
            negative["loaded_validation"]["command"]["undetermined_exit_codes"],
            [-1],
        )
        self.assertEqual(
            arbitrary["loaded_validation"]["command"]["undetermined_exit_codes"],
            [18446744073709551616000000000000000000000000000001],
        )
        relations = {
            "identity-selector-order": "RI_ORDERED_SELECTOR_IDENTITY",
            "identity-command-argument-order": "RI_ORDERED_ARGUMENT_IDENTITY",
            "identity-environment-value": "RI_EXACT_ENVIRONMENT_IDENTITY",
            "identity-validation-id": "RI_EXACT_VALIDATION_IDENTITY",
            "identity-prerequisite-value": "RI_EXACT_PREREQUISITE_IDENTITY",
            "identity-target-value": "RI_EXACT_TARGET_IDENTITY",
            "identity-exit-code-value": "RI_EXACT_EXIT_CODE_IDENTITY",
            "identity-authority-responsibility-value":
                "RI_EXACT_PROFILE_AUTHORITY_IDENTITY",
            "identity-instance-kind-value": "RI_EXACT_INSTANCE_KIND_IDENTITY",
        }
        for suffix, metric in relations.items():
            value = decode_transport(
                self.vectors[f"repository-integrity.profile.{suffix}"]
                ["expected_observation"]["value"]
            )
            self.assertEqual(value["identity_relation"]["relation"], "NOT_EQUAL")
            print(f"{metric}=NOT_EQUAL")

    def test_evidence_class_order_is_semantically_unordered(self) -> None:
        original = decode_transport(
            self.vectors["evidence-requirements.registry.explicit-classes-multiple"]
            ["expected_observation"]["value"]
        )
        reversed_value = decode_transport(
            self.vectors[
                "evidence-requirements.registry.explicit-multi-class-reversed"
            ]["expected_observation"]["value"]
        )
        self.assertEqual(original, reversed_value)
        classes = self._carrier(
            "evidence-requirements.registry.explicit-multi-class-reversed"
        )["evidence_requirements"]["requirements"]["gate-a"]
        self.assertEqual(
            classes["evidence_classes"]["classes"], ["class-b", "class-a"]
        )
        print("EVIDENCE_CLASS_DECLARATION_ORDER_SEMANTIC_EFFECT=none")

    def test_metadata_and_source_audit_boundaries_are_mechanical(self) -> None:
        metrics = audit_issue66_metadata(
            self.responsibilities, self.cases, self.coverage
        )
        self.assertEqual(metrics["derivation_mismatches"], 0)
        self.assertEqual(metrics["blocking_gaps_nonempty"], 0)
        self.assertEqual(metrics["context_runtime_overclaims"], 0)
        self.assertEqual(metrics["gb_read_only_unrelated_derivation_violations"], 0)
        self.assertEqual(metrics["gb_read_only_finite_witness_count"], 1)
        self.assertEqual(metrics["gb_read_only_witness_classification"], "PASS")
        self.assertEqual(metrics["gb_read_only_heading_classification"], "MIXED_CONTEXT")
        self.assertEqual(metrics["coverage_authority_violations"], 0)
        self.assertEqual(metrics["coverage_authority"], "PASS")
        self.assertEqual(metrics["coverage_full_vector_map_size"], 1082)
        self.assertEqual(metrics["coverage_relevant_headings"], 43)
        self.assertEqual(metrics["open_domain_source_audit_required"], "yes")
        print("ISSUE66_CONTEXT_ONLY_RUNTIME_DERIVATION_OVERCLAIMS=0")
        print("GB_READ_ONLY_UNRELATED_DERIVATION_VIOLATIONS=0")
        print("GB_READ_ONLY_FINITE_WITNESS_COUNT=1")
        print("GB_READ_ONLY_WITNESS_CLASSIFICATION=PASS")
        print("GB_READ_ONLY_HEADING_CLASSIFICATION=MIXED_CONTEXT")
        print("ISSUE66_COVERAGE_AUTHORITY_BINDING_VIOLATIONS=0")
        print("ISSUE66_COVERAGE_AUTHORITY_BINDING=PASS")
        print("COVERAGE_AUDIT_FULL_VECTOR_MAP_SIZE=1082")
        print("ISSUE66_RELEVANT_COVERED_HEADINGS=43")
        print("ISSUE66_OPEN_DOMAIN_SOURCE_AUDIT_REQUIRED=yes")

    def test_read_only_vector_claims_only_observed_nonmutation(self) -> None:
        distinction = self.vectors[
            "evidence-requirements.registry.read-only"
        ]["state_distinctions"]
        self.assertEqual(
            distinction,
            ["Evidence Requirements loading leaves the fixture repository unchanged"],
        )
        encoded = json.dumps(distinction).lower()
        self.assertNotIn("source resolution", encoded)
        self.assertNotIn("external reads", encoded)
        self.assertEqual(len(ISSUE66_RESPONSIBILITIES), 5)


class Issue66CoverageAuditScopeTest(unittest.TestCase):
    contract = {
        "source_kind": "repository",
        "repository": "example/repository",
        "commit": "0123456789abcdef",
        "path": "docs/contracts/projection-integrity.md",
    }

    @classmethod
    def _key(cls, heading: str) -> str:
        authority = {
            **cls.contract,
            "clause": heading,
            "sha256": None,
        }
        return authority_key(authority)

    @classmethod
    def _vector(
        cls,
        vector_id: str,
        responsibility_id: str,
        heading: str,
        *,
        authority: bool = True,
        derivation: bool = True,
    ) -> dict[str, object]:
        item = {
            **cls.contract,
            "clause": heading,
            "sha256": None,
        }
        return {
            "vector_id": vector_id,
            "authorities": [item] if authority else [],
            "expected_observation": {
                "derivation": {
                    "authority_keys": [cls._key(heading)] if derivation else [],
                }
            },
        }

    @staticmethod
    def _cases(*entries: tuple[str, dict[str, object]]):
        grouped = {}
        for responsibility_id, vector in entries:
            grouped.setdefault(
                responsibility_id,
                {"responsibility_id": responsibility_id, "vectors": []},
            )["vectors"].append(vector)
        return grouped

    @classmethod
    def _coverage(
        cls,
        heading: str,
        responsibilities: list[str],
        refs: list[str],
        disposition: str = "covered",
    ) -> list[dict[str, object]]:
        return [
            {
                "contract": cls.contract,
                "headings": [
                    {
                        "heading_path": heading,
                        "disposition": disposition,
                        "responsibility_ids": responsibilities,
                        "vector_refs": refs,
                    }
                ],
            }
        ]

    def test_a_unrelated_heading_is_excluded(self) -> None:
        responsibility = "accepted-adr-body.immutability"
        vector_id = f"{responsibility}.accepted-creation"
        vector = self._vector(vector_id, responsibility, "Unrelated")
        cases = self._cases((responsibility, vector))
        coverage = self._coverage("Unrelated", [responsibility], [vector_id])
        vectors = {vector_id: (responsibility, vector)}
        self.assertFalse(heading_is_issue66_relevant(coverage[0]["headings"][0], vectors))
        self.assertEqual(coverage_authority_binding_violations(coverage, cases), [])
        print("UNRELATED_COVERAGE_HEADING_EXCLUDED=PASS")

    def test_b_valid_issue66_reference(self) -> None:
        responsibility = "projection-registry.registry"
        vector_id = f"{responsibility}.reference-mode"
        vector = self._vector(vector_id, responsibility, "Binding")
        cases = self._cases((responsibility, vector))
        coverage = self._coverage("Binding", [responsibility], [vector_id])
        self.assertEqual(coverage_authority_binding_violations(coverage, cases), [])

    def test_c_cross_owner_reference(self) -> None:
        local = "projection-registry.registry.validation-reference"
        upstream = (
            "repository-governance-model.load."
            "projection-integrity-requires-repository-integrity"
        )
        heading = "Repository Integrity validation binding"
        cases = self._cases(
            ("projection-registry.registry", self._vector(local, "projection-registry.registry", heading)),
            ("repository-governance-model.load", self._vector(upstream, "repository-governance-model.load", heading)),
        )
        coverage = self._coverage(
            heading,
            ["projection-registry.registry", "repository-governance-model.load"],
            [local, upstream],
        )
        self.assertEqual(coverage_authority_binding_violations(coverage, cases), [])
        print("CROSS_OWNER_COVERAGE_REFERENCE_TEST=PASS")

    def test_d_dangling_reference(self) -> None:
        coverage = self._coverage(
            "Binding", ["projection-registry.registry"], ["missing.vector"]
        )
        self.assertEqual(
            [item["kind"] for item in coverage_authority_binding_violations(coverage, {})],
            ["dangling_vector_ref"],
        )

    def test_e_missing_issue66_responsibility(self) -> None:
        responsibility = "projection-registry.registry"
        vector_id = f"{responsibility}.reference-mode"
        vector = self._vector(vector_id, responsibility, "Binding")
        cases = self._cases((responsibility, vector))
        coverage = self._coverage(
            "Binding", ["repository-governance-model.load"], [vector_id]
        )
        self.assertEqual(
            [item["kind"] for item in coverage_authority_binding_violations(coverage, cases)],
            ["responsibility_mismatch"],
        )
        print("ISSUE66_REFERENCE_SCOPE_MISMATCH_DETECTED=PASS")

    def test_f_missing_authority(self) -> None:
        responsibility = "projection-registry.registry"
        vector_id = f"{responsibility}.reference-mode"
        vector = self._vector(
            vector_id, responsibility, "Binding", authority=False
        )
        cases = self._cases((responsibility, vector))
        coverage = self._coverage("Binding", [responsibility], [vector_id])
        self.assertEqual(
            [item["kind"] for item in coverage_authority_binding_violations(coverage, cases)],
            ["authority_missing"],
        )

    def test_g_missing_derivation_authority(self) -> None:
        responsibility = "projection-registry.registry"
        vector_id = f"{responsibility}.reference-mode"
        vector = self._vector(
            vector_id, responsibility, "Binding", derivation=False
        )
        cases = self._cases((responsibility, vector))
        coverage = self._coverage("Binding", [responsibility], [vector_id])
        self.assertEqual(
            [item["kind"] for item in coverage_authority_binding_violations(coverage, cases)],
            ["derivation_authority_missing"],
        )

    def test_h_context_derivation_is_separate(self) -> None:
        responsibility = "projection-registry.registry"
        vector_id = f"{responsibility}.reference-mode"
        heading = "Authority and evidence boundaries"
        pure_context_key = next(
            key
            for key in ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS
            if "docs/contracts/projection-integrity.md#Authority and evidence boundaries"
            in key
        )
        self.assertIn(pure_context_key, ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS)
        print("SYNTHETIC_PURE_CONTEXT_KEY_IS_CANONICAL=PASS")
        vector = self._vector(vector_id, responsibility, heading)
        vector["expected_observation"]["derivation"]["authority_keys"] = [
            pure_context_key
        ]
        cases = self._cases((responsibility, vector))
        coverage = self._coverage(
            heading, [responsibility], [vector_id], disposition="context"
        )
        self.assertEqual(coverage_authority_binding_violations(coverage, cases), [])
        self.assertEqual(context_only_runtime_overclaims(coverage, cases), 1)
        print("SYNTHETIC_PURE_CONTEXT_DERIVATION_DETECTED=PASS")
        prefix, commit_and_suffix = pure_context_key.split("@", 1)
        _, suffix = commit_and_suffix.split(":", 1)
        modified_key = f"{prefix}@{'f' * 40}:{suffix}"
        self.assertNotIn(modified_key, ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS)
        vector["expected_observation"]["derivation"]["authority_keys"] = [
            modified_key
        ]
        self.assertEqual(context_only_runtime_overclaims(coverage, cases), 0)
        print("PURE_CONTEXT_AUTHORITY_EXACT_IDENTITY_TEST=PASS")
        print("CONTEXT_AUDIT_SEPARATED_FROM_COVERAGE_BINDING_AUDIT=PASS")
        print("PURE_CONTEXT_SYNTHETIC_TEST_ISOLATION=PASS")


if __name__ == "__main__":
    unittest.main()
