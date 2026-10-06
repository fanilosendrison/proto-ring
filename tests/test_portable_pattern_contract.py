from __future__ import annotations

import ast
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "contracts" / "portable-pattern.md"
README = ROOT / "README.md"
PROJECT = ROOT / "pyproject.toml"
IMPLEMENTATIONS = (
    ROOT / "src" / "proto_ring" / "portable_pattern.py",
    ROOT / "src" / "proto_ring" / "portable_pattern_matching.py",
)


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

    def test_implementation_has_exact_module_boundary_and_no_regex_engine(self) -> None:
        for path in IMPLEMENTATIONS:
            self.assertTrue(path.is_file())
            tree = ast.parse(path.read_text(encoding="utf-8"))
            imports = {
                alias.name
                for node in ast.walk(tree)
                if isinstance(node, ast.Import)
                for alias in node.names
            }
            imports.update(
                node.module
                for node in ast.walk(tree)
                if isinstance(node, ast.ImportFrom) and node.module
            )
            self.assertNotIn("re", imports)
            self.assertNotIn("regex", imports)

    def test_package_version_is_0_16_1(self) -> None:
        self.assertIn('version = "0.16.1"', PROJECT.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
