from __future__ import annotations

from dataclasses import fields
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import yaml

from proto_ring import governance_bootstrap, repository_state
from proto_ring.repository_governance_state import (
    RepositoryGovernanceStage,
    RepositoryGovernanceState,
    RepositoryGovernanceStateError,
    load,
)


COMMIT = "a" * 40


def _capability(route: str, target: str) -> dict[str, object]:
    return {"configuration": {}, "routes": {route: target}}


class RepositoryGovernanceStateTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.repository = Path(temporary.name)
        subprocess.run(
            ["git", "init", "-q", os.fspath(self.repository)], check=True
        )

    def write_frontmatter(self, path: str, metadata: dict[str, object]) -> None:
        target = self.repository / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(
            f"---\n{yaml.safe_dump(metadata, sort_keys=False)}---\n# Carrier\n",
            encoding="utf-8",
        )

    def configure(
        self,
        *optional_capabilities: str,
        route_overrides: dict[str, str] | None = None,
        binding_scope: dict[str, object] | None = None,
        evidence_target: dict[str, object] | None = None,
        repository_command: tuple[str, ...] = ("-c", "pass"),
    ) -> None:
        declared = set(optional_capabilities)
        routes = {
            "shared_governance_provider": ("registry", "governance-bindings.md"),
            "governance_authority": ("profile", "governance-authority.md"),
            "governed_objects": ("profile", "governed-objects.md"),
            "repository_integrity": ("profile", "repository-integrity.md"),
            "projection_integrity": ("registry", "projection-registry.md"),
            "evidence_requirements": ("registry", "evidence-requirements.md"),
            "architecture_decisions": ("profile", "decisions"),
            "authoritative_ref_monotonicity": ("binding", "arm-binding.md"),
        }
        overrides = route_overrides or {}
        capabilities = {
            capability_id: _capability(route_id, overrides.get(capability_id, target))
            for capability_id, (route_id, target) in routes.items()
            if capability_id in declared
            or capability_id
            in {"shared_governance_provider", "governance_authority"}
        }
        governance = {
            "model_version": 2,
            "provider": {
                "id": "proto-ring",
                "binding": {
                    "capability": "shared_governance_provider",
                    "route": "registry",
                },
            },
            "capabilities": capabilities,
        }
        self.write_frontmatter(
            "AGENTS.md", {"repository_governance": governance}
        )

        sources: dict[str, object] = {
            "bindings": {
                "repository_target": overrides.get(
                    "shared_governance_provider", "governance-bindings.md"
                )
            }
        }
        responsibilities: dict[str, object] = {
            "provider-binding": {
                "roles": {"bindings": "authority"},
                "precedence": [],
            }
        }
        if "governed_objects" in declared:
            sources["object-owner"] = {}
            responsibilities["object-duty"] = {
                "roles": {"object-owner": "authority"},
                "precedence": [],
            }
        if "repository_integrity" in declared:
            sources["integrity"] = {
                "repository_target": overrides.get(
                    "repository_integrity", "repository-integrity.md"
                )
            }
            responsibilities.update(
                {
                    "integrity-profile": {
                        "roles": {"integrity": "authority"},
                        "precedence": [],
                    },
                    "repository-validation": {
                        "roles": {"integrity": "authority"},
                        "precedence": [],
                    },
                }
            )
        if "projection_integrity" in declared:
            sources.update(
                {
                    "projection-registry": {
                        "repository_target": overrides.get(
                            "projection_integrity", "projection-registry.md"
                        )
                    },
                    "canonical": {},
                    "secondary": {},
                }
            )
            responsibilities.update(
                {
                    "projection-registry": {
                        "roles": {"projection-registry": "authority"},
                        "precedence": [],
                    },
                    "projection-fact": {
                        "roles": {
                            "canonical": "authority",
                            "secondary": "secondary_representation",
                        },
                        "precedence": [],
                    },
                }
            )
        if "evidence_requirements" in declared:
            sources.update(
                {
                    "evidence-registry": {
                        "repository_target": overrides.get(
                            "evidence_requirements", "evidence-requirements.md"
                        )
                    },
                    "subject": {},
                    "candidates": {},
                }
            )
            responsibilities.update(
                {
                    "evidence-registry": {
                        "roles": {"evidence-registry": "authority"},
                        "precedence": [],
                    },
                    "evidence-required": {
                        "roles": {"subject": "authority"},
                        "precedence": [],
                    },
                    "candidate-custody": {
                        "roles": {"candidates": "non_authoritative"},
                        "precedence": [],
                    },
                }
            )
        self.write_frontmatter(
            overrides.get("governance_authority", "governance-authority.md"),
            {
                "governance_authority": {
                    "model_version": 1,
                    "sources": sources,
                    "responsibilities": responsibilities,
                }
            },
        )

        bindings: dict[str, object] = {
            "provider": {
                "kind": "executable_provider",
                "scope": {"kind": "logical_provider"},
                "identity": {
                    "repository": "provider/repository",
                    "commit": COMMIT,
                },
                "authority": {
                    "responsibility": "provider-binding",
                    "source": "bindings",
                },
            }
        }
        if binding_scope is not None:
            bindings["contract"] = {
                "kind": "governance_contract",
                "scope": binding_scope,
                "identity": {
                    "repository": "provider/repository",
                    "commit": COMMIT,
                    "path": "docs/contracts/example.md",
                },
                "authority": {
                    "responsibility": "provider-binding",
                    "source": "bindings",
                },
            }
        self.write_frontmatter(
            overrides.get(
                "shared_governance_provider", "governance-bindings.md"
            ),
            {
                "governance_bindings": {
                    "model_version": 1,
                    "source": "bindings",
                    "bindings": bindings,
                }
            },
        )

        if "governed_objects" in declared:
            self.write_frontmatter(
                overrides.get("governed_objects", "governed-objects.md"),
                {
                    "governed_objects": {
                        "model_version": 1,
                        "interfaces": {
                            "repository": {
                                "objects": {
                                    "governance": {
                                        "responsibilities": ["object-duty"],
                                        "relations": [],
                                    }
                                }
                            }
                        },
                    }
                },
            )
        if "repository_integrity" in declared:
            self.write_frontmatter(
                overrides.get("repository_integrity", "repository-integrity.md"),
                {
                    "repository_integrity": {
                        "model_version": 1,
                        "authority": {
                            "responsibility": "integrity-profile",
                            "source": "integrity",
                        },
                        "environments": ["python"],
                        "continue_after_non_satisfied": True,
                        "validations": {
                            "check": {
                                "responsibility": "repository-validation",
                                "prerequisites": [],
                                "instances": {"kind": "single"},
                                "command": {
                                    "kind": "command",
                                    "environment": "python",
                                    "arguments": list(repository_command),
                                    "undetermined_exit_codes": [],
                                },
                            }
                        },
                        "order": ["check"],
                    }
                },
            )
        if "projection_integrity" in declared:
            self.write_frontmatter(
                overrides.get("projection_integrity", "projection-registry.md"),
                {
                    "projection_registry": {
                        "model_version": 1,
                        "authority": {
                            "responsibility": "projection-registry",
                            "source": "projection-registry",
                        },
                        "projections": {
                            "primary": {
                                "responsibility": "projection-fact",
                                "canonical_source": "canonical",
                                "secondary_source": "secondary",
                                "mode": "reference",
                                "validation": "check",
                            }
                        },
                    }
                },
            )
        if "evidence_requirements" in declared:
            requirement: dict[str, object] = {
                "responsibility": "evidence-required",
                "instances": {"kind": "single"},
                "evidence_classes": {
                    "kind": "explicit",
                    "classes": ["qualification"],
                },
                "subject": {"source": "subject"},
                "context": {"required": False},
                "candidates": {"source": "candidates"},
            }
            if evidence_target is not None:
                requirement["target"] = evidence_target
            self.write_frontmatter(
                overrides.get("evidence_requirements", "evidence-requirements.md"),
                {
                    "evidence_requirements": {
                        "model_version": 1,
                        "authority": {
                            "responsibility": "evidence-registry",
                            "source": "evidence-registry",
                        },
                        "requirements": {"gate": requirement},
                    }
                },
            )
        if "architecture_decisions" in declared:
            (self.repository / overrides.get("architecture_decisions", "decisions")).mkdir(
                parents=True, exist_ok=True
            )
        if "authoritative_ref_monotonicity" in declared:
            target = self.repository / overrides.get(
                "authoritative_ref_monotonicity", "arm-binding.md"
            )
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("https://provider.invalid/ruleset\n", encoding="utf-8")

    def assert_stage(
        self, stage: RepositoryGovernanceStage
    ) -> unittest.case._AssertRaisesContext[RepositoryGovernanceStateError]:
        return self.assertRaisesRegex(
            RepositoryGovernanceStateError, f"^{stage.value}:"
        )

    def test_minimal_valid_v2_state_and_exact_frozen_shape(self) -> None:
        self.configure()
        state = load(self.repository)
        self.assertEqual(
            [
                "repository",
                "observed_state",
                "bootstrap",
                "repository_governance_model",
                "governance_authority",
                "governed_objects",
                "governance_bindings",
                "repository_integrity",
                "projection_registry",
                "evidence_requirements",
            ],
            [field.name for field in fields(RepositoryGovernanceState)],
        )
        self.assertIsNone(state.governed_objects)
        self.assertIsNone(state.repository_integrity)
        self.assertIsNone(state.projection_registry)
        self.assertIsNone(state.evidence_requirements)
        with self.assertRaises(Exception):
            state.repository = Path("elsewhere")  # type: ignore[misc]

    def test_v1_is_rejected_without_changing_v1_model_loading(self) -> None:
        self.configure()
        agents = (self.repository / "AGENTS.md").read_text(encoding="utf-8")
        agents = agents.replace("model_version: 2", "model_version: 1", 1)
        agents = agents.replace("route: registry", "route: binding", 1)
        agents = agents.replace("registry: governance-bindings.md", "binding: governance-bindings.md", 1)
        agents = agents.replace(
            "  governance_authority:\n    configuration: {}\n    routes:\n      profile: governance-authority.md\n",
            "",
            1,
        )
        (self.repository / "AGENTS.md").write_text(agents, encoding="utf-8")
        with self.assert_stage(RepositoryGovernanceStage.REPOSITORY_GOVERNANCE_MODEL):
            load(self.repository)

    def test_optional_governed_objects_and_integrity_absence_and_presence(self) -> None:
        self.configure("governed_objects", "repository_integrity")
        state = load(self.repository)
        self.assertIsNotNone(state.governed_objects)
        self.assertIsNotNone(state.repository_integrity)
        self.assertIsNone(state.projection_registry)

    def test_projection_loads_with_existing_dependencies(self) -> None:
        self.configure("repository_integrity", "projection_integrity")
        state = load(self.repository)
        self.assertIsNotNone(state.repository_integrity)
        self.assertEqual({"primary"}, set(state.projection_registry.projections))  # type: ignore[union-attr]

    def test_evidence_requirements_load_without_governed_objects(self) -> None:
        self.configure("evidence_requirements")
        state = load(self.repository)
        self.assertEqual({"gate"}, set(state.evidence_requirements.requirements))  # type: ignore[union-attr]
        self.assertIsNone(state.governed_objects)

    def test_invalid_evidence_governed_object_target_is_attributed(self) -> None:
        self.configure(
            "evidence_requirements",
            evidence_target={"interface": "missing", "object": "missing"},
        )
        with self.assert_stage(RepositoryGovernanceStage.EVIDENCE_REQUIREMENTS):
            load(self.repository)

    def test_invalid_governed_object_binding_scope_is_attributed(self) -> None:
        self.configure(
            "governed_objects",
            binding_scope={
                "kind": "governed_object",
                "interface": "repository",
                "object": "missing",
            },
        )
        with self.assert_stage(RepositoryGovernanceStage.GOVERNANCE_BINDINGS):
            load(self.repository)

    def test_capability_scoped_binding_to_undeclared_capability_is_rejected(self) -> None:
        self.configure(
            binding_scope={"kind": "capability", "capability": "projection_integrity"}
        )
        with self.assert_stage(RepositoryGovernanceStage.COMPOSITION):
            load(self.repository)

    def test_malformed_authority_and_projection_diagnostics_are_attributed(self) -> None:
        self.configure()
        (self.repository / "governance-authority.md").write_text(
            "not frontmatter\n", encoding="utf-8"
        )
        with self.assert_stage(RepositoryGovernanceStage.GOVERNANCE_AUTHORITY):
            load(self.repository)

        self.configure("repository_integrity", "projection_integrity")
        (self.repository / "projection-registry.md").write_text(
            "not frontmatter\n", encoding="utf-8"
        )
        with self.assert_stage(RepositoryGovernanceStage.PROJECTION_INTEGRITY):
            load(self.repository)

    def test_architecture_decisions_do_not_trigger_eager_adr_scan(self) -> None:
        self.configure("architecture_decisions")
        malformed = self.repository / "decisions" / "not-an-adr.md"
        malformed.write_bytes(b"\xff\x00not an ADR")
        self.assertIn(
            "architecture_decisions",
            load(self.repository).repository_governance_model.capabilities,
        )

    def test_arm_does_not_call_a_network_provider(self) -> None:
        self.configure("authoritative_ref_monotonicity")
        with mock.patch("urllib.request.urlopen", side_effect=AssertionError("network")):
            state = load(self.repository)
        self.assertIn(
            "authoritative_ref_monotonicity",
            state.repository_governance_model.capabilities,
        )

    def test_stable_exact_state_uses_second_pass_values_and_one_bootstrap_per_pass(self) -> None:
        self.configure()
        real_bootstrap_load = governance_bootstrap.load
        observed_bootstraps: list[governance_bootstrap.GovernanceBootstrap] = []

        def record_bootstrap(repository: Path):
            bootstrap = real_bootstrap_load(repository)
            observed_bootstraps.append(bootstrap)
            return bootstrap

        with mock.patch(
            "proto_ring.repository_governance_state.governance_bootstrap.load",
            side_effect=record_bootstrap,
        ) as bootstrap_load, mock.patch(
            "proto_ring.repository_governance_state.repository_governance_model.load",
            side_effect=AssertionError("must construct from loaded bootstrap"),
        ):
            state = load(self.repository)
        self.assertEqual(2, bootstrap_load.call_count)
        self.assertEqual(
            repository_state.capture(self.repository, state.observed_state.scope_paths),
            state.observed_state,
        )
        self.assertIs(state.bootstrap, observed_bootstraps[1])
        self.assertIsNot(state.bootstrap, observed_bootstraps[0])

    def test_repository_mutation_during_construction_fails_closed(self) -> None:
        self.configure()
        real_load = governance_bootstrap.load
        calls = 0

        def mutate_on_second_pass(repository: Path):
            nonlocal calls
            calls += 1
            if calls == 2:
                with (self.repository / "governance-bindings.md").open(
                    "a", encoding="utf-8"
                ) as carrier:
                    carrier.write("# concurrent mutation\n")
            return real_load(repository)

        with mock.patch(
            "proto_ring.repository_governance_state.governance_bootstrap.load",
            side_effect=mutate_on_second_pass,
        ):
            with self.assert_stage(RepositoryGovernanceStage.REPOSITORY_STATE):
                load(self.repository)

    def test_observation_scope_drift_fails_closed(self) -> None:
        self.configure()
        from proto_ring import repository_governance_state as state_module

        real_scope = state_module._observation_scope
        calls = 0

        def drift(*args, **kwargs):
            nonlocal calls
            calls += 1
            scope = real_scope(*args, **kwargs)
            return scope if calls == 1 else scope + ("drift",)

        with mock.patch.object(state_module, "_observation_scope", side_effect=drift):
            with self.assert_stage(RepositoryGovernanceStage.COMPOSITION):
                load(self.repository)

    def test_ignored_routed_file_is_in_the_exact_observation(self) -> None:
        (self.repository / ".gitignore").write_text(
            "ignored-bindings.md\n", encoding="utf-8"
        )
        self.configure(
            route_overrides={
                "shared_governance_provider": "ignored-bindings.md"
            }
        )
        first = load(self.repository).observed_state
        with (self.repository / "ignored-bindings.md").open("a", encoding="utf-8") as carrier:
            carrier.write("# changed while ignored\n")
        second = load(self.repository).observed_state
        self.assertNotEqual(first.identity, second.identity)
        self.assertIn("ignored-bindings.md", second.scope_paths)

    def test_contained_return_route_expression_is_not_repository_state_scope(self) -> None:
        declared = f"../{self.repository.name}/governance-bindings.md"
        self.configure(
            route_overrides={"shared_governance_provider": declared}
        )
        self.assertTrue((self.repository / "governance-bindings.md").is_file())

        state = load(self.repository)
        route = (
            state.repository_governance_model
            .capabilities["shared_governance_provider"]
            .routes["registry"]
        )

        self.assertEqual(declared, route.declared_path)
        self.assertNotIn(declared, state.observed_state.scope_paths)
        self.assertIn("governance-bindings.md", state.observed_state.scope_paths)

    def test_contained_return_route_resolved_target_mutation_is_observed(self) -> None:
        declared = f"../{self.repository.name}/governance-bindings.md"
        self.configure(
            route_overrides={"shared_governance_provider": declared}
        )
        first = load(self.repository)

        with (self.repository / "governance-bindings.md").open(
            "a", encoding="utf-8"
        ) as carrier:
            carrier.write("# explanatory revision\n")
        second = load(self.repository)

        self.assertNotEqual(
            first.observed_state.identity,
            second.observed_state.identity,
        )

    def test_routed_symlink_binding_and_target_are_both_observed(self) -> None:
        self.configure()
        first_target = self.repository / "bindings-a.md"
        second_target = self.repository / "bindings-b.md"
        content = (self.repository / "governance-bindings.md").read_bytes()
        first_target.write_bytes(content)
        second_target.write_bytes(content)
        (self.repository / "governance-bindings.md").unlink()
        (self.repository / "governance-bindings.md").symlink_to(first_target.name)
        first = load(self.repository).observed_state
        (self.repository / "governance-bindings.md").unlink()
        (self.repository / "governance-bindings.md").symlink_to(second_target.name)
        second = load(self.repository).observed_state
        self.assertNotEqual(first.identity, second.identity)
        self.assertIn("governance-bindings.md", second.scope_paths)
        self.assertIn("bindings-b.md", second.scope_paths)

    def test_ignored_symlinked_bootstrap_target_bytes_are_observed(self) -> None:
        self.configure()
        target = self.repository / ".governance" / "bootstrap.md"
        target.parent.mkdir()
        payload = (self.repository / "AGENTS.md").read_text(encoding="utf-8")
        target.write_text(
            f"{payload}# fixture revision: first\n", encoding="utf-8"
        )
        (self.repository / ".gitignore").write_text(
            ".governance/\n", encoding="utf-8"
        )
        (self.repository / "AGENTS.md").unlink()
        (self.repository / "AGENTS.md").symlink_to(".governance/bootstrap.md")

        first = load(self.repository).observed_state
        self.assertIn("AGENTS.md", first.scope_paths)
        self.assertIn(".governance/bootstrap.md", first.scope_paths)

        target.write_text(
            target.read_text(encoding="utf-8").replace(
                "fixture revision: first", "fixture revision: second", 1
            ),
            encoding="utf-8",
        )
        second = load(self.repository).observed_state
        self.assertNotEqual(first.identity, second.identity)

    def test_root_bootstrap_symlink_binding_itself_is_observed(self) -> None:
        self.configure()
        governance = self.repository / ".governance"
        governance.mkdir()
        bootstrap = governance / "bootstrap.md"
        bootstrap.write_bytes((self.repository / "AGENTS.md").read_bytes())
        (governance / "binding-a").symlink_to("bootstrap.md")
        (governance / "binding-b").symlink_to("bootstrap.md")
        (self.repository / ".gitignore").write_text(
            ".governance/\n", encoding="utf-8"
        )
        carrier = self.repository / "AGENTS.md"
        carrier.unlink()
        carrier.symlink_to(".governance/binding-a")

        first = load(self.repository).observed_state
        carrier.unlink()
        carrier.symlink_to(".governance/binding-b")
        second = load(self.repository).observed_state

        self.assertEqual(first.scope_paths, second.scope_paths)
        self.assertIn("AGENTS.md", second.scope_paths)
        self.assertIn(".governance/bootstrap.md", second.scope_paths)
        self.assertNotEqual(first.identity, second.identity)

    def test_external_bootstrap_symlink_target_fails_closed(self) -> None:
        self.configure()
        outside = TemporaryDirectory()
        self.addCleanup(outside.cleanup)
        target = Path(outside.name) / "bootstrap.md"
        target.write_bytes((self.repository / "AGENTS.md").read_bytes())
        carrier = self.repository / "AGENTS.md"
        carrier.unlink()
        carrier.symlink_to(target)

        self.assertEqual(
            target.resolve(), governance_bootstrap.load(self.repository).carrier.resolve()
        )
        with self.assertRaises(RepositoryGovernanceStateError) as raised:
            load(self.repository)
        self.assertIs(
            RepositoryGovernanceStage.REPOSITORY_STATE,
            raised.exception.diagnostic.stage,
        )
        self.assertIn(
            "bootstrap-resolved target is outside the repository observation root",
            raised.exception.diagnostic.detail,
        )

    def test_construction_is_pure_and_does_not_execute_integrity_commands(self) -> None:
        marker = self.repository / "must-not-exist"
        self.configure(
            "repository_integrity",
            repository_command=("-c", f"open({os.fspath(marker)!r}, 'w').close()"),
        )
        before = subprocess.run(
            ["git", "-C", os.fspath(self.repository), "status", "--porcelain", "-z"],
            check=True,
            capture_output=True,
        ).stdout
        load(self.repository)
        after = subprocess.run(
            ["git", "-C", os.fspath(self.repository), "status", "--porcelain", "-z"],
            check=True,
            capture_output=True,
        ).stdout
        self.assertEqual(before, after)
        self.assertFalse(marker.exists())


if __name__ == "__main__":
    unittest.main()
