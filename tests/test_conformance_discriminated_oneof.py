import unittest
from unittest import mock

from jsonschema import Draft202012Validator, ValidationError

import conformance_corpus_test_support as support
import conformance_discriminated_oneof as oneof
from conformance_schema_test_support import (
    discriminated_branches,
    first_validation_error,
    normalize_validation_error,
)


class ConformanceDiscriminatedOneOfTest(unittest.TestCase):
    def test_json_equal_matches_profile_semantics(self):
        self.assertTrue(support._json_equal("a", "a"))
        self.assertFalse(support._json_equal("a", "b"))
        self.assertFalse(support._json_equal(True, 1))
        self.assertFalse(support._json_equal(False, 0))
        self.assertTrue(support._json_equal(1, 1))
        self.assertFalse(support._json_equal(1, 2))
        self.assertTrue(support._json_equal([1, {"value": False}], [1, {"value": False}]))
        self.assertFalse(support._json_equal([1, {"value": False}], [1, {"value": 0}]))

    def test_oneof_discriminator_eligibility_matches_profile_contract(self):
        branches = discriminated_branches()
        self.assertEqual(
            support._oneof_const_discriminator(branches),
            ("kind", ("A", "B", "C")),
        )
        multiple = [
            {"properties": {"kind": {"const": "A"}, "type": {"const": "X"}}},
            {"properties": {"kind": {"const": "B"}, "type": {"const": "Y"}}},
        ]
        self.assertEqual(
            support._oneof_const_discriminator(multiple),
            ("kind", ("A", "B")),
        )
        duplicate = [
            {"properties": {"kind": {"const": "A"}}},
            {"properties": {"kind": {"const": "A"}}},
        ]
        missing_properties = [branches[0], {"type": "object"}]
        missing_const = [
            branches[0], {"properties": {"kind": {"type": "string"}}},
        ]
        for one_of in (duplicate, missing_properties, missing_const, [], None):
            with self.subTest(one_of=one_of):
                self.assertIsNone(support._oneof_const_discriminator(one_of))

        uri = support._DRAFT_2020_12_SCHEMA_URI
        safe_root = {
            "$schema": uri,
            "$defs": {"payload": {"oneOf": branches}},
            "$ref": "#/$defs/payload",
        }
        self.assertTrue(support._schema_allows_discriminated_oneof_fastpath(safe_root))
        unsafe_roots = {
            "missing-root-schema": {"oneOf": branches},
            "draft4-root": {"$schema": "http://json-schema.org/draft-04/schema#", "oneOf": branches},
            "custom-root": {"$schema": "https://example.invalid/custom", "oneOf": branches},
            "nested-draft4": {"$schema": uri, "properties": {"x": {"$schema": "http://json-schema.org/draft-04/schema#"}}},
            "nested-custom": {"$schema": uri, "properties": {"x": {"$schema": "https://example.invalid/custom"}}},
            "nested-draft2020": {"$schema": uri, "properties": {"x": {"$schema": uri}}},
            "external-ref": {"$schema": uri, "$ref": "other.json#/$defs/x"},
            "external-dynamic-ref": {"$schema": uri, "$dynamicRef": "urn:example:x"},
        }
        for name, schema in unsafe_roots.items():
            with self.subTest(name=name):
                self.assertFalse(
                    support._schema_allows_discriminated_oneof_fastpath(schema)
                )

    def test_discriminated_oneof_valid_selected_branch_skips_stock_oneof(self):
        schema = {
            "$schema": support._DRAFT_2020_12_SCHEMA_URI,
            "oneOf": discriminated_branches(),
        }
        self.assertTrue(support._schema_allows_discriminated_oneof_fastpath(schema))
        with mock.patch.object(
            oneof, "_ORIGINAL_ONEOF", side_effect=AssertionError("stock oneOf invoked")
        ) as stock_oneof:
            support._DiscriminatedDraft202012Validator(schema).validate(
                {"kind": "B", "b": 2}
            )
        stock_oneof.assert_not_called()

    def test_discriminated_oneof_fallback_error_tree_matches_stock(self):
        schema = {
            "$schema": support._DRAFT_2020_12_SCHEMA_URI,
            "oneOf": discriminated_branches(),
        }
        duplicate_schema = {
            "$schema": support._DRAFT_2020_12_SCHEMA_URI,
            "oneOf": [
                {"properties": {"kind": {"const": "A"}}},
                {"properties": {"kind": {"const": "A"}}},
            ],
        }
        for name, candidate, instance in (
            ("selected-invalid", schema, {"kind": "B"}),
            ("unknown", schema, {"kind": "UNKNOWN"}),
            ("missing", schema, {"b": 2}),
            ("non-object", schema, 42),
            ("duplicate-const", duplicate_schema, {"kind": "A"}),
        ):
            with self.subTest(name=name):
                stock = first_validation_error(Draft202012Validator, candidate, instance)
                accelerated = first_validation_error(
                    support._DiscriminatedDraft202012Validator, candidate, instance
                )
                self.assertEqual(
                    normalize_validation_error(accelerated),
                    normalize_validation_error(stock),
                )

        uri = support._DRAFT_2020_12_SCHEMA_URI
        unsafe_cases = {
            "inherited-custom": (
                {
                    "$schema": "https://example.invalid/custom-dialect",
                    "type": "object",
                    "properties": {"payload": {"oneOf": discriminated_branches()}},
                    "required": ["payload"],
                },
                {"payload": {"kind": "B"}},
            ),
            "oneof-containing-custom": (
                {
                    "$schema": uri,
                    "type": "object",
                    "properties": {"payload": {"$schema": "https://example.invalid/custom-dialect", "oneOf": discriminated_branches()}},
                    "required": ["payload"],
                },
                {"payload": {"kind": "B"}},
            ),
            "draft4-branch": (
                {
                    "$schema": uri,
                    "oneOf": [
                        {**branch, "$schema": "http://json-schema.org/draft-04/schema#"}
                        for branch in discriminated_branches()
                    ],
                },
                {"kind": "B"},
            ),
            "draft4-discriminator": (
                {
                    "$schema": uri,
                    "oneOf": [
                        {
                            **branch,
                            "properties": {
                                **branch["properties"],
                                "kind": {
                                    **branch["properties"]["kind"],
                                    "$schema": "http://json-schema.org/draft-04/schema#",
                                },
                            },
                        }
                        for branch in discriminated_branches()
                    ],
                },
                {"kind": "B"},
            ),
        }
        for name, (candidate, instance) in unsafe_cases.items():
            with self.subTest(name=name):
                Draft202012Validator.check_schema(candidate)
                self.assertFalse(
                    support._schema_allows_discriminated_oneof_fastpath(candidate)
                )
                stock = first_validation_error(Draft202012Validator, candidate, instance)
                with mock.patch.object(
                    support,
                    "_DiscriminatedDraft202012Validator",
                    side_effect=AssertionError("custom validator instantiated"),
                ):
                    with self.assertRaises(ValidationError) as selected:
                        support._validate_schema_job(0, candidate, instance)
                self.assertEqual(
                    normalize_validation_error(selected.exception),
                    normalize_validation_error(stock),
                )

    def test_discriminated_validator_overrides_only_oneof(self):
        self.assertIs(
            support._DiscriminatedDraft202012Validator.VALIDATORS["oneOf"],
            support._discriminator_aware_one_of,
        )
        self.assertIs(
            Draft202012Validator.VALIDATORS["oneOf"], support._ORIGINAL_ONEOF
        )
        for keyword, handler in Draft202012Validator.VALIDATORS.items():
            if keyword != "oneOf":
                with self.subTest(keyword=keyword):
                    self.assertIs(
                        support._DiscriminatedDraft202012Validator.VALIDATORS[keyword],
                        handler,
                    )


if __name__ == "__main__":
    unittest.main()
