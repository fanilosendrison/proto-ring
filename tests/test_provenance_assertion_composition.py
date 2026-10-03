from __future__ import annotations

from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import yaml

from proto_ring import governance_authority, projection_registry
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
from proto_ring.repository_integrity import ConsumerIntegrityProfile, ProfileAuthority


class ProvenanceAssertionCompositionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.repository = Path(temporary.name)
        self.registry_path = self.repository / "projections.md"
        self.registry_route = ResolvedGovernanceRoute(
            ("capabilities", "projection_integrity", "routes", "registry"),
            "projections.md",
            self.registry_path,
        )
        self.integrity = ConsumerIntegrityProfile(
            self.repository,
            self.repository / "integrity.md",
            1,
            ProfileAuthority("repository-integrity", "canonical"),
            frozenset(),
            True,
            {"assertion_correspondence": mock.sentinel.validation},  # type: ignore[arg-type]
            ("assertion_correspondence",),
            "profile-identity",
        )

    @staticmethod
    def requirement(subject_identity: bytes | None) -> EvidenceRequirement:
        return EvidenceRequirement(
            frozenset({"embedded-assertion"}),
            subject_identity,
        )

    def authority_profile(
        self, *, canonical_has_authority: bool = True
    ) -> GovernanceAuthorityProfile:
        assertion_roles = {
            "secondary": SourceRole.SECONDARY_REPRESENTATION,
            "alternative": SourceRole.AUTHORITY,
        }
        if canonical_has_authority:
            assertion_roles["canonical"] = SourceRole.AUTHORITY
        return GovernanceAuthorityProfile(
            self.repository,
            self.repository / "authority.md",
            1,
            {
                "registry": GovernedSource("registry", self.registry_route),
                "canonical": GovernedSource("canonical", None),
                "alternative": GovernedSource("alternative", None),
                "secondary": GovernedSource("secondary", None),
                "candidate": GovernedSource("candidate", None),
            },
            {
                "projection-registry": GovernedResponsibility(
                    "projection-registry",
                    {"registry": SourceRole.AUTHORITY},
                    (),
                ),
                "embedded_assertion": GovernedResponsibility(
                    "embedded_assertion",
                    assertion_roles,
                    (),
                ),
            },
        )

    def write_projection_registry(self) -> None:
        metadata = {
            "okf_version": "1.0",
            "projection_registry": {
                "model_version": 1,
                "authority": {
                    "responsibility": "projection-registry",
                    "source": "registry",
                },
                "projections": {
                    "embedded-assertion": {
                        "responsibility": "embedded_assertion",
                        "canonical_source": "canonical",
                        "secondary_source": "secondary",
                        "mode": "mechanically_validated_maintained",
                        "validation": "assertion_correspondence",
                    }
                },
            },
        }
        self.registry_path.write_text(
            f"---\n{yaml.safe_dump(metadata, sort_keys=False)}---\n",
            encoding="utf-8",
        )

    def load_projection(
        self, authority: GovernanceAuthorityProfile
    ) -> projection_registry.ProjectionRegistry:
        self.write_projection_registry()
        return projection_registry.load(
            self.repository,
            self.registry_route,
            authority,
            self.integrity,
        )

    def test_known_exact_match(self) -> None:
        result = evaluate(
            self.requirement(b"expected-assertion"),
            EvidenceBinding("embedded-assertion", b"expected-assertion"),
        )

        self.assertIs(BindingStatus.MATCH, result)

    def test_known_exact_mismatch(self) -> None:
        result = evaluate(
            self.requirement(b"expected-assertion"),
            EvidenceBinding("embedded-assertion", b"other-assertion"),
        )

        self.assertIs(BindingStatus.MISMATCH, result)

    def test_observed_candidate_does_not_determine_unknown_current_identity(self) -> None:
        result = evaluate(
            self.requirement(None),
            EvidenceBinding("embedded-assertion", b"candidate-assertion"),
        )

        self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_undeclared_authority_is_not_a_source_role(self) -> None:
        state = governance_authority.role_of(
            self.authority_profile(),
            "embedded_assertion",
            "candidate",
        )

        self.assertIs(governance_authority.AuthorityLookupState.UNDECLARED, state)
        self.assertNotIsInstance(state, SourceRole)

    def test_known_projection_registry_relation_preserves_declared_identities(self) -> None:
        projection = self.load_projection(
            self.authority_profile()
        ).projections["embedded-assertion"]

        self.assertEqual("canonical", projection.canonical_source_id)
        self.assertEqual("secondary", projection.secondary_source_id)
        self.assertIs(
            projection_registry.ProjectionMode.MECHANICALLY_VALIDATED_MAINTAINED,
            projection.mode,
        )
        self.assertEqual("assertion_correspondence", projection.validation_id)

    def test_projection_registry_refuses_missing_canonical_authority_role(self) -> None:
        with self.assertRaises(projection_registry.ProjectionRegistryError):
            self.load_projection(self.authority_profile(canonical_has_authority=False))


if __name__ == "__main__":
    unittest.main()
