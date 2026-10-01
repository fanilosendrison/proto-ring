from __future__ import annotations

from dataclasses import fields
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import yaml

from proto_ring import governance_bootstrap, governance_routing
from proto_ring import repository_governance_model as model_module


PROFILE = "docs/adr/adr-profile.yaml"
AUTHORITY_PROFILE = "docs/repository-governance/governance-authority.md"
GOVERNED_OBJECTS_PROFILE = "docs/repository-governance/governed-objects.md"
BINDING = "docs/repository-governance/consumer-binding.md"


def canonical_model(binding: str = BINDING) -> dict[str, object]:
    return {
        "model_version": 1,
        "provider": {
            "id": "proto-ring",
            "binding": {
                "capability": "shared_governance_provider",
                "route": "binding",
            },
        },
        "capabilities": {
            "architecture_decisions": {
                "configuration": {},
                "routes": {"profile": PROFILE},
            },
            "shared_governance_provider": {
                "configuration": {"required": True},
                "routes": {"binding": binding},
            },
        },
    }


class RepositoryGovernanceModelTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repository = Path(self.temporary.name)
        self.governance = canonical_model()
        self.write_target(PROFILE)
        self.write_target(BINDING)
        self.write_agents()

    def write_target(self, relative: str) -> Path:
        target = self.repository / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("test target\n", encoding="utf-8")
        return target

    def write_agents(self, body: str = "# Directives\n") -> None:
        payload = yaml.safe_dump(
            {"repository_governance": self.governance}, sort_keys=False
        )
        (self.repository / "AGENTS.md").write_text(
            f"---\n{payload}---\n{body}", encoding="utf-8"
        )

    def load(self) -> model_module.RepositoryGovernanceModel:
        self.write_agents()
        return model_module.load(self.repository)

    def assert_fails(self) -> None:
        self.write_agents()
        with self.assertRaises(model_module.RepositoryGovernanceModelError):
            model_module.load(self.repository)

    @property
    def capabilities(self) -> dict[str, object]:
        return self.governance["capabilities"]  # type: ignore[return-value]

    def add_governed_objects(self, routes: dict[str, str] | None = None) -> None:
        self.capabilities["governance_authority"] = {
            "configuration": {}, "routes": {"profile": AUTHORITY_PROFILE}
        }
        self.capabilities["governed_objects"] = {
            "configuration": {},
            "routes": {"profile": GOVERNED_OBJECTS_PROFILE} if routes is None else routes,
        }
        self.write_target(AUTHORITY_PROFILE)
        self.write_target(GOVERNED_OBJECTS_PROFILE)

    def test_public_api_is_exact(self) -> None:
        self.assertEqual(
            [
                "GovernanceCapability",
                "LogicalGovernanceProvider",
                "ProviderBindingReference",
                "RepositoryGovernanceModel",
                "RepositoryGovernanceModelError", "load",
                "validate_binding_capabilities",
            ],
            model_module.__all__,
        )

    def test_turnlock_style_model_loads(self) -> None:
        binding = "docs/repository-governance/turnlock-rust-binding.md"
        self.governance = canonical_model(binding)
        self.write_target(binding)
        self.assertEqual(model_module.load(self.repository).model_version, 1)

    def test_ruu_style_model_loads(self) -> None:
        binding = "docs/repository-governance/ruu-binding.md"
        self.governance = canonical_model(binding)
        self.write_target(binding)
        self.assertEqual(model_module.load(self.repository).provider.id, "proto-ring")

    def test_repository_and_carrier_are_preserved(self) -> None:
        model = self.load()
        self.assertEqual(model.repository, self.repository)
        self.assertEqual(model.carrier, self.repository / "AGENTS.md")

    def test_model_version_is_one(self) -> None:
        self.assertEqual(self.load().model_version, 1)

    def test_logical_provider_is_proto_ring(self) -> None:
        self.assertEqual(self.load().provider.id, "proto-ring")

    def test_provider_binding_reference_is_exact(self) -> None:
        binding = self.load().provider.binding
        self.assertEqual(binding.capability, "shared_governance_provider")
        self.assertEqual(binding.route, "binding")

    def test_architecture_profile_route_resolves(self) -> None:
        route = self.load().capabilities["architecture_decisions"].routes["profile"]
        self.assertEqual(route.declared_path, PROFILE)
        self.assertEqual(route.target, (self.repository / PROFILE).resolve())

    def test_shared_provider_binding_route_resolves(self) -> None:
        route = self.load().capabilities["shared_governance_provider"].routes["binding"]
        self.assertEqual(route.declared_path, BINDING)
        self.assertEqual(route.target, (self.repository / BINDING).resolve())

    def test_governance_authority_profile_route_resolves(self) -> None:
        self.capabilities["governance_authority"] = {
            "configuration": {},
            "routes": {"profile": AUTHORITY_PROFILE},
        }
        self.write_target(AUTHORITY_PROFILE)
        route = self.load().capabilities["governance_authority"].routes["profile"]
        self.assertEqual(route.declared_path, AUTHORITY_PROFILE)
        self.assertEqual(route.target, (self.repository / AUTHORITY_PROFILE).resolve())

    def test_governance_authority_requires_profile_route(self) -> None:
        self.capabilities["governance_authority"] = {
            "configuration": {},
            "routes": {},
        }
        self.assert_fails()

    def test_governed_objects_profile_route_resolves_and_succeeds(self) -> None:
        self.add_governed_objects()
        route = self.load().capabilities["governed_objects"].routes["profile"]
        self.assertEqual(route.declared_path, GOVERNED_OBJECTS_PROFILE)
        self.assertEqual(route.target, (self.repository / GOVERNED_OBJECTS_PROFILE).resolve())

    def test_governed_objects_requires_profile_route(self) -> None:
        self.add_governed_objects({})
        self.assert_fails()

    def test_governed_objects_without_governance_authority_fails(self) -> None:
        self.capabilities["governed_objects"] = {
            "configuration": {}, "routes": {"profile": GOVERNED_OBJECTS_PROFILE}
        }
        self.write_target(GOVERNED_OBJECTS_PROFILE)
        with self.assertRaisesRegex(
            model_module.RepositoryGovernanceModelError,
            "governed_objects capability requires governance_authority capability",
        ):
            self.load()

    def test_architecture_decisions_remains_optional_with_governed_objects(self) -> None:
        del self.capabilities["architecture_decisions"]
        self.add_governed_objects()
        self.assertNotIn("architecture_decisions", self.load().capabilities)

    def test_configuration_unknown_descendants_are_preserved(self) -> None:
        configuration = {"required": True, "unknown": {"items": [1, None, "x"]}}
        self.capabilities["shared_governance_provider"]["configuration"] = configuration  # type: ignore[index]
        self.assertEqual(
            self.load().capabilities["shared_governance_provider"].configuration,
            configuration,
        )

    def test_additional_route_on_supported_capability_is_resolved(self) -> None:
        self.capabilities["architecture_decisions"]["routes"]["extra"] = "extra.md"  # type: ignore[index]
        self.write_target("extra.md")
        self.assertEqual(
            self.load().capabilities["architecture_decisions"].routes["extra"].target,
            (self.repository / "extra.md").resolve(),
        )

    def test_capability_construction_order_is_deterministic(self) -> None:
        self.governance["capabilities"] = dict(reversed(list(self.capabilities.items())))
        self.assertEqual(
            list(self.load().capabilities),
            ["architecture_decisions", "shared_governance_provider"],
        )

    def test_route_construction_order_is_deterministic(self) -> None:
        routes = {"zeta": "zeta.md", "profile": PROFILE, "alpha": "alpha.md"}
        self.capabilities["architecture_decisions"]["routes"] = routes  # type: ignore[index]
        self.write_target("zeta.md")
        self.write_target("alpha.md")
        self.assertEqual(
            list(self.load().capabilities["architecture_decisions"].routes),
            ["alpha", "profile", "zeta"],
        )

    def test_loader_delegates_to_governance_bootstrap(self) -> None:
        with mock.patch.object(
            governance_bootstrap, "load", wraps=governance_bootstrap.load
        ) as loader:
            self.load()
        loader.assert_called_once_with(self.repository)

    def test_routes_delegate_to_governance_routing(self) -> None:
        with mock.patch.object(
            governance_routing, "resolve_path", wraps=governance_routing.resolve_path
        ) as resolver:
            self.load()
        routes = [call.args[2] for call in resolver.call_args_list]
        self.assertIn(
            ("capabilities", "architecture_decisions", "routes", "profile"), routes
        )
        self.assertIn(
            ("capabilities", "shared_governance_provider", "routes", "binding"),
            routes,
        )

    def test_prose_is_not_reconstructed(self) -> None:
        self.write_agents("repository_governance:\n  model_version: wrong\n")
        self.assertEqual(model_module.load(self.repository).model_version, 1)

    def test_requirements_txt_is_not_consulted(self) -> None:
        (self.repository / "requirements.txt").write_text("not valid\n", encoding="utf-8")
        self.assertEqual(self.load().provider.id, "proto-ring")

    def test_model_has_no_executable_provider_sha_field(self) -> None:
        names = {field.name for field in fields(model_module.RepositoryGovernanceModel)}
        self.assertNotIn("executable_provider_sha", names)
        self.assertNotIn("contract_pins", names)

    def test_legacy_compact_bootstrap_can_parse_but_model_fails_closed(self) -> None:
        self.governance = {
            "architecture_decisions": {"profile_path": PROFILE},
            "shared_governance_provider": {
                "required": True,
                "binding_path": BINDING,
            },
        }
        self.write_agents()
        self.assertIn(
            "architecture_decisions",
            governance_bootstrap.load(self.repository).repository_governance,
        )
        with self.assertRaises(model_module.RepositoryGovernanceModelError):
            model_module.load(self.repository)

    def test_missing_model_version_fails(self) -> None:
        del self.governance["model_version"]
        self.assert_fails()

    def test_boolean_model_version_fails(self) -> None:
        self.governance["model_version"] = True
        self.assert_fails()

    def test_string_model_version_fails(self) -> None:
        self.governance["model_version"] = "1"
        self.assert_fails()

    def test_other_model_version_fails(self) -> None:
        self.governance["model_version"] = 3
        self.assert_fails()

    def test_extra_root_model_key_fails(self) -> None:
        self.governance["extra"] = None
        self.assert_fails()

    def test_missing_provider_fails(self) -> None:
        del self.governance["provider"]
        self.assert_fails()

    def test_provider_not_mapping_fails(self) -> None:
        self.governance["provider"] = "proto-ring"
        self.assert_fails()

    def test_unknown_provider_id_fails(self) -> None:
        self.governance["provider"]["id"] = "other"  # type: ignore[index]
        self.assert_fails()

    def test_extra_provider_key_fails(self) -> None:
        self.governance["provider"]["extra"] = None  # type: ignore[index]
        self.assert_fails()

    def test_provider_binding_not_mapping_fails(self) -> None:
        self.governance["provider"]["binding"] = "binding"  # type: ignore[index]
        self.assert_fails()

    def test_provider_binding_wrong_capability_fails(self) -> None:
        self.governance["provider"]["binding"]["capability"] = "architecture_decisions"  # type: ignore[index]
        self.assert_fails()

    def test_provider_binding_wrong_route_fails(self) -> None:
        self.governance["provider"]["binding"]["route"] = "profile"  # type: ignore[index]
        self.assert_fails()

    def test_extra_provider_binding_key_fails(self) -> None:
        self.governance["provider"]["binding"]["extra"] = None  # type: ignore[index]
        self.assert_fails()

    def test_capabilities_not_mapping_fails(self) -> None:
        self.governance["capabilities"] = []
        self.assert_fails()

    def test_missing_shared_governance_provider_capability_fails(self) -> None:
        del self.capabilities["shared_governance_provider"]
        self.assert_fails()

    def test_unknown_capability_fails(self) -> None:
        self.capabilities["unknown"] = {"configuration": {}, "routes": {}}
        self.assert_fails()

    def test_capability_not_mapping_fails(self) -> None:
        self.capabilities["architecture_decisions"] = []
        self.assert_fails()

    def test_missing_configuration_fails(self) -> None:
        del self.capabilities["architecture_decisions"]["configuration"]  # type: ignore[index]
        self.assert_fails()

    def test_configuration_not_mapping_fails(self) -> None:
        self.capabilities["architecture_decisions"]["configuration"] = []  # type: ignore[index]
        self.assert_fails()

    def test_missing_routes_fails(self) -> None:
        del self.capabilities["architecture_decisions"]["routes"]  # type: ignore[index]
        self.assert_fails()

    def test_routes_not_mapping_fails(self) -> None:
        self.capabilities["architecture_decisions"]["routes"] = []  # type: ignore[index]
        self.assert_fails()

    def test_extra_capability_structural_key_fails(self) -> None:
        self.capabilities["architecture_decisions"]["extra"] = None  # type: ignore[index]
        self.assert_fails()

    def test_architecture_decisions_missing_profile_route_fails(self) -> None:
        self.capabilities["architecture_decisions"]["routes"] = {}  # type: ignore[index]
        self.assert_fails()

    def test_shared_provider_missing_binding_route_fails(self) -> None:
        self.capabilities["shared_governance_provider"]["routes"] = {}  # type: ignore[index]
        self.assert_fails()

    def test_empty_route_id_fails(self) -> None:
        self.capabilities["architecture_decisions"]["routes"][""] = "extra.md"  # type: ignore[index]
        self.assert_fails()

    def test_non_string_route_value_fails(self) -> None:
        self.capabilities["architecture_decisions"]["routes"]["profile"] = True  # type: ignore[index]
        self.assert_fails()

    def test_empty_route_value_fails(self) -> None:
        self.capabilities["architecture_decisions"]["routes"]["profile"] = ""  # type: ignore[index]
        self.assert_fails()

    def test_missing_route_target_fails(self) -> None:
        (self.repository / PROFILE).unlink()
        self.assert_fails()

    def test_absolute_route_target_fails(self) -> None:
        self.capabilities["architecture_decisions"]["routes"]["profile"] = "/tmp/profile"  # type: ignore[index]
        self.assert_fails()

    def test_escaping_route_target_fails(self) -> None:
        self.capabilities["architecture_decisions"]["routes"]["profile"] = "../profile"  # type: ignore[index]
        self.assert_fails()

    def test_symlink_escape_route_target_fails(self) -> None:
        outside = self.repository.parent / f"{self.repository.name}-outside"
        outside.write_text("outside\n", encoding="utf-8")
        self.addCleanup(outside.unlink)
        route = self.repository / "escape"
        route.symlink_to(outside)
        self.capabilities["architecture_decisions"]["routes"]["profile"] = "escape"  # type: ignore[index]
        self.assert_fails()


if __name__ == "__main__":
    unittest.main()
