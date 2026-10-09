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


class Issue98ExactEvidenceBindingTransportBoundaryTest(unittest.TestCase):
    def test_invalid_transport_fails_before_semantic_evaluation(self) -> None:
        cases = (
            ("evidence-class-boolean", "evidence_class", True, TypeError),
            ("subject-integer", "subject_hex", 1, TypeError),
            # A wrong transport encoding is not a candidate opaque context value,
            # even when semantic context participation is disabled.
            ("ignored-context-integer", "context_hex", 1, TypeError),
            ("subject-invalid-hex", "subject_hex", "zz", ValueError),
            ("ignored-context-invalid-hex", "context_hex", "zz", ValueError),
        )
        for label, field, value, error_type in cases:
            with self.subTest(label=label):
                binding = copy.deepcopy(VALID_BINDING)
                binding[field] = value
                with patch.object(adapter, "evaluate") as evaluate:
                    with self.assertRaises(error_type):
                        adapter.binding_status(copy.deepcopy(VALID_REQUIREMENT), binding)
                    evaluate.assert_not_called()
        print(f"ISSUE98_TRANSPORT_BOUNDARY_TESTS={len(cases)}")
        print("ISSUE98_TRANSPORT_BOUNDARY_PASS=all")


if __name__ == "__main__":
    unittest.main()
