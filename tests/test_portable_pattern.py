from __future__ import annotations

import unittest

from proto_ring import portable_pattern
from proto_ring.portable_pattern import (
    PortablePattern,
    PortablePatternError,
    PortablePatternMatch,
    full_match,
)


class PortablePatternApiTests(unittest.TestCase):
    def test_public_surface_and_error_boundary_are_exact(self) -> None:
        self.assertEqual(
            [
                "PortablePattern",
                "PortablePatternError",
                "PortablePatternMatch",
                "full_match",
            ],
            portable_pattern.__all__,
        )
        self.assertTrue(issubclass(PortablePatternError, ValueError))

    def test_properties_and_top_level_operation(self) -> None:
        pattern = PortablePattern(r"(?P<first>a)(?P<second>b)")
        self.assertEqual(r"(?P<first>a)(?P<second>b)", pattern.source)
        self.assertEqual(2, pattern.capture_count)
        self.assertEqual(("first", "second"), pattern.capture_names)
        direct = pattern.full_match("ab")
        top_level = full_match(pattern.source, "ab")
        self.assertIsInstance(direct, PortablePatternMatch)
        self.assertEqual(direct, top_level)
        with self.assertRaises(AttributeError):
            pattern.source = "changed"  # type: ignore[misc]

    def test_non_string_and_surrogate_domains_are_rejected(self) -> None:
        with self.assertRaises(PortablePatternError):
            PortablePattern(1)  # type: ignore[arg-type]
        with self.assertRaises(PortablePatternError):
            PortablePattern("\ud800")
        pattern = PortablePattern("a")
        with self.assertRaises(PortablePatternError):
            pattern.full_match(1)  # type: ignore[arg-type]
        with self.assertRaises(PortablePatternError):
            pattern.full_match("\udfff")


