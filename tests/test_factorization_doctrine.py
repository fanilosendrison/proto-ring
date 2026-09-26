from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]


class FactorizationDoctrineTests(unittest.TestCase):
    def read_document(self, relative_path: str) -> str:
        return (REPOSITORY_ROOT / relative_path).read_text(encoding="utf-8")

    def assert_in_order(self, document: str, *fragments: str) -> None:
        previous_position = -1
        for fragment in fragments:
            position = document.index(fragment)
            self.assertGreater(position, previous_position, fragment)
            previous_position = position

    def test_readme_preserves_turnlock_first_extraction_flow(self) -> None:
        readme = self.read_document("README.md")
        self.assert_in_order(
            readme,
            "observe Turnlock governance",
            "identify the maximum consumer-independent extraction candidate",
            "confront the candidate with Ruu and other concrete consumers",
            "narrow false generalizations",
            "strengthen with better consumer-independent guarantees",
            "establish the proto-ring contract or mechanism",
            "validate it independently",
            "adopt it back into consumers without weakening their guarantees",
        )
        self.assertIn("Turnlock is the primary extraction source", readme)
        self.assertIn("it is not derived as `Turnlock ∩ Ruu`", readme)

    def test_normative_profile_rejects_symmetric_extraction(self) -> None:
        profile = self.read_document(
            "docs/repository-governance/proto-ring-engineering.md"
        )
        self.assertIn("Turnlock is currently the primary source", profile)
        self.assertIn(
            "The absence of an equivalent mechanism in Ruu MUST NOT by itself",
            profile,
        )
        self.assertIn("shared = intersection(Turnlock, Ruu)", profile)
        self.assertIn("Turnlock contains X\n≠\nX automatically belongs", profile)
        self.assertNotIn("observe concrete governance in multiple consumers", profile)
        self.assertNotIn(
            "Proto-ring factorization MUST NOT treat any consumer as a privileged reference",
            profile,
        )

    def test_historical_audit_identifies_current_normative_supersession(self) -> None:
        audit = self.read_document("docs/bootstrap/turnlock-extraction-audit.md")
        self.assertIn("Status: non-normative bootstrap analysis", audit)
        self.assertIn("Its extraction-method assumptions are", audit)
        self.assertIn("superseded by the current normative", audit)
        self.assertIn("Turnlock is the primary", audit)
        self.assertIn("absence of an equivalent Ruu mechanism does not block", audit)


if __name__ == "__main__":
    unittest.main()
