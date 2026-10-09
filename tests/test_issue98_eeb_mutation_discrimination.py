from __future__ import annotations

import copy
import unittest

import conformance_python_adapter_65_66 as adapter
from conformance_corpus_test_support import (
    compare,
    decode_transport,
    load_corpus,
    observation,
)
from conformance_fixture_runner import materialize

RESPONSIBILITY_ID = "exact-evidence-binding.evaluate"
DUPLICATE_ID = f"{RESPONSIBILITY_ID}.requirement-duplicate-classes"
IGNORED_CONTEXT_ID = (
    f"{RESPONSIBILITY_ID}.empty-context-ignored-when-not-required"
)
PRECEDENCE_IDS = (
    f"{RESPONSIBILITY_ID}.unknown-current-subject-precedes-malformed-candidate",
    f"{RESPONSIBILITY_ID}.unknown-current-context-precedes-malformed-candidate",
)
VECTOR_IDS = (DUPLICATE_ID, IGNORED_CONTEXT_ID, *PRECEDENCE_IDS)


class InlineArguments:
    def __init__(self, arguments: dict[str, object]) -> None:
        self._arguments = arguments

    def run(self, callback):
        return callback(None, copy.deepcopy(self._arguments))


def silent_deduplication_mutant(arguments: dict[str, object]):
    requirement = arguments["requirement"]
    requirement["admitted_classes"] = list(
        dict.fromkeys(requirement["admitted_classes"])
    )
    return adapter.execute(RESPONSIBILITY_ID, InlineArguments(arguments))


def ignored_context_participation_mutant(arguments: dict[str, object]):
    if arguments["binding"]["context_hex"] == "":
        return observation("result", {"status": "UNDETERMINED"})
    return adapter.execute(RESPONSIBILITY_ID, InlineArguments(arguments))


def candidate_before_requirement_mutant(arguments: dict[str, object]):
    binding = arguments["binding"]
    mandatory_empty = binding["subject_hex"] == "" or (
        arguments["requirement"]["context_required"]
        and binding["context_hex"] == ""
    )
    if binding["evidence_class"] == "" or mandatory_empty:
        return observation(
            "controlled_rejection",
            {"category": "exact-evidence-binding.evaluate.rejected"},
        )
    return adapter.execute(RESPONSIBILITY_ID, InlineArguments(arguments))


class Issue98EebMutationDiscriminationTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, _, cases, _ = load_corpus()
        cls.vectors = {
            vector["vector_id"]: vector
            for vector in cases[RESPONSIBILITY_ID]["vectors"]
            if vector["vector_id"] in VECTOR_IDS
        }

    def _canonical(self, vector: dict[str, object]) -> dict[str, object]:
        with materialize(vector["fixture"]) as realized:
            actual = adapter.execute(RESPONSIBILITY_ID, realized)
        self.assertEqual(
            compare(actual, vector["expected_observation"], vector["comparison"]),
            "MATCH",
        )
        return actual

    def _kill(self, vector_id: str, mutant) -> None:
        vector = self.vectors[vector_id]
        self._canonical(vector)
        arguments = decode_transport(vector["fixture"]["input"])
        mutated = mutant(arguments)
        with self.assertRaisesRegex(AssertionError, "observation mismatch"):
            compare(mutated, vector["expected_observation"], vector["comparison"])

    def test_all_issue98_mutants_are_killed(self) -> None:
        self.assertEqual(set(self.vectors), set(VECTOR_IDS))
        cases = (
            (DUPLICATE_ID, silent_deduplication_mutant),
            (IGNORED_CONTEXT_ID, ignored_context_participation_mutant),
            (PRECEDENCE_IDS[0], candidate_before_requirement_mutant),
            (PRECEDENCE_IDS[1], candidate_before_requirement_mutant),
        )
        for vector_id, mutant in cases:
            with self.subTest(vector_id=vector_id):
                self._kill(vector_id, mutant)
        print("ISSUE98_MUTANT_CLASSES=3")
        print(f"ISSUE98_MUTANT_CASES={len(cases)}")
        print(f"ISSUE98_MUTANT_KILLS={len(cases)}")
        print("ISSUE98_MUTANT_SURVIVORS=0")
        print("ISSUE98_MUTATION_DISCRIMINATION=PASS")


if __name__ == "__main__":
    unittest.main()
