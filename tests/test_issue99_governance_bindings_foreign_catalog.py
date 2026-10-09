from __future__ import annotations

import copy
import unittest

import conformance_python_adapter_65_66 as adapter
from conformance_corpus_test_support import compare, decode_transport, load_corpus
from conformance_fixture_runner import materialize

RESPONSIBILITY_ID = "governance-bindings.registry"
VECTOR_ID = "governance-bindings.registry.foreign-catalog-repository"


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


class Issue99GovernanceBindingsForeignCatalogTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, _, cases, _ = load_corpus()
        vectors = cases[RESPONSIBILITY_ID]["vectors"]
        selected = [vector for vector in vectors if vector["vector_id"] == VECTOR_ID]
        if len(selected) != 1:
            raise AssertionError("ISSUE99_FOREIGN_CATALOG_VECTOR_COUNT")
        cls.vector = selected[0]

    def test_foreign_catalog_repository_is_the_sole_invalid_distinction(self) -> None:
        with materialize(self.vector["fixture"]) as realized:
            realized.run(
                lambda root, arguments: adapter.precheck_support(
                    RESPONSIBILITY_ID, root, arguments
                )
            )
        print("FOREIGN_GB_CATALOG_SUPPORT_PRECHECK=PASS")

        with materialize(self.vector["fixture"]) as realized:
            baseline = adapter.execute(RESPONSIBILITY_ID, realized)
        self.assertEqual(baseline["kind"], "controlled_rejection")
        self.assertEqual(
            decode_transport(baseline["value"]),
            {"category": "governance-bindings.registry.rejected"},
        )
        self.assertEqual(
            compare(
                baseline,
                self.vector["expected_observation"],
                self.vector["comparison"],
            ),
            "MATCH",
        )
        print("BASELINE=controlled_rejection governance-bindings.registry.rejected")
        print("ISSUE99_FOREIGN_CATALOG_CANONICAL_REJECTION=PASS")

        with materialize(self.vector["fixture"]) as realized:
            mutated_fixture = ArgumentMutatingFixture(realized)
            mutant = adapter.execute(RESPONSIBILITY_ID, mutated_fixture)
        self.assertIsNotNone(mutated_fixture.original_arguments)
        expected_mutated_arguments = copy.deepcopy(
            mutated_fixture.original_arguments
        )
        self.assertEqual(
            expected_mutated_arguments.pop("foreign_support_repository"),
            "catalog",
        )
        self.assertEqual(
            mutated_fixture.mutated_arguments,
            expected_mutated_arguments,
            "the mutant must remove only the foreign repository selector",
        )
        self.assertEqual(mutant["kind"], "result")
        with self.assertRaisesRegex(AssertionError, "observation mismatch"):
            compare(
                mutant,
                self.vector["expected_observation"],
                self.vector["comparison"],
            )
        print("MUTANT=result")
        print("ISSUE99_MUTANT_CLASSES=1")
        print("ISSUE99_MUTANT_CASES=1")
        print("ISSUE99_MUTANT_KILLS=1")
        print("ISSUE99_MUTANT_SURVIVORS=0")
        print("ISSUE99_MUTATION_DISCRIMINATION=PASS")

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
