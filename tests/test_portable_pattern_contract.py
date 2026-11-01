from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "portable-pattern.md"
README = ROOT / "README.md"


class PortablePatternContractTests(unittest.TestCase):
    def test_contract_records_every_execution_boundary(self) -> None:
        contract = CONTRACT.read_text(encoding="utf-8")
        for required in (
            "Canonical Portable Pattern contract",
            "full_match(pattern, input)",
            "Unicode scalar-value strings",
            "unescaped dot",
            "literal dot",
            "empty sequence",
            "Alternatives are ordered",
            "last successful participation",
            "unbounded quantifier",
            "empty string",
            "[a-b-c]",
            "→ invalid",
        ):
            with self.subTest(required=required):
                self.assertIn(required, contract)

    def test_readme_already_links_contract(self) -> None:
        self.assertIn(
            "[Canonical Portable Pattern](docs/contracts/portable-pattern.md)",
            README.read_text(encoding="utf-8"),
        )



if __name__ == "__main__":
    unittest.main()
