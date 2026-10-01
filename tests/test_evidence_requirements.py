from __future__ import annotations

from copy import deepcopy
from dataclasses import fields
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import yaml

from proto_ring import evidence_requirements, structured_data
from proto_ring.exact_evidence_binding import (
    BindingStatus,
    EvidenceBinding,
    EvidenceRequirement,
    evaluate,
)
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


class EvidenceRequirementsTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.repository = Path(temporary.name)
        self.registry_path = self.repository / "evidence.md"
        route = self.route()
        self.authority = GovernanceAuthorityProfile(
            self.repository,
            self.repository / "authority.md",
            1,
            {
                "registry": GovernedSource("registry", route),
                "owner": GovernedSource("owner", None),
                "secondary": GovernedSource("secondary", None),
                "non-authority": GovernedSource("non-authority", None),
                "logical": GovernedSource("logical", None),
                "protocol": GovernedSource("protocol", None),
                "candidates": GovernedSource("candidates", None),
            },
            {
                "evidence-registry": GovernedResponsibility(
                    "evidence-registry", {"registry": SourceRole.AUTHORITY}, ()
                ),
                "evidence-required": GovernedResponsibility(
                    "evidence-required",
                    {
                        "owner": SourceRole.AUTHORITY,
                        "secondary": SourceRole.SECONDARY_REPRESENTATION,
                        "non-authority": SourceRole.NON_AUTHORITATIVE,
                    },
                    (),
                ),
            },
        )
        self.payload: object = self.valid_payload()

    def route(self, path: Path | None = None) -> ResolvedGovernanceRoute:
        target = path or self.registry_path
        return ResolvedGovernanceRoute(
            ("capabilities", "evidence_requirements", "routes", "registry"),
            "evidence.md",
            target,
        )

    @staticmethod
    def single_requirement() -> dict[str, object]:
        return {
            "responsibility": "evidence-required",
            "instances": {"kind": "single"},
            "evidence_classes": {
                "kind": "explicit",
                "classes": ["assurance-decomposition"],
            },
            "subject": {"source": "owner"},
            "context": {"required": False},
            "candidates": {"source": "candidates"},
        }

    def valid_payload(self) -> dict[str, object]:
        return {
            "model_version": 1,
            "authority": {
                "responsibility": "evidence-registry",
                "source": "registry",
            },
            "requirements": {"gate-a": self.single_requirement()},
        }

    def write_registry(self, body: str = "# Evidence requirements\n") -> None:
        metadata = {"okf_version": "1.0", "evidence_requirements": self.payload}
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
    ) -> evidence_requirements.EvidenceRequirementRegistry:
        self.write_registry()
        return evidence_requirements.load(
            self.repository,
            route or self.route(),
            authority or self.authority,
            catalog,
        )

    def assert_fails(self, **kwargs: object) -> None:
        with self.assertRaises(evidence_requirements.EvidenceRequirementsError):
            self.load(**kwargs)  # type: ignore[arg-type]

    def test_public_model_is_exact_frozen_and_load_is_non_executing(self) -> None:
        marker = self.repository / "must-not-run"
        payload = self.valid_payload()
        payload["requirements"]["gate-a"]["subject"] = {  # type: ignore[index]
            "source": f"open({marker!r}, 'w')"
        }
        self.payload = self.valid_payload()
        loaded = self.load()
        self.assertFalse(marker.exists())
        self.assertEqual(1, loaded.model_version)
        self.assertEqual("registry", loaded.authority.source_id)
        self.assertEqual({"gate-a"}, set(loaded.requirements))
        self.assertEqual(
            [field.name for field in fields(evidence_requirements.PersistentEvidenceRequirement)],
            [
                "id", "responsibility_id", "target", "instances",
                "evidence_classes", "subject_source_id", "context",
                "candidate_source_id",
            ],
        )
        for model in (
            evidence_requirements.RegistryAuthority,
            evidence_requirements.RequirementInstantiation,
            evidence_requirements.EvidenceClassAdmission,
            evidence_requirements.ContextBinding,
            evidence_requirements.PersistentEvidenceRequirement,
            evidence_requirements.EvidenceRequirementRegistry,
        ):
            self.assertTrue(model.__dataclass_params__.frozen)

    def test_exact_single_and_source_instance_forms(self) -> None:
        single = self.load().requirements["gate-a"].instances
        self.assertIs(evidence_requirements.InstantiationKind.SINGLE, single.kind)
        self.assertIsNone(single.source_id)

        payload = self.valid_payload()
        payload["requirements"]["gate-a"]["instances"] = {  # type: ignore[index]
            "kind": "source", "source": "logical"
        }
        self.payload = payload
        source = self.load().requirements["gate-a"].instances
        self.assertIs(evidence_requirements.InstantiationKind.SOURCE, source.kind)
        self.assertEqual("logical", source.source_id)

    def test_exact_explicit_and_source_evidence_class_forms(self) -> None:
        explicit = self.load().requirements["gate-a"].evidence_classes
        self.assertEqual(frozenset({"assurance-decomposition"}), explicit.explicit_classes)

        payload = self.valid_payload()
        payload["requirements"]["gate-a"]["evidence_classes"] = {  # type: ignore[index]
            "kind": "source", "source": "secondary"
        }
        self.payload = payload
        source = self.load().requirements["gate-a"].evidence_classes
        self.assertIs(evidence_requirements.EvidenceClassKind.SOURCE, source.kind)
        self.assertEqual("secondary", source.source_id)

    def test_context_forms_preserve_not_required_and_required_unknown_boundary(self) -> None:
        not_required = self.load().requirements["gate-a"].context
        self.assertFalse(not_required.required)
        self.assertIsNone(not_required.source_id)

        payload = self.valid_payload()
        payload["requirements"]["gate-a"]["context"] = {  # type: ignore[index]
            "required": True, "source": "protocol"
        }
        self.payload = payload
        required = self.load().requirements["gate-a"].context
        self.assertTrue(required.required)
        self.assertEqual("protocol", required.source_id)

        concrete = EvidenceRequirement(
            frozenset({"assurance-decomposition"}), b"subject", None, True
        )
        self.assertIs(
            BindingStatus.UNDETERMINED,
            evaluate(concrete, EvidenceBinding("assurance-decomposition", b"subject", b"context")),
        )

    def test_referenced_sources_exist_without_universal_role_requirement(self) -> None:
        payload = self.valid_payload()
        requirement = payload["requirements"]["gate-a"]  # type: ignore[index]
        requirement["instances"] = {"kind": "source", "source": "logical"}
        requirement["evidence_classes"] = {"kind": "source", "source": "secondary"}
        requirement["subject"] = {"source": "non-authority"}
        requirement["context"] = {"required": True, "source": "protocol"}
        requirement["candidates"] = {"source": "candidates"}
        self.payload = payload
        loaded = self.load().requirements["gate-a"]
        self.assertEqual("logical", loaded.instances.source_id)
        self.assertEqual("secondary", loaded.evidence_classes.source_id)
        self.assertEqual("non-authority", loaded.subject_source_id)

    def test_registry_requirement_and_nested_exact_keys_fail_closed(self) -> None:
        cases = (
            ((), "extra"),
            (("authority",), "extra"),
            (("requirements", "gate-a"), "extra"),
            (("requirements", "gate-a", "subject"), "extra"),
            (("requirements", "gate-a", "candidates"), "extra"),
        )
        for path, key in cases:
            with self.subTest(path=path):
                payload = self.valid_payload()
                target: object = payload
                for segment in path:
                    target = target[segment]  # type: ignore[index]
                target[key] = None  # type: ignore[index]
                self.payload = payload
                self.assert_fails()

    def test_invalid_instance_class_and_context_combinations_fail_closed(self) -> None:
        mutations = (
            ("instances", {"kind": "single", "source": "logical"}),
            ("instances", {"kind": "source"}),
            ("instances", {"kind": "other"}),
            ("evidence_classes", {"kind": "explicit", "classes": []}),
            ("evidence_classes", {"kind": "explicit", "classes": ["x", "x"]}),
            ("evidence_classes", {"kind": "source"}),
            ("context", {"required": False, "source": "protocol"}),
            ("context", {"required": True}),
            ("context", {"required": "true", "source": "protocol"}),
        )
        for field, value in mutations:
            with self.subTest(field=field, value=value):
                payload = self.valid_payload()
                payload["requirements"]["gate-a"][field] = value  # type: ignore[index]
                self.payload = payload
                self.assert_fails()

    def test_all_foreign_references_fail_closed(self) -> None:
        paths = (
            ("requirements", "gate-a", "instances", "source"),
            ("requirements", "gate-a", "evidence_classes", "source"),
            ("requirements", "gate-a", "subject", "source"),
            ("requirements", "gate-a", "context", "source"),
            ("requirements", "gate-a", "candidates", "source"),
        )
        for path in paths:
            payload = self.valid_payload()
            requirement = payload["requirements"]["gate-a"]  # type: ignore[index]
            requirement["instances"] = {"kind": "source", "source": "logical"}
            requirement["evidence_classes"] = {"kind": "source", "source": "secondary"}
            requirement["context"] = {"required": True, "source": "protocol"}
            target: object = payload
            for segment in path[:-1]:
                target = target[segment]  # type: ignore[index]
            target[path[-1]] = "missing"  # type: ignore[index]
            self.payload = payload
            with self.subTest(path=path):
                self.assert_fails()

    def catalog(self, responsibilities: frozenset[str]) -> GovernedObjectCatalog:
        governed_object = GovernedObject("contract", responsibilities, frozenset())
        return GovernedObjectCatalog(
            self.repository,
            self.repository / "objects.md",
            1,
            {"contracts": GovernedInterface("contracts", {"contract": governed_object})},
        )

    def test_optional_target_requires_existing_participating_object(self) -> None:
        payload = self.valid_payload()
        payload["requirements"]["gate-a"]["target"] = {  # type: ignore[index]
            "interface": "contracts", "object": "contract"
        }
        self.payload = payload
        self.assert_fails()
        loaded = self.load(catalog=self.catalog(frozenset({"evidence-required"})))
        self.assertEqual("contract", loaded.requirements["gate-a"].target.object_id)  # type: ignore[union-attr]
        self.assert_fails(catalog=self.catalog(frozenset({"evidence-registry"})))

    def test_registry_authority_and_repository_boundaries_fail_closed(self) -> None:
        payload = self.valid_payload()
        payload["authority"]["source"] = "owner"  # type: ignore[index]
        self.payload = payload
        self.assert_fails()
        for source in (
            GovernedSource("registry", None),
            GovernedSource("registry", self.route(self.repository / "other.md")),
        ):
            authority = deepcopy(self.authority)
            authority.sources["registry"] = source
            self.payload = self.valid_payload()
            self.assert_fails(authority=authority)
        foreign = deepcopy(self.authority)
        foreign = GovernanceAuthorityProfile(
            self.repository / "foreign", foreign.carrier, 1,
            foreign.sources, foreign.responsibilities,
        )
        self.assert_fails(authority=foreign)

    def test_turnlock_and_ruu_patterns_are_expressible_without_local_semantics(self) -> None:
        payload = self.valid_payload()
        current = self.single_requirement()
        protocol = deepcopy(current)
        protocol["context"] = {"required": True, "source": "protocol"}
        registered = deepcopy(current)
        registered["instances"] = {"kind": "source", "source": "logical"}
        registered["evidence_classes"] = {"kind": "source", "source": "secondary"}
        recorded = deepcopy(registered)
        recorded["evidence_classes"] = {"kind": "explicit", "classes": ["recorded_output"]}
        payload["requirements"] = {
            "current-subject": current,
            "current-protocol": protocol,
            "registered-artifacts": registered,
            "recorded-output": recorded,
        }
        self.payload = payload
        loaded = self.load()
        self.assertEqual(4, len(loaded.requirements))
        self.assertIsNone(loaded.requirements["current-subject"].target)
        self.assertEqual(
            frozenset({"recorded_output"}),
            loaded.requirements["recorded-output"].evidence_classes.explicit_classes,
        )

    def test_structured_data_body_io_and_empty_registry_boundaries(self) -> None:
        self.payload = {**self.valid_payload(), "requirements": {}}
        self.write_registry("evidence_requirements:\n  model_version: wrong\n")
        self.assertEqual({}, evidence_requirements.load(
            self.repository, self.route(), self.authority
        ).requirements)
        with mock.patch.object(Path, "read_bytes", side_effect=OSError("denied")):
            self.assert_fails()
        with mock.patch.object(
            structured_data,
            "parse_frontmatter_bytes",
            side_effect=structured_data.StructuredDataError("bad"),
        ):
            self.assert_fails()


if __name__ == "__main__":
    unittest.main()
