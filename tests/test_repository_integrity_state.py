#!/usr/bin/env python3
from __future__ import annotations

import os
import stat
from pathlib import Path
from unittest import mock

from proto_ring.repository_integrity import (
    CommandObligation,
    IntegrityProfile,
    IntegrityVerdict,
    ObligationStatus,
    evaluate,
)
from repository_integrity_test_support import RepositoryFixtureTestCase


class RepositoryStateTests(RepositoryFixtureTestCase):
    def test_preexisting_dirty_states_are_admissible(self) -> None:
        preparations = (
            lambda: self.write("tracked.txt", "modified\n"),
            lambda: (self.write("tracked.txt", "staged\n"), self.git("add", "tracked.txt")),
            lambda: self.write("loose.txt", "untracked\n"),
        )
        for index, prepare in enumerate(preparations):
            with self.subTest(index=index):
                if index:
                    self.write("tracked.txt", "original\n")
                    self.git("reset", "-q", "HEAD", "--", "tracked.txt")
                    loose = self.repo / "loose.txt"
                    if loose.exists():
                        loose.unlink()
                prepare()
                result = self.run_integrity(
                    [CommandObligation("ok", self.python("pass"))]
                )
                self.assertEqual(IntegrityVerdict.PASS, result.verdict)
                self.assertEqual(
                    result.baseline_state_identity, result.final_state_identity
                )

    def test_content_creation_deletion_and_index_mutations_are_detected(self) -> None:
        cases = (
            ("tracked-content", self.write_code("tracked.txt", "mutated\n")),
            ("untracked-create", self.write_code("created.txt", "new\n")),
            ("tracked-delete", self.delete_code("tracked.txt")),
        )
        for name, command in cases:
            with self.subTest(name=name):
                result = self.run_integrity([CommandObligation(name, command)])
                self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
                self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
                self.assertNotEqual(
                    result.baseline_state_identity, result.final_state_identity
                )
                self.write("tracked.txt", "original\n")
                created = self.repo / "created.txt"
                if created.exists():
                    created.unlink()

        self.write("tracked.txt", "unstaged\n")
        result = self.run_integrity(
            [CommandObligation("stage", ("git", "add", "tracked.txt"))]
        )
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)

    def test_mutation_of_already_dirty_paths_is_detected(self) -> None:
        self.write("tracked.txt", "dirty-one\n")
        tracked = self.run_integrity(
            [CommandObligation("dirty", self.write_code("tracked.txt", "dirty-two\n"))]
        )
        self.assertEqual(ObligationStatus.VIOLATED, tracked.obligations[0].status)

        self.write("loose.txt", "one\n")
        untracked = self.run_integrity(
            [CommandObligation("loose", self.write_code("loose.txt", "two\n"))]
        )
        self.assertEqual(ObligationStatus.VIOLATED, untracked.obligations[0].status)

    def test_head_and_mode_mutations_are_detected(self) -> None:
        moved = self.run_integrity(
            [CommandObligation("commit", ("git", "commit", "--allow-empty", "-q", "-m", "probe"))]
        )
        self.assertEqual(ObligationStatus.VIOLATED, moved.obligations[0].status)

        target = self.repo / "tracked.txt"
        changed = self.run_integrity(
            [
                CommandObligation(
                    "chmod",
                    self.python(f"import os\nos.chmod({os.fspath(target)!r}, 0o755)"),
                )
            ]
        )
        self.assertEqual(ObligationStatus.VIOLATED, changed.obligations[0].status)
        self.assertTrue(stat.S_IMODE(os.stat(target).st_mode) & stat.S_IXUSR)

    def test_mutation_stops_later_execution(self) -> None:
        marker = self.root / "later"
        result = self.run_integrity(
            [
                CommandObligation("mutate", self.write_code("created.txt", "new\n")),
                CommandObligation("later", self.marker_code(marker)),
            ]
        )
        self.assertFalse(marker.exists())
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertEqual(ObligationStatus.UNDETERMINED, result.obligations[1].status)

    def test_repository_gitignore_is_authoritative_but_ambient_ignores_are_not(self) -> None:
        self.write(".gitignore", "ignored/\n")
        self.git("add", ".gitignore")
        self.git("commit", "-q", "-m", "ignore")
        self.write("ignored/item.txt", "one\n")
        ignored = self.run_integrity(
            [CommandObligation("ignored", self.write_code("ignored/item.txt", "two\n"))]
        )
        self.assertEqual(IntegrityVerdict.PASS, ignored.verdict)

        excludes = self.root / "global-excludes"
        excludes.write_text("global.txt\n", encoding="utf-8")
        config = self.root / "gitconfig"
        config.write_text(f"[core]\n\texcludesFile = {excludes}\n", encoding="utf-8")
        self.write("global.txt", "one\n")
        with mock.patch.dict(
            os.environ,
            {"GIT_CONFIG_GLOBAL": os.fspath(config), "GIT_CONFIG_NOSYSTEM": "1"},
        ):
            global_result = self.run_integrity(
                [CommandObligation("global", self.write_code("global.txt", "two\n"))]
            )
        self.assertEqual(IntegrityVerdict.NON_PASS, global_result.verdict)

        info = self.repo / ".git" / "info" / "exclude"
        info.write_text("info.txt\n", encoding="utf-8")
        self.write("info.txt", "one\n")
        info_result = self.run_integrity(
            [CommandObligation("info", self.write_code("info.txt", "two\n"))]
        )
        self.assertEqual(IntegrityVerdict.NON_PASS, info_result.verdict)

    def test_projection_validation_is_pure(self) -> None:
        self.write("source.txt", "v2\n")
        self.write("projection.txt", "v1\n")
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "projection")
        validator = self.python(
            "import sys\nfrom pathlib import Path\n"
            "sys.exit(0 if Path('source.txt').read_text() == Path('projection.txt').read_text() else 7)"
        )
        stale = self.run_integrity([CommandObligation("check", validator)])
        self.assertEqual(ObligationStatus.VIOLATED, stale.obligations[0].status)
        self.assertEqual("v1\n", (self.repo / "projection.txt").read_text())

        generator = self.python(
            "from pathlib import Path\n"
            "Path('projection.txt').write_text(Path('source.txt').read_text())"
        )
        repair = self.run_integrity([CommandObligation("repair", generator)])
        self.assertEqual(ObligationStatus.VIOLATED, repair.obligations[0].status)
        identical = self.run_integrity([CommandObligation("rewrite", generator)])
        self.assertEqual(IntegrityVerdict.PASS, identical.verdict)

    def test_invalid_repository_roots_fail_closed_without_execution(self) -> None:
        plain = self.root / "plain"
        plain.mkdir()
        subdirectory = self.repo / "subdirectory"
        subdirectory.mkdir()
        for repository in (plain, subdirectory):
            marker = self.root / f"marker-{repository.name}"
            result = self.run_integrity(
                [CommandObligation("probe", self.marker_code(marker))],
                repository=repository,
            )
            self.assertFalse(marker.exists())
            self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
            self.assertIsNone(result.baseline_state_identity)
            self.assertEqual(ObligationStatus.UNDETERMINED, result.obligations[0].status)

    def test_internal_git_state_capture_ignores_ambient_redirection(self) -> None:
        profile = IntegrityProfile(())
        control = evaluate(
            self.repo,
            profile,
            env=self.environment(),
            evaluation_context_identity="test",
        )
        alternate = self.root / "alternate-index"
        with mock.patch.dict(os.environ, {"GIT_INDEX_FILE": os.fspath(alternate)}):
            polluted = evaluate(
                self.repo,
                profile,
                env=self.environment(),
                evaluation_context_identity="test",
            )
        self.assertEqual(control.baseline_state_identity, polluted.baseline_state_identity)

        other = self.root / "other"
        other.mkdir()
        self.git_in(other, "init", "-q")
        with mock.patch.dict(
            os.environ,
            {"GIT_DIR": os.fspath(other / ".git"), "GIT_WORK_TREE": os.fspath(other)},
        ):
            redirected = evaluate(
                self.repo,
                profile,
                env=self.environment(),
                evaluation_context_identity="test",
            )
        self.assertEqual(control.baseline_state_identity, redirected.baseline_state_identity)

    def test_consumer_git_environment_is_preserved(self) -> None:
        expected = self.root / "consumer-index"
        environment = self.environment()
        environment["GIT_INDEX_FILE"] = os.fspath(expected)
        command = self.python(
            "import os, sys\n"
            f"sys.exit(0 if os.environ.get('GIT_INDEX_FILE') == {os.fspath(expected)!r} else 73)"
        )
        result = self.run_integrity(
            [CommandObligation("consumer", command)], env=environment
        )
        self.assertEqual(IntegrityVerdict.PASS, result.verdict)


if __name__ == "__main__":
    import unittest

    unittest.main()
