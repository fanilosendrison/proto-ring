from __future__ import annotations

import unittest

from proto_ring.exact_evidence_binding import (
    BindingStatus,
    EvidenceBinding,
    EvidenceRequirement,
    evaluate,
)


class EvidenceRequirementValidationTests(unittest.TestCase):
    def test_admitted_classes_must_be_frozenset(self) -> None:
        for value in ({"check"}, ("check",), ["check"], "check"):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    EvidenceRequirement(value, b"subject")  # type: ignore[arg-type]

    def test_admitted_classes_must_not_be_empty(self) -> None:
        with self.assertRaises(ValueError):
            EvidenceRequirement(frozenset(), b"subject")

    def test_admitted_class_members_must_be_strings(self) -> None:
        with self.assertRaises(ValueError):
            EvidenceRequirement(frozenset({"check", 1}), b"subject")  # type: ignore[arg-type]

    def test_admitted_class_members_must_not_be_empty(self) -> None:
        with self.assertRaises(ValueError):
            EvidenceRequirement(frozenset({""}), b"subject")

    def test_class_identifiers_are_not_trimmed_or_normalized(self) -> None:
        requirement = EvidenceRequirement(frozenset({" check "}), b"subject")

        self.assertEqual(frozenset({" check "}), requirement.admitted_classes)

    def test_non_none_subject_identity_must_be_bytes(self) -> None:
        for value in ("subject", bytearray(b"subject"), 1):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    EvidenceRequirement(
                        frozenset({"check"}), value  # type: ignore[arg-type]
                    )

    def test_non_none_subject_identity_must_not_be_empty(self) -> None:
        with self.assertRaises(ValueError):
            EvidenceRequirement(frozenset({"check"}), b"")

    def test_non_none_context_identity_must_be_bytes(self) -> None:
        for value in ("context", bytearray(b"context"), 1):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    EvidenceRequirement(
                        frozenset({"check"}),
                        b"subject",
                        value,  # type: ignore[arg-type]
                        True,
                    )

    def test_non_none_context_identity_must_not_be_empty(self) -> None:
        with self.assertRaises(ValueError):
            EvidenceRequirement(
                frozenset({"check"}), b"subject", b"", True
            )

    def test_context_required_must_be_bool(self) -> None:
        for value in (0, 1, "true", None):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    EvidenceRequirement(
                        frozenset({"check"}),
                        b"subject",
                        None,
                        value,  # type: ignore[arg-type]
                    )

    def test_context_identity_is_rejected_when_context_is_not_required(self) -> None:
        with self.assertRaises(ValueError):
            EvidenceRequirement(
                frozenset({"check"}), b"subject", b"context", False
            )

    def test_required_unknown_context_is_structurally_valid(self) -> None:
        requirement = EvidenceRequirement(
            frozenset({"check"}), b"subject", None, True
        )

        self.assertTrue(requirement.context_required)
        self.assertIsNone(requirement.context_identity)


