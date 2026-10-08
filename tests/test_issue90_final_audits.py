from __future__ import annotations

import json
from pathlib import Path
import unittest

import yaml

from conformance_corpus_test_support import decode_transport, load_corpus


class Issue90FinalAuditTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, cls.responsibilities, cls.cases, cls.coverage = load_corpus()

    @staticmethod
    def _arguments(vector: dict[str, object]) -> dict[str, object]:
        fixture = vector["fixture"]
        return decode_transport(fixture.get("invocation", fixture.get("input")))

    @staticmethod
    def _carrier(vector: dict[str, object]) -> dict[str, object]:
        arguments = Issue90FinalAuditTest._arguments(vector)
        step = next(
            step
            for step in vector["fixture"]["steps"]
            if step.get("op") == "write_utf8"
            and step.get("path") == arguments["path"]
        )
        return yaml.safe_load(step["text"].split("---", 2)[1])

    def test_projection_support_has_one_variant_per_family(self) -> None:
        vectors = self.cases["projection-registry.registry"]["vectors"]
        supports = {
            family: {
                json.dumps(
                    self._arguments(vector)["semantic_inputs"][family],
                    sort_keys=True,
                    separators=(",", ":"),
                )
                for vector in vectors
            }
            for family in ("integrity_profile", "bindings")
        }
        self.assertEqual(len(supports["integrity_profile"]), 1)
        self.assertEqual(len(supports["bindings"]), 1)

    def test_integrity_support_matches_owning_loader_witness(self) -> None:
        projection = self.cases["projection-registry.registry"]["vectors"][0]
        common = dict(self._arguments(projection)["semantic_inputs"]["integrity_profile"])
        common.pop("identity")
        owner = next(
            vector
            for vector in self.cases["repository-integrity.profile"]["vectors"]
            if vector["vector_id"]
            == "repository-integrity.profile.projection-support-witness"
        )
        profile = self._carrier(owner)["repository_integrity"]
        actual = {
            "authority": profile["authority"],
            "carrier": self._arguments(owner)["path"],
            "continue_after_non_satisfied": profile["continue_after_non_satisfied"],
            "environments": profile["environments"],
            "order": profile["order"],
            "validations": [
                {
                    "arguments": profile["validations"][item]["command"]["arguments"],
                    "environment": profile["validations"][item]["command"]["environment"],
                    "id": item,
                    "prerequisites": profile["validations"][item]["prerequisites"],
                    "responsibility": profile["validations"][item]["responsibility"],
                    "undetermined_exit_codes": profile["validations"][item]["command"]["undetermined_exit_codes"],
                }
                for item in profile["order"]
            ],
        }
        self.assertEqual(actual, common)
        expected = decode_transport(owner["expected_observation"]["value"])
        self.assertTrue(expected["identity_present"])
        self.assertEqual(set(expected["loaded_validations"]), {"1", "current", "custody"})

    def test_binding_support_matches_owning_loader_witness(self) -> None:
        projection = self.cases["projection-registry.registry"]["vectors"][0]
        common = self._arguments(projection)["semantic_inputs"]["bindings"]
        owner = next(
            vector
            for vector in self.cases["governance-bindings.registry"]["vectors"]
            if vector["vector_id"]
            == "governance-bindings.registry.projection-support-witness"
        )
        registry = self._carrier(owner)["governance_bindings"]
        bindings = []
        for binding_id, binding in sorted(registry["bindings"].items()):
            item = {
                "id": binding_id,
                "kind": binding["kind"],
                "scope_kind": binding["scope"]["kind"],
                "repository": binding["identity"]["repository"],
                "commit": binding["identity"]["commit"],
                "responsibility": binding["authority"]["responsibility"],
                "source": binding["authority"]["source"],
            }
            if "capability" in binding["scope"]:
                item["capability"] = binding["scope"]["capability"]
            if "path" in binding["identity"]:
                item["path"] = binding["identity"]["path"]
            bindings.append(item)
        actual = {
            "carrier": self._arguments(owner)["path"],
            "source": registry["source"],
            "bindings": bindings,
        }
        self.assertEqual(actual, common)
        expected = decode_transport(owner["expected_observation"]["value"])
        self.assertEqual(expected["loaded_binding"], {
            "id": "provider",
            "kind": "executable_provider",
            "scope": {"kind": "logical_provider", "capability": None, "interface": None, "object": None},
            "identity": {"repository": "provider/repository", "commit": "a" * 40, "path": None},
            "authority": {"responsibility": "fact", "source": "canonical"},
        })

    def test_projection_adapter_does_not_reload_support(self) -> None:
        source = (Path(__file__).parent / "conformance_python_adapter_65_66.py").read_text()
        function = source.split("def _projection_result", 1)[1].split("def precheck_support", 1)[0]
        self.assertNotIn("repository_integrity.load", function)
        self.assertNotIn("governance_bindings.load", function)
        self.assertIn("support_roots = _projection_support_roots", function)
        self.assertIn("integrity_profile(\n        support_roots[\"integrity_profile\"]", function)
        self.assertIn("binding_registry(support_roots[\"bindings\"]", function)

    def test_duplicate_nonmapping_claims_are_collapsed(self) -> None:
        integrity_ids = {
            vector["vector_id"]
            for vector in self.cases["repository-integrity.profile"]["vectors"]
            if "instances declaration must be a mapping" in vector["state_distinctions"]
        }
        context_ids = {
            vector["vector_id"]
            for vector in self.cases["evidence-requirements.registry"]["vectors"]
            if "context declaration must be a mapping" in vector["state_distinctions"]
        }
        self.assertEqual(
            integrity_ids, {"repository-integrity.profile.instances-non-mapping"}
        )
        self.assertEqual(
            context_ids, {"evidence-requirements.registry.context-non-mapping"}
        )

    def test_rgm_dependency_has_exact_two_witnesses(self) -> None:
        ids = {
            vector["vector_id"]
            for vector in self.cases["repository-governance-model.load"]["vectors"]
            if "projection-integrity" in vector["vector_id"]
        }
        self.assertEqual(ids, {
            "repository-governance-model.load.projection-integrity-requires-repository-integrity",
            "repository-governance-model.load.projection-integrity-with-repository-integrity",
        })


if __name__ == "__main__":
    unittest.main()
