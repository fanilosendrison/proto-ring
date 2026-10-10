from __future__ import annotations

import json
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "conformance" / "v1"
AUTHORITY_SHA = "1f103322d6816effcc102e35cbdd1e14bd33ba84"
CONTRACT_PATH = "docs/contracts/repository-state.md"
NEW_VECTOR_IDS = {
    "repository-state.capture.ambient-global-ignore-config",
    "repository-state.capture.ambient-namespace-redirect",
    "repository-state.capture.ambient-object-database-redirect",
    "repository-state.capture.ambient-worktree-redirect",
    "repository-state.capture.concurrent-content-only-mutation",
    "repository-state.capture.concurrent-path-membership-mutation",
    "repository-state.capture.directory-mode-change",
    "repository-state.capture.final-symlink-outside-observed",
    "repository-state.capture.git-internal-unrelated-change",
    "repository-state.capture.index-mode-only",
    "repository-state.capture.index-object-only",
    "repository-state.capture.index-path-binding",
    "repository-state.capture.index-stage-nonzero",
    "repository-state.capture.info-exclude-untracked-still-governed",
    "repository-state.capture.intermediate-symlink-escape-rejected",
    "repository-state.capture.non-git-root",
    "repository-state.capture.path-case-distinct",
    "repository-state.capture.path-unicode-normalization-distinct",
    "repository-state.capture.path-whitespace-distinct",
    "repository-state.capture.repository-ignore-untracked-excluded",
    "repository-state.capture.scope-absolute-rejected",
    "repository-state.capture.scope-dot-segment-rejected",
    "repository-state.capture.scope-order-duplicate-identity",
    "repository-state.capture.scope-parent-segment-rejected",
    "repository-state.capture.tracked-repository-ignore",
    "repository-state.capture.untracked-content-mutation",
}
REQUIRED_DISTINCTION_FRAGMENTS = (
    "plain non-Git directory is rejected as root",
    "ambient worktree/repository redirection",
    "ambient index redirection",
    "ambient object-database redirection",
    "ambient namespace redirection",
    "tracked paths remain governed when repository ignore matches",
    "repository-owned .gitignore may exclude untracked paths",
    ".git/info/exclude cannot remove",
    "user-global ignore configuration cannot remove",
    "absolute explicit scope is rejected",
    "dot-segment explicit scope is rejected",
    "parent-traversal explicit scope is rejected",
    "final in-repository symlink object remains observable",
    "intermediate-parent symlink escape is rejected",
    "not created through trimming",
    "not created through case folding",
    "not created through Unicode normalization",
    "index object identity participates",
    "index mode participates",
    "non-stage-zero index stage participates",
    "index object-to-path association participates",
    "unrelated .git implementation bytes do not participate",
    "explicit directory relevant mode participates",
    "already-untracked content mutation changes state",
    "same exact state/effective observed path set yields equal opaque identity",
    "scope order/duplicate metadata does not alter identity",
    "content-only concurrent drift fails closed",
    "path-membership concurrent drift fails closed",
)


def load(relative: str):
    return json.loads((CORPUS / relative).read_text(encoding="utf-8"))