class ExactEvidenceBindingEvaluationTests(unittest.TestCase):
    def requirement(
        self,
        *,
        admitted_classes: frozenset[str] = frozenset({"check"}),
        subject_identity: bytes | None = b"subject",
        context_identity: bytes | None = None,
        context_required: bool = False,
    ) -> EvidenceRequirement:
        return EvidenceRequirement(
            admitted_classes,
            subject_identity,
            context_identity,
            context_required,
        )

    def test_unknown_current_subject_is_undetermined(self) -> None:
        result = evaluate(
            self.requirement(subject_identity=None),
            EvidenceBinding("check", b"subject"),
        )

        self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_unknown_current_subject_precedes_class_mismatch(self) -> None:
        result = evaluate(
            self.requirement(subject_identity=None),
            EvidenceBinding("other", b"subject"),
        )

        self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_required_unknown_current_context_is_undetermined(self) -> None:
        result = evaluate(
            self.requirement(context_required=True, context_identity=None),
            EvidenceBinding("check", b"subject", b"context"),
        )

        self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_absent_evidence_is_undetermined(self) -> None:
        self.assertIs(
            BindingStatus.UNDETERMINED,
            evaluate(self.requirement(), None),
        )

    def test_missing_evidence_class_is_undetermined(self) -> None:
        result = evaluate(
            self.requirement(), EvidenceBinding(None, b"subject")
        )

        self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_malformed_or_empty_evidence_class_is_undetermined(self) -> None:
        for value in (1, b"check", ""):
            with self.subTest(value=value):
                result = evaluate(
                    self.requirement(),
                    EvidenceBinding(value, b"subject"),  # type: ignore[arg-type]
                )
                self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_missing_evidence_subject_is_undetermined(self) -> None:
        result = evaluate(
            self.requirement(), EvidenceBinding("check", None)
        )

        self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_malformed_or_empty_evidence_subject_is_undetermined(self) -> None:
        for value in ("subject", bytearray(b"subject"), b""):
            with self.subTest(value=value):
                result = evaluate(
                    self.requirement(),
                    EvidenceBinding("check", value),  # type: ignore[arg-type]
                )
                self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_required_missing_evidence_context_is_undetermined(self) -> None:
        result = evaluate(
            self.requirement(
                context_required=True, context_identity=b"context"
            ),
            EvidenceBinding("check", b"subject", None),
        )

        self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_required_malformed_or_empty_evidence_context_is_undetermined(
        self,
    ) -> None:
        requirement = self.requirement(
            context_required=True, context_identity=b"context"
        )
        for value in ("context", bytearray(b"context"), b""):
            with self.subTest(value=value):
                result = evaluate(
                    requirement,
                    EvidenceBinding(
                        "check", b"subject", value  # type: ignore[arg-type]
                    ),
                )
                self.assertIs(BindingStatus.UNDETERMINED, result)

    def test_unadmitted_evidence_class_is_mismatch(self) -> None:
        result = evaluate(
            self.requirement(), EvidenceBinding("other", b"subject")
        )

        self.assertIs(BindingStatus.MISMATCH, result)

    def test_subject_mismatch_is_mismatch(self) -> None:
        result = evaluate(
            self.requirement(), EvidenceBinding("check", b"other-subject")
        )

        self.assertIs(BindingStatus.MISMATCH, result)

    def test_required_context_mismatch_is_mismatch(self) -> None:
        result = evaluate(
            self.requirement(
                context_required=True, context_identity=b"context"
            ),
            EvidenceBinding("check", b"subject", b"other-context"),
        )

        self.assertIs(BindingStatus.MISMATCH, result)

    def test_exact_class_and_subject_without_required_context_match(self) -> None:
        result = evaluate(
            self.requirement(), EvidenceBinding("check", b"subject")
        )

        self.assertIs(BindingStatus.MATCH, result)

    def test_exact_class_subject_and_required_context_match(self) -> None:
        result = evaluate(
            self.requirement(
                context_required=True, context_identity=b"context"
            ),
            EvidenceBinding("check", b"subject", b"context"),
        )

        self.assertIs(BindingStatus.MATCH, result)

    def test_either_explicitly_admitted_class_may_match(self) -> None:
        requirement = self.requirement(
            admitted_classes=frozenset({"check", "review"})
        )

        for evidence_class in ("check", "review"):
            with self.subTest(evidence_class=evidence_class):
                result = evaluate(
                    requirement,
                    EvidenceBinding(evidence_class, b"subject"),
                )
                self.assertIs(BindingStatus.MATCH, result)

    def test_no_implicit_class_fallback(self) -> None:
        requirement = self.requirement(
            admitted_classes=frozenset({"check-v2"})
        )

        result = evaluate(
            requirement, EvidenceBinding("check", b"subject")
        )

        self.assertIs(BindingStatus.MISMATCH, result)

    def test_evidence_context_is_ignored_when_not_required(self) -> None:
        for value in (b"unrelated", "malformed", b""):
            with self.subTest(value=value):
                result = evaluate(
                    self.requirement(),
                    EvidenceBinding(
                        "check", b"subject", value  # type: ignore[arg-type]
                    ),
                )
                self.assertIs(BindingStatus.MATCH, result)

    def test_historical_match_does_not_authorize_changed_current_subject(self) -> None:
        evidence = EvidenceBinding("check", b"historical-subject")
        historical = self.requirement(subject_identity=b"historical-subject")
        current = self.requirement(subject_identity=b"current-subject")

        self.assertIs(BindingStatus.MATCH, evaluate(historical, evidence))
        self.assertIs(BindingStatus.MISMATCH, evaluate(current, evidence))

    def test_historical_match_does_not_authorize_changed_current_context(self) -> None:
        evidence = EvidenceBinding(
            "check", b"subject", b"historical-context"
        )
        historical = self.requirement(
            context_required=True,
            context_identity=b"historical-context",
        )
        current = self.requirement(
            context_required=True,
            context_identity=b"current-context",
        )

        self.assertIs(BindingStatus.MATCH, evaluate(historical, evidence))
        self.assertIs(BindingStatus.MISMATCH, evaluate(current, evidence))

    def test_public_api_is_exact(self) -> None:
        from proto_ring import exact_evidence_binding

        self.assertEqual(
            [
                "BindingStatus",
                "EvidenceRequirement",
                "EvidenceBinding",
                "evaluate",
            ],
            exact_evidence_binding.__all__,
        )


if __name__ == "__main__":
    unittest.main()
