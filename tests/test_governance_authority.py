from __future__ import annotations

from dataclasses import fields
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import yaml

from proto_ring import governance_authority, governance_routing


class GovernanceAuthorityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repository = Path(self.temporary.name)
        self.profile = self.repository / "authority.md"
        self.payload: object = {
            "model_version": 1,
            "sources": {},
            "responsibilities": {},
        }

    def write_target(self, relative: str) -> Path:
        target = self.repository / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("target\n", encoding="utf-8")
        return target

    def write_profile(self, body: str = "# Explanation\n") -> None:
        metadata = {
            "okf_version": "1.0",
            "governance_authority": self.payload,
        }
        text = yaml.safe_dump(metadata, sort_keys=False)
        self.profile.write_text(f"---\n{text}---\n{body}", encoding="utf-8")

    def route(self) -> governance_routing.ResolvedGovernanceRoute:
        return governance_routing.ResolvedGovernanceRoute(
            route=("capabilities", "governance_authority", "routes", "profile"),
            declared_path="authority.md",
            target=self.profile,
        )

    def load(self) -> governance_authority.GovernanceAuthorityProfile:
        self.write_profile()
        return governance_authority.load(self.repository, self.route())

    def assert_fails(self) -> None:
        self.write_profile()
        with self.assertRaises(governance_authority.GovernanceAuthorityError):
            governance_authority.load(self.repository, self.route())

    def authority_payload(
        self,
        roles: dict[str, str] | None = None,
        precedence: list[object] | None = None,
    ) -> dict[str, object]:
        return {
            "model_version": 1,
            "sources": {"a": {}, "b": {}},
            "responsibilities": {
                "r": {
                    "roles": roles or {"a": "authority", "b": "authority"},
                    "precedence": precedence or [],
                }
            },
        }

    def test_public_api_is_exact(self) -> None:
        self.assertEqual(
            [
                "GovernanceAuthorityError",
                "SourceRole",
                "AuthorityLookupState",
                "GovernedSource",
                "PrecedenceEdge",
                "GovernedResponsibility",
                "GovernanceAuthorityProfile",
                "load",
                "role_of",
                "authority_sources",
                "sources_with_role",
                "outranks",
            ],
            governance_authority.__all__,
        )

    def test_valid_minimal_empty_profile_needs_no_product_intent(self) -> None:
        loaded = self.load()
        self.assertEqual(loaded.model_version, 1)
        self.assertEqual(loaded.sources, {})
        self.assertEqual(loaded.responsibilities, {})
        self.assertEqual(loaded.repository, self.repository)
        self.assertEqual(loaded.carrier, self.profile)

    def test_exact_role_vocabulary(self) -> None:
        self.assertEqual(
            ["authority", "secondary_representation", "non_authoritative"],
            [role.value for role in governance_authority.SourceRole],
        )
        self.assertEqual(
            ["unknown_responsibility", "undeclared"],
            [state.value for state in governance_authority.AuthorityLookupState],
        )

    def test_one_authority_and_exact_queries(self) -> None:
        self.payload = {
            "model_version": 1,
            "sources": {"a": {}, "other": {}},
            "responsibilities": {
                "r": {
                    "roles": {"a": "authority", "other": "non_authoritative"},
                    "precedence": [],
                }
            },
        }
        loaded = self.load()
        self.assertEqual(
            governance_authority.role_of(loaded, "r", "a"),
            governance_authority.SourceRole.AUTHORITY,
        )
        self.assertEqual(
            governance_authority.sources_with_role(
                loaded, "r", governance_authority.SourceRole.NON_AUTHORITATIVE
            ),
            frozenset({"other"}),
        )
        self.assertEqual(
            governance_authority.authority_sources(loaded, "r"), frozenset({"a"})
        )

    def test_multiple_incomparable_coauthorities_are_valid_without_winner(self) -> None:
        self.payload = self.authority_payload()
        loaded = self.load()
        self.assertEqual(
            governance_authority.authority_sources(loaded, "r"),
            frozenset({"a", "b"}),
        )
        self.assertFalse(governance_authority.outranks(loaded, "r", "a", "b"))
        self.assertFalse(governance_authority.outranks(loaded, "r", "b", "a"))
        profile_fields = {field.name for field in fields(type(loaded))}
        self.assertFalse(profile_fields & {"winner", "winning_authority"})

    def test_same_source_has_different_roles_by_responsibility(self) -> None:
        self.payload = {
            "model_version": 1,
            "sources": {"a": {}},
            "responsibilities": {
                "first": {"roles": {"a": "authority"}, "precedence": []},
                "second": {
                    "roles": {"a": "non_authoritative"},
                    "precedence": [],
                },
            },
        }
        loaded = self.load()
        self.assertIs(
            governance_authority.role_of(loaded, "first", "a"),
            governance_authority.SourceRole.AUTHORITY,
        )
        self.assertIs(
            governance_authority.role_of(loaded, "second", "a"),
            governance_authority.SourceRole.NON_AUTHORITATIVE,
        )

    def test_lookup_states_remain_distinct(self) -> None:
        self.payload = self.authority_payload()
        loaded = self.load()
        self.assertIs(
            governance_authority.role_of(loaded, "missing", "a"),
            governance_authority.AuthorityLookupState.UNKNOWN_RESPONSIBILITY,
        )
        self.assertIs(
            governance_authority.role_of(loaded, "r", "missing"),
            governance_authority.AuthorityLookupState.UNDECLARED,
        )
        self.assertIs(
            governance_authority.authority_sources(loaded, "missing"),
            governance_authority.AuthorityLookupState.UNKNOWN_RESPONSIBILITY,
        )
        self.assertIs(
            governance_authority.outranks(loaded, "missing", "a", "b"),
            governance_authority.AuthorityLookupState.UNKNOWN_RESPONSIBILITY,
        )

    def test_known_empty_responsibility_has_empty_authority_set(self) -> None:
        self.payload = {
            "model_version": 1,
            "sources": {},
            "responsibilities": {"known": {"roles": {}, "precedence": []}},
        }
        loaded = self.load()
        self.assertEqual(
            governance_authority.authority_sources(loaded, "known"), frozenset()
        )

    def test_secondary_with_authority_is_valid(self) -> None:
        self.payload = self.authority_payload(
            {"a": "authority", "b": "secondary_representation"}
        )
        loaded = self.load()
        self.assertEqual(
            governance_authority.sources_with_role(
                loaded, "r", governance_authority.SourceRole.SECONDARY_REPRESENTATION
            ),
            frozenset({"b"}),
        )

    def test_secondary_without_authority_fails(self) -> None:
        self.payload = self.authority_payload(
            {"a": "secondary_representation", "b": "non_authoritative"}
        )
        self.assert_fails()

    def test_precedence_and_transitive_outranking(self) -> None:
        self.payload = {
            "model_version": 1,
            "sources": {"a": {}, "b": {}, "c": {}},
            "responsibilities": {
                "r": {
                    "roles": {"a": "authority", "b": "authority", "c": "authority"},
                    "precedence": [
                        {"higher_source": "a", "lower_source": "b"},
                        {"higher_source": "b", "lower_source": "c"},
                        {"higher_source": "a", "lower_source": "c"},
                    ],
                }
            },
        }
        loaded = self.load()
        self.assertTrue(governance_authority.outranks(loaded, "r", "a", "c"))
        self.assertFalse(governance_authority.outranks(loaded, "r", "c", "a"))
        self.assertEqual(len(loaded.responsibilities["r"].precedence), 3)

    def test_invalid_precedence_graphs_fail(self) -> None:
        invalid_edges = {
            "self": [{"higher_source": "a", "lower_source": "a"}],
            "duplicate": [
                {"higher_source": "a", "lower_source": "b"},
                {"higher_source": "a", "lower_source": "b"},
            ],
            "cycle": [
                {"higher_source": "a", "lower_source": "b"},
                {"higher_source": "b", "lower_source": "a"},
            ],
        }
        for name, edges in invalid_edges.items():
            with self.subTest(name=name):
                self.payload = self.authority_payload(precedence=edges)
                self.assert_fails()

    def test_precedence_endpoint_must_be_declared_authority(self) -> None:
        cases = (
            ({"a": "authority", "b": "non_authoritative"}, "b"),
            ({"a": "authority", "b": "secondary_representation"}, "b"),
        )
        for roles, lower in cases:
            with self.subTest(roles=roles):
                self.payload = self.authority_payload(
                    roles,
                    [{"higher_source": "a", "lower_source": lower}],
                )
                self.assert_fails()
        self.payload = self.authority_payload(
            precedence=[{"higher_source": "a", "lower_source": "unknown"}]
        )
        self.assert_fails()

    def test_repository_targets_use_canonical_routing(self) -> None:
        self.write_target("docs/shared.md")
        self.payload = {
            "model_version": 1,
            "sources": {
                "a": {"repository_target": "docs/shared.md"},
                "b": {"repository_target": "docs/shared.md"},
                "external": {},
            },
            "responsibilities": {
                "r": {
                    "roles": {
                        "a": "authority",
                        "b": "authority",
                        "external": "non_authoritative",
                    },
                    "precedence": [],
                }
            },
        }
        with mock.patch.object(
            governance_routing, "resolve_path", wraps=governance_routing.resolve_path
        ) as resolver:
            loaded = self.load()
        self.assertEqual(resolver.call_count, 2)
        target_a = loaded.sources["a"].repository_target
        target_b = loaded.sources["b"].repository_target
        self.assertIsNotNone(target_a)
        self.assertIsNotNone(target_b)
        self.assertEqual(target_a.target, target_b.target)  # type: ignore[union-attr]
        self.assertEqual(  # type: ignore[union-attr]
            target_a.declared_path, target_b.declared_path
        )
        self.assertIsNone(loaded.sources["external"].repository_target)

    def test_body_prose_is_ignored(self) -> None:
        self.write_profile("governance_authority:\n  model_version: wrong\n")
        self.assertEqual(
            governance_authority.load(self.repository, self.route()).model_version, 1
        )

    def test_unused_source_and_unknown_role_fail(self) -> None:
        self.payload = self.authority_payload({"a": "authority"})
        self.assert_fails()
        self.payload = self.authority_payload({"a": "unknown", "b": "authority"})
        self.assert_fails()

    def test_role_source_missing_from_sources_fails(self) -> None:
        self.payload = self.authority_payload()
        del self.payload["sources"]["b"]  # type: ignore[index]
        self.assert_fails()

    def test_invalid_profile_structures_fail_closed(self) -> None:
        valid = self.authority_payload()
        cases: list[tuple[str, object]] = [
            ("not mapping", []),
            ("unknown direct key", {**valid, "extra": None}),
            ("boolean version", {**valid, "model_version": True}),
            ("other version", {**valid, "model_version": 2}),
            ("sources not mapping", {**valid, "sources": []}),
            ("responsibilities not mapping", {**valid, "responsibilities": []}),
            ("empty source id", {**valid, "sources": {"": {}}}),
            ("source not mapping", {**valid, "sources": {"a": []}}),
            ("unknown source key", {**valid, "sources": {"a": {"extra": "x"}}}),
            ("empty target", {**valid, "sources": {"a": {"repository_target": ""}}}),
            ("empty responsibility id", {**valid, "responsibilities": {"": {"roles": {}, "precedence": []}}}),
            ("responsibility not mapping", {**valid, "responsibilities": {"r": []}}),
            ("missing roles", {**valid, "responsibilities": {"r": {"precedence": []}}}),
            ("missing precedence", {**valid, "responsibilities": {"r": {"roles": {}}}}),
            ("unknown responsibility key", {**valid, "responsibilities": {"r": {"roles": {}, "precedence": [], "extra": None}}}),
            ("roles not mapping", {**valid, "responsibilities": {"r": {"roles": [], "precedence": []}}}),
            ("precedence not sequence", {**valid, "responsibilities": {"r": {"roles": {}, "precedence": {}}}}),
        ]
        for name, payload in cases:
            with self.subTest(name=name):
                self.payload = payload
                self.assert_fails()

    def test_invalid_precedence_entries_fail_closed(self) -> None:
        invalid = (
            "edge",
            {},
            {"higher_source": "a"},
            {"higher_source": "a", "lower_source": "b", "extra": None},
            {"higher_source": "", "lower_source": "b"},
        )
        for edge in invalid:
            with self.subTest(edge=edge):
                self.payload = self.authority_payload(precedence=[edge])
                self.assert_fails()

    def test_absent_governance_authority_fails(self) -> None:
        self.profile.write_text("---\nokf_version: \"1.0\"\n---\n", encoding="utf-8")
        with self.assertRaises(governance_authority.GovernanceAuthorityError):
            governance_authority.load(self.repository, self.route())


if __name__ == "__main__":
    unittest.main()
