from __future__ import annotations

from pathlib import Path
import re
from tempfile import TemporaryDirectory
import unittest

import yaml

from proto_ring.governance_bindings import (
    BindingAuthority,
    BindingKind,
    GovernanceBinding,
    GovernanceBindingRegistry,
    GovernanceBindingScope,
    GovernanceContractIdentity,
    ScopeKind,
)
from proto_ring import repository_governance_model as model_module


CONTRACT = Path("docs/contracts/repository-governance-model.md")
REQUIRED_ROUTES = {
    "architecture_decisions": "profile",
    "governance_authority": "profile",
    "governed_objects": "profile",
    "shared_governance_provider": "registry",
    "projection_integrity": "registry",
    "repository_integrity": "profile",
    "evidence_requirements": "registry",
    "authoritative_ref_monotonicity": "binding",
}
MANDATORY = frozenset({"shared_governance_provider", "governance_authority"})


def capability(route: str, target: str) -> dict[str, object]:
    return {"configuration": {}, "routes": {route: target}}


def canonical_v2() -> dict[str, object]:
    return {
        "model_version": 2,
        "provider": {
            "id": "proto-ring",
            "binding": {
                "capability": "shared_governance_provider",
                "route": "registry",
            },
        },
        "capabilities": {
            "shared_governance_provider": capability(
                "registry", "governance-bindings.md"
            ),
            "governance_authority": capability(
                "profile", "governance-authority.md"
            ),
        },
    }


