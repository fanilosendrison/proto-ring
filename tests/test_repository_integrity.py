#!/usr/bin/env python3
from __future__ import annotations

import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from proto_ring.repository_integrity import (
    CommandObligation,
    IntegrityProfile,
    IntegrityVerdict,
    ObligationStatus,
    evaluate,
)


class RepositoryFixtureTestCase(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()

        subprocess.run(
            ["git", "init", "-q", os.fspath(self.repo)],
            check=True,
            capture_output=True,
            text=True,
        )
        self.git("config", "user.name", "Proto Ring Test")
        self.git("config", "user.email", "proto-ring@example.invalid")
        self.git("config", "commit.gpgsign", "false")

        self.write("tracked.txt", "original\n")
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "initial")

    def git(self, *args: str) -> subprocess.CompletedProcess[str]:
        completed = subprocess.run(
            ["git", "-C", os.fspath(self.repo), *args],
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            raise AssertionError(
                f"git {' '.join(args)} failed: {completed.stderr.strip()}"
            )
        return completed

    def git_in(
        self, repository: Path, *args: str
    ) -> subprocess.CompletedProcess[str]:
        completed = subprocess.run(
            ["git", "-C", os.fspath(repository), *args],
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            raise AssertionError(
                f"git {' '.join(args)} failed: {completed.stderr.strip()}"
            )
        return completed

    def write(self, relative: str, content: str) -> Path:
        path = self.repo / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def environment(self) -> dict[str, str]:
        return dict(os.environ)

    def run_integrity(
        self,
        obligations: list[CommandObligation],
        *,
        continue_after: bool = True,
        env: dict[str, str] | None = None,
        repository: Path | None = None,
    ):
        profile = IntegrityProfile(tuple(obligations), continue_after)
        return evaluate(
            repository if repository is not None else self.repo,
            profile,
            env=self.environment() if env is None else env,
        )

    def python(self, code: str) -> tuple[str, ...]:
        return (sys.executable, "-c", code)

    def exit_code(self, code: int) -> tuple[str, ...]:
        return self.python(f"import sys\nsys.exit({code})")

    def write_code(self, relative: str, content: str) -> tuple[str, ...]:
        target = self.repo / relative
        return self.python(
            "from pathlib import Path\n"
            f"Path({os.fspath(target)!r}).write_text({content!r}, encoding='utf-8')"
        )

    def delete_code(self, relative: str) -> tuple[str, ...]:
        target = self.repo / relative
        return self.python(
            "from pathlib import Path\n" f"Path({os.fspath(target)!r}).unlink()"
        )

    def marker_code(self, marker: Path, content: str = "ran") -> tuple[str, ...]:
        return self.python(
            "from pathlib import Path\n"
            f"Path({os.fspath(marker)!r}).write_text({content!r})"
        )

    def append_code(self, log: Path, token: str) -> tuple[str, ...]:
        return self.python(
            "from pathlib import Path\n"
            f"path = Path({os.fspath(log)!r})\n"
            "path.write_text((path.read_text() if path.exists() else '') + "
            f"{token!r})"
        )

    def status_porcelain(self) -> str:
        return self.git("status", "--porcelain").stdout

    # ── 10.1 Basic verdicts ──────────────────────────────────────────────

    def test_clean_repository_satisfied_obligation_passes(self) -> None:
        result = self.run_integrity(
            [CommandObligation("ok", self.python("pass"))]
        )

        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        self.assertEqual(ObligationStatus.SATISFIED, result.obligations[0].status)
        self.assertEqual(0, result.obligations[0].returncode)
        self.assertIsNone(result.obligations[0].detail)
        self.assertIsNotNone(result.baseline_state_identity)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertEqual((), result.errors)
        self.assertEqual(64, len(result.profile_identity))

    def test_nonzero_exit_is_violated_non_pass(self) -> None:
        result = self.run_integrity(
            [CommandObligation("fail", self.exit_code(7))]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        obligation = result.obligations[0]
        self.assertEqual(ObligationStatus.VIOLATED, obligation.status)
        self.assertEqual(7, obligation.returncode)
        self.assertEqual("command exited with 7", obligation.detail)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertEqual((), result.errors)

    def test_missing_executable_is_undetermined_non_pass(self) -> None:
        result = self.run_integrity(
            [
                CommandObligation(
                    "missing", ("proto-ring-definitely-missing-executable",)
                )
            ]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        obligation = result.obligations[0]
        self.assertEqual(ObligationStatus.UNDETERMINED, obligation.status)
        self.assertIsNone(obligation.returncode)
        self.assertIn("command could not be started", obligation.detail)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )

    def test_configured_undetermined_exit_code_is_undetermined(self) -> None:
        result = self.run_integrity(
            [
                CommandObligation(
                    "probe",
                    self.exit_code(42),
                    undetermined_exit_codes=frozenset({42}),
                )
            ]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        obligation = result.obligations[0]
        self.assertEqual(ObligationStatus.UNDETERMINED, obligation.status)
        self.assertEqual(42, obligation.returncode)
        self.assertEqual("command exited with 42", obligation.detail)

    # ── 10.2 Execution policy ────────────────────────────────────────────

    def test_continuation_enabled_executes_second_obligation(self) -> None:
        marker = self.root / "second-ran.marker"

        result = self.run_integrity(
            [
                CommandObligation("first", self.exit_code(7)),
                CommandObligation("second", self.marker_code(marker)),
            ],
            continue_after=True,
        )

        self.assertTrue(marker.exists())
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertEqual(ObligationStatus.SATISFIED, result.obligations[1].status)
        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)

    def test_continuation_disabled_marks_second_obligation_undetermined(
        self,
    ) -> None:
        marker = self.root / "second-ran.marker"

        result = self.run_integrity(
            [
                CommandObligation("first", self.exit_code(7)),
                CommandObligation("second", self.marker_code(marker)),
            ],
            continue_after=False,
        )

        self.assertFalse(marker.exists())
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        second = result.obligations[1]
        self.assertEqual(ObligationStatus.UNDETERMINED, second.status)
        self.assertIsNone(second.returncode)
        self.assertEqual(
            "obligation skipped after previous non-satisfied result", second.detail
        )
        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)

    def test_obligation_order_is_preserved(self) -> None:
        log = self.root / "order.log"

        result = self.run_integrity(
            [
                CommandObligation("first", self.append_code(log, "1")),
                CommandObligation("second", self.append_code(log, "2")),
                CommandObligation("third", self.append_code(log, "3")),
            ]
        )

        self.assertEqual("123", log.read_text(encoding="utf-8"))
        self.assertEqual(
            ["first", "second", "third"],
            [obligation.name for obligation in result.obligations],
        )
        self.assertEqual(IntegrityVerdict.PASS, result.verdict)

    # ── 10.3 Pre-existing dirty state ────────────────────────────────────

    def test_preexisting_modified_tracked_file_passes(self) -> None:
        self.write("tracked.txt", "modified\n")
        self.assertEqual(" M tracked.txt\n", self.status_porcelain())

        result = self.run_integrity(
            [CommandObligation("ok", self.python("pass"))]
        )

        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        self.assertEqual(ObligationStatus.SATISFIED, result.obligations[0].status)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )

    def test_preexisting_staged_file_passes(self) -> None:
        self.write("tracked.txt", "staged\n")
        self.git("add", "tracked.txt")
        self.assertEqual("M  tracked.txt\n", self.status_porcelain())

        result = self.run_integrity(
            [CommandObligation("ok", self.python("pass"))]
        )

        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        self.assertEqual(ObligationStatus.SATISFIED, result.obligations[0].status)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )

    def test_preexisting_untracked_nonignored_file_passes(self) -> None:
        self.write("loose.txt", "untracked\n")
        self.assertEqual("?? loose.txt\n", self.status_porcelain())

        result = self.run_integrity(
            [CommandObligation("ok", self.python("pass"))]
        )

        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        self.assertEqual(ObligationStatus.SATISFIED, result.obligations[0].status)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )

    # ── 10.4 Content-sensitive purity ────────────────────────────────────

    def test_clean_tracked_file_mutation_detected(self) -> None:
        result = self.run_integrity(
            [
                CommandObligation(
                    "mutate", self.write_code("tracked.txt", "mutated\n")
                )
            ]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        obligation = result.obligations[0]
        self.assertEqual(ObligationStatus.VIOLATED, obligation.status)
        self.assertEqual(0, obligation.returncode)
        self.assertEqual(
            "repository state changed during obligation mutate", obligation.detail
        )
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertTrue(result.errors)

    def test_new_governed_untracked_file_creation_detected(self) -> None:
        result = self.run_integrity(
            [
                CommandObligation(
                    "create", self.write_code("created.txt", "new\n")
                )
            ]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertTrue((self.repo / "created.txt").exists())

    def test_tracked_file_deletion_detected(self) -> None:
        result = self.run_integrity(
            [CommandObligation("delete", self.delete_code("tracked.txt"))]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertFalse((self.repo / "tracked.txt").exists())

    def test_already_dirty_tracked_file_content_mutation_detected(self) -> None:
        self.write("tracked.txt", "dirty-one\n")
        self.assertEqual(" M tracked.txt\n", self.status_porcelain())

        result = self.run_integrity(
            [
                CommandObligation(
                    "mutate-dirty", self.write_code("tracked.txt", "dirty-two\n")
                )
            ]
        )

        self.assertEqual(" M tracked.txt\n", self.status_porcelain())
        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertEqual(
            "dirty-two\n", (self.repo / "tracked.txt").read_text(encoding="utf-8")
        )

    def test_already_present_untracked_file_content_mutation_detected(self) -> None:
        self.write("loose.txt", "one\n")
        self.assertEqual("?? loose.txt\n", self.status_porcelain())

        result = self.run_integrity(
            [
                CommandObligation(
                    "mutate-untracked", self.write_code("loose.txt", "two\n")
                )
            ]
        )

        self.assertEqual("?? loose.txt\n", self.status_porcelain())
        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertEqual(
            "two\n", (self.repo / "loose.txt").read_text(encoding="utf-8")
        )

    def test_staged_index_only_mutation_detected(self) -> None:
        self.write("tracked.txt", "unstaged-change\n")
        content_before = (self.repo / "tracked.txt").read_bytes()

        result = self.run_integrity(
            [CommandObligation("stage", ("git", "add", "tracked.txt"))]
        )

        self.assertEqual(content_before, (self.repo / "tracked.txt").read_bytes())
        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )

    def test_head_movement_detected(self) -> None:
        result = self.run_integrity(
            [
                CommandObligation(
                    "commit",
                    ("git", "commit", "--allow-empty", "-q", "-m", "probe"),
                )
            ]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertTrue(result.errors)

    @unittest.skipIf(os.name == "nt", "POSIX permission bits are required")
    def test_mode_only_mutation_detected(self) -> None:
        target = self.repo / "tracked.txt"
        result = self.run_integrity(
            [
                CommandObligation(
                    "chmod",
                    self.python(
                        "import os\n" f"os.chmod({os.fspath(target)!r}, 0o755)"
                    ),
                )
            ]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertEqual(
            "original\n", target.read_text(encoding="utf-8")
        )
        self.assertTrue(stat.S_IMODE(os.stat(target).st_mode) & stat.S_IXUSR)

    # ── 10.5 Mutation stops later evaluation ─────────────────────────────

    def test_mutation_stops_later_obligations(self) -> None:
        marker = self.root / "later.marker"

        result = self.run_integrity(
            [
                CommandObligation(
                    "mutate", self.write_code("created.txt", "new\n")
                ),
                CommandObligation("later", self.marker_code(marker)),
            ],
            continue_after=True,
        )

        self.assertFalse(marker.exists())
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        later = result.obligations[1]
        self.assertEqual(ObligationStatus.UNDETERMINED, later.status)
        self.assertIsNone(later.returncode)
        self.assertEqual(
            "obligation skipped after previous non-satisfied result", later.detail
        )
        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)

    # ── 10.6 Ignore scope ────────────────────────────────────────────────

    def test_repository_gitignore_excluded_artifact_is_not_governed(self) -> None:
        self.write(".gitignore", "ignored-artifacts/\n")
        self.git("add", ".gitignore")
        self.git("commit", "-q", "-m", "ignore rules")
        self.write("ignored-artifacts/tmp.txt", "one\n")

        result = self.run_integrity(
            [
                CommandObligation(
                    "mutate-ignored",
                    self.write_code("ignored-artifacts/tmp.txt", "two\n"),
                )
            ]
        )

        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        self.assertEqual(ObligationStatus.SATISFIED, result.obligations[0].status)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertEqual(
            "two\n",
            (self.repo / "ignored-artifacts/tmp.txt").read_text(encoding="utf-8"),
        )

    def test_ambient_global_ignore_is_not_authoritative(self) -> None:
        excludes = self.root / "global-excludes"
        excludes.write_text("globally-ignored.txt\n", encoding="utf-8")
        global_config = self.root / "gitconfig-global"
        global_config.write_text(
            f"[core]\n\texcludesFile = {os.fspath(excludes)}\n",
            encoding="utf-8",
        )
        self.write("globally-ignored.txt", "one\n")

        with mock.patch.dict(
            os.environ,
            {
                "GIT_CONFIG_GLOBAL": os.fspath(global_config),
                "GIT_CONFIG_NOSYSTEM": "1",
            },
        ):
            result = self.run_integrity(
                [
                    CommandObligation(
                        "mutate-globally-ignored",
                        self.write_code("globally-ignored.txt", "two\n"),
                    )
                ]
            )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )

    def test_git_info_exclude_is_not_authoritative(self) -> None:
        info_exclude = self.repo / ".git" / "info" / "exclude"
        info_exclude.parent.mkdir(parents=True, exist_ok=True)
        info_exclude.write_text("info-excluded.txt\n", encoding="utf-8")
        self.write("info-excluded.txt", "one\n")

        result = self.run_integrity(
            [
                CommandObligation(
                    "mutate-info-excluded",
                    self.write_code("info-excluded.txt", "two\n"),
                )
            ]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertEqual(ObligationStatus.VIOLATED, result.obligations[0].status)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )

    # ── 10.7 Profile identity ────────────────────────────────────────────

    def identity_for(
        self,
        obligations: tuple[CommandObligation, ...],
        continue_after: bool = True,
    ) -> str:
        profile = IntegrityProfile(obligations, continue_after)
        return evaluate(self.repo, profile, env=self.environment()).profile_identity

    def test_profile_identity_stable_for_identical_ordered_profile_data(
        self,
    ) -> None:
        first = (
            CommandObligation("one", self.python("pass")),
            CommandObligation("two", self.python("pass"), frozenset({3})),
        )
        second = (
            CommandObligation("one", self.python("pass")),
            CommandObligation("two", self.python("pass"), frozenset({3})),
        )

        self.assertEqual(self.identity_for(first), self.identity_for(second))

    def test_profile_identity_changes_when_obligation_order_changes(self) -> None:
        first = (
            CommandObligation("one", self.python("pass")),
            CommandObligation("two", self.python("pass")),
        )
        second = (
            CommandObligation("two", self.python("pass")),
            CommandObligation("one", self.python("pass")),
        )

        self.assertNotEqual(self.identity_for(first), self.identity_for(second))

    def test_profile_identity_changes_when_argv_changes(self) -> None:
        first = (CommandObligation("one", self.python("pass")),)
        second = (CommandObligation("one", self.python("pass") + ("extra",)),)

        self.assertNotEqual(self.identity_for(first), self.identity_for(second))

    def test_profile_identity_changes_when_continuation_policy_changes(
        self,
    ) -> None:
        obligations = (CommandObligation("one", self.python("pass")),)

        self.assertNotEqual(
            self.identity_for(obligations, continue_after=True),
            self.identity_for(obligations, continue_after=False),
        )

    def test_profile_identity_changes_when_undetermined_exit_codes_change(
        self,
    ) -> None:
        first = (
            CommandObligation("one", self.python("pass"), frozenset({3})),
        )
        second = (
            CommandObligation("one", self.python("pass"), frozenset({4})),
        )

        self.assertNotEqual(self.identity_for(first), self.identity_for(second))

    def test_zero_undetermined_exit_code_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            CommandObligation(
                "bad", self.python("pass"), frozenset({0})
            )

    # ── 10.8 Projection behavior ─────────────────────────────────────────

    def make_projection_pair(self, projection_content: str) -> None:
        self.write("source.txt", "v2\n")
        self.write("projection.txt", projection_content)
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "projection pair")

    def test_stale_projection_pure_validator_non_pass(self) -> None:
        self.make_projection_pair("v1\n")
        validator = self.python(
            "import sys\n"
            "from pathlib import Path\n"
            "sys.exit(0 if Path('projection.txt').read_text() "
            "== Path('source.txt').read_text() else 7)"
        )

        result = self.run_integrity(
            [CommandObligation("projection-check", validator)]
        )

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        obligation = result.obligations[0]
        self.assertEqual(ObligationStatus.VIOLATED, obligation.status)
        self.assertEqual(7, obligation.returncode)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertEqual(
            "v1\n", (self.repo / "projection.txt").read_text(encoding="utf-8")
        )

    def test_projection_repair_generator_cannot_pass(self) -> None:
        self.make_projection_pair("v1\n")
        generator = self.python(
            "from pathlib import Path\n"
            "Path('projection.txt').write_text(Path('source.txt').read_text())"
        )

        result = self.run_integrity([CommandObligation("regenerate", generator)])

        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        obligation = result.obligations[0]
        self.assertEqual(ObligationStatus.VIOLATED, obligation.status)
        self.assertEqual(0, obligation.returncode)
        self.assertNotEqual(
            result.baseline_state_identity, result.final_state_identity
        )
        self.assertEqual(
            "v2\n", (self.repo / "projection.txt").read_text(encoding="utf-8")
        )

    def test_projection_identical_rewrite_passes(self) -> None:
        self.make_projection_pair("v2\n")
        generator = self.python(
            "from pathlib import Path\n"
            "Path('projection.txt').write_text(Path('source.txt').read_text())"
        )

        result = self.run_integrity([CommandObligation("regenerate", generator)])

        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        obligation = result.obligations[0]
        self.assertEqual(ObligationStatus.SATISFIED, obligation.status)
        self.assertEqual(0, obligation.returncode)
        self.assertEqual(
            result.baseline_state_identity, result.final_state_identity
        )

    # ── 10.9 Fail-closed root/state ──────────────────────────────────────

    def test_non_git_directory_fails_closed_without_executing(self) -> None:
        plain = self.root / "plain"
        plain.mkdir()
        marker = self.root / "executed.marker"

        result = self.run_integrity(
            [
                CommandObligation("first", self.marker_code(marker)),
                CommandObligation("second", self.python("pass")),
            ],
            repository=plain,
        )

        self.assertFalse(marker.exists())
        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertIsNone(result.baseline_state_identity)
        self.assertIsNone(result.final_state_identity)
        for obligation in result.obligations:
            self.assertEqual(ObligationStatus.UNDETERMINED, obligation.status)
            self.assertIsNone(obligation.returncode)
        self.assertTrue(result.errors)
        self.assertIn(
            "baseline repository state could not be determined", result.errors[0]
        )

    def test_repository_subdirectory_fails_closed_without_executing(self) -> None:
        subdirectory = self.repo / "subdirectory"
        subdirectory.mkdir()
        marker = self.root / "executed.marker"

        result = self.run_integrity(
            [
                CommandObligation("first", self.marker_code(marker)),
                CommandObligation("second", self.python("pass")),
            ],
            repository=subdirectory,
        )

        self.assertFalse(marker.exists())
        self.assertEqual(IntegrityVerdict.NON_PASS, result.verdict)
        self.assertIsNone(result.baseline_state_identity)
        self.assertIsNone(result.final_state_identity)
        for obligation in result.obligations:
            self.assertEqual(ObligationStatus.UNDETERMINED, obligation.status)
        self.assertTrue(result.errors)

    # ── 10.10 Explicit evaluation environment ────────────────────────────

    def test_explicit_environment_reaches_consumer_commands(self) -> None:
        probe = self.python(
            "import os\n"
            "assert 'RI_AMBIENT_VARIABLE' not in os.environ\n"
            "assert os.environ.get('RI_EXPLICIT_VARIABLE') == 'present'\n"
        )

        with mock.patch.dict(
            os.environ, {"RI_AMBIENT_VARIABLE": "ambient"}
        ):
            result = self.run_integrity(
                [CommandObligation("env-check", probe)],
                env={"RI_EXPLICIT_VARIABLE": "present"},
            )

        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        obligation = result.obligations[0]
        self.assertEqual(ObligationStatus.SATISFIED, obligation.status)
        self.assertEqual(0, obligation.returncode)
        self.assertIsNone(obligation.detail)

    # ── Correction: consumer output and internal Git isolation ──────────

    def external_script(self, inner_code: str) -> str:
        return (
            "import os\n"
            "import sys\n"
            "from pathlib import Path\n"
            "from proto_ring.repository_integrity import (\n"
            "    CommandObligation,\n"
            "    IntegrityProfile,\n"
            "    IntegrityVerdict,\n"
            "    evaluate,\n"
            ")\n"
            f"obligation = CommandObligation('probe', (sys.executable, '-c', {inner_code!r}))\n"
            "result = evaluate(\n"
            f"    Path({os.fspath(self.repo)!r}),\n"
            "    IntegrityProfile((obligation,)),\n"
            "    env=dict(os.environ),\n"
            ")\n"
            "sys.exit(0 if result.verdict == IntegrityVerdict.PASS else 91)\n"
        )

    def test_consumer_command_stdout_is_inherited_by_caller(self) -> None:
        script = self.external_script(
            "print('RI-STDOUT-SENTINEL', flush=True)"
        )

        completed = subprocess.run(
            [sys.executable, "-c", script],
            capture_output=True,
            text=True,
            env=dict(os.environ),
            check=False,
        )

        self.assertEqual(0, completed.returncode)
        self.assertIn("RI-STDOUT-SENTINEL", completed.stdout)

    def test_consumer_command_stderr_is_inherited_by_caller(self) -> None:
        script = self.external_script(
            "import sys\n"
            "print('RI-STDERR-SENTINEL', file=sys.stderr, flush=True)"
        )

        completed = subprocess.run(
            [sys.executable, "-c", script],
            capture_output=True,
            text=True,
            env=dict(os.environ),
            check=False,
        )

        self.assertEqual(0, completed.returncode)
        self.assertIn("RI-STDERR-SENTINEL", completed.stderr)

    def test_ambient_git_index_file_cannot_redirect_state_capture(self) -> None:
        empty_profile = IntegrityProfile(())

        control = evaluate(
            self.repo,
            empty_profile,
            env=self.environment(),
        )
        self.assertEqual(IntegrityVerdict.PASS, control.verdict)
        self.assertIsNotNone(control.baseline_state_identity)

        alternate_index = self.root / "ambient-alternate-index"
        self.assertFalse(alternate_index.exists())

        with mock.patch.dict(
            os.environ,
            {"GIT_INDEX_FILE": os.fspath(alternate_index)},
        ):
            polluted = evaluate(
                self.repo,
                empty_profile,
                env=self.environment(),
            )

        self.assertEqual(IntegrityVerdict.PASS, polluted.verdict)
        self.assertEqual(
            control.baseline_state_identity,
            polluted.baseline_state_identity,
        )
        self.assertEqual(
            polluted.baseline_state_identity,
            polluted.final_state_identity,
        )

    def test_ambient_git_dir_and_work_tree_cannot_redirect_explicit_repository(
        self,
    ) -> None:
        empty_profile = IntegrityProfile(())

        control = evaluate(
            self.repo,
            empty_profile,
            env=self.environment(),
        )
        self.assertEqual(IntegrityVerdict.PASS, control.verdict)

        other_repo = self.root / "other-repo"
        other_repo.mkdir()
        self.git_in(other_repo, "init", "-q")
        self.git_in(other_repo, "config", "user.name", "Proto Ring Other Test")
        self.git_in(
            other_repo,
            "config",
            "user.email",
            "proto-ring-other@example.invalid",
        )
        self.git_in(other_repo, "config", "commit.gpgsign", "false")
        (other_repo / "other.txt").write_text("other\n", encoding="utf-8")
        self.git_in(other_repo, "add", "-A")
        self.git_in(other_repo, "commit", "-q", "-m", "other")

        with mock.patch.dict(
            os.environ,
            {
                "GIT_DIR": os.fspath(other_repo / ".git"),
                "GIT_WORK_TREE": os.fspath(other_repo),
            },
        ):
            polluted = evaluate(
                self.repo,
                empty_profile,
                env=self.environment(),
            )

        self.assertEqual(IntegrityVerdict.PASS, polluted.verdict)
        self.assertEqual(
            control.baseline_state_identity,
            polluted.baseline_state_identity,
        )
        self.assertEqual(
            polluted.baseline_state_identity,
            polluted.final_state_identity,
        )

    def test_consumer_git_environment_is_preserved_exactly(self) -> None:
        consumer_index = self.root / "consumer-owned-index"

        explicit_env = self.environment()
        explicit_env["GIT_INDEX_FILE"] = os.fspath(consumer_index)
        explicit_env["RI_CONSUMER_ENV_SENTINEL"] = "present"

        command = self.python(
            "import os\n"
            "import sys\n"
            f"expected_index = {os.fspath(consumer_index)!r}\n"
            "ok = (\n"
            "    os.environ.get('GIT_INDEX_FILE') == expected_index\n"
            "    and os.environ.get('RI_CONSUMER_ENV_SENTINEL') == 'present'\n"
            ")\n"
            "sys.exit(0 if ok else 73)\n"
        )

        result = self.run_integrity(
            [CommandObligation("consumer-env", command)],
            env=explicit_env,
        )

        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        self.assertEqual(
            ObligationStatus.SATISFIED,
            result.obligations[0].status,
        )
        self.assertEqual(0, result.obligations[0].returncode)


if __name__ == "__main__":
    unittest.main()
