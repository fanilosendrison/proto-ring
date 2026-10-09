from __future__ import annotations

import copy
import unittest

import yaml

import conformance_python_adapter_65_66 as adapter
from conformance_corpus_test_support import compare, decode_transport, load_corpus
from conformance_fixture_runner import materialize

RESPONSIBILITY_ID = "evidence-requirements.registry"
VECTOR_ID = "evidence-requirements.registry.foreign-catalog-repository"
SOURCE_VECTOR_ID = "evidence-requirements.registry.loaded-source-context-target"


class ArgumentMutatingFixture:
    def __init__(self, realized) -> None:
        self._realized = realized
        self.original_arguments = None
        self.mutated_arguments = None

    def run(self, callback):
        def wrapped(root, arguments):
            self.original_arguments = copy.deepcopy(arguments)
            mutated = copy.deepcopy(arguments)
            removed = mutated.pop("foreign_support_repository")
            if removed != "catalog":
                raise AssertionError("unexpected foreign catalog transport selection")
            self.mutated_arguments = copy.deepcopy(mutated)
            return callback(root, mutated)

        return self._realized.run(wrapped)


class TransportReplacingFixture:
    def __init__(self, realized, replacement) -> None:
        self._realized = realized
        self._replacement = replacement

    def run(self, callback):
        def wrapped(root, arguments):
            mutated = copy.deepcopy(arguments)
            mutated["foreign_support_repository"] = self._replacement
            return callback(root, mutated)

        return self._realized.run(wrapped)


class Issue102EvidenceRequirementsForeignCatalogTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, _, cases, _ = load_corpus()
        vectors = cases[RESPONSIBILITY_ID]["vectors"]
        selected = [vector for vector in vectors if vector["vector_id"] == VECTOR_ID]
        source = [
            vector for vector in vectors if vector["vector_id"] == SOURCE_VECTOR_ID
        ]
        if len(selected) != 1 or len(source) != 1:
            raise AssertionError("ISSUE102_FOREIGN_CATALOG_VECTOR_COUNT")
        cls.vector = selected[0]
        cls.source_vector = source[0]

    def _assert_target_is_otherwise_valid(self, root, arguments) -> None:
        self.assertTrue((root / "foreign").is_dir())
        write_step = next(
            step
            for step in self.vector["fixture"]["steps"]
            if step["op"] == "write_utf8" and step["path"] == arguments["path"]
        )
        registry = yaml.safe_load(write_step["text"].split("---", 2)[1])[
            "evidence_requirements"
        ]
        requirement = registry["requirements"][arguments["observe_requirement_id"]]
        target = requirement["target"]
        catalog = arguments["semantic_inputs"]["catalog"]
        interface = catalog["interfaces"].get(target["interface"])
        self.assertIsNotNone(interface, "target interface must exist")
        governed_object = interface.get(target["object"])
        self.assertIsNotNone(governed_object, "target object must exist")
        self.assertTrue(
            governed_object["responsibilities"],
            "target object responsibilities must be nonempty",
        )
        self.assertIn(
            requirement["responsibility"],
            governed_object["responsibilities"],
            "target object must participate in the requirement responsibility",
        )
        adapter.precheck_support(RESPONSIBILITY_ID, root, arguments)

    def test_foreign_catalog_repository_is_the_sole_invalid_distinction(self) -> None:
        with materialize(self.vector["fixture"]) as realized:
            realized.run(self._assert_target_is_otherwise_valid)
        print("FOREIGN_EVIDENCE_CATALOG_SUPPORT_PRECHECK=PASS")
        print("ISSUE102_TARGET_OTHERWISE_VALID=PASS")

        with materialize(self.vector["fixture"]) as realized:
            baseline = adapter.execute(RESPONSIBILITY_ID, realized)
        self.assertEqual(baseline["kind"], "controlled_rejection")
        self.assertEqual(
            decode_transport(baseline["value"]),
            {"category": "evidence-requirements.registry.rejected"},
        )
        self.assertEqual(
            compare(
                baseline,
                self.vector["expected_observation"],
                self.vector["comparison"],
            ),
            "MATCH",
        )
        print("ISSUE102_FOREIGN_CATALOG_CANONICAL_REJECTION=PASS")
        print("NEW_S4_GAP_VECTORS=1")
        print("NEW_S4_GAP_PYTHON_MATCH=1/1")

        with materialize(self.vector["fixture"]) as realized:
            mutated_fixture = ArgumentMutatingFixture(realized)
            mutant = adapter.execute(RESPONSIBILITY_ID, mutated_fixture)
        expected_arguments = copy.deepcopy(mutated_fixture.original_arguments)
        self.assertEqual(expected_arguments.pop("foreign_support_repository"), "catalog")
        self.assertEqual(mutated_fixture.mutated_arguments, expected_arguments)
        self.assertEqual(mutant["kind"], "result")
        self.assertEqual(
            compare(
                mutant,
                self.source_vector["expected_observation"],
                self.source_vector["comparison"],
            ),
            "MATCH",
            "the accepted mutant must preserve the cloned target declaration",
        )
        with self.assertRaisesRegex(AssertionError, "observation mismatch"):
            compare(
                mutant,
                self.vector["expected_observation"],
                self.vector["comparison"],
            )
        print("MUTANT=result")
        print("ISSUE102_MUTANT_CLASSES=1")
        print("ISSUE102_MUTANT_CASES=1")
        print("ISSUE102_MUTANT_KILLS=1")
        print("ISSUE102_MUTANT_SURVIVORS=0")
        print("ISSUE102_MUTATION_DISCRIMINATION=PASS")

    def test_invalid_foreign_catalog_transport_is_a_harness_failure(self) -> None:
        for replacement in ("bindings", {"type": "string", "value": "catalog"}):
            with self.subTest(replacement=replacement):
                with materialize(self.vector["fixture"]) as realized:
                    malformed = TransportReplacingFixture(realized, replacement)
                    with self.assertRaisesRegex(
                        ValueError,
                        "invalid foreign_support_repository transport value",
                    ):
                        adapter.execute(RESPONSIBILITY_ID, malformed)


if __name__ == "__main__":
    unittest.main()
