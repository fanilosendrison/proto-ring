from __future__ import annotations

import importlib.machinery
from pathlib import Path
import sys
import tempfile
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "tools"
sys.path.insert(0, str(TOOLS))
from python_package_absence import (  # noqa: E402
    PythonPackageAbsenceError,
    assert_python_package_absent,
)


class ProductionDistributionBoundaryTests(unittest.TestCase):
    def test_root_has_no_installable_python_package(self) -> None:
        for relative in ("pyproject.toml", "setup.py", "setup.cfg"):
            with self.subTest(relative=relative):
                self.assertFalse((ROOT / relative).exists())
        self.assertFalse((ROOT / "src" / "proto_ring").exists())

    def test_proto_ring_is_absent_with_root_and_src_on_sys_path(self) -> None:
        assert_python_package_absent(ROOT)

        cases = (
            ("tests module", Path("tests/proto_ring.py"), ""),
            ("tests package", Path("tests/proto_ring/__init__.py"), ""),
            ("tools module", Path("tools/proto_ring.py"), ""),
            (
                "ambient distribution",
                Path("site/proto_ring-9.9.9.dist-info/METADATA"),
                "Metadata-Version: 2.1\nName: proto-ring\nVersion: 9.9.9\n",
            ),
        )
        for label, relative, content in cases:
            with self.subTest(label=label), tempfile.TemporaryDirectory() as temporary:
                repository = Path(temporary)
                target = repository / relative
                target.parent.mkdir(parents=True)
                target.write_text(content, encoding="utf-8")
                paths = (
                    str(repository),
                    str(repository / "src"),
                    str(repository / "tests"),
                    str(repository / "tools"),
                    str(repository / "site"),
                )
                with self.assertRaises(PythonPackageAbsenceError):
                    assert_python_package_absent(repository, paths)

    def test_meta_path_finder_cannot_expose_retired_package(self) -> None:
        class RetiredPackageFinder:
            def __init__(self) -> None:
                self.paths = []

            def find_spec(self, fullname, path=None, _target=None):
                self.paths.append(path)
                if fullname == "proto_ring":
                    return importlib.machinery.ModuleSpec(fullname, loader=None)
                return None

        finder = RetiredPackageFinder()
        sys.meta_path.insert(0, finder)
        try:
            with self.assertRaises(PythonPackageAbsenceError):
                assert_python_package_absent(ROOT, ())
            self.assertEqual(finder.paths, [None])
        finally:
            sys.meta_path.remove(finder)

    def test_rust_compatibility_binary_is_the_distribution_surface(self) -> None:
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertIn("crates/proto-ring-compat", workspace["workspace"]["members"])
        self.assertFalse(workspace["workspace"]["package"]["publish"])

        manifest = tomllib.loads(
            (ROOT / "crates" / "proto-ring-compat" / "Cargo.toml").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(manifest["package"]["name"], "proto-ring-compat")
        self.assertEqual(
            manifest["bin"],
            [{"name": "proto-ring-compat", "path": "src/main.rs"}],
        )
        self.assertTrue(
            (ROOT / "crates" / "proto-ring-compat" / "src" / "main.rs").is_file()
        )


if __name__ == "__main__":
    unittest.main()