class PortablePatternSyntaxTests(unittest.TestCase):
    def assert_invalid(self, *patterns: str) -> None:
        for pattern in patterns:
            with self.subTest(pattern=pattern):
                with self.assertRaises(PortablePatternError):
                    PortablePattern(pattern)

    def test_exact_literal_and_unicode_matching(self) -> None:
        pattern = PortablePattern("abc")
        self.assertIsNotNone(pattern.full_match("abc"))
        self.assertIsNone(pattern.full_match("xabc"))
        self.assertIsNone(pattern.full_match("abcx"))
        self.assertIsNotNone(PortablePattern("é").full_match("é"))
        self.assertIsNone(PortablePattern("é").full_match("e\u0301"))

    def test_anchors_are_exact_and_end_has_no_final_lf_exception(self) -> None:
        pattern = PortablePattern("^abc$")
        self.assertIsNotNone(pattern.full_match("abc"))
        self.assertIsNone(pattern.full_match("abc\n"))
        self.assertIsNone(pattern.full_match("xabc"))

    def test_dot_is_reserved_unless_escaped(self) -> None:
        self.assertIsNotNone(PortablePattern(r"\.").full_match("."))
        self.assertIsNone(PortablePattern(r"\.").full_match("x"))
        self.assert_invalid(".")

    def test_character_classes_and_exact_dash_algorithm(self) -> None:
        positive = (
            ("[a-z]", "m"),
            ("[A-Z]", "M"),
            ("[0-9]", "5"),
            ("[a-z0-9-]", "-"),
            ("[a-z0-9-]", "q"),
            ("[^`]", "x"),
            (r"[\-]", "-"),
            (r"[\]]", "]"),
            (r"[\\]", "\\"),
            ("[.]", "."),
            ("[-a]", "-"),
            ("[-a]", "a"),
            ("[a-]", "-"),
            ("[a-]", "a"),
        )
        for source, text in positive:
            with self.subTest(source=source, text=text):
                self.assertIsNotNone(PortablePattern(source).full_match(text))
        self.assertIsNone(PortablePattern("[^`]").full_match("`"))
        self.assert_invalid("[]", "[^]", "[z-a]", "[é-ê]", "[a-b-c]")

    def test_empty_sequences_groups_and_alternatives(self) -> None:
        self.assertIsNotNone(PortablePattern("").full_match(""))
        empty_group = PortablePattern("()")
        match = empty_group.full_match("")
        self.assertIsNotNone(match)
        assert match is not None
        self.assertEqual("", match.group(1))
        for pattern in ("a|", "|a", "(a|)"):
            with self.subTest(pattern=pattern):
                self.assertIsNotNone(PortablePattern(pattern).full_match(""))
                self.assertIsNotNone(PortablePattern(pattern).full_match("a"))

    def test_group_forms_and_capture_metadata(self) -> None:
        match = PortablePattern("(a)(b)").full_match("ab")
        self.assertIsNotNone(match)
        assert match is not None
        self.assertEqual("a", match.group(1))
        self.assertEqual("b", match.group(2))
        nested = PortablePattern("((a))").full_match("a")
        self.assertIsNotNone(nested)
        assert nested is not None
        self.assertEqual("a", nested.group(1))
        self.assertEqual("a", nested.group(2))
        named = PortablePattern(r"(?P<first>a)(?P<second>b)")
        result = named.full_match("ab")
        self.assertIsNotNone(result)
        assert result is not None
        self.assertEqual(result.group(1), result.group("first"))
        self.assertEqual(result.group(2), result.group("second"))
        self.assert_invalid(r"(?P<x>a)(?P<x>b)", r"(?P<1x>a)")

    def test_absent_empty_and_invalid_capture_lookups(self) -> None:
        absent = PortablePattern("(a)?b").full_match("b")
        empty = PortablePattern("()b").full_match("b")
        self.assertIsNotNone(absent)
        self.assertIsNotNone(empty)
        assert absent is not None and empty is not None
        self.assertIsNone(absent.group(1))
        self.assertEqual("", empty.group(1))
        for key in (True, object()):
            with self.subTest(key=key):
                with self.assertRaises(TypeError):
                    absent.group(key)  # type: ignore[arg-type]
        for index in (0, -1, 2):
            with self.subTest(index=index):
                with self.assertRaises(IndexError):
                    absent.group(index)
        with self.assertRaises(KeyError):
            absent.group("missing")

    def test_alternation_order_uses_first_complete_derivation(self) -> None:
        match = PortablePattern(r"(?P<x>a|aa)(?P<y>a|)").full_match("aa")
        self.assertIsNotNone(match)
        assert match is not None
        self.assertEqual("a", match.group("x"))
        self.assertEqual("a", match.group("y"))
        self.assertIsNotNone(PortablePattern("a|b").full_match("b"))

    def test_greedy_backtracking_and_repeated_capture_semantics(self) -> None:
        greedy = PortablePattern(r"(?P<x>a*)a").full_match("aaa")
        repeated = PortablePattern(r"(?P<x>a)+").full_match("aaa")
        retained = PortablePattern(r"(?:(?P<x>a)?b)+").full_match("abb")
        self.assertIsNotNone(greedy)
        self.assertIsNotNone(repeated)
        self.assertIsNotNone(retained)
        assert greedy is not None and repeated is not None and retained is not None
        self.assertEqual("aa", greedy.group("x"))
        self.assertEqual("a", repeated.group("x"))
        self.assertEqual("a", retained.group("x"))

    def test_quantifier_forms_and_invalid_extensions(self) -> None:
        valid = (
            ("a?", ""),
            ("a*", "aaa"),
            ("a+", "a"),
            ("a{0}", ""),
            ("a{2}", "aa"),
            ("a{2,}", "aaa"),
            ("a{2,4}", "aaaa"),
        )
        for source, text in valid:
            with self.subTest(source=source):
                self.assertIsNotNone(PortablePattern(source).full_match(text))
        self.assert_invalid(
            "a{01}", "a{00}", "a{+1}", "a{-1}", "a{,1}", "a{}",
            "a{2,1}", "a**", "a*+", "a*?", "a??", "a+?", "a{2}?",
            "^*", "$+",
        )

    def test_unbounded_nullable_rejected_but_bounded_admitted(self) -> None:
        self.assert_invalid("()*", "()+", "(){1,}", "(a?)*", "(a|)*")
        self.assertIsNotNone(PortablePattern("(){0}").full_match(""))
        self.assertIsNotNone(PortablePattern("(){2}").full_match(""))
        self.assertIsNotNone(PortablePattern("(a?){3}").full_match(""))

    def test_bounded_nullable_empty_capture_repeats_finitely(self) -> None:
        match = PortablePattern("(){2}").full_match("")
        self.assertIsNotNone(match)
        assert match is not None
        self.assertEqual("", match.group(1))

    def test_bounded_optional_child_reports_final_empty_participation(self) -> None:
        for text in ("", "a"):
            with self.subTest(text=text):
                match = PortablePattern("(a?){3}").full_match(text)
                self.assertIsNotNone(match)
                assert match is not None
                self.assertEqual("", match.group(1))

    def test_bounded_nullable_large_count_uses_no_repeat_recursion(self) -> None:
        for source in ("(){1500}", "(a?){1500}"):
            with self.subTest(source=source):
                match = PortablePattern(source).full_match("")
                self.assertIsNotNone(match)
                assert match is not None
                self.assertEqual("", match.group(1))

    def test_unsupported_native_regex_features_are_rejected(self) -> None:
        self.assert_invalid(
            r"\d", r"\w", r"\s", r"\1", "(?=a)", "(?!a)",
            "(?<=a)", "(?<!a)", "(?i:a)", "(?i)", "(?#x)",
            "(?>a)", "(?|a)", "(?P=x)",
        )

    def test_huge_consuming_minimum_is_pruned_without_runtime_digit_parsing(self) -> None:
        bound = "1" + "0" * 4300
        pattern = PortablePattern(f"a{{{bound}}}")
        self.assertIsNone(pattern.full_match("a"))


class PortablePatternConsumerVectorTests(unittest.TestCase):
    def test_adr_filename_vector(self) -> None:
        pattern = PortablePattern(
            r"^adr-(?P<number>[0-9]{3})-(?P<slug>[a-z0-9]+(?:-[a-z0-9]+)*)\.md$"
        )
        match = pattern.full_match("adr-001-example-name.md")
        self.assertIsNotNone(match)
        assert match is not None
        self.assertEqual("001", match.group("number"))
        self.assertEqual("example-name", match.group("slug"))

    def test_adr_id_and_terminology_key_vectors(self) -> None:
        self.assertIsNotNone(full_match(r"^ADR-[0-9]{3}$", "ADR-001"))
        self.assertIsNone(full_match(r"^ADR-[0-9]{3}$", "ADR-01"))
        self.assertIsNotNone(full_match(r"^[a-z][a-z0-9-]*$", "alpha-key"))

    def test_terminology_anchor_vectors(self) -> None:
        link = full_match(
            r"\[`[^`]+`\]\(#(term-[a-z0-9-]+)\)",
            "[`term-alpha`](#term-alpha)",
        )
        anchor = full_match(
            r'<a id="(term-[a-z0-9-]+)"></a>',
            '<a id="term-alpha"></a>',
        )
        self.assertIsNotNone(link)
        self.assertIsNotNone(anchor)
        assert link is not None and anchor is not None
        self.assertEqual("term-alpha", link.group(1))
        self.assertEqual("term-alpha", anchor.group(1))


if __name__ == "__main__":
    unittest.main()
