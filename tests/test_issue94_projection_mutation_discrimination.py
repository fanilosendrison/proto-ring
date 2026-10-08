from __future__ import annotations

import copy
import unittest

import conformance_python_adapter_65_66 as adapter
import conformance_python_adapter_66_support as support
from conformance_corpus_test_support import compare, decode_transport, load_corpus
from conformance_fixture_runner import materialize
from proto_ring import projection_registry

RESPONSIBILITY_ID = "projection-registry.registry"
FOREIGN_VECTOR_IDS = (
    "projection-registry.registry.foreign-integrity-repository",
    "projection-registry.registry.foreign-catalog-repository",
    "projection-registry.registry.foreign-bindings-repository",
)
TARGET_VECTOR_ID = (
    "projection-registry.registry.target-without-responsibility-participation"
)
MUTATION_VECTOR_IDS = (*FOREIGN_VECTOR_IDS, TARGET_VECTOR_ID)


class ArgumentMutatingFixture:
    def __init__(self, realized, mutator) -> None:
        self._realized = realized
        self._mutator = mutator
        self.mutated_arguments = None

    def run(self, callback):
        def wrapped(root, arguments):
            mutated = self._mutator(copy.deepcopy(arguments))
            self.mutated_arguments = copy.deepcopy(mutated)
            return callback(root, mutated)

        return self._realized.run(wrapped)


def ignore_foreign_support_repository(arguments):
    arguments.pop("foreign_support_repository", None)
    return arguments


def _reconstruct_projection_support(root, arguments):
    semantic_inputs = arguments["semantic_inputs"]
    support_roots = adapter._projection_support_roots(root, arguments)
    authority = adapter._authority(root, semantic_inputs["authority"])
    integrity = support.integrity_profile(
        support_roots["integrity_profile"],
        semantic_inputs["integrity_profile"],
    )
    catalog = (
        adapter._catalog(support_roots["catalog"], semantic_inputs["catalog"])
        if arguments.get("with_catalog")
        else None
    )
    bindings = (
        support.binding_registry(
            support_roots["bindings"], semantic_inputs["bindings"]
        )
        if arguments.get("with_bindings")
        else None
    )
    route = adapter._route(root, arguments["path"], "projection_registry")
    return authority, integrity, catalog, bindings, route


def _project_success(loaded, arguments, repository_unchanged):
    value = {
        "model_version": loaded.model_version,
        "projections": sorted(loaded.projections),
        "modes": {
            key: item.mode.value
            for key, item in sorted(loaded.projections.items())
        },
    }
    if arguments.get("qualified_observation"):
        value["registry"] = "ACCEPTED"
        value["loaded"] = {
            key: {
                "canonical_source": item.canonical_source_id,
                "secondary_source": item.secondary_source_id,
                "mode": item.mode.value,
            }
            for key, item in sorted(loaded.projections.items())
        }
    if arguments.get("observe_registry_authority"):
        value["registry_authority"] = {
            "responsibility": loaded.authority.responsibility_id,
            "source": loaded.authority.source_id,
        }
    observed = arguments.get("observe_projection_id")
    if observed is not None:
        value["loaded_projection"] = support.projection_observation(
            loaded.projections[observed]
        )
    if repository_unchanged is not None:
        value["repository_unchanged"] = repository_unchanged
    return adapter._success(value)


def participation_required_mutant(root, arguments):
    before = (
        support.repository_snapshot(root)
        if arguments.get("observe_read_only")
        else None
    )
    authority, integrity, catalog, bindings, route = _reconstruct_projection_support(
        root, arguments
    )
    try:
        loaded = projection_registry.load(
            root,
            route,
            authority,
            integrity,
            catalog,
            bindings,
        )
    except projection_registry.ProjectionRegistryError:
        return adapter._rejection(RESPONSIBILITY_ID)
    for projection in loaded.projections.values():
        if projection.target is None:
            continue
        target_object = catalog.interfaces[
            projection.target.interface_id
        ].objects[projection.target.object_id]
        if projection.responsibility_id not in target_object.responsibilities:
            return adapter._rejection(RESPONSIBILITY_ID)
    unchanged = None if before is None else support.repository_snapshot(root) == before
    return _project_success(loaded, arguments, unchanged)


