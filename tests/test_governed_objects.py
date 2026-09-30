from __future__ import annotations

from copy import deepcopy
from dataclasses import fields
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import yaml

from proto_ring import governance_authority, governed_objects, structured_data
from proto_ring.governance_routing import ResolvedGovernanceRoute


class GovernedObjectsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repository = Path(self.temporary.name)
        self.profile = self.repository / "governed.md"
        self.payload: object = {"model_version": 1, "interfaces": {}}
        self.authority = governance_authority.GovernanceAuthorityProfile(
            repository=self.repository,
            carrier=self.repository / "authority.md",
            model_version=1,
            sources={},
            responsibilities={
                identity: governance_authority.GovernedResponsibility(
                    id=identity, roles={}, precedence=()
                )
                for identity in ("r", "r2")
            },
        )

    def write_profile(self, body: str = "# Explanation\n") -> None:
        metadata = {"okf_version": "1.0", "governed_objects": self.payload}
        text = yaml.safe_dump(metadata, sort_keys=False)
        self.profile.write_text(f"---\n{text}---\n{body}", encoding="utf-8")

    def route(self) -> ResolvedGovernanceRoute:
        return ResolvedGovernanceRoute(
            route=("capabilities", "governed_objects", "routes", "profile"),
            declared_path="governed.md",
            target=self.profile,
        )

    def load(self) -> governed_objects.GovernedObjectCatalog:
        self.write_profile()
        return governed_objects.load(self.repository, self.route(), self.authority)

    def assert_fails(self) -> None:
        self.write_profile()
        with self.assertRaises(governed_objects.GovernedObjectsError):
            governed_objects.load(self.repository, self.route(), self.authority)

    @staticmethod
    def object(
        responsibilities: list[object] | None = None,
        relations: list[object] | None = None,
    ) -> dict[str, object]:
        return {
            "responsibilities": ["r"] if responsibilities is None else responsibilities,
            "relations": [] if relations is None else relations,
        }

    @staticmethod
    def relation(
        relation: str = "depends_on",
        responsibility: str = "r",
        interface: str = "a",
        object_id: str = "one",
    ) -> dict[str, object]:
        return {
            "relation": relation,
            "responsibility": responsibility,
            "target": {"interface": interface, "object": object_id},
        }

    def interfaces(self, value: dict[str, object]) -> None:
        self.payload = {"model_version": 1, "interfaces": value}

    def relational_payload(self) -> dict[str, object]:
        return {
            "model_version": 1,
            "interfaces": {"a": {"objects": {"one": self.object(
                relations=[self.relation()]
            )}}},
        }

    def test_public_api_and_exact_model_load_valid_catalog(self) -> None:
        self.assertEqual(
            [
                "GovernedObjectRef",
                "GovernedRelation",
                "GovernedObject",
                "GovernedInterface",
                "GovernedObjectCatalog",
                "GovernedObjectsError",
                "load",
            ],
            governed_objects.__all__,
        )
        self.interfaces({"a": {"objects": {"one": self.object()}}})
        loaded = self.load()
        self.assertEqual(loaded.repository, self.repository)
        self.assertEqual(loaded.carrier, self.profile)
        self.assertEqual(loaded.model_version, 1)
        self.assertEqual(loaded.interfaces["a"].objects["one"].id, "one")
        self.assertEqual(
            loaded.interfaces["a"].objects["one"].responsibilities,
            frozenset({"r"}),
        )
        for model in (
            governed_objects.GovernedObjectRef,
            governed_objects.GovernedRelation,
            governed_objects.GovernedObject,
            governed_objects.GovernedInterface,
            governed_objects.GovernedObjectCatalog,
        ):
            self.assertTrue(model.__dataclass_params__.frozen)
        self.assertEqual(
            [field.name for field in fields(governed_objects.GovernedObjectRef)],
            ["interface_id", "object_id"],
        )

    def test_empty_interfaces_and_empty_objects_are_accepted(self) -> None:
        self.assertEqual(self.load().interfaces, {})
        self.interfaces({"empty": {"objects": {}}})
        self.assertEqual(self.load().interfaces["empty"].objects, {})

    def test_empty_responsibilities_are_rejected(self) -> None:
        self.interfaces({"a": {"objects": {"one": self.object([])}}})
        self.assert_fails()

    def test_known_responsibility_with_empty_roles_is_accepted(self) -> None:
        self.interfaces({"a": {"objects": {"one": self.object(["r"])}}})
        self.assertEqual(self.load().interfaces["a"].objects["one"].responsibilities, frozenset({"r"}))

    def test_unknown_and_duplicate_responsibilities_are_rejected(self) -> None:
        for responsibilities in (["missing"], ["r", "r"]):
            with self.subTest(responsibilities=responsibilities):
                self.interfaces({"a": {"objects": {"one": self.object(responsibilities)}}})
                self.assert_fails()

    def test_zero_relations_are_valid(self) -> None:
        self.interfaces({"a": {"objects": {"one": self.object(relations=[])}}})
        self.assertEqual(self.load().interfaces["a"].objects["one"].relations, frozenset())

    def test_cross_interface_forward_relation_is_accepted(self) -> None:
        relation = self.relation(interface="b", object_id="later")
        self.interfaces(
            {
                "a": {"objects": {"one": self.object(relations=[relation])}},
                "b": {"objects": {"later": self.object()}},
            }
        )
        loaded_relation = next(iter(self.load().interfaces["a"].objects["one"].relations))
        self.assertEqual(
            loaded_relation.target,
            governed_objects.GovernedObjectRef("b", "later"),
        )

    def test_self_relation_is_accepted(self) -> None:
        self.interfaces(
            {"a": {"objects": {"one": self.object(relations=[self.relation()])}}}
        )
        self.assertEqual(len(self.load().interfaces["a"].objects["one"].relations), 1)

    def test_cycle_is_accepted(self) -> None:
        self.interfaces(
            {
                "a": {
                    "objects": {
                        "one": self.object(relations=[self.relation(object_id="two")]),
                        "two": self.object(relations=[self.relation(object_id="one")]),
                    }
                }
            }
        )
        self.assertEqual(len(self.load().interfaces["a"].objects), 2)

    def test_relation_responsibility_must_belong_to_source(self) -> None:
        relation = self.relation(responsibility="r2")
        self.interfaces({"a": {"objects": {"one": self.object(relations=[relation])}}})
        self.assert_fails()

    def test_missing_target_interface_and_object_are_rejected(self) -> None:
        cases = (
            self.relation(interface="missing"),
            self.relation(object_id="missing"),
        )
        for relation in cases:
            with self.subTest(relation=relation):
                self.interfaces({"a": {"objects": {"one": self.object(relations=[relation])}}})
                self.assert_fails()

    def test_exact_duplicate_relation_is_rejected(self) -> None:
        relation = self.relation()
        self.interfaces({"a": {"objects": {"one": self.object(relations=[relation, relation])}}})
        self.assert_fails()

    def test_same_target_with_different_relation_is_allowed(self) -> None:
        relations = [self.relation("first"), self.relation("second")]
        self.interfaces({"a": {"objects": {"one": self.object(relations=relations)}}})
        self.assertEqual(len(self.load().interfaces["a"].objects["one"].relations), 2)

    def test_same_target_with_owned_different_responsibility_is_allowed(self) -> None:
        relations = [self.relation(responsibility="r"), self.relation(responsibility="r2")]
        self.interfaces(
            {"a": {"objects": {"one": self.object(["r", "r2"], relations)}}}
        )
        self.assertEqual(len(self.load().interfaces["a"].objects["one"].relations), 2)

    def test_case_distinct_ids_remain_distinct(self) -> None:
        self.interfaces(
            {
                "A": {"objects": {"Object": self.object()}},
                "a": {"objects": {"object": self.object()}},
            }
        )
        loaded = self.load()
        self.assertEqual(set(loaded.interfaces), {"A", "a"})
        self.assertIn("Object", loaded.interfaces["A"].objects)
        self.assertIn("object", loaded.interfaces["a"].objects)

    def test_unknown_structural_keys_fail_at_every_model_layer(self) -> None:
        base = {
            "model_version": 1,
            "interfaces": {
                "a": {
                    "objects": {
                        "one": self.object(relations=[self.relation()]),
                    }
                }
            },
        }
        paths = (
            (),
            ("interfaces", "a"),
            ("interfaces", "a", "objects", "one"),
            ("interfaces", "a", "objects", "one", "relations", 0),
            ("interfaces", "a", "objects", "one", "relations", 0, "target"),
        )
        for path in paths:
            with self.subTest(path=path):
                payload = deepcopy(base)
                target: object = payload
                for segment in path:
                    target = target[segment]  # type: ignore[index]
                target["extra"] = None  # type: ignore[index]
                self.payload = payload
                self.assert_fails()

    def test_missing_required_keys_are_rejected_explicitly(self) -> None:
        cases = (
            ("governed_objects.model_version", ("model_version",)),
            ("governed_objects.interfaces", ("interfaces",)),
            ("interface.objects", ("interfaces", "a", "objects")),
            ("object.responsibilities", ("interfaces", "a", "objects", "one", "responsibilities")),
            ("object.relations", ("interfaces", "a", "objects", "one", "relations")),
            ("relation.relation", ("interfaces", "a", "objects", "one", "relations", 0, "relation")),
            ("relation.responsibility", ("interfaces", "a", "objects", "one", "relations", 0, "responsibility")),
            ("relation.target", ("interfaces", "a", "objects", "one", "relations", 0, "target")),
            ("target.interface", ("interfaces", "a", "objects", "one", "relations", 0, "target", "interface")),
            ("target.object", ("interfaces", "a", "objects", "one", "relations", 0, "target", "object")),
        )
        for rule, path in cases:
            with self.subTest(rule=rule):
                payload = deepcopy(self.relational_payload())
                parent: object = payload
                for segment in path[:-1]:
                    parent = parent[segment]  # type: ignore[index]
                del parent[path[-1]]  # type: ignore[index]
                self.payload = payload
                self.assert_fails()

    def test_malformed_required_structures_are_rejected_explicitly(self) -> None:
        cases = (
            ("governed_objects non-mapping", (), []),
            ("interfaces non-mapping", ("interfaces",), []),
            ("interface declaration non-mapping", ("interfaces", "a"), []),
            ("objects non-mapping", ("interfaces", "a", "objects"), []),
            ("object declaration non-mapping", ("interfaces", "a", "objects", "one"), []),
            ("responsibilities non-sequence", ("interfaces", "a", "objects", "one", "responsibilities"), {}),
            ("responsibility non-string", ("interfaces", "a", "objects", "one", "responsibilities", 0), 1),
            ("relations non-sequence", ("interfaces", "a", "objects", "one", "relations"), {}),
            ("relation declaration non-mapping", ("interfaces", "a", "objects", "one", "relations", 0), []),
            ("relation identity non-string", ("interfaces", "a", "objects", "one", "relations", 0, "relation"), 1),
            ("relation responsibility non-string", ("interfaces", "a", "objects", "one", "relations", 0, "responsibility"), 1),
            ("target non-mapping", ("interfaces", "a", "objects", "one", "relations", 0, "target"), []),
            ("target interface non-string", ("interfaces", "a", "objects", "one", "relations", 0, "target", "interface"), 1),
            ("target object non-string", ("interfaces", "a", "objects", "one", "relations", 0, "target", "object"), 1),
        )
        for rule, path, value in cases:
            with self.subTest(rule=rule):
                payload = deepcopy(self.relational_payload())
                if not path:
                    self.payload = value
                else:
                    parent: object = payload
                    for segment in path[:-1]:
                        parent = parent[segment]  # type: ignore[index]
                    parent[path[-1]] = value  # type: ignore[index]
                    self.payload = payload
                self.assert_fails()

    def test_empty_interface_identity_is_rejected(self) -> None:
        self.interfaces({"": {"objects": {}}})
        self.assert_fails()

    def test_empty_object_identity_is_rejected(self) -> None:
        self.interfaces({"a": {"objects": {"": self.object()}}})
        self.assert_fails()

    def test_empty_object_responsibility_identity_is_rejected(self) -> None:
        self.interfaces({"a": {"objects": {"one": self.object([""])}}})
        self.assert_fails()

    def test_empty_relation_responsibility_identity_is_rejected(self) -> None:
        self.interfaces({"a": {"objects": {"one": self.object(relations=[self.relation(responsibility="")])}}})
        self.assert_fails()

    def test_empty_relation_identity_is_rejected(self) -> None:
        self.interfaces({"a": {"objects": {"one": self.object(relations=[self.relation(relation="")])}}})
        self.assert_fails()

    def test_empty_target_interface_identity_is_rejected(self) -> None:
        self.interfaces({"a": {"objects": {"one": self.object(relations=[self.relation(interface="")])}}})
        self.assert_fails()

    def test_empty_target_object_identity_is_rejected(self) -> None:
        self.interfaces({"a": {"objects": {"one": self.object(relations=[self.relation(object_id="")])}}})
        self.assert_fails()

    def test_invalid_versions_are_rejected(self) -> None:
        for version in (True, "1", 1.0, 2, None):
            with self.subTest(version=version):
                self.payload = {"model_version": version, "interfaces": {}}
                self.assert_fails()

    def test_body_prose_is_ignored(self) -> None:
        self.write_profile("governed_objects:\n  model_version: wrong\n")
        self.assertEqual(
            governed_objects.load(self.repository, self.route(), self.authority).model_version,
            1,
        )

    def test_authority_profile_from_another_repository_is_rejected(self) -> None:
        other = governance_authority.GovernanceAuthorityProfile(
            repository=self.repository / "other",
            carrier=self.authority.carrier,
            model_version=1,
            sources={},
            responsibilities=self.authority.responsibilities,
        )
        self.write_profile()
        with self.assertRaises(governed_objects.GovernedObjectsError):
            governed_objects.load(self.repository, self.route(), other)

    def test_read_failure_is_wrapped_with_diagnostic(self) -> None:
        with mock.patch.object(Path, "read_bytes", side_effect=OSError("denied")):
            with self.assertRaisesRegex(governed_objects.GovernedObjectsError, "denied"):
                governed_objects.load(self.repository, self.route(), self.authority)

    def test_structured_data_failure_is_wrapped_with_diagnostic(self) -> None:
        self.profile.write_bytes(b"invalid")
        with mock.patch.object(
            structured_data,
            "parse_frontmatter_bytes",
            side_effect=structured_data.StructuredDataError("bad data"),
        ):
            with self.assertRaisesRegex(governed_objects.GovernedObjectsError, "bad data"):
                governed_objects.load(self.repository, self.route(), self.authority)


if __name__ == "__main__":
    unittest.main()
