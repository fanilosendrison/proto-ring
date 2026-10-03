from __future__ import annotations

from dataclasses import FrozenInstanceError
import os
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

from proto_ring import repository_state


EXPECTED_IDENTITIES = {
    "clean_tracked": "c96d9ef5d1202610ad72ce9bf45823ecd591b19ddb1eb33ac7d22cef20c2954a",
    "detached_head": "76bedd7f5bdf48cf4e54a17f08fffe596529fdb41319337482fe556b2228c3f2",
    "dirty_tracked": "e9505943f1ffe1643b8eb38b065ab802afe73fcf3041335bbfe9b742db100838",
    "file_mode": "255339e011a612bb4224532167bd27687bf45b13ce989778bc91cc7ab15d503b",
    "staged": "fdff09577440cf35ea92154bec82ad8d984f50fc4077cd425d6a0b7703968919",
    "symlink": "b470201ef34b37f742e8efdcf8ed3087e73c23727fe058abe50587007fe701a5",
    "untracked": "3e428d0b69cd77f1750258ce6736ebb7505efdf6dfd288feb02272c2a691ef97",
}


class RepositoryStateTests(unittest.TestCase):
    def git_environment(self) -> dict[str, str]:
        return {
            **os.environ,
            "GIT_AUTHOR_NAME": "Vector Author",
            "GIT_AUTHOR_EMAIL": "vector@example.invalid",
            "GIT_COMMITTER_NAME": "Vector Committer",
            "GIT_COMMITTER_EMAIL": "vector@example.invalid",
            "GIT_AUTHOR_DATE": "2001-02-03T04:05:06+0000",
            "GIT_COMMITTER_DATE": "2001-02-03T04:05:06+0000",
        }

    def git(self, repository: Path, *arguments: str) -> None:
        subprocess.run(
            ("git", "-C", os.fspath(repository), *arguments),
            env=self.git_environment(),
            check=True,
            capture_output=True,
        )

    def repository(self) -> tuple[TemporaryDirectory[str], Path]:
        temporary = TemporaryDirectory()
        repository = Path(temporary.name)
        self.git(repository, "init", "-q", "-b", "main")
        self.git(repository, "config", "user.name", "Vector")
        self.git(repository, "config", "user.email", "vector@example.invalid")
        self.git(repository, "config", "core.fileMode", "true")
        (repository / "tracked.txt").write_text("original\n", encoding="utf-8")
        self.git(repository, "add", "tracked.txt")
        self.git(repository, "commit", "-q", "-m", "baseline")
        return temporary, repository

    def identity_for(self, state_name: str) -> str:
        temporary, repository = self.repository()
        self.addCleanup(temporary.cleanup)
        if state_name == "dirty_tracked":
            (repository / "tracked.txt").write_text("dirty\n", encoding="utf-8")
        elif state_name == "staged":
            (repository / "tracked.txt").write_text("staged\n", encoding="utf-8")
            self.git(repository, "add", "tracked.txt")
        elif state_name == "untracked":
            (repository / "untracked.txt").write_text("untracked\n", encoding="utf-8")
        elif state_name == "detached_head":
            self.git(repository, "checkout", "-q", "--detach", "HEAD")
        elif state_name == "file_mode":
            os.chmod(repository / "tracked.txt", 0o755)
        elif state_name == "symlink":
            (repository / "tracked.txt").unlink()
            (repository / "target.txt").write_text("target\n", encoding="utf-8")
            (repository / "tracked.txt").symlink_to("target.txt")
        return repository_state.capture(repository).identity

    def test_existing_state_identity_vectors_are_exactly_preserved(self) -> None:
        for state_name, expected in EXPECTED_IDENTITIES.items():
            with self.subTest(state=state_name):
                self.assertEqual(expected, self.identity_for(state_name))

    def test_public_state_is_frozen_and_scope_is_canonical(self) -> None:
        temporary, repository = self.repository()
        self.addCleanup(temporary.cleanup)
        state = repository_state.capture(
            repository, ("tracked.txt", "absent.txt", "tracked.txt")
        )
        self.assertEqual(repository.resolve(), state.repository)
        self.assertEqual(("absent.txt", "tracked.txt"), state.scope_paths)
        self.assertEqual(64, len(state.identity))
        with self.assertRaises(FrozenInstanceError):
            state.identity = "changed"  # type: ignore[misc]
        self.assertEqual(
            [
                "RepositoryState",
                "StateCaptureError",
                "resolve_worktree_root",
                "discover_repository_paths",
                "capture",
            ],
            repository_state.__all__,
        )

    def test_explicit_regular_symlink_directory_and_absent_kinds_are_observed(self) -> None:
        temporary, repository = self.repository()
        self.addCleanup(temporary.cleanup)
        directory = repository / "routed"
        directory.mkdir()
        baseline = repository_state.capture(repository, ("routed",)).identity

        os.chmod(directory, 0o700)
        mode_changed = repository_state.capture(repository, ("routed",)).identity
        self.assertNotEqual(baseline, mode_changed)

        directory.rmdir()
        absent = repository_state.capture(repository, ("routed",)).identity
        self.assertNotEqual(mode_changed, absent)
        directory.write_text("regular\n", encoding="utf-8")
        regular = repository_state.capture(repository, ("routed",)).identity
        self.assertNotEqual(absent, regular)
        directory.unlink()
        directory.symlink_to("tracked.txt")
        symlink = repository_state.capture(repository, ("routed",)).identity
        self.assertNotEqual(regular, symlink)

    def test_explicit_directory_does_not_recursively_hash_ignored_descendants(self) -> None:
        temporary, repository = self.repository()
        self.addCleanup(temporary.cleanup)
        (repository / ".gitignore").write_text("ignored/\n", encoding="utf-8")
        self.git(repository, "add", ".gitignore")
        self.git(repository, "commit", "-q", "-m", "ignore")
        ignored = repository / "ignored"
        ignored.mkdir()
        child = ignored / "child.txt"
        child.write_text("one\n", encoding="utf-8")
        before = repository_state.capture(repository, ("ignored",))
        child.write_text("two\n", encoding="utf-8")
        after = repository_state.capture(repository, ("ignored",))
        self.assertEqual(before.identity, after.identity)

    def test_capture_is_read_only_and_rejects_non_root(self) -> None:
        temporary, repository = self.repository()
        self.addCleanup(temporary.cleanup)
        subdirectory = repository / "subdirectory"
        subdirectory.mkdir()
        before = subprocess.run(
            ("git", "-C", os.fspath(repository), "status", "--porcelain=v1", "-z"),
            check=True,
            capture_output=True,
        ).stdout
        repository_state.capture(repository, ("missing.txt",))
        after = subprocess.run(
            ("git", "-C", os.fspath(repository), "status", "--porcelain=v1", "-z"),
            check=True,
            capture_output=True,
        ).stdout
        self.assertEqual(before, after)
        with self.assertRaises(repository_state.StateCaptureError):
            repository_state.capture(subdirectory)

    def test_ambient_git_redirection_does_not_change_identity(self) -> None:
        temporary, repository = self.repository()
        self.addCleanup(temporary.cleanup)
        control = repository_state.capture(repository)
        alternate = repository.parent / "alternate-index"
        with mock.patch.dict(os.environ, {"GIT_INDEX_FILE": os.fspath(alternate)}):
            polluted = repository_state.capture(repository)
        self.assertEqual(control.identity, polluted.identity)

    def test_git_discovered_untracked_nested_repository_fails_closed(self) -> None:
        temporary, repository = self.repository()
        self.addCleanup(temporary.cleanup)
        nested = repository / "nested"
        nested.mkdir()
        self.git(nested, "init", "-q", "-b", "main")
        (nested / "nested.txt").write_text("nested\n", encoding="utf-8")
        self.git(nested, "add", "nested.txt")
        self.git(nested, "commit", "-q", "-m", "nested")
        self.assertIn("nested/", repository_state.discover_repository_paths(repository))

        with self.assertRaisesRegex(
            repository_state.StateCaptureError,
            "unsupported filesystem object kind",
        ):
            repository_state.capture(repository)

    def test_git_discovered_submodule_fails_closed_unless_explicit(self) -> None:
        temporary, repository = self.repository()
        source_temporary, source = self.repository()
        self.addCleanup(temporary.cleanup)
        self.addCleanup(source_temporary.cleanup)
        self.git(
            repository,
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            os.fspath(source),
            "submod",
        )
        self.git(repository, "commit", "-q", "-m", "submodule")
        self.assertIn("submod", repository_state.discover_repository_paths(repository))

        with self.assertRaisesRegex(
            repository_state.StateCaptureError,
            "unsupported filesystem object kind",
        ):
            repository_state.capture(repository)
        explicit = repository_state.capture(repository, ("submod",))
        self.assertEqual(("submod",), explicit.scope_paths)

    def test_discovered_nested_repository_succeeds_only_when_explicit(self) -> None:
        temporary, repository = self.repository()
        self.addCleanup(temporary.cleanup)
        nested = repository / "nested"
        nested.mkdir()
        self.git(nested, "init", "-q", "-b", "main")
        with self.assertRaises(repository_state.StateCaptureError):
            repository_state.capture(repository)

        explicit = repository_state.capture(repository, ("nested",))
        self.assertEqual(("nested",), explicit.scope_paths)
        (nested / "ignored-internal.txt").write_text("changed\n", encoding="utf-8")
        after_internal_change = repository_state.capture(repository, ("nested",))
        self.assertEqual(explicit.identity, after_internal_change.identity)

    def test_discovery_canonicalization_changes_only_one_terminal_marker(self) -> None:
        self.assertEqual(
            b"ordinary.txt",
            repository_state._discovered_path_identity(b"ordinary.txt"),
        )
        self.assertEqual(
            b"nested",
            repository_state._discovered_path_identity(b"nested/"),
        )
        self.assertEqual(
            b"nested/",
            repository_state._discovered_path_identity(b"nested//"),
        )
        self.assertEqual(b"/", repository_state._discovered_path_identity(b"/"))

    def test_old_repository_integrity_state_module_is_removed(self) -> None:
        old_module = Path(repository_state.__file__).with_name(
            "repository_integrity_state.py"
        )
        self.assertFalse(old_module.exists())


if __name__ == "__main__":
    unittest.main()
