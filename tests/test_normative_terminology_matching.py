from __future__ import annotations

import unittest

from proto_ring.normative_terminology import discover_occurrences, parse_registry
from test_normative_terminology import FORMAT, POLICY, SPEC


class TerminologyMatchingTests(unittest.TestCase):
    def occurrences(self, specification: str):
        entries, registry_errors = parse_registry(specification, FORMAT)
        self.assertEqual([], registry_errors)
        occurrences, errors = discover_occurrences(specification, entries, POLICY)
        self.assertEqual([], errors)
        return occurrences

    def candidate_is_discovered(self, text: str) -> bool:
        addition = f"# 4. Candidate\n\n{text}\n\n"
        changed = SPEC.replace("# 8. Summary", addition + "# 8. Summary")
        return any(
            not item.canonical and item.section == "4"
            for item in self.occurrences(changed)
        )

    def test_prose_and_equation_cues_are_discovered(self) -> None:
        additions = """
# 4. Candidate forms

Alpha means one candidate.

Alpha is defined as another candidate.

```
alpha = candidate equation
```

"""
        changed = SPEC.replace("# 8. Summary", additions + "# 8. Summary")
        candidates = [
            item
            for item in self.occurrences(changed)
            if not item.canonical and item.section == "4"
        ]
        self.assertEqual(3, len(candidates))
        self.assertTrue(all(item.concepts == ("alpha",) for item in candidates))

    def test_expression_boundaries_aliases_and_deprecated_wording(self) -> None:
        additions = """
# 4. Boundaries

Alphabet means a different token.

Kalpha is a Unicode-boundary definition.

A first concept is an alias definition.

Old alpha refers to deprecated wording.

"""
        changed = SPEC.replace("# 8. Summary", additions + "# 8. Summary")
        candidates = [
            item
            for item in self.occurrences(changed)
            if not item.canonical and item.section == "4"
        ]
        self.assertEqual(
            {
                "a9b7bb1a649e30f697695f1e1246d4299f6535f03a9ee1292adad2d5b4ea231c",
                "d33b9d44e6bbfbf189ce953bb687c2468923d97426cbc439d50d750074b052dc",
                "1587d875d692908d0ec1fb63024776f319eebd7d1a26a46ed74223239e545523",
            },
            {item.fingerprint for item in candidates},
        )
        self.assertNotIn(
            "90f9ee3fee7215fb6c6e59af9975e0bb42b75d46090108926b6cb9d210def3f1",
            {item.fingerprint for item in candidates},
        )

    def test_assignment_cues_and_tilde_fences_preserve_boundaries(self) -> None:
        additions = """
# 4. Equations

~~~text
alpha := assigned concept
~~~

alpha == comparison only

Alpha is defined by its governing paragraph.

"""
        changed = SPEC.replace("# 8. Summary", additions + "# 8. Summary")
        candidates = [
            item
            for item in self.occurrences(changed)
            if not item.canonical and item.section == "4"
        ]
        self.assertEqual(
            {
                "3ff9e624cf0c25261e0bdf56fdb4f4e124d7892dcc710de80c267ecd277d52b9",
                "f5e0a025760c4effeea055d47afaa4b3cf19daa112f2e70c65ac993a6864f1f6",
            },
            {item.fingerprint for item in candidates},
        )
        self.assertNotIn(
            "2e188172665664998788c4875849f208f1adbb0fef0eb3b8943e6d57dabc349f",
            {item.fingerprint for item in candidates},
        )

    def test_expression_matching_uses_full_unicode_casefold(self) -> None:
        changed = SPEC.replace("| `alpha` | `alpha` |", "| `alpha` | `Straße` |")
        changed = changed.replace("An **alpha** is", "A **Straße** is", 1)
        changed = changed.replace("Alpha refers", "Straße refers", 1)
        changed = changed.replace(
            "# 8. Summary",
            "# 4. Casefold\n\nSTRASSE means an expanded casefold definition.\n\n# 8. Summary",
        )
        candidates = [
            item
            for item in self.occurrences(changed)
            if not item.canonical and item.section == "4"
        ]
        self.assertEqual(
            {"1d5c1c9468d5d66f719d8fc9ec6f68eff36ed848ba5b46e62bda0b03159ee251"},
            {item.fingerprint for item in candidates},
        )

    def test_punctuation_bearing_canonical_alias_and_deprecated_terms(self) -> None:
        changed = SPEC.replace("| `alpha` | `alpha` |", "| `alpha` | `snake_case` |")
        changed = changed.replace("`first concept`", "`~/.config`", 1)
        changed = changed.replace("`old alpha`", "`old_name`", 1)
        changed = changed.replace("An **alpha** is", "A **snake_case** is", 1)
        changed = changed.replace("Alpha refers", "snake_case refers", 1)
        changed = changed.replace(
            "# 8. Summary",
            "# 4. Punctuation\n\n"
            "snakecase means a different concept.\n\n"
            "`~/.config` means the alias concept.\n\n"
            "~~old_name~~ refers to deprecated wording.\n\n"
            "# 8. Summary",
        )
        occurrences = self.occurrences(changed)
        canonical = [
            item
            for item in occurrences
            if item.canonical and item.concepts == ("alpha",)
        ]
        self.assertEqual(1, len(canonical))
        candidates = {
            item.fingerprint
            for item in occurrences
            if not item.canonical and item.section == "4"
        }
        self.assertEqual(
            {
                "d47e00cfc44c901d85571f902666780fe1e116f74556db4bd8dc632d6bd7c0f0",
                "1f2c6631bcc253fe9fa34cc77efa68dcd5e150d25c287ae2118ba4bf391e57c7",
            },
            candidates,
        )
        self.assertNotIn(
            "6f331132eff3cb0cb9f0eb019cb5a18eae0a1ea29756eabf19d0063e539a14d7",
            candidates,
        )

    def test_curly_single_quotes_are_definition_delimiters(self) -> None:
        additions = """
# 4. Curly quotes

‘Alpha’ is a curly quoted definition.

‘alpha’ = curly quoted equation

"""
        changed = SPEC.replace("# 8. Summary", additions + "# 8. Summary")
        candidates = [
            item
            for item in self.occurrences(changed)
            if not item.canonical and item.section == "4"
        ]
        self.assertEqual(
            {
                "1e008bd8b749a8ed67961be05ba5da0de06dcbb4dba82b8270470767d81c4676",
                "c038b075eef645f9d111ac59f9fcd71dc7bfac46f5cfcd3e31449269894fd16c",
            },
            {item.fingerprint for item in candidates},
        )

    def test_mean_and_means_are_exact_ascii_case_insensitive_cues(self) -> None:
        for cue in ("mean", "means", "MEAN", "MEANS"):
            with self.subTest(cue=cue):
                self.assertTrue(
                    self.candidate_is_discovered(f"Alpha {cue} one candidate.")
                )
        for noncue in ("meaning", "meant"):
            with self.subTest(noncue=noncue):
                self.assertFalse(
                    self.candidate_is_discovered(f"Alpha {noncue} one candidate.")
                )

    def test_definition_cues_use_ascii_case_only(self) -> None:
        self.assertFalse(self.candidate_is_discovered("Alpha iſ one candidate."))
        self.assertFalse(self.candidate_is_discovered("Alpha İs one candidate."))
        self.assertTrue(self.candidate_is_discovered("Alpha IS one candidate."))
        self.assertTrue(self.candidate_is_discovered("Alpha Is one candidate."))

    def test_cue_boundary_is_ascii_defined(self) -> None:
        for text in ("Alpha is_name", "Alpha is9", "Alpha isa"):
            with self.subTest(text=text):
                self.assertFalse(self.candidate_is_discovered(text))
        for text in (
            "Alpha is: one.",
            "Alpha is. one.",
            "Alpha isé candidate.",
            "Alpha is\U0001e4d0 candidate.",
        ):
            with self.subTest(text=text):
                self.assertTrue(self.candidate_is_discovered(text))

    def test_multiword_cues_use_contract_whitespace(self) -> None:
        self.assertTrue(
            self.candidate_is_discovered(
                "Alpha is\u2007defined\u2007as one candidate."
            )
        )
        self.assertTrue(
            self.candidate_is_discovered("Alpha refers\u001cto one candidate.")
        )

    def test_equations_use_contract_whitespace(self) -> None:
        for operator in ("=", ":="):
            with self.subTest(operator=operator):
                self.assertTrue(
                    self.candidate_is_discovered(
                        f"\u2007*Alpha*\u2007{operator}\u2007one candidate"
                    )
                )
        self.assertFalse(
            self.candidate_is_discovered("\u2007*Alpha*\u2007==\u2007comparison")
        )


if __name__ == "__main__":
    unittest.main()
