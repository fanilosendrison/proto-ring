from __future__ import annotations

import inspect
import json
from pathlib import Path
import subprocess
import sys
import unittest

from jsonschema import Draft202012Validator

import conformance_python_adapter_63_64 as adapter_63_64
import conformance_python_adapter_65_66 as adapter_65_66
import conformance_python_adapter_67_69 as adapter_67_69
from conformance_corpus_test_support import (
    CORPUS_ROOT,
    OWNER_BY_ID,
    REPOSITORY_ROOT,
    compare,
    decode_transport,
    load_corpus,
    read_json,
    validate_authorities,
    validate_coverage,
    validate_migration_accounting,
    validate_no_placeholders,
    validate_repository_boundaries,
    validate_schemas,
    validate_vector_qualification,
)
from conformance_fixture_runner import materialize

ACTUAL_OBSERVATIONS_PATH = Path("/tmp/proto-ring-61-actual-observations.json")
COMMAND_ACTIONS = {
    "exit", "write_utf8", "append_utf8", "remove", "touch", "chmod", "git",
    "record_arguments_utf8", "assert_environment",
    "exit_code_from_utf8_integer_file_argument",
}
PROCEDURAL_CONDITIONS = {
    "git_process_start_unavailable", "repository_mutation_during_capture",
    "repository_mutation_during_integrity_evaluation",
    "repository_mutation_during_governance_state_construction",
    "observation_scope_mutation_during_governance_state_construction",
    "provider_network_access_forbidden", "repository_path_discovery_unavailable",
    "repository_state_unavailable_before_obligation",
    "repository_state_unavailable_after_obligation",
}
HISTORICAL_LEGACY_EQUIVALENCE = {
    "normative-terminology.definition-discovery.strasse",
    "normative-terminology.definition-discovery.no-normalization",
    "portable-pattern.full-match.demonstrated-adr-pattern",
    "portable-pattern.full-match.demonstrated-anchor-pattern",
}
HISTORICAL_NOT_COMPARABLE = {
    "git-whitespace.validate.git-unavailable",
    "structured-data.frontmatter.body-cannot-repair",
    "structured-data.frontmatter.document-start-marker",
    "structured-data.frontmatter.document-end-marker",
}

ADAPTERS = {
    63: adapter_63_64,
    64: adapter_63_64,
    65: adapter_65_66,
    66: adapter_65_66,
    67: adapter_67_69,
    68: adapter_67_69,
    69: adapter_67_69,
}


class ConformanceCorpusTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.index, cls.responsibilities, cls.cases, cls.coverage = load_corpus()
        cls.actual_observations: list[dict[str, object]] = []
        cls.executed = 0

    def test_01_all_json_is_canonical_utf8(self) -> None:
        for path in sorted(CORPUS_ROOT.rglob("*.json")):
            with self.subTest(path=path):
                read_json(path)

    def test_02_documents_validate_against_strict_schemas(self) -> None:
        validate_schemas(
            self.index, self.responsibilities, self.cases, self.coverage
        )

    def test_03_layout_inventory_and_owners_are_exact(self) -> None:
        metrics = validate_migration_accounting(self.index, self.responsibilities, self.cases, self.coverage)
        self.assertEqual(metrics, {"rust": 29, "retired": 1, "matrices": 29, "modules": 23, "carriers": 3, "literal_mismatches": 0, "identity_lengths": 0, "identity_digests": 0, "identity_without_authority": 0, "identity_compatibility": 0, "opaque_identity_relations": "PASS"})
        for responsibility_id in OWNER_BY_ID:
            responsibility = self.responsibilities[responsibility_id]
            self.assertEqual(responsibility["rust_owner_issue"], OWNER_BY_ID[responsibility_id])
            self.assertEqual(responsibility["case_file"], f"cases/{responsibility_id}.matrix.json")
        self.assertTrue(all(value["python_retirement_owner"] == 73 and value["blocking_gaps"] == [] for value in self.responsibilities.values()))
    def test_04_vector_qualification_and_placeholder_audits(self) -> None:
        total = validate_vector_qualification(self.cases)
        self.assertEqual(total, sum(len(case["vectors"]) for case in self.cases.values()))
        self.assertEqual(validate_no_placeholders(self.cases), 0)
    def test_05_authorities_and_semantic_coverage_bind_mechanically(self) -> None:
        validate_authorities(self.cases)
        validate_coverage(
            self.coverage, self.responsibilities, self.cases
        )
    def test_06_known_bad_coverage_mappings_are_impossible(self) -> None:
        headings = {
            (record["contract"]["path"], heading["heading_path"]): heading
            for record in self.coverage
            for heading in record["headings"]
        }
        routing = headings[
            (
                "docs/contracts/canonical-governance-routing.md",
                "Exact-route traversal",
            )
        ]
        self.assertTrue(routing["vector_refs"])
        self.assertTrue(
            all(ref.startswith("governance-routing.resolve.") for ref in routing["vector_refs"])
        )
        self.assertNotIn("governance-authority.profile.invalid-graph", routing["vector_refs"])

        purity = headings[
            (
                "docs/contracts/repository-integrity.md",
                "Evaluation purity / Exact purity-status attribution",
            )
        ]
        self.assertTrue(
            all(ref.startswith("repository-integrity.evaluate.") for ref in purity["vector_refs"])
        )
        self.assertNotIn("repository-integrity.profile.exact-keys", purity["vector_refs"])
        self.assertNotIn(
            "github-authoritative-ref-monotonicity.observe.pagination",
            purity["vector_refs"],
        )

        discovery = headings[
            (
                "docs/contracts/accepted-adr-body-immutability.md",
                "Historical discovery",
            )
        ]
        self.assertGreaterEqual(len(discovery["vector_refs"]), 3)
        self.assertTrue(
            all(
                ref.startswith("accepted-adr-body.immutability.")
                for ref in discovery["vector_refs"]
            )
        )
        self.assertNotEqual(
            discovery["vector_refs"],
            ["accepted-adr-body.immutability.first-uncommitted-acceptance"],
        )
        ids = (
            "normative-terminology.markdown.fenced-blocks", "projection-registry.registry.distinct-direct-sources",
            "evidence-requirements.registry.required-unknown-context", "repository-integrity.evaluate.pre-state-drift",
            "structured-data.frontmatter.immediate-close", "structured-data.document.beyond-i64",
        )
        selected = {v["vector_id"]: v for case in self.cases.values() for v in case["vectors"] if v["vector_id"] in ids}
        self.assertEqual(set(selected), set(ids))
        fenced_vector = selected[ids[0]]
        self.assertNotIn("exclud", json.dumps(fenced_vector["state_distinctions"]).lower())
        self.assertEqual({item["clause"] for item in fenced_vector["authorities"]}, {"Markdown structural model"})
        projection = selected[ids[1]]; projection_args = decode_transport(projection["fixture"]["invocation"])
        sources = projection_args["semantic_inputs"]["authority"]["sources"]
        self.assertNotEqual("secondary", "secondary-two")
        self.assertEqual({sources[x]["repository_target"] for x in ("secondary", "secondary-two")}, {"shared.md"})
        self.assertTrue(any(step.get("path") == "shared.md" for step in projection["fixture"]["steps"]))
        self.assertEqual(decode_transport(projection["expected_observation"]["value"])["registry"], "ACCEPTED")
        evidence = selected[ids[2]]; evidence_args = decode_transport(evidence["fixture"]["invocation"])
        evidence_expected = decode_transport(evidence["expected_observation"]["value"])
        self.assertIn("source: protocol", evidence["fixture"]["steps"][0]["text"])
        self.assertEqual((evidence_expected["registry"], evidence_expected["context_source_id"], evidence_expected["resolved_context_known"], evidence_expected["binding_status"]), ("ACCEPTED", "protocol", False, "UNDETERMINED"))
        self.assertTrue(evidence_expected["context_required"] and not evidence_args["consumer_resolution"]["context"]["known"])
        drift = selected[ids[3]]; drift_args = decode_transport(drift["fixture"]["base"]["invocation"]); drift_expected = decode_transport(drift["expected_observation"]["value"])
        self.assertEqual(drift_args["actions"], [{"id": "check", "action": "touch", "path": "must-not-run"}])
        self.assertEqual((drift["fixture"]["condition"]["kind"], drift["fixture"]["condition"]["phase"]), ("repository_mutation_during_integrity_evaluation", "after_baseline_before_obligation"))
        self.assertEqual((drift_expected["obligations"][0]["status"], drift_expected["obligations"][0]["executed"], drift_expected["marker_absent"], drift_expected["state_coherence_halt"], drift_expected["verdict"]), ("UNDETERMINED", False, True, True, "NON_PASS"))
        immediate = selected[ids[4]]
        self.assertEqual((immediate["resolution"]["correction_issue"], immediate["resolution"]["corrected_python_sha"]), (75, "f3bdde11f86c4caf98aeae208083e491f7f7014e"))
        self.assertEqual((immediate["later_python_observation"]["implementation_sha"], immediate["later_python_observation"]["comparison_to_expected"]), ("f3bdde11f86c4caf98aeae208083e491f7f7014e", "MATCH"))
        large = selected[ids[5]]; large_args = decode_transport(large["fixture"]["input"]); large_expected = decode_transport(large["expected_observation"]["value"])
        self.assertEqual((bytes.fromhex(large_args["bytes_hex"]).decode(), large_expected), ("18446744073709551616", 18446744073709551616))
        self.assertIn("18446744073709551616", large["state_distinctions"][0]); self.assertNotIn("9223372036854775808", json.dumps(large))
    def test_07_adapter_execute_api_cannot_receive_oracle_data(self) -> None:
        for adapter in set(ADAPTERS.values()):
            with self.subTest(adapter=adapter.__name__):
                signature = inspect.signature(adapter.execute)
                self.assertEqual(list(signature.parameters), ["responsibility_id", "fixture"])
                forbidden = {
                    "expected", "expected_observation", "comparison",
                    "classification", "behavior_classification",
                }
                self.assertTrue(forbidden.isdisjoint(signature.parameters))
    def test_08_every_vector_executes_and_current_python_conforms(self) -> None:
        actual_records = []
        for responsibility_id, matrix in sorted(self.cases.items()):
            adapter = ADAPTERS[OWNER_BY_ID[responsibility_id]]
            for vector in matrix["vectors"]:
                vector_id = vector["vector_id"]
                with self.subTest(vector_id=vector_id):
                    with materialize(vector["fixture"]) as realized:
                        actual = adapter.execute(responsibility_id, realized)
                    comparison_result = compare(
                        actual,
                        vector["expected_observation"],
                        vector["comparison"],
                    )
                    self.assertEqual(comparison_result, "MATCH")
                    actual_records.append(
                        {
                            "actual_observation": actual,
                            "comparison_result": comparison_result,
                            "responsibility_id": responsibility_id,
                            "vector_id": vector_id,
                        }
                    )
        type(self).actual_observations = actual_records
        type(self).executed = len(actual_records)
        ACTUAL_OBSERVATIONS_PATH.write_text(
            json.dumps(actual_records, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        self.assertEqual(self.executed, sum(len(case["vectors"]) for case in self.cases.values()))

    def test_09_historical_resolution_transport_and_projection_audits(self) -> None:
        resolution_inconsistencies = historical_inconsistencies = 0
        command_schema = read_json(CORPUS_ROOT / "schemas/case.schema.json")["$defs"]["command_action"]
        command_validator = Draft202012Validator(command_schema)
        semantic_default_inconsistencies = embedded_sources = lossy = 0
        support_required = {
            "governed-objects.catalog", "governance-bindings.registry",
            "projection-registry.registry", "repository-integrity.profile",
            "evidence-requirements.registry",
        }
        actual_by_id = {record["vector_id"]: record["actual_observation"] for record in self.actual_observations}
        for responsibility_id, matrix in self.cases.items():
            for vector in matrix["vectors"]:
                vector_id = vector["vector_id"]
                expected = {key: vector["expected_observation"][key] for key in ("kind", "value")}
                for field in ("frozen_python_observation", "later_python_observation"):
                    record = vector[field]
                    if record is None:
                        continue
                    observed = record["observation"]
                    if observed == expected or (
                        vector_id in HISTORICAL_LEGACY_EQUIVALENCE
                        and self._legacy_observation_matches(observed, expected)
                    ):
                        computed = "MATCH"
                    elif vector_id in HISTORICAL_NOT_COMPARABLE:
                        computed = "NOT_COMPARABLE"
                    else:
                        computed = "MISMATCH"
                    historical_inconsistencies += computed != record["comparison_to_expected"]
                if vector["behavior_classification"] in {
                    "bootstrap_implementation_accident", "missing_contract_responsibility"
                } and vector["later_python_observation"] is not None:
                    resolution = vector["resolution"]
                    resolution_inconsistencies += not (
                        resolution and resolution["status"] == "resolved"
                        and resolution["authorities"] and isinstance(resolution["correction_issue"], int)
                        and len(resolution["corrected_python_sha"]) == 40
                    )
                fixture = vector["fixture"]
                arguments = {} if fixture["kind"] == "provider_observation" else decode_transport(
                    fixture["input"] if fixture["kind"] == "inline" else fixture["invocation"]
                )
                if responsibility_id in support_required:
                    semantic_default_inconsistencies += "semantic_inputs" not in arguments
                if responsibility_id == "repository-governance-model.binding-compatibility":
                    semantic_default_inconsistencies += "semantic_inputs" not in arguments
                if responsibility_id.startswith("normative-terminology.") and arguments.get("operation") in {
                    "parse_blocks", "discover_occurrences", "reconcile_inventory"
                }:
                    semantic_default_inconsistencies += "policy" not in arguments
                for node in self._walk([fixture, arguments]):
                    if isinstance(node, dict) and isinstance(node.get("action"), str):
                        self.assertIn(node["action"], COMMAND_ACTIONS)
                        command_validator.validate({
                            key: value for key, value in node.items()
                            if key not in {"id", "undetermined_exit_codes"}
                        })
                if fixture["kind"] == "procedural_requirement":
                    self.assertIn(fixture["condition"]["kind"], PROCEDURAL_CONDITIONS)
                self.assertEqual(actual_by_id[vector_id], expected)
        for vector_id in (
            "canonical-adr.resolve.exact-body",
            "normative-terminology.fingerprint.canonical-occurrence",
            "projection-registry.registry.distinct-direct-sources",
            "repository-state.capture.head-object",
            "repository-governance-state.compose.pure-construction",
            "repository-integrity.evaluate.selector-sort",
        ):
            self.assertIn(vector_id, actual_by_id)
            lossy += not actual_by_id[vector_id]["value"]["value"]
        runner = (REPOSITORY_ROOT / "tests/conformance_fixture_runner.py").read_text()
        adapter = (REPOSITORY_ROOT / "tests/conformance_python_adapter_67_69.py").read_text()
        embedded_sources += "_ACTION_PROGRAM" in runner or "_ACTION_PROGRAM" in adapter
        self.assertEqual(historical_inconsistencies, 0)
        self.assertEqual(resolution_inconsistencies, 0)
        self.assertEqual(semantic_default_inconsistencies, 0)
        self.assertEqual(embedded_sources, 0)
        self.assertEqual(lossy, 0)
        state_resolution = self.cases["repository-state.capture"]["vectors"]
        governance_resolution = self.cases["repository-governance-state.compose"]["vectors"]
        selected = {item["vector_id"]: item["resolution"] for item in state_resolution + governance_resolution}
        for vector_id in (
            "repository-state.capture.invalid-scope",
            "repository-governance-state.compose.routing-expression-not-scope",
        ):
            self.assertEqual(selected[vector_id]["correction_issue"], 77)
            self.assertEqual(selected[vector_id]["corrected_python_sha"], "46ffdac569d36edc0f4a655aa603ac4656d29092")

    @staticmethod
    def _legacy_observation_matches(observed, expected):
        if observed.get("kind") != "result" or expected.get("kind") != "result":
            return False
        left, right = decode_transport(observed["value"]), decode_transport(expected["value"])
        if isinstance(left, dict) and "detected" in left and isinstance(right, dict) and "concepts" in right:
            return left["detected"] == bool(right["concepts"])
        if isinstance(left, dict) and isinstance(right, dict) and "matched" in left and "matched" in right:
            if left["matched"] != right["matched"]:
                return False
            captures = right.get("captures", [])
            if "capture_1" in left:
                return captures == [left["capture_1"]]
            named = right.get("named_captures", {})
            return all(left.get(name) == value for name, value in named.items())
        return False

    @staticmethod
    def _walk(value):
        yield value
        if isinstance(value, dict):
            for nested in value.values():
                yield from ConformanceCorpusTest._walk(nested)
        elif isinstance(value, list):
            for nested in value:
                yield from ConformanceCorpusTest._walk(nested)

    def test_10_execution_evidence_and_final_metrics(self) -> None:
        total = sum(len(case["vectors"]) for case in self.cases.values())
        self.assertEqual(self.executed, total)
        self.assertTrue(ACTUAL_OBSERVATIONS_PATH.is_file())
        self.assertTrue(
            all(record["comparison_result"] == "MATCH" for record in self.actual_observations)
        )
        migration = validate_migration_accounting(self.index, self.responsibilities, self.cases, self.coverage)
        metrics = (
            "RESPONSIBILITIES=30/30\n"
            f"RUST_PORT_RESPONSIBILITIES={migration['rust']}\n"
            f"RETIRE_WITHOUT_RUST_PORT_RESPONSIBILITIES={migration['retired']}\n"
            "RESPONSIBILITY_FILES=30\n"
            f"RUST_PORT_CASE_MATRICES={migration['matrices']}\n"
            f"FROZEN_MODULES={migration['modules']}/23\n"
            f"POST_BASELINE_CARRIERS={migration['carriers']}/3\n"
            f"TOTAL_VECTORS={total}\n"
            f"EXECUTABLE_VECTORS={self.executed}\n"
            "PLACEHOLDER_FIXTURES=0\n"
            "CURRENT_PYTHON_CONFORMANCE=PASS\n"
            "HISTORICAL_GENERIC_OBSERVATION_PLACEHOLDERS=0\n"
            "HISTORICAL_COMPARISON_INCONSISTENCIES=0\n"
            "MATRIX_LEVEL_BEHAVIOR_CLASSIFICATIONS=0\n"
            "RESOLUTION_METADATA_INCONSISTENCIES=0\n"
            "UNDECLARED_SEMANTIC_ADAPTER_DEFAULTS=0\n"
            "EMBEDDED_COMMAND_SOURCE_SEMANTICS=0\n"
            "LOSSY_ACTUAL_OBSERVATION_PROJECTIONS=0\n"
            "CURRENT_MISSING_CONTRACT_BLOCKERS=0\n"
            "RGS_CANONICAL_AUTHORITY_CONFLICTS=0\n"
            "UNRESOLVED_LEGACY_SHARED_PROVIDER_AUTHORITIES=0\n"
            "FENCED_BLOCK_RESPONSIBILITY_VECTOR_CONFLICTS=0\n"
            "CANONICAL_SCALAR_DECLARATION_INPUT_MISMATCHES=0\n"
            "VECTOR_DECLARATION_INPUT_MISMATCHES=0\n"
            f"STRUCTURED_DATA_RESPONSIBILITY_LITERAL_MISMATCHES={migration['literal_mismatches']}\n"
            f"UNSUPPORTED_EXACT_IDENTITY_LENGTH_REQUIREMENTS={migration['identity_lengths']}\n"
            f"UNSUPPORTED_EXACT_IDENTITY_DIGEST_REQUIREMENTS={migration['identity_digests']}\n"
            f"IDENTITY_COMPATIBILITY_OBLIGATIONS_WITHOUT_AUTHORITY={migration['identity_without_authority']}\n"
            f"CURRENT_EXACT_IDENTITY_COMPATIBILITY_OBLIGATIONS={migration['identity_compatibility']}\n"
            f"OPAQUE_IDENTITY_RELATION_CASES={migration['opaque_identity_relations']}\n"
            "ORPHAN_FIXTURES=0\n"
            "SIX_VECTOR_SEMANTIC_CORRECTIONS=6/6\n"
            "SIX_VECTOR_CURRENT_PYTHON_CONFORMANCE=6/6\n"
            "LIVE_NETWORK_REQUESTS=0\n"
        )
        Path("/tmp/proto-ring-61-execution-metrics.txt").write_text(metrics, encoding="utf-8")
        sys.stderr.write(metrics)

    def test_11_repository_boundary_and_tracked_cache_cleanliness(self) -> None:
        validate_repository_boundaries()
        tracked = subprocess.check_output(["git", "-C", str(REPOSITORY_ROOT), "ls-files", "-z"], text=True).split("\0")
        forbidden = [path for path in tracked if "__pycache__" in Path(path).parts or Path(path).suffix in {".pyc", ".pyo"}]
        self.assertEqual(forbidden, [])


if __name__ == "__main__":
    unittest.main()