class RepositoryGovernanceModelV2Tests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.repository = Path(temporary.name)
        self.governance = canonical_v2()
        self.write_declared_targets()

    @property
    def capabilities(self) -> dict[str, object]:
        return self.governance["capabilities"]  # type: ignore[return-value]

    def write_target(self, target: str, data: bytes = b"target\n") -> None:
        path = self.repository / target
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def write_declared_targets(self) -> None:
        for declaration in self.capabilities.values():
            routes = declaration["routes"]  # type: ignore[index]
            for target in routes.values():  # type: ignore[union-attr]
                self.write_target(target)  # type: ignore[arg-type]

    def write_agents(self) -> None:
        payload = yaml.safe_dump(
            {"repository_governance": self.governance}, sort_keys=False
        )
        (self.repository / "AGENTS.md").write_text(
            f"---\n{payload}---\n# Directives\n", encoding="utf-8"
        )

    def load(self) -> model_module.RepositoryGovernanceModel:
        self.write_declared_targets()
        self.write_agents()
        return model_module.load(self.repository)

    def assert_fails(self, pattern: str | None = None) -> None:
        self.write_declared_targets()
        self.write_agents()
        context = (
            self.assertRaisesRegex(model_module.RepositoryGovernanceModelError, pattern)
            if pattern is not None
            else self.assertRaises(model_module.RepositoryGovernanceModelError)
        )
        with context:
            model_module.load(self.repository)

    def add_all_capabilities(self) -> None:
        for capability_id, route_id in REQUIRED_ROUTES.items():
            self.capabilities[capability_id] = capability(
                route_id, f"targets/{capability_id}.md"
            )

    def test_minimal_v2_model_loads_with_exact_provider_reference(self) -> None:
        model = self.load()
        self.assertEqual(2, model.model_version)
        self.assertEqual("proto-ring", model.provider.id)
        self.assertEqual("shared_governance_provider", model.provider.binding.capability)
        self.assertEqual("registry", model.provider.binding.route)
        self.assertEqual(MANDATORY, frozenset(model.capabilities))

    def test_exact_eight_capabilities_and_required_routes_load(self) -> None:
        self.add_all_capabilities()
        model = self.load()
        self.assertEqual(frozenset(REQUIRED_ROUTES), frozenset(model.capabilities))
        for capability_id, route_id in REQUIRED_ROUTES.items():
            with self.subTest(capability=capability_id):
                route = model.capabilities[capability_id].routes[route_id]
                self.assertEqual(
                    (self.repository / f"targets/{capability_id}.md").resolve(),
                    route.target,
                )

    def test_contract_table_and_implementation_specification_agree_exactly(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        section = contract.split(
            "The exact model-version-2 capability and required-route table is:", 1
        )[1].split("This table is exhaustive and normative.", 1)[0]
        rows = dict(re.findall(r"^\| `([^`]+)` \| `([^`]+)` \|$", section, re.MULTILINE))
        implementation = {
            capability_id: next(iter(route_ids))
            for capability_id, route_ids in
            model_module._MODEL_VERSIONS[2].required_routes.items()
        }
        self.assertEqual(REQUIRED_ROUTES, rows)
        self.assertEqual(REQUIRED_ROUTES, implementation)
        self.assertEqual(MANDATORY, model_module._MODEL_VERSIONS[2].mandatory_capabilities)

    def test_each_missing_mandatory_capability_fails(self) -> None:
        for capability_id in sorted(MANDATORY):
            self.governance = canonical_v2()
            del self.capabilities[capability_id]
            with self.subTest(capability=capability_id):
                self.assert_fails(f"{capability_id} capability is required")

    def test_each_capability_requires_its_normative_route(self) -> None:
        for capability_id, route_id in REQUIRED_ROUTES.items():
            self.governance = canonical_v2()
            self.add_all_capabilities()
            self.capabilities[capability_id]["routes"] = {}  # type: ignore[index]
            with self.subTest(capability=capability_id):
                self.assert_fails(
                    f"capability {capability_id} is missing route: {route_id}"
                )

    def test_v1_binding_route_is_not_accepted_by_v2(self) -> None:
        self.governance["provider"]["binding"]["route"] = "binding"  # type: ignore[index]
        self.assert_fails("shared_governance_provider/registry")

    def test_v2_registry_route_is_not_a_v1_behavior_change(self) -> None:
        self.governance["model_version"] = 1
        self.governance["provider"]["binding"]["route"] = "binding"  # type: ignore[index]
        self.capabilities["shared_governance_provider"] = capability(
            "binding", "consumer-binding.md"
        )
        del self.capabilities["governance_authority"]
        self.assertEqual(1, self.load().model_version)

    def test_projection_integrity_requires_repository_integrity(self) -> None:
        self.capabilities["projection_integrity"] = capability(
            "registry", "projection-registry.md"
        )
        self.assert_fails(
            "projection_integrity capability requires repository_integrity capability"
        )
        self.capabilities["repository_integrity"] = capability(
            "profile", "repository-integrity.md"
        )
        self.assertIn("projection_integrity", self.load().capabilities)

    def test_evidence_requirements_has_no_object_or_integrity_dependency(self) -> None:
        self.capabilities["evidence_requirements"] = capability(
            "registry", "evidence-requirements.md"
        )
        model = self.load()
        self.assertIn("evidence_requirements", model.capabilities)
        self.assertNotIn("governed_objects", model.capabilities)
        self.assertNotIn("repository_integrity", model.capabilities)

    def test_authoritative_ref_has_no_additional_dependency(self) -> None:
        self.capabilities["authoritative_ref_monotonicity"] = capability(
            "binding", "arm-binding.md"
        )
        self.assertIn("authoritative_ref_monotonicity", self.load().capabilities)

    def test_lower_level_module_names_are_not_capabilities(self) -> None:
        for capability_id in (
            "adr_metadata", "canonical_adr", "accepted_adr_body",
            "git_whitespace", "governance_routing", "structured_data",
            "exact_evidence_binding",
        ):
            self.governance = canonical_v2()
            self.capabilities[capability_id] = capability("profile", "target.md")
            with self.subTest(capability=capability_id):
                self.assert_fails(f"unsupported capability: {capability_id}")

    def test_configuration_and_additional_routes_remain_opaque(self) -> None:
        declaration = self.capabilities["governance_authority"]  # type: ignore[assignment]
        declaration["configuration"] = {"required": True, "consumer": [1, None]}
        declaration["routes"]["consumer-local"] = "consumer-local.md"
        model = self.load()
        loaded = model.capabilities["governance_authority"]
        self.assertEqual(
            {"required": True, "consumer": [1, None]}, loaded.configuration
        )
        self.assertIn("consumer-local", loaded.routes)

    def test_route_targets_are_resolved_but_not_loaded(self) -> None:
        self.write_target("governance-bindings.md", b"\xff\x00not structured data")
        self.write_agents()
        model = model_module.load(self.repository)
        self.assertEqual(
            (self.repository / "governance-bindings.md").resolve(),
            model.capabilities["shared_governance_provider"].routes["registry"].target,
        )

    def registry(self, capability_id: str, repository: Path | None = None) -> GovernanceBindingRegistry:
        binding = GovernanceBinding(
            "contract",
            BindingKind.GOVERNANCE_CONTRACT,
            GovernanceBindingScope(ScopeKind.CAPABILITY, capability_id),
            GovernanceContractIdentity("owner/repository", "a" * 40, "contract.md"),
            BindingAuthority("contract-authority", "registry"),
        )
        return GovernanceBindingRegistry(
            repository or self.repository,
            self.repository / "governance-bindings.md",
            1,
            "registry",
            {"contract": binding},
        )

    def test_declared_capability_scoped_contract_binding_composes(self) -> None:
        model_module.validate_binding_capabilities(
            self.load(), self.registry("governance_authority")
        )

    def test_undeclared_capability_scoped_contract_binding_fails(self) -> None:
        with self.assertRaisesRegex(
            model_module.RepositoryGovernanceModelError,
            "undeclared capability: projection_integrity",
        ):
            model_module.validate_binding_capabilities(
                self.load(), self.registry("projection_integrity")
            )

    def test_binding_composition_rejects_v1_and_foreign_repository(self) -> None:
        model = self.load()
        v1 = model_module.RepositoryGovernanceModel(
            model.repository, model.carrier, 1, model.provider, model.capabilities
        )
        with self.assertRaisesRegex(
            model_module.RepositoryGovernanceModelError, "requires model_version 2"
        ):
            model_module.validate_binding_capabilities(
                v1, self.registry("governance_authority")
            )
        with self.assertRaisesRegex(
            model_module.RepositoryGovernanceModelError, "repository differs"
        ):
            model_module.validate_binding_capabilities(
                model,
                self.registry("governance_authority", self.repository / "foreign"),
            )


if __name__ == "__main__":
    unittest.main()
