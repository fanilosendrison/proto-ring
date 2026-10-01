from __future__ import annotations

from copy import deepcopy
from dataclasses import fields
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import yaml

from proto_ring import governance_bindings, structured_data
from proto_ring.governance_authority import (
    GovernedResponsibility,
    GovernedSource,
    GovernanceAuthorityProfile,
    SourceRole,
)
from proto_ring.governance_routing import ResolvedGovernanceRoute
from proto_ring.governed_objects import (
    GovernedInterface,
    GovernedObject,
    GovernedObjectCatalog,
)

COMMIT = "a" * 40
OTHER_COMMIT = "b" * 40


class GovernanceBindingsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repository = Path(self.temporary.name)
        self.registry_path = self.repository / "bindings.md"
        registry_route = self.route()
        self.authority = GovernanceAuthorityProfile(
            repository=self.repository,
            carrier=self.repository / "authority.md",
            model_version=1,
            sources={
                "registry": GovernedSource("registry", registry_route),
                "owner": GovernedSource("owner", None),
                "other": GovernedSource("other", None),
            },
            responsibilities={
                "provider_binding": GovernedResponsibility(
                    "provider_binding",
                    {"registry": SourceRole.AUTHORITY},
                    (),
                ),
                "contract_binding": GovernedResponsibility(
                    "contract_binding",
                    {
                        "owner": SourceRole.AUTHORITY,
                        "registry": SourceRole.SECONDARY_REPRESENTATION,
                        "other": SourceRole.NON_AUTHORITATIVE,
                    },
                    (),
                ),
            },
        )
        self.payload: object = self.valid_payload()

    def route(self, path: Path | None = None) -> ResolvedGovernanceRoute:
        target = self.registry_path if path is None else path
        return ResolvedGovernanceRoute(("capabilities", "x", "routes", "registry"), "bindings.md", target)

    @staticmethod
    def executable() -> dict[str, object]:
        return {
            "kind": "executable_provider",
            "scope": {"kind": "logical_provider"},
            "identity": {"repository": "provider/repository", "commit": COMMIT},
            "authority": {"responsibility": "provider_binding", "source": "registry"},
        }

    @staticmethod
    def contract(
        *,
        scope: dict[str, object] | None = None,
        commit: str = COMMIT,
        path: str = "docs/contracts/example.md",
    ) -> dict[str, object]:
        return {
            "kind": "governance_contract",
            "scope": scope or {"kind": "capability", "capability": "example"},
            "identity": {
                "repository": "provider/repository",
                "commit": commit,
                "path": path,
            },
            "authority": {"responsibility": "contract_binding", "source": "owner"},
        }

    def valid_payload(self) -> dict[str, object]:
        return {
            "model_version": 1,
            "source": "registry",
            "bindings": {"provider": self.executable(), "contract": self.contract()},
        }

    def write_registry(self, body: str = "# Explanatory body\n") -> None:
        metadata = {"okf_version": "1.0", "governance_bindings": self.payload}
        self.registry_path.write_text(
            f"---\n{yaml.safe_dump(metadata, sort_keys=False)}---\n{body}",
            encoding="utf-8",
        )

    def load(
        self,
        *,
        authority: GovernanceAuthorityProfile | None = None,
        catalog: GovernedObjectCatalog | None = None,
        route: ResolvedGovernanceRoute | None = None,
    ) -> governance_bindings.GovernanceBindingRegistry:
        self.write_registry()
        return governance_bindings.load(
            self.repository,
            route or self.route(),
            authority or self.authority,
            catalog,
        )

    def assert_fails(self, **kwargs: object) -> None:
        with self.assertRaises(governance_bindings.GovernanceBindingsError):
            self.load(**kwargs)  # type: ignore[arg-type]

    def catalog(self, object_id: str = "object") -> GovernedObjectCatalog:
        governed_object = GovernedObject(object_id, frozenset({"contract_binding"}), frozenset())
        return GovernedObjectCatalog(
            self.repository,
            self.repository / "objects.md",
            1,
            {"interface": GovernedInterface("interface", {object_id: governed_object})},
        )

    def test_public_api_and_valid_model_are_exact_and_frozen(self) -> None:
        self.assertEqual(
            governance_bindings.__all__,
            [
                "BindingKind", "ScopeKind", "ExecutableProviderIdentity",
                "GovernanceContractIdentity", "BindingAuthority",
                "GovernanceBindingScope", "GovernanceBinding",
                "GovernanceBindingRegistry", "GovernanceBindingsError", "load",
            ],
        )
        loaded = self.load()
        self.assertEqual(loaded.model_version, 1)
        self.assertEqual(loaded.source_id, "registry")
        self.assertEqual(set(loaded.bindings), {"provider", "contract"})
        self.assertEqual(
            [field.name for field in fields(governance_bindings.GovernanceBinding)],
            ["id", "kind", "scope", "identity", "authority"],
        )
        for model in (
            governance_bindings.ExecutableProviderIdentity,
            governance_bindings.GovernanceContractIdentity,
            governance_bindings.BindingAuthority,
            governance_bindings.GovernanceBindingScope,
            governance_bindings.GovernanceBinding,
            governance_bindings.GovernanceBindingRegistry,
        ):
            self.assertTrue(model.__dataclass_params__.frozen)

    def test_exact_kind_and_scope_vocabularies(self) -> None:
        self.assertEqual([kind.value for kind in governance_bindings.BindingKind], ["executable_provider", "governance_contract"])
        self.assertEqual([kind.value for kind in governance_bindings.ScopeKind], ["logical_provider", "capability", "governed_object"])
        bindings = self.valid_payload()["bindings"]
        bindings["logical_contract"] = self.contract(scope={"kind": "logical_provider"})  # type: ignore[index]
        bindings["object_contract"] = self.contract(  # type: ignore[index]
            scope={"kind": "governed_object", "interface": "interface", "object": "object"},
            path="docs/contracts/object.md",
        )
        self.payload = {**self.valid_payload(), "bindings": bindings}
        loaded = self.load(catalog=self.catalog())
        self.assertEqual(loaded.bindings["logical_contract"].scope.kind, governance_bindings.ScopeKind.LOGICAL_PROVIDER)
        self.assertEqual(loaded.bindings["object_contract"].scope.governed_object.object_id, "object")  # type: ignore[union-attr]

    def test_registry_and_binding_exact_keys_fail_closed(self) -> None:
        cases = (
            ("registry missing", ("source",), None),
            ("registry extra", ("extra",), None),
            ("binding missing", ("bindings", "contract", "authority"), None),
            ("binding extra", ("bindings", "contract", "extra"), None),
        )
        for name, path, marker in cases:
            with self.subTest(name=name):
                payload = self.valid_payload()
                parent: object = payload
                for segment in path[:-1]:
                    parent = parent[segment]  # type: ignore[index]
                if "missing" in name:
                    del parent[path[-1]]  # type: ignore[index]
                else:
                    parent[path[-1]] = marker  # type: ignore[index]
                self.payload = payload
                self.assert_fails()

    def test_scope_exact_shapes_and_kind_rules_fail_closed(self) -> None:
        invalid_scopes = (
            {},
            {"kind": "unknown"},
            {"kind": "logical_provider", "capability": "x"},
            {"kind": "capability"},
            {"kind": "capability", "capability": ""},
            {"kind": "governed_object", "interface": "i"},
            {"kind": "governed_object", "interface": "i", "object": "o", "extra": None},
        )
        for scope in invalid_scopes:
            with self.subTest(scope=scope):
                payload = self.valid_payload()
                payload["bindings"]["contract"]["scope"] = scope  # type: ignore[index]
                self.payload = payload
                self.assert_fails()
        payload = self.valid_payload()
        payload["bindings"]["provider"]["scope"] = {"kind": "capability", "capability": "x"}  # type: ignore[index]
        self.payload = payload
        self.assert_fails()

    def test_identity_shapes_commits_and_binding_ids_fail_closed(self) -> None:
        cases = (
            ("unknown kind", ("bindings", "contract", "kind"), "other"),
            ("empty binding id", ("bindings",), "empty-id"),
            ("short commit", ("bindings", "contract", "identity", "commit"), "abc"),
            ("uppercase commit", ("bindings", "contract", "identity", "commit"), "A" * 40),
            ("empty repository", ("bindings", "contract", "identity", "repository"), ""),
            ("empty path", ("bindings", "contract", "identity", "path"), ""),
        )
        for name, path, value in cases:
            with self.subTest(name=name):
                payload = self.valid_payload()
                if name == "empty binding id":
                    payload["bindings"][""] = payload["bindings"].pop("contract")  # type: ignore[index]
                else:
                    target: object = payload
                    for segment in path[:-1]:
                        target = target[segment]  # type: ignore[index]
                    target[path[-1]] = value  # type: ignore[index]
                self.payload = payload
                self.assert_fails()
        for binding_id, extra_key in (("provider", "path"), ("contract", "extra")):
            payload = self.valid_payload()
            payload["bindings"][binding_id]["identity"][extra_key] = "x"  # type: ignore[index]
            self.payload = payload
            self.assert_fails()

    def test_authority_references_and_roles_fail_closed(self) -> None:
        mutations = (
            ("responsibility", "missing"),
            ("source", "missing"),
            ("source", "other"),
        )
        for key, value in mutations:
            with self.subTest(key=key, value=value):
                payload = self.valid_payload()
                payload["bindings"]["contract"]["authority"][key] = value  # type: ignore[index]
                self.payload = payload
                self.assert_fails()
        authority = deepcopy(self.authority)
        authority.responsibilities["contract_binding"].roles["registry"] = SourceRole.NON_AUTHORITATIVE
        self.assert_fails(authority=authority)

    def test_registry_source_must_exist_and_resolve_to_carrier(self) -> None:
        payload = self.valid_payload()
        payload["source"] = "missing"
        self.payload = payload
        self.assert_fails()
        for target in (None, self.route(self.repository / "other.md")):
            authority = deepcopy(self.authority)
            authority.sources["registry"] = GovernedSource("registry", target)
            self.payload = self.valid_payload()
            self.assert_fails(authority=authority)

    def test_governed_object_scope_requires_resolved_same_repository_catalog(self) -> None:
        payload = self.valid_payload()
        payload["bindings"]["contract"]["scope"] = {  # type: ignore[index]
            "kind": "governed_object", "interface": "interface", "object": "object"
        }
        self.payload = payload
        self.assert_fails()
        self.assertEqual(self.load(catalog=self.catalog()).bindings["contract"].scope.kind, governance_bindings.ScopeKind.GOVERNED_OBJECT)
        self.assert_fails(catalog=self.catalog("other"))
        foreign = self.catalog()
        foreign = GovernedObjectCatalog(self.repository / "foreign", foreign.carrier, 1, foreign.interfaces)
        self.assert_fails(catalog=foreign)

    def test_cardinality_and_contract_revision_ambiguity(self) -> None:
        for bindings in ({"contract": self.contract()}, {"one": self.executable(), "two": self.executable()}):
            with self.subTest(bindings=bindings):
                self.payload = {"model_version": 1, "source": "registry", "bindings": bindings}
                self.assert_fails()
        payload = self.valid_payload()
        payload["bindings"]["conflict"] = self.contract(commit=OTHER_COMMIT)  # type: ignore[index]
        self.payload = payload
        self.assert_fails()
        payload["bindings"]["conflict"]["scope"] = {"kind": "capability", "capability": "other"}  # type: ignore[index]
        self.payload = payload
        self.assertEqual(len(self.load().bindings), 3)

    def test_repeated_identity_and_same_sha_distinct_identity_kinds_are_valid(self) -> None:
        payload = self.valid_payload()
        payload["bindings"]["duplicate_identity"] = self.contract(  # type: ignore[index]
            scope={"kind": "logical_provider"}
        )
        self.payload = payload
        loaded = self.load()
        self.assertEqual(loaded.bindings["provider"].identity.commit, loaded.bindings["contract"].identity.commit)
        self.assertEqual(len(loaded.bindings), 3)

    def test_invalid_versions_and_top_level_structures_fail_closed(self) -> None:
        for value in (True, "1", 2, None, []):
            with self.subTest(value=value):
                self.payload = value if value == [] else {**self.valid_payload(), "model_version": value}
                self.assert_fails()
        self.payload = {**self.valid_payload(), "bindings": []}
        self.assert_fails()

    def test_structured_data_read_repository_and_body_boundaries(self) -> None:
        self.write_registry("governance_bindings:\n  model_version: wrong\n")
        self.assertEqual(
            governance_bindings.load(self.repository, self.route(), self.authority).model_version,
            1,
        )
        foreign_authority = deepcopy(self.authority)
        foreign_authority = GovernanceAuthorityProfile(
            self.repository / "foreign", foreign_authority.carrier, 1,
            foreign_authority.sources, foreign_authority.responsibilities,
        )
        self.payload = self.valid_payload()
        self.assert_fails(authority=foreign_authority)
        with mock.patch.object(Path, "read_bytes", side_effect=OSError("denied")):
            self.assert_fails()
        with mock.patch.object(
            structured_data, "parse_frontmatter_bytes",
            side_effect=structured_data.StructuredDataError("invalid"),
        ):
            self.assert_fails()


if __name__ == "__main__":
    unittest.main()
