from __future__ import annotations

import hashlib
from importlib import resources
import inspect
import unittest

from proto_ring import _normative_terminology_unicode as unicode14


EXPECTED_WHITESPACE = frozenset(
    {
        0x0009,
        0x000A,
        0x000B,
        0x000C,
        0x000D,
        0x001C,
        0x001D,
        0x001E,
        0x001F,
        0x0020,
        0x0085,
        0x00A0,
        0x1680,
        0x2000,
        0x2001,
        0x2002,
        0x2003,
        0x2004,
        0x2005,
        0x2006,
        0x2007,
        0x2008,
        0x2009,
        0x200A,
        0x2028,
        0x2029,
        0x202F,
        0x205F,
        0x3000,
    }
)


class NormativeTerminologyUnicodeTests(unittest.TestCase):
    def test_vendored_data_identity(self) -> None:
        self.assertEqual("14.0.0", unicode14.UNICODE_VERSION)
        resource = resources.files("proto_ring").joinpath(
            "data", "unicode-14.0.0", "CaseFolding.txt"
        )
        data = resource.read_bytes()
        self.assertEqual(
            unicode14._CASE_FOLDING_SHA256,
            hashlib.sha256(data).hexdigest(),
        )
        self.assertEqual(b"# CaseFolding-14.0.0.txt", data.splitlines()[0])

    def test_exact_case_fold_vectors(self) -> None:
        vectors = {
            "ABC": "abc",
            "Straße": "strasse",
            "STRASSE": "strasse",
            "\u212a": "k",
            "\u0130": "i\u0307",
            "\U0001e4d0": "\U0001e4d0",
        }
        for source, expected in vectors.items():
            with self.subTest(source=source):
                self.assertEqual(expected, unicode14.casefold(source))

    def test_case_folding_does_not_normalize(self) -> None:
        composed = unicode14.casefold("é")
        decomposed = unicode14.casefold("e\u0301")
        self.assertEqual("é", composed)
        self.assertEqual("e\u0301", decomposed)
        self.assertNotEqual(composed, decomposed)

    def test_exact_contract_whitespace_set(self) -> None:
        self.assertEqual(EXPECTED_WHITESPACE, unicode14._WHITESPACE_CODEPOINTS)
        for codepoint in EXPECTED_WHITESPACE:
            with self.subTest(codepoint=f"U+{codepoint:04X}"):
                self.assertTrue(unicode14.is_whitespace(chr(codepoint)))
        self.assertFalse(unicode14.is_whitespace("\u180e"))
        self.assertFalse(unicode14.is_whitespace("A"))

    def test_strip_and_split_use_only_contract_whitespace(self) -> None:
        self.assertEqual(
            ["alpha", "beta"],
            unicode14.split_whitespace("alpha\u2007beta"),
        )
        self.assertEqual(
            ["alpha", "beta"],
            unicode14.split_whitespace("\u001calpha\u001fbeta\u3000"),
        )
        self.assertEqual(
            "alpha\u180ebeta",
            unicode14.strip("\u2007alpha\u180ebeta\u3000"),
        )
        self.assertEqual(
            ["alpha\u180ebeta"],
            unicode14.split_whitespace("alpha\u180ebeta"),
        )

    def test_runtime_uses_no_network_client_or_subprocess(self) -> None:
        source = inspect.getsource(unicode14)
        for forbidden in (
            "urllib",
            "requests",
            "http",
            "socket",
            "subprocess",
            "curl",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
