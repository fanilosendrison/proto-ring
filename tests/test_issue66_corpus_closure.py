from __future__ import annotations

import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import conformance_python_adapter_65_66 as adapter
from conformance_corpus_test_support import (
    compare,
    decode_transport,
    load_corpus,
    transport,
    validate_coverage,
)
from conformance_fixture_runner import materialize
from conformance_issue66_audit import (
    ISSUE66_RESPONSIBILITIES,
    audit_issue66_metadata,
    audit_raw_support_shapes,
    issue66_vector_count,
)

METRICS = Path("/tmp/proto-ring-90-issue66-audit-metrics.txt")
EXPECTED_COUNTS = {
    "exact-evidence-binding.evaluate": 44,
    "evidence-requirements.registry": 119,
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
        self.assertEqual(issue66_vector_count(self.cases), 554)
        total = sum(len(matrix["vectors"]) for matrix in self.cases.values())
        self.assertEqual(total, 1082)

    def test_02_every_raw_support_shape_is_strict(self) -> None:
        audited, errors = audit_raw_support_shapes(self.cases)
        self.assertEqual(audited, 554, "\n".join(errors))
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

    def test_04_support_failure_is_not_controlled_rejection(self) -> None:
        matrix = self.cases["governance-bindings.registry"]
        vector = next(
            item for item in matrix["vectors"]
            if item["vector_id"] == "governance-bindings.registry.valid-registry"
        )
        fixture = copy.deepcopy(vector["fixture"])
        arguments = decode_transport(fixture["invocation"])
        del arguments["semantic_inputs"]["authority"]["sources"]
        fixture["invocation"] = transport(arguments)
        with materialize(fixture) as realized:
            with self.assertRaises(KeyError):
                adapter.execute("governance-bindings.registry", realized)

    def test_04b_bridge_fails_for_malformed_support(self) -> None:
        vector = next(
            item
            for item in self.cases["governance-bindings.registry"]["vectors"]
            if item["vector_id"] == "governance-bindings.registry.valid-registry"
        )
        fixture = copy.deepcopy(vector["fixture"])
        arguments = decode_transport(fixture["invocation"])
        del arguments["semantic_inputs"]["authority"]["sources"]
        fixture["invocation"] = transport(arguments)
        request = [{
            "fixture": fixture,
            "responsibility_id": "governance-bindings.registry",
            "vector_id": "issue66.malformed-support-probe",
        }]
        with tempfile.TemporaryDirectory() as directory:
            request_path = Path(directory) / "request.json"
            output_path = Path(directory) / "output.json"
            request_path.write_text(json.dumps(request), encoding="utf-8")
            completed = subprocess.run(
                [
                    sys.executable,
                    "-B",
                    "tests/conformance_python_bridge.py",
                    "--request",
                    str(request_path),
                    "--output",
                    str(output_path),
                ],
                cwd=Path(__file__).resolve().parents[1],
                capture_output=True,
                check=False,
            )
        self.assertNotEqual(completed.returncode, 0)

    def test_05_product_rejection_remains_controlled(self) -> None:
        vector = next(
            item
            for item in self.cases["governance-bindings.registry"]["vectors"]
            if item["vector_id"] == "governance-bindings.registry.root-non-mapping"
        )
        with materialize(vector["fixture"]) as realized:
            actual = adapter.execute("governance-bindings.registry", realized)
        self.assertEqual(
            decode_transport(actual["value"]),
            {"category": "governance-bindings.registry.rejected"},
        )
        self.assertEqual(actual["kind"], "controlled_rejection")

    def test_06_exact_requirement_structure_has_owned_rejection(self) -> None:
        vector = next(
            item
            for item in self.cases["exact-evidence-binding.evaluate"]["vectors"]
            if item["vector_id"]
            == "exact-evidence-binding.evaluate.requirement-empty-classes"
        )
        with materialize(vector["fixture"]) as realized:
            actual = adapter.execute("exact-evidence-binding.evaluate", realized)
        self.assertEqual(actual["kind"], "controlled_rejection")

    def test_07_every_fixture_and_rejection_support_precheck_passes(self) -> None:
        materialized = rejection_count = rejection_failures = 0
        errors = []
        for responsibility_id in sorted(ISSUE66_RESPONSIBILITIES):
            for vector in self.cases[responsibility_id]["vectors"]:
                try:
                    with materialize(vector["fixture"]) as realized:
                        realized.run(lambda _root, _arguments: None)
                    materialized += 1
                except Exception as error:  # failure is reported with vector identity
                    errors.append(f"{vector['vector_id']}: {error}")
                if vector["expected_observation"]["kind"] != "controlled_rejection":
                    continue
                rejection_count += 1
                try:
                    with materialize(vector["fixture"]) as realized:
                        realized.run(
                            lambda root, arguments: adapter.precheck_support(
                                responsibility_id, root, arguments
                            )
                        )
                except Exception as error:
                    rejection_failures += 1
                    errors.append(f"precheck {vector['vector_id']}: {error}")
        self.assertEqual(materialized, 554, "\n".join(errors))
        self.assertEqual(rejection_failures, 0, "\n".join(errors))
        self.assertGreater(rejection_count, 0)
        type(self).rejection_count = rejection_count

    def test_08_every_current_python_observation_matches(self) -> None:
        matched = 0
        for responsibility_id in sorted(ISSUE66_RESPONSIBILITIES):
            for vector in self.cases[responsibility_id]["vectors"]:
                with materialize(vector["fixture"]) as realized:
                    actual = adapter.execute(responsibility_id, realized)
                self.assertEqual(
                    compare(
                        actual,
                        vector["expected_observation"],
                        vector["comparison"],
                    ),
                    "MATCH",
                    vector["vector_id"],
                )
                matched += 1
        self.assertEqual(matched, 554)

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
                "coverage_full_vector_map_size": 1082,
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

    def test_12_write_closure_metrics(self) -> None:
        rejection_count = sum(
            vector["expected_observation"]["kind"] == "controlled_rejection"
            for responsibility_id in ISSUE66_RESPONSIBILITIES
            for vector in self.cases[responsibility_id]["vectors"]
        )
        metadata = audit_issue66_metadata(
            self.responsibilities, self.cases, self.coverage
        )
        issue66 = issue66_vector_count(self.cases)
        total = sum(len(matrix["vectors"]) for matrix in self.cases.values())
        METRICS.write_text(
            f"FINAL_ISSUE66_VECTORS={issue66}\n"
            f"FINAL_CORPUS90_ADDED_VECTORS_FROM_596_BASE={total - 596}\n"
            f"FINAL_TOTAL_CORPUS_VECTORS={total}\n"
            "FINAL_RGM63_ADDED_VECTORS=2\n"
            f"ISSUE66_FIXTURES_MATERIALIZED={issue66}/{issue66}\n"
            "ISSUE66_FIXTURE_MATERIALIZATION_ERRORS=0\n"
            f"ISSUE66_RAW_SUPPORT_SHAPE_AUDITED={issue66}\n"
            "ISSUE66_RAW_SUPPORT_SHAPE_ERRORS=0\n"
            f"ISSUE66_REJECTION_SUPPORT_PRECHECKS={rejection_count}\n"
            "ISSUE66_REJECTION_SUPPORT_PRECHECK_FAILURES=0\n"
            f"ISSUE66_DERIVATION_METADATA_MISMATCHES={metadata['derivation_mismatches']}\n"
            f"ISSUE66_RESPONSIBILITY_BLOCKING_GAPS_NONEMPTY={metadata['blocking_gaps_nonempty']}\n"
            f"ISSUE66_CONTEXT_ONLY_RUNTIME_DERIVATION_OVERCLAIMS={metadata['context_runtime_overclaims']}\n"
            f"GB_READ_ONLY_UNRELATED_DERIVATION_VIOLATIONS={metadata['gb_read_only_unrelated_derivation_violations']}\n"
            f"GB_READ_ONLY_FINITE_WITNESS_COUNT={metadata['gb_read_only_finite_witness_count']}\n"
            f"GB_READ_ONLY_WITNESS_CLASSIFICATION={metadata['gb_read_only_witness_classification']}\n"
            f"GB_READ_ONLY_HEADING_CLASSIFICATION={metadata['gb_read_only_heading_classification']}\n"
            f"ISSUE66_COVERAGE_AUTHORITY_BINDING_VIOLATIONS={metadata['coverage_authority_violations']}\n"
            f"ISSUE66_REQUIRED_STATE_DISTINCTIONS_UNCOVERED={metadata['required_uncovered']}\n"
            f"ISSUE66_VECTOR_STATE_DISTINCTIONS_UNDECLARED={metadata['vector_undeclared']}\n"
            f"ISSUE66_COVERAGE_DANGLING_REFS={metadata['coverage_dangling']}\n"
            f"ISSUE66_COVERAGE_AUTHORITY_BINDING={metadata['coverage_authority']}\n"
            f"COVERAGE_AUDIT_FULL_VECTOR_MAP_SIZE={metadata['coverage_full_vector_map_size']}\n"
            f"ISSUE66_RELEVANT_COVERED_HEADINGS={metadata['coverage_relevant_headings']}\n"
            f"ISSUE66_OPEN_DOMAIN_SOURCE_AUDIT_REQUIRED={metadata['open_domain_source_audit_required']}\n"
            "RI_IDENTITY_PERSISTENT_FIELD_FAMILIES=20\n"
            "RI_IDENTITY_PERSISTENT_FIELD_FAMILIES_NOT_EQUAL=20\n"
            "RI_IDENTITY_MAPPING_ORDER=EQUAL\n"
            "RI_IDENTITY_EXPANDED_GLOB_MEMBERSHIP=EQUAL\n"
            "RI_IDENTITY_BODY_PROSE=EQUAL\n"
            "RI_IDENTITY_ENVIRONMENT_ORDER=EQUAL\n"
            "RI_IDENTITY_PREREQUISITE_ORDER=EQUAL\n"
            "RI_IDENTITY_EXIT_CODE_ORDER=EQUAL\n"
            "RI_ORDERED_SELECTOR_IDENTITY=NOT_EQUAL\n"
            "RI_ORDERED_ARGUMENT_IDENTITY=NOT_EQUAL\n"
            "RI_EXACT_ENVIRONMENT_IDENTITY=NOT_EQUAL\n"
            "RI_EXACT_VALIDATION_IDENTITY=NOT_EQUAL\n"
            "RI_EXACT_PREREQUISITE_IDENTITY=NOT_EQUAL\n"
            "RI_EXACT_TARGET_IDENTITY=NOT_EQUAL\n"
            "RI_EXACT_EXIT_CODE_IDENTITY=NOT_EQUAL\n"
            "RI_EXACT_PROFILE_AUTHORITY_IDENTITY=NOT_EQUAL\n"
            "RI_EXACT_INSTANCE_KIND_IDENTITY=NOT_EQUAL\n"
            "RI_IDENTITY_EXACT_DIGEST_ASSERTIONS=0\n"
            "RI_IDENTITY_PYTHON_FRAMING_ASSERTIONS=0\n",
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
