from __future__ import annotations

import copy
import unittest
from unittest.mock import patch

import conformance_python_adapter_66_evidence as adapter


VALID_REQUIREMENT = {
    "admitted_classes": ["check"],
    "subject_hex": "7375626a656374",
    "context_required": False,
    "context_hex": None,
}
VALID_BINDING = {
    "evidence_class": "check",
    "subject_hex": "7375626a656374",
    "context_hex": None,
}
DUPLICATE_REQUIREMENT = {
    **VALID_REQUIREMENT,
    "admitted_classes": ["check", "check"],
}


class Issue66ExactEvidenceBindingCombinedTransportBoundaryTest(unittest.TestCase):
    def test_combined_transport_failures_precede_semantic_rejection(self) -> None:
        cases = (
            ("binding-evidence-class-boolean", "binding", "evidence_class", True, TypeError),
            ("binding-subject-integer", "binding", "subject_hex", 1, TypeError),
            ("binding-subject-invalid-hex", "binding", "subject_hex", "zz", ValueError),
            ("binding-context-invalid-hex", "binding", "context_hex", "zz", ValueError),
            ("requirement-subject-integer", "requirement", "subject_hex", 1, TypeError),
            ("requirement-subject-invalid-hex", "requirement", "subject_hex", "zz", ValueError),
            (
                "requirement-context-required-string",
                "requirement",
                "context_required",
                "false",
                TypeError,
            ),
            ("requirement-context-invalid-hex", "requirement", "context_hex", "zz", ValueError),
        )
        harness_failures = 0
        controlled_rejections = 0
        for label, target, field, value, error_type in cases:
            with self.subTest(label=label):
                requirement = copy.deepcopy(DUPLICATE_REQUIREMENT)
                binding = copy.deepcopy(VALID_BINDING)
                target_value = requirement if target == "requirement" else binding
                target_value[field] = value
                with patch.object(adapter, "evaluate") as evaluate:
                    with self.assertRaises(error_type):
                        result = adapter.binding_status(requirement, binding)
                        if result is None:
                            controlled_rejections += 1
                    evaluate.assert_not_called()
                harness_failures += 1

        with patch.object(adapter, "evaluate") as evaluate:
            self.assertIsNone(
                adapter.binding_status(
                    copy.deepcopy(DUPLICATE_REQUIREMENT),
                    copy.deepcopy(VALID_BINDING),
                )
            )
            evaluate.assert_not_called()

        print(f"ISSUE66_EEB_COMBINED_TRANSPORT_CASES={len(cases)}")
        print(
            "ISSUE66_EEB_COMBINED_TRANSPORT_HARNESS_FAILURES="
            f"{harness_failures}"
        )
        print(
            "ISSUE66_EEB_COMBINED_TRANSPORT_CONTROLLED_REJECTIONS="
            f"{controlled_rejections}"
        )
        print("ISSUE66_EEB_DUPLICATE_CLASSES_SEMANTIC_REJECTION=PASS")


if __name__ == "__main__":
    unittest.main()
