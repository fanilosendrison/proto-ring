from __future__ import annotations

from copy import deepcopy
from dataclasses import fields
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import yaml

from proto_ring import projection_registry, structured_data
from proto_ring.governance_authority import (
    GovernedResponsibility,
    GovernedSource,
    GovernanceAuthorityProfile,
    SourceRole,
)
from proto_ring.governance_bindings import (
    BindingAuthority,
    BindingKind,
    ExecutableProviderIdentity,
    GovernanceBinding,
    GovernanceBindingRegistry,
    GovernanceBindingScope,
    ScopeKind,
)
from proto_ring.governance_routing import ResolvedGovernanceRoute
from proto_ring.governed_objects import (
    GovernedInterface,
    GovernedObject,
    GovernedObjectCatalog,
)
from proto_ring.repository_integrity import (
    ConsumerIntegrityProfile,
    ProfileAuthority,
)


class ProjectionRegistryTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.repository = Path(temporary.name)
        self.registry_path = self.repository / "projections.md"
        shared_target = self.route(self.repository / "shared.md")
        self.authority = GovernanceAuthorityProfile(
            self.repository,
            self.repository / "authority.md",
            1,
            {
                "registry": GovernedSource("registry", self.route()),
                "canonical": GovernedSource("canonical", None),
                "other-canonical": GovernedSource("other-canonical", None),
                "secondary": GovernedSource("secondary", shared_target),
                "secondary-two": GovernedSource("secondary-two", shared_target),
                "generator": GovernedSource("generator", None),
                "boundary": GovernedSource("boundary", None),
                "wrong": GovernedSource("wrong", None),
            },
            {
                "projection-registry": GovernedResponsibility(
                    "projection-registry", {"registry": SourceRole.AUTHORITY}, ()
                ),
                "fact": GovernedResponsibility(
                    "fact",
                    {
                        "canonical": SourceRole.AUTHORITY,
                        "other-canonical": SourceRole.AUTHORITY,
                        "secondary": SourceRole.SECONDARY_REPRESENTATION,
                        "secondary-two": SourceRole.SECONDARY_REPRESENTATION,
                        "generator": SourceRole.NON_AUTHORITATIVE,
                        "boundary": SourceRole.AUTHORITY,
                        "wrong": SourceRole.NON_AUTHORITATIVE,
                    },
                    (),
                ),
            },
        )
        self.integrity = ConsumerIntegrityProfile(
            self.repository,
            self.repository / "integrity.md",
            1,
            ProfileAuthority("integrity", "canonical"),
            frozenset(),
            True,
            {"current": mock.sentinel.current, "custody": mock.sentinel.custody},  # type: ignore[arg-type]
            ("current", "custody"),
            "profile-identity",
        )
        self.payload: object = self.valid_payload()

    def route(self, path: Path | None = None) -> ResolvedGovernanceRoute:
        target = path or self.registry_path
        return ResolvedGovernanceRoute(
            ("capabilities", "projection_integrity", "routes", "registry"),
            "projections.md",
            target,
        )

    @staticmethod
    def projection(
        *,
        mode: str = "reference",
        secondary: str = "secondary",
        canonical: str = "canonical",
        validation: str = "current",
    ) -> dict[str, object]:
        value: dict[str, object] = {
            "responsibility": "fact",
            "canonical_source": canonical,
            "secondary_source": secondary,
            "mode": mode,
            "validation": validation,
        }
        if mode == "generated":
            value["generator_source"] = "generator"
        if mode == "bounded_historical_snapshot":
            value["boundary_source"] = "boundary"
            value["validation"] = "custody"
        return value

    def valid_payload(self) -> dict[str, object]:
        return {
            "model_version": 1,
            "authority": {
                "responsibility": "projection-registry",
                "source": "registry",
            },
            "projections": {"primary": self.projection()},
        }

    def write_registry(self, body: str = "# Projection Registry\n") -> None:
        metadata = {"okf_version": "1.0", "projection_registry": self.payload}
        self.registry_path.write_text(
            f"---\n{yaml.safe_dump(metadata, sort_keys=False)}---\n{body}",
            encoding="utf-8",
        )

    def load(
        self,
        *,
        authority: GovernanceAuthorityProfile | None = None,
        integrity: ConsumerIntegrityProfile | None = None,
        catalog: GovernedObjectCatalog | None = None,
        bindings: GovernanceBindingRegistry | None = None,
        route: ResolvedGovernanceRoute | None = None,
    ) -> projection_registry.ProjectionRegistry:
        self.write_registry()
        return projection_registry.load(
            self.repository,
            route or self.route(),
            authority or self.authority,
            integrity or self.integrity,
            catalog,
            bindings,
        )

    def assert_fails(self, **kwargs: object) -> None:
        with self.assertRaises(projection_registry.ProjectionRegistryError):
            self.load(**kwargs)  # type: ignore[arg-type]

    def test_public_model_is_exact_frozen_and_loading_is_read_only(self) -> None:
        loaded = self.load()
        self.assertEqual(1, loaded.model_version)
        self.assertEqual("registry", loaded.authority.source_id)
        self.assertEqual({"primary"}, set(loaded.projections))
        self.assertEqual(
            [field.name for field in fields(projection_registry.Projection)],
            [
                "id", "responsibility_id", "canonical_source_id",
                "secondary_source_id", "mode", "validation_id",
                "generator_source_id", "boundary_source_id", "target",
                "binding_id",
            ],
        )
        for model in (
            projection_registry.RegistryAuthority,
            projection_registry.Projection,
            projection_registry.ProjectionRegistry,
        ):
            self.assertTrue(model.__dataclass_params__.frozen)

    def test_all_four_modes_load_with_exact_specializations(self) -> None:
        payload = self.valid_payload()
        maintained = self.projection(mode="mechanically_validated_maintained")
        maintained["target"] = {"interface": "profiles", "object": "catalog"}
        payload["projections"] = {
            "reference": self.projection(mode="reference"),
            "generated": self.projection(mode="generated", secondary="secondary-two"),
            "maintained": maintained,
            "historical": self.projection(mode="bounded_historical_snapshot"),
        }
        self.payload = payload
        loaded = self.load(catalog=self.catalog()).projections
        self.assertEqual(set(projection_registry.ProjectionMode), {item.mode for item in loaded.values()})
        self.assertEqual("generator", loaded["generated"].generator_source_id)
        self.assertEqual("boundary", loaded["historical"].boundary_source_id)

    def test_registry_projection_and_target_exact_keys_fail_closed(self) -> None:
        paths = (
            (),
            ("authority",),
            ("projections", "primary"),
        )
        for path in paths:
            payload = self.valid_payload()
            target: object = payload
            for segment in path:
                target = target[segment]  # type: ignore[index]
            target["extra"] = None  # type: ignore[index]
            self.payload = payload
            with self.subTest(path=path):
                self.assert_fails()

    def test_mode_specific_fields_are_required_and_forbidden(self) -> None:
        cases = (
            self.projection(mode="generated") | {"generator_source": ""},
            self.projection(mode="reference") | {"generator_source": "generator"},
            self.projection(mode="bounded_historical_snapshot") | {"boundary_source": "wrong"},
            self.projection(mode="reference") | {"boundary_source": "boundary"},
            self.projection(mode="unknown"),
        )
        missing_generator = self.projection(mode="generated")
        del missing_generator["generator_source"]
        missing_boundary = self.projection(mode="bounded_historical_snapshot")
        del missing_boundary["boundary_source"]
        for value in (*cases, missing_generator, missing_boundary):
            self.payload = {**self.valid_payload(), "projections": {"invalid": value}}
            with self.subTest(value=value):
                self.assert_fails()

    def test_direct_source_roles_and_distinct_sources_are_required(self) -> None:
        cases = (
            self.projection(canonical="secondary"),
            self.projection(secondary="canonical"),
            self.projection(secondary="canonical", canonical="canonical"),
            self.projection(canonical="missing"),
        )
        for value in cases:
            self.payload = {**self.valid_payload(), "projections": {"invalid": value}}
            with self.subTest(value=value):
                self.assert_fails()

    def test_validation_must_exist_in_same_repository_integrity_profile(self) -> None:
        self.payload = {
            **self.valid_payload(),
            "projections": {"invalid": self.projection(validation="missing")},
        }
        self.assert_fails()
        foreign = deepcopy(self.integrity)
        foreign = ConsumerIntegrityProfile(
            self.repository / "foreign", foreign.carrier, foreign.model_version,
            foreign.authority, foreign.environments,
            foreign.continue_after_non_satisfied, foreign.validations,
            foreign.order, foreign.identity,
        )
        self.payload = self.valid_payload()
        self.assert_fails(integrity=foreign)

    def catalog(self, object_id: str = "catalog") -> GovernedObjectCatalog:
        item = GovernedObject(object_id, frozenset({"fact"}), frozenset())
        return GovernedObjectCatalog(
            self.repository,
            self.repository / "objects.md",
            1,
            {"profiles": GovernedInterface("profiles", {object_id: item})},
        )

    def test_optional_target_resolves_without_replacing_sources(self) -> None:
        payload = self.valid_payload()
        payload["projections"]["primary"]["target"] = {  # type: ignore[index]
            "interface": "profiles", "object": "catalog"
        }
        self.payload = payload
        self.assert_fails()
        loaded = self.load(catalog=self.catalog()).projections["primary"]
        self.assertEqual("catalog", loaded.target.object_id)  # type: ignore[union-attr]
        self.assertEqual("canonical", loaded.canonical_source_id)
        self.assertEqual("secondary", loaded.secondary_source_id)

    def bindings(self, responsibility: str = "fact", source: str = "canonical") -> GovernanceBindingRegistry:
        binding = GovernanceBinding(
            "provider",
            BindingKind.EXECUTABLE_PROVIDER,
            GovernanceBindingScope(ScopeKind.LOGICAL_PROVIDER),
            ExecutableProviderIdentity("owner/repository", "a" * 40),
            BindingAuthority(responsibility, source),
        )
        return GovernanceBindingRegistry(
            self.repository, self.repository / "bindings.md", 1, "registry", {"provider": binding}
        )

    def test_optional_binding_must_match_existing_binding_authority(self) -> None:
        payload = self.valid_payload()
        payload["projections"]["primary"]["binding"] = "provider"  # type: ignore[index]
        self.payload = payload
        self.assert_fails()
        loaded = self.load(bindings=self.bindings()).projections["primary"]
        self.assertEqual("provider", loaded.binding_id)
        self.assert_fails(bindings=self.bindings(source="other-canonical"))
        self.assert_fails(bindings=self.bindings(responsibility="projection-registry"))

    def test_active_ambiguity_rejects_conflicts_but_distinct_sources_are_direct(self) -> None:
        payload = self.valid_payload()
        payload["projections"] = {
            "one": self.projection(mode="reference"),
            "conflict": self.projection(mode="mechanically_validated_maintained"),
        }
        self.payload = payload
        self.assert_fails()
        payload["projections"]["conflict"]["secondary_source"] = "secondary-two"  # type: ignore[index]
        self.payload = payload
        loaded = self.load()
        self.assertEqual(
            {"canonical"}, {item.canonical_source_id for item in loaded.projections.values()}
        )

    def test_executable_binding_registry_and_realization_are_separate_direct_relations(self) -> None:
        payload = self.valid_payload()
        registry_projection = self.projection(secondary="secondary")
        realization = self.projection(mode="generated", secondary="secondary-two")
        registry_projection["binding"] = "provider"
        realization["binding"] = "provider"
        payload["projections"] = {"registry": registry_projection, "realization": realization}
        self.payload = payload
        loaded = self.load(bindings=self.bindings())
        self.assertEqual(2, len(loaded.projections))
        self.assertEqual(
            {"canonical"}, {item.canonical_source_id for item in loaded.projections.values()}
        )
        self.assertEqual("generator", loaded.projections["realization"].generator_source_id)

    def test_historical_relations_do_not_create_active_conflicts(self) -> None:
        payload = self.valid_payload()
        payload["projections"] = {
            "current": self.projection(),
            "historical": self.projection(mode="bounded_historical_snapshot", canonical="other-canonical"),
        }
        self.payload = payload
        self.assertEqual(2, len(self.load().projections))

    def test_consumer_shapes_fit_without_paths_install_or_qualification_policy(self) -> None:
        payload = self.valid_payload()
        modes = (
            "generated", "mechanically_validated_maintained", "reference",
            "bounded_historical_snapshot",
        )
        projections: dict[str, object] = {}
        for index, mode in enumerate(modes):
            secondary = "secondary" if index % 2 == 0 else "secondary-two"
            value = self.projection(mode=mode, secondary=secondary)
            if mode not in ("generated", "bounded_historical_snapshot"):
                value["target"] = {"interface": "profiles", "object": "catalog"}
            projections[f"shape-{index}"] = value
        payload["projections"] = projections
        self.payload = payload
        loaded = self.load(catalog=self.catalog())
        self.assertEqual(4, len(loaded.projections))

    def test_authority_version_body_and_controlled_io_fail_closed(self) -> None:
        for version in (True, "1", 2, None):
            self.payload = {**self.valid_payload(), "model_version": version}
            with self.subTest(version=version):
                self.assert_fails()
        self.payload = {**self.valid_payload(), "projections": {}}
        self.write_registry("projection_registry:\n  model_version: wrong\n")
        self.assertEqual({}, projection_registry.load(
            self.repository, self.route(), self.authority, self.integrity
        ).projections)
        with mock.patch.object(Path, "read_bytes", side_effect=OSError("denied")):
            self.assert_fails()
        with mock.patch.object(
            structured_data, "parse_frontmatter_bytes",
            side_effect=structured_data.StructuredDataError("bad"),
        ):
            self.assert_fails()


if __name__ == "__main__":
    unittest.main()
