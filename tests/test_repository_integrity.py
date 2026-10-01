#!/usr/bin/env python3
from __future__ import annotations

from copy import deepcopy
from pathlib import Path

import yaml

from proto_ring.governance_authority import (
    GovernanceAuthorityProfile,
    GovernedResponsibility,
    GovernedSource,
    SourceRole,
)
from proto_ring.governance_routing import ResolvedGovernanceRoute
from proto_ring.governed_objects import (
    GovernedInterface,
    GovernedObject,
    GovernedObjectCatalog,
)
from proto_ring.repository_integrity import (
    RepositoryIntegrityError,
    load,
)
from repository_integrity_test_support import RepositoryFixtureTestCase


class PersistentProfileTests(RepositoryFixtureTestCase):
    def setUp(self) -> None:
        super().setUp()
        self.carrier = self.repo / "integrity-profile.md"
        self.route = ResolvedGovernanceRoute(
            ("repository_integrity",),
            "integrity-profile.md",
            self.carrier,
        )
        sources = {
            "profile-source": GovernedSource("profile-source", self.route),
            "other-source": GovernedSource("other-source", None),
        }
        responsibilities = {
            "repository-integrity-profile": GovernedResponsibility(
                "repository-integrity-profile",
                {"profile-source": SourceRole.AUTHORITY},
                (),
            ),
            "repository-validation": GovernedResponsibility(
                "repository-validation",
                {"profile-source": SourceRole.AUTHORITY},
                (),
            ),
        }
        self.authority = GovernanceAuthorityProfile(
            self.repo,
            self.repo / "authority.md",
            1,
            sources,
            responsibilities,
        )

    def profile_data(self) -> dict[str, object]:
        return {
            "model_version": 1,
            "authority": {
                "responsibility": "repository-integrity-profile",
                "source": "profile-source",
            },
            "environments": ["python"],
            "continue_after_non_satisfied": True,
            "validations": {
                "syntax": {
                    "responsibility": "repository-validation",
                    "prerequisites": [],
                    "instances": {"kind": "single"},
                    "command": {
                        "kind": "command",
                        "environment": "python",
                        "arguments": ["-m", "py_compile"],
                        "undetermined_exit_codes": [2],
                    },
                }
            },
            "order": ["syntax"],
        }

    def write_profile(self, profile: dict[str, object]) -> None:
        metadata = yaml.safe_dump(
            {"repository_integrity": profile},
            allow_unicode=True,
            sort_keys=False,
        )
        self.carrier.write_text(f"---\n{metadata}---\n# Profile\n", encoding="utf-8")

    def load_profile(
        self,
        profile: dict[str, object] | None = None,
        *,
        catalog: GovernedObjectCatalog | None = None,
    ):
        self.write_profile(profile or self.profile_data())
        return load(self.repo, self.route, self.authority, catalog)

    def assert_rejected(self, profile: dict[str, object], message: str) -> None:
        self.write_profile(profile)
        with self.assertRaisesRegex(RepositoryIntegrityError, message):
            load(self.repo, self.route, self.authority)

    def test_loads_exact_model_without_executing_commands(self) -> None:
        marker = self.root / "must-not-exist"
        profile = self.profile_data()
        command = profile["validations"]["syntax"]["command"]  # type: ignore[index]
        command["arguments"] = ["-c", f"open({str(marker)!r}, 'w').close()"]  # type: ignore[index]
        loaded = self.load_profile(profile)
        self.assertFalse(marker.exists())
        self.assertEqual(1, loaded.model_version)
        self.assertEqual(("syntax",), loaded.order)
        self.assertEqual("python", loaded.validations["syntax"].command.environment)
        self.assertEqual(64, len(loaded.identity))

    def test_profile_and_nested_structures_reject_missing_or_unknown_keys(self) -> None:
        unknown = self.profile_data()
        unknown["unexpected"] = True
        self.assert_rejected(unknown, "unexpected key")

        missing = self.profile_data()
        del missing["order"]
        self.assert_rejected(missing, "missing key")

        nested = self.profile_data()
        nested["validations"]["syntax"]["command"]["success_codes"] = [0]  # type: ignore[index]
        self.assert_rejected(nested, "unexpected key")

        instance = self.profile_data()
        instance["validations"]["syntax"]["instances"]["mode"] = "for_each"  # type: ignore[index]
        self.assert_rejected(instance, "unexpected key")

    def test_environments_and_command_binding_fail_closed(self) -> None:
        duplicate = self.profile_data()
        duplicate["environments"] = ["python", "python"]
        self.assert_rejected(duplicate, "duplicate ValidationEnvironmentId")

        undeclared = self.profile_data()
        undeclared["validations"]["syntax"]["command"]["environment"] = "rust"  # type: ignore[index]
        self.assert_rejected(undeclared, "undeclared validation environment")

        success_zero = self.profile_data()
        success_zero["validations"]["syntax"]["command"]["undetermined_exit_codes"] = [0]  # type: ignore[index]
        self.assert_rejected(success_zero, "undetermined_exit_codes is invalid")

    def test_order_must_be_an_exact_total_order(self) -> None:
        profile = self._two_validations()
        for order in (["first"], ["first", "first"], ["first", "foreign"]):
            candidate = deepcopy(profile)
            candidate["order"] = order
            with self.subTest(order=order):
                self.assert_rejected(candidate, "every ValidationId exactly once")

    def _two_validations(self) -> dict[str, object]:
        profile = self.profile_data()
        first = profile["validations"].pop("syntax")  # type: ignore[union-attr]
        profile["validations"] = {  # type: ignore[index]
            "first": first,
            "second": deepcopy(first),
        }
        profile["order"] = ["first", "second"]
        return profile

    def test_prerequisites_reject_duplicates_unknown_self_and_forward_edges(self) -> None:
        cases = (
            (("second",), "first", "prerequisite must precede"),
            (("first", "first"), "second", "duplicate prerequisite"),
            (("unknown",), "second", "unknown prerequisite"),
            (("second",), "second", "cannot depend on itself"),
        )
        for prerequisites, validation_id, message in cases:
            profile = self._two_validations()
            profile["validations"][validation_id]["prerequisites"] = list(prerequisites)  # type: ignore[index]
            with self.subTest(prerequisites=prerequisites):
                self.assert_rejected(profile, message)

    def test_repository_path_forms_and_restricted_globs_load(self) -> None:
        profile = self.profile_data()
        profile["validations"]["syntax"]["instances"] = {  # type: ignore[index]
            "kind": "repository_paths",
            "mode": "for_each",
            "selectors": [
                {"kind": "path", "path": "pyproject.toml"},
                {"kind": "glob", "glob": "src/*/*.py"},
            ],
        }
        loaded = self.load_profile(profile)
        instances = loaded.validations["syntax"].instances
        self.assertEqual("repository_paths", instances.kind)
        self.assertEqual(["path", "glob"], [item.kind for item in instances.selectors])

    def test_forbidden_selector_syntax_is_rejected(self) -> None:
        values = ("/absolute", "../parent", "src/**/x.py", "src/?.py", "src/[ab].py")
        for value in values:
            profile = self.profile_data()
            profile["validations"]["syntax"]["instances"] = {  # type: ignore[index]
                "kind": "repository_paths",
                "mode": "for_each",
                "selectors": [{"kind": "glob", "glob": value}],
            }
            with self.subTest(value=value):
                self.assert_rejected(profile, "relative POSIX|forbidden|unsupported")

    def test_mapping_declaration_order_does_not_change_profile_identity(self) -> None:
        first = self._two_validations()
        first["validations"]["second"]["prerequisites"] = ["first"]  # type: ignore[index]
        one = self.load_profile(first).identity
        reversed_validations = dict(reversed(list(first["validations"].items())))  # type: ignore[union-attr]
        first["validations"] = reversed_validations
        two = self.load_profile(first).identity
        self.assertEqual(one, two)

    def test_profile_identity_excludes_expanded_glob_membership(self) -> None:
        profile = self.profile_data()
        profile["validations"]["syntax"]["instances"] = {  # type: ignore[index]
            "kind": "repository_paths",
            "mode": "for_each",
            "selectors": [{"kind": "glob", "glob": "dynamic/*.py"}],
        }
        before = self.load_profile(profile).identity
        self.write("dynamic/new.py", "pass\n")
        after = self.load_profile(profile).identity
        self.assertEqual(before, after)

    def test_profile_carrier_must_be_authoritative_source(self) -> None:
        profile = self.profile_data()
        profile["authority"]["source"] = "other-source"  # type: ignore[index]
        self.assert_rejected(profile, "not authority")

    def test_optional_target_must_exist_and_participate_in_responsibility(self) -> None:
        governed = GovernedObject(
            "contract",
            frozenset({"repository-validation"}),
            frozenset(),
        )
        catalog = GovernedObjectCatalog(
            self.repo,
            self.repo / "objects.md",
            1,
            {"contracts": GovernedInterface("contracts", {"contract": governed})},
        )
        profile = self.profile_data()
        profile["validations"]["syntax"]["target"] = {  # type: ignore[index]
            "interface": "contracts",
            "object": "contract",
        }
        loaded = self.load_profile(profile, catalog=catalog)
        self.assertEqual("contract", loaded.validations["syntax"].target.object_id)

        with self.assertRaisesRegex(RepositoryIntegrityError, "catalog is required"):
            load(self.repo, self.route, self.authority)


if __name__ == "__main__":
    import unittest

    unittest.main()
