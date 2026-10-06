#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import tempfile
import unittest
from unittest import mock

from proto_ring import git_whitespace

from git_whitespace_test_support import (
    ZERO_SHA,
    commit_all,
    make_fixture,
    write_event,
)


class GitWhitespaceCheckerTests(unittest.TestCase):
    def test_clean_repository_passes(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
                commit_all(fixture, "clean")
                self.assertEqual([], git_whitespace.check(fixture, env={}))

    def test_unstaged_whitespace_violation_fails(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "clean.txt").write_text(
                    "first\nsecond\nthird\n", encoding="utf-8"
                )
                commit_all(fixture, "clean")
                (fixture / "clean.txt").write_text(
                    "first\nsecond   \nthird\n", encoding="utf-8"
                )

                errors = git_whitespace.check(fixture, env={})
                joined = "\n".join(errors)
                self.assertIn("unstaged", joined)
                self.assertIn("clean.txt:2", joined)

    def test_staged_whitespace_violation_fails(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
                commit_all(fixture, "clean")
                (fixture / "clean.txt").write_text("clean   \n", encoding="utf-8")
                subprocess.run(
                    ["git", "-C", str(fixture), "add", "clean.txt"],
                    check=True,
                    capture_output=True,
                )

                errors = git_whitespace.check(fixture, env={})
                joined = "\n".join(errors)
                self.assertIn("staged", joined)

    def test_github_push_range_detects_committed_whitespace(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
                before = commit_all(fixture, "clean")
                (fixture / "bad.txt").write_text("bad   \n", encoding="utf-8")
                after = commit_all(fixture, "bad")

                event_path = write_event(
                    fixture, "push-event.json", {"before": before, "after": after}
                )
                errors = git_whitespace.check(
                    fixture,
                    env={
                        "GITHUB_EVENT_NAME": "push",
                        "GITHUB_EVENT_PATH": str(event_path),
                    },
                )
                joined = "\n".join(errors)
                self.assertIn("GitHub push committed-range whitespace check", joined)
                self.assertIn("bad.txt", joined)

    def test_github_pull_request_range_detects_committed_whitespace(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
                base = commit_all(fixture, "base")
                (fixture / "bad.txt").write_text("bad   \n", encoding="utf-8")
                head = commit_all(fixture, "head")

                event_path = write_event(
                    fixture,
                    "pr-event.json",
                    {"pull_request": {"base": {"sha": base}, "head": {"sha": head}}},
                )
                errors = git_whitespace.check(
                    fixture,
                    env={
                        "GITHUB_EVENT_NAME": "pull_request",
                        "GITHUB_EVENT_PATH": str(event_path),
                    },
                )
                joined = "\n".join(errors)
                self.assertIn(
                    "GitHub pull-request committed-range whitespace check", joined
                )
                self.assertIn("bad.txt", joined)

    def test_github_push_root_range_detects_committed_whitespace(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "root.txt").write_text("root   \n", encoding="utf-8")
                after = commit_all(fixture, "root with whitespace")

                event_path = write_event(
                    fixture, "push-root-event.json", {"before": ZERO_SHA, "after": after}
                )
                errors = git_whitespace.check(
                    fixture,
                    env={
                        "GITHUB_EVENT_NAME": "push",
                        "GITHUB_EVENT_PATH": str(event_path),
                    },
                )
                joined = "\n".join(errors)
                self.assertIn("GitHub push committed-range whitespace check", joined)
                self.assertIn("root.txt", joined)

    def test_clean_github_push_range_passes(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "first.txt").write_text("first\n", encoding="utf-8")
                before = commit_all(fixture, "first")
                (fixture / "second.txt").write_text("second\n", encoding="utf-8")
                after = commit_all(fixture, "second")

                event_path = write_event(
                    fixture, "clean-push-event.json", {"before": before, "after": after}
                )
                self.assertEqual(
                    [],
                    git_whitespace.check(
                        fixture,
                        env={
                            "GITHUB_EVENT_NAME": "push",
                            "GITHUB_EVENT_PATH": str(event_path),
                        },
                    ),
                )

    def test_clean_github_pull_request_range_passes(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "first.txt").write_text("first\n", encoding="utf-8")
                base = commit_all(fixture, "base")
                (fixture / "second.txt").write_text("second\n", encoding="utf-8")
                head = commit_all(fixture, "head")

                event_path = write_event(
                    fixture,
                    "clean-pr-event.json",
                    {"pull_request": {"base": {"sha": base}, "head": {"sha": head}}},
                )
                self.assertEqual(
                    [],
                    git_whitespace.check(
                        fixture,
                        env={
                            "GITHUB_EVENT_NAME": "pull_request",
                            "GITHUB_EVENT_PATH": str(event_path),
                        },
                    ),
                )

    def test_other_github_event_does_not_require_event_payload(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
                commit_all(fixture, "clean")

                errors = git_whitespace.check(
                    fixture, env={"GITHUB_EVENT_NAME": "workflow_dispatch"}
                )
                self.assertEqual([], errors)

    def test_other_github_event_still_checks_local_whitespace(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)

                (fixture / "clean.txt").write_text(
                    "clean\n",
                    encoding="utf-8",
                )

                commit_all(fixture, "clean")

                (fixture / "clean.txt").write_text(
                    "bad   \n",
                    encoding="utf-8",
                )

                errors = git_whitespace.check(
                    fixture,
                    env={
                        "GITHUB_EVENT_NAME": "workflow_dispatch",
                    },
                )

                joined = "\n".join(errors)
                self.assertIn("unstaged whitespace check", joined)
                self.assertIn("clean.txt:1", joined)

    def test_git_process_start_failure_is_controlled(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                with mock.patch(
                    "proto_ring.git_whitespace.subprocess.run",
                    side_effect=OSError("git unavailable"),
                ):
                    errors = git_whitespace.check(fixture, env={})
                self.assertTrue(errors)
                self.assertIn("unstaged whitespace check", "\n".join(errors))

    def test_root_push_empty_tree_process_start_failure_is_controlled(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                event_path = write_event(
                    fixture,
                    "root-push.json",
                    {"before": ZERO_SHA, "after": "revision"},
                )
                with mock.patch(
                    "proto_ring.git_whitespace.subprocess.run",
                    side_effect=OSError("git unavailable"),
                ):
                    errors = git_whitespace.check(
                        fixture,
                        env={
                            "GITHUB_EVENT_NAME": "push",
                            "GITHUB_EVENT_PATH": str(event_path),
                        },
                    )
                self.assertTrue(errors)
                self.assertIn("cannot resolve the Git empty tree", "\n".join(errors))


if __name__ == "__main__":
    unittest.main()
