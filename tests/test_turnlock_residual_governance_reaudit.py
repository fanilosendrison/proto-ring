from collections import Counter
from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
AUDIT_PATH = (
    REPOSITORY_ROOT
    / "docs"
    / "bootstrap"
    / "turnlock-residual-governance-reaudit.md"
)

EXPECTED_CANDIDATE_IDS = (
    "WORK-01",
    "WORK-02",
    "WORK-03",
    "WORK-04",
    "WORK-05",
    "WORK-06",
    "DISC-01",
    "DISC-02",
    "DISC-03",
    "ADR-01",
    "ADR-02",
    "ADR-03",
    "ADR-04",
    "ADR-05",
    "ADR-06",
    "ADR-07",
    "ADR-08",
    "ADR-09",
    "ADR-10",
    "ADR-11",
    "FORMAL-01",
    "FORMAL-02",
    "FORMAL-03",
    "FORMAL-04",
    "FORMAL-05",
    "EVID-01",
    "EVID-02",
    "EVID-03",
    "EVID-04",
    "EVID-05",
    "EVID-06",
    "CHECK-01",
    "CHECK-02",
    "CHECK-03",
    "CHECK-04",
    "CHECK-05",
    "CHECK-06",
    "VERIFY-01",
    "MAP-01",
    "MAP-02",
    "MAP-03",
    "NEW-01",
    "NEW-02",
    "NEW-03",
    "NEW-04",
    "NEW-05",
    "NEW-06",
    "NEW-07",
)

TERMINAL_DISPOSITIONS = {
    "COVERED",
    "EXTRACT",
    "DEFER_UNSTABLE",
    "CONSUMER",
    "EXTERNAL",
    "OBSOLETE",
}
FORBIDDEN_FINAL_VALUES = {"UNRESOLVED", "UNKNOWN", "TBD", "NARROW"}
EXPECTED_BASELINES = (
    "ee14ae8ef67640d0bb53c90a7cd00e08580a68ce",
    "0c2fd68974466df37105682d9aa8d034ed22dad2",
    "e27adc9290b4781396cb5c9745014ca5427e98ac",
)
FOLLOW_UP_URL = "https://github.com/fanilosendrison/proto-ring/issues/20"
MATRIX_HEADER = (
    "| Candidate | Responsibility | Evidence and provenance | Ruu confrontation "
    "| Owner or overlap | Analysis and maturity | Disposition "
    "| Trigger or follow-up |"
)
MATRIX_SEPARATOR = (
    "| --------- | -------------- | ----------------------- | ----------------- "
    "| ---------------- | --------------------- | ----------- "
    "| -------------------- |"
)


def audit_matrix_rows(document: str) -> list[tuple[str, ...]]:
    lines = document.splitlines()
    header_index = lines.index(MATRIX_HEADER)
    if lines[header_index + 1] != MATRIX_SEPARATOR:
        raise AssertionError("candidate matrix separator does not match its header")

    rows: list[tuple[str, ...]] = []
    for line in lines[header_index + 2 :]:
        if not line.startswith("|"):
            break
        cells = tuple(cell.strip() for cell in line.split("|")[1:-1])
        if len(cells) != 8:
            raise AssertionError(f"candidate row has {len(cells)} cells: {line}")
        rows.append(cells)
    return rows


class TurnlockResidualGovernanceReauditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.document = AUDIT_PATH.read_text(encoding="utf-8")
        cls.rows = audit_matrix_rows(cls.document)
        cls.rows_by_id = {row[0]: row for row in cls.rows}

    def test_audit_document_exists(self) -> None:
        self.assertTrue(AUDIT_PATH.is_file())

    def test_exact_evidence_baselines_are_recorded(self) -> None:
        for baseline in EXPECTED_BASELINES:
            self.assertIn(baseline, self.document)

    def test_candidate_matrix_is_complete_and_unique(self) -> None:
        observed_ids = [row[0] for row in self.rows]
        self.assertEqual(Counter(observed_ids), Counter(EXPECTED_CANDIDATE_IDS))
        for candidate_id in EXPECTED_CANDIDATE_IDS:
            self.assertEqual(observed_ids.count(candidate_id), 1)

    def test_every_candidate_has_one_terminal_disposition(self) -> None:
        for row in self.rows:
            candidate_id, disposition = row[0], row[6]
            self.assertIn(disposition, TERMINAL_DISPOSITIONS, candidate_id)
            self.assertEqual(
                sum(disposition == value for value in TERMINAL_DISPOSITIONS),
                1,
                candidate_id,
            )
            for forbidden in FORBIDDEN_FINAL_VALUES:
                self.assertNotIn(forbidden, row, candidate_id)

    def test_exactly_one_extract_candidate_is_evid_01(self) -> None:
        extracts = [row[0] for row in self.rows if row[6] == "EXTRACT"]
        self.assertEqual(extracts, ["EVID-01"])

    def test_deferred_candidates_have_reaudit_triggers(self) -> None:
        for row in self.rows:
            if row[6] == "DEFER_UNSTABLE":
                self.assertNotIn(row[7], {"", "—"}, row[0])

    def test_covered_and_external_rows_name_their_owner(self) -> None:
        for row in self.rows:
            if row[6] in {"COVERED", "EXTERNAL"}:
                self.assertNotIn(row[4], {"", "—"}, row[0])

    def test_extract_row_links_the_actual_follow_up_issue(self) -> None:
        self.assertIn(FOLLOW_UP_URL, self.rows_by_id["EVID-01"][7])

    def test_bounded_conclusion_is_explicit(self) -> None:
        self.assertIn(
            "For the exact audited baselines, every currently identified residual "
            "Turnlock\nrepository-governance candidate has been adjudicated.",
            self.document,
        )
        self.assertIn(
            "Exactly one residual responsibility is currently justified for new "
            "proto-ring\nextraction:",
            self.document,
        )
        self.assertIn(
            "No other additional extraction is justified at the current Turnlock "
            "maturity\nstate.",
            self.document,
        )
        self.assertIn("not a permanent\nexhaustion claim", self.document)
        self.assertIn(
            "Turnlock's method and specification remain evolving", self.document
        )


if __name__ == "__main__":
    unittest.main()