class RepositoryStateConformance109Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.matrix = load("cases/repository-state.capture.matrix.json")
        cls.responsibility = load(
            "responsibilities/repository-state.capture.json"
        )
        cls.coverage = load("coverage/repository-state.json")
        cls.index = load("index.json")

    def test_exact_vector_inventory_is_53_with_all_26_additions(self) -> None:
        vector_ids = [item["vector_id"] for item in self.matrix["vectors"]]
        self.assertEqual(53, len(vector_ids))
        self.assertEqual(vector_ids, sorted(vector_ids))
        self.assertEqual(NEW_VECTOR_IDS, NEW_VECTOR_IDS & set(vector_ids))
        self.assertEqual(26, len(NEW_VECTOR_IDS))

    def test_every_repository_state_authority_uses_current_sha(self) -> None:
        responsibility_authorities = [
            authority
            for authority in self.responsibility["authorities"]
            if authority.get("path") == CONTRACT_PATH
        ]
        self.assertTrue(responsibility_authorities)
        self.assertEqual(
            {AUTHORITY_SHA},
            {authority["commit"] for authority in responsibility_authorities},
        )
        for vector in self.matrix["vectors"]:
            expected_keys = set(
                vector["expected_observation"]["derivation"]["authority_keys"]
            )
            authorities = [
                authority
                for authority in vector["authorities"]
                if authority.get("path") == CONTRACT_PATH
            ]
            with self.subTest(vector=vector["vector_id"]):
                self.assertTrue(authorities)
                self.assertEqual(
                    {AUTHORITY_SHA},
                    {authority["commit"] for authority in authorities},
                )
                for authority in authorities:
                    key = (
                        f"repo:{authority['repository']}@{AUTHORITY_SHA}:"
                        f"{CONTRACT_PATH}#{authority['clause']}"
                    )
                    self.assertIn(key, expected_keys)
                self.assertFalse(
                    any(
                        old in key
                        for key in expected_keys
                        for old in ("@0b140d050587", "@5ba3ef457786")
                    )
                )

    def test_coverage_and_composition_classification_are_exact(self) -> None:
        self.assertEqual(AUTHORITY_SHA, self.coverage["contract"]["commit"])
        headings = {
            item["heading_path"]: item for item in self.coverage["headings"]
        }
        composition = headings["Composition boundaries"]
        self.assertEqual("context", composition["disposition"])
        self.assertEqual([], composition["vector_refs"])
        self.assertEqual([], composition["responsibility_ids"])
        self.assertIsNone(composition["coverage_assertion"])
        exact_path = headings["Exact path identity"]
        self.assertIn("mandatory #67 source audit", exact_path["coverage_assertion"])
        coherence = headings["Capture coherence"]
        self.assertIn("do not prove absence of arbitrary ABA", coherence["reason"])
        self.assertIn("separately established coherence basis", coherence["reason"])
        identity = headings["Opaque exact identity"]
        self.assertIn("does not prove that a coherence basis existed", identity["reason"])
        self.assertIn("does not prove absence of arbitrary ABA", identity["reason"])

    def test_snapshot_is_indexed_and_byte_exact(self) -> None:
        relative = (
            f"fixtures/authorities/{AUTHORITY_SHA}/{CONTRACT_PATH}"
        )
        self.assertIn(relative, self.index["authority_snapshot_files"])
        self.assertEqual(
            (ROOT / CONTRACT_PATH).read_bytes(),
            (CORPUS / relative).read_bytes(),
        )

    def test_expected_observations_do_not_prescribe_python_digest(self) -> None:
        encoded = json.dumps(
            [
                vector["expected_observation"]
                for vector in self.matrix["vectors"]
            ],
            ensure_ascii=False,
            sort_keys=True,
        ).lower()
        self.assertNotIn("identity_length", encoded)
        self.assertNotIn("sha-256", encoded)
        self.assertIsNone(
            re.search(r'"identity"\s*:\s*"[0-9a-f]{64}"', encoded)
        )
        self.assertNotIn("compatibility_obligation", encoded)

    def test_required_state_distinctions_include_issue_109_closure(self) -> None:
        distinctions = "\n".join(
            self.responsibility["required_state_distinctions"]
        )
        for fragment in REQUIRED_DISTINCTION_FRAGMENTS:
            with self.subTest(fragment=fragment):
                self.assertIn(fragment, distinctions)

    def test_coherence_metadata_does_not_require_mutation_isolation(self) -> None:
        documents = (self.matrix, self.responsibility, self.coverage)
        mentions: list[str] = []

        def collect(value) -> None:
            if isinstance(value, str):
                if "mutation isolation" in value.lower():
                    mentions.append(value.lower())
            elif isinstance(value, list):
                for item in value:
                    collect(item)
            elif isinstance(value, dict):
                for item in value.values():
                    collect(item)

        for document in documents:
            collect(document)
        self.assertTrue(mentions)
        for mention in mentions:
            with self.subTest(mention=mention):
                self.assertIn("one sufficient realization", mention)
                self.assertIn("coherence basis", mention)
                self.assertIn("not the normative mechanism", mention)


    def test_python_regressions_are_bootstrap_evidence_only(self) -> None:
        accounting = self.responsibility["source_test_accounting"]
        self.assertEqual(1, len(accounting))
        record = accounting[0]
        self.assertEqual("bootstrap_evidence_only", record["disposition"])
        self.assertEqual("tests/test_repository_state.py", record["path"])
        self.assertEqual(
            {
                "test_content_change_after_first_path_observation_fails_closed",
                "test_cross_path_nonexistent_combination_fails_closed",
            },
            set(record["methods"]),
        )
        self.assertIn("do not prove arbitrary ABA exclusion", record["reason"])
        self.assertIn("do not establish that a coherence basis existed", record["reason"])


if __name__ == "__main__":
    unittest.main()