def inspect_target_nonparticipation(root, arguments):
    authority, integrity, catalog, bindings, route = _reconstruct_projection_support(
        root, arguments
    )
    loaded = projection_registry.load(
        root,
        route,
        authority,
        integrity,
        catalog,
        bindings,
    )
    projection = loaded.projections[arguments["observe_projection_id"]]
    target = projection.target
    interface = None if target is None else catalog.interfaces.get(target.interface_id)
    target_object = (
        None
        if interface is None or target.object_id not in interface.objects
        else interface.objects[target.object_id]
    )
    return {
        "target_non_null": target is not None,
        "interface_exists": interface is not None,
        "object_exists": target_object is not None,
        "responsibilities_nonempty": bool(
            target_object is not None and target_object.responsibilities
        ),
        "projection_responsibility_absent": bool(
            target_object is not None
            and projection.responsibility_id not in target_object.responsibilities
        ),
    }


class Issue94ProjectionMutationDiscriminationTest(unittest.TestCase):
    def _vector(self, vector_id):
        _, _, cases, _ = load_corpus()
        vectors = cases[RESPONSIBILITY_ID]["vectors"]
        selected = [
            vector for vector in vectors
            if vector["vector_id"] in MUTATION_VECTOR_IDS
        ]
        self.assertEqual(len(selected), 4, "ISSUE94_MUTATION_VECTOR_COUNT")
        self.assertEqual(
            {vector["vector_id"] for vector in selected},
            set(MUTATION_VECTOR_IDS),
            "ISSUE94_MUTATION_VECTOR_INVENTORY",
        )
        return next(vector for vector in selected if vector["vector_id"] == vector_id)

    def _baseline(self, vector):
        with materialize(vector["fixture"]) as realized:
            actual = adapter.execute(RESPONSIBILITY_ID, realized)
        self.assertEqual(
            compare(actual, vector["expected_observation"], vector["comparison"]),
            "MATCH",
            f"ISSUE94_MUTATION_BASELINE_MISMATCH: {vector['vector_id']}",
        )
        return actual

    def _assert_foreign_mutant_killed(self, vector_id, metric):
        vector = self._vector(vector_id)
        baseline = self._baseline(vector)
        self.assertEqual(baseline["kind"], "controlled_rejection")
        with materialize(vector["fixture"]) as realized:
            mutated = ArgumentMutatingFixture(
                realized, ignore_foreign_support_repository
            )
            mutant = adapter.execute(RESPONSIBILITY_ID, mutated)
        detail = (
            f"baseline={baseline!r} mutant={mutant!r} "
            f"invocation={mutated.mutated_arguments!r}"
        )
        self.assertEqual(mutant["kind"], "result", detail)
        with self.assertRaisesRegex(AssertionError, "observation mismatch", msg=detail):
            compare(mutant, vector["expected_observation"], vector["comparison"])
        print(f"{metric}=KILLED")

    def test_foreign_integrity_repository_kills_ignore_repository_mutant(self):
        self._assert_foreign_mutant_killed(
            FOREIGN_VECTOR_IDS[0], "FOREIGN_INTEGRITY_IGNORE_REPOSITORY_MUTANT"
        )

    def test_foreign_catalog_repository_kills_ignore_repository_mutant(self):
        self._assert_foreign_mutant_killed(
            FOREIGN_VECTOR_IDS[1], "FOREIGN_CATALOG_IGNORE_REPOSITORY_MUTANT"
        )

    def test_foreign_bindings_repository_kills_ignore_repository_mutant(self):
        self._assert_foreign_mutant_killed(
            FOREIGN_VECTOR_IDS[2], "FOREIGN_BINDINGS_IGNORE_REPOSITORY_MUTANT"
        )

    def test_target_nonparticipation_kills_participation_required_mutant(self):
        vector = self._vector(TARGET_VECTOR_ID)
        baseline = self._baseline(vector)
        self.assertEqual(baseline["kind"], "result")
        with materialize(vector["fixture"]) as realized:
            isolation = realized.run(inspect_target_nonparticipation)
        self.assertEqual(
            isolation,
            {
                "target_non_null": True,
                "interface_exists": True,
                "object_exists": True,
                "responsibilities_nonempty": True,
                "projection_responsibility_absent": True,
            },
            "ISSUE94_TARGET_MUTANT_FIXTURE_NOT_ISOLATED",
        )
        with materialize(vector["fixture"]) as realized:
            mutant = realized.run(participation_required_mutant)
        self.assertEqual(mutant["kind"], "controlled_rejection")
        self.assertEqual(
            decode_transport(mutant["value"]),
            {"category": "projection-registry.registry.rejected"},
        )
        with self.assertRaisesRegex(AssertionError, "observation mismatch"):
            compare(mutant, vector["expected_observation"], vector["comparison"])
        print("ISSUE94_TARGET_NONPARTICIPATION_FIXTURE_ISOLATED=PASS")
        print("TARGET_PARTICIPATION_REQUIRED_MUTANT=KILLED")


if __name__ == "__main__":
    unittest.main()
