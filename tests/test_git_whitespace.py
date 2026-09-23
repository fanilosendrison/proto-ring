#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from proto_ring import git_whitespace

ZERO_SHA = "0" * 40


def make_fixture(temporary: str) -> Path:
    fixture = Path(temporary) / "repo"
    fixture.mkdir()
    subprocess.run(["git", "init", "-q", str(fixture)], check=True)
    subprocess.run(
        ["git", "-C", str(fixture), "config", "user.name", "proto-ring Test"],
        check=True,
        capture_output=True,
    )
    subprocess.run(
        ["git", "-C", str(fixture), "config", "user.email", "proto-ring@example.invalid"],
        check=True,
        capture_output=True,
    )
    return fixture


def commit_all(fixture: Path, message: str) -> str:
    subprocess.run(
        ["git", "-C", str(fixture), "add", "-A"], check=True, capture_output=True
    )
    subprocess.run(
        ["git", "-C", str(fixture), "commit", "-q", "-m", message],
        check=True,
        capture_output=True,
    )
    result = subprocess.run(
        ["git", "-C", str(fixture), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def write_event(fixture: Path, name: str, payload: dict) -> Path:
    event_path = fixture.parent / name
    event_path.write_text(json.dumps(payload), encoding="utf-8")
    return event_path


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

    def test_github_push_with_missing_event_data_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)
            (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
            commit_all(fixture, "clean")

            event_path = write_event(fixture, "bad-event.json", {"after": "abc"})
            errors = git_whitespace.check(
                fixture,
                env={
                    "GITHUB_EVENT_NAME": "push",
                    "GITHUB_EVENT_PATH": str(event_path),
                },
            )
            joined = "\n".join(errors)
            self.assertIn("cannot determine GitHub push whitespace range", joined)
            self.assertIn("event.before/event.after missing", joined)

    def test_github_push_with_missing_after_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)

            (fixture / "clean.txt").write_text(
                "clean\n",
                encoding="utf-8",
            )

            before = commit_all(fixture, "before")

            event_path = write_event(
                fixture,
                "missing-after-push-event.json",
                {
                    "before": before,
                },
            )

            errors = git_whitespace.check(
                fixture,
                env={
                    "GITHUB_EVENT_NAME": "push",
                    "GITHUB_EVENT_PATH": str(event_path),
                },
            )

            joined = "\n".join(errors)
            self.assertIn(
                "cannot determine GitHub push whitespace range",
                joined,
            )

            self.assertIn(
                "event.before/event.after missing",
                joined,
            )

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

    def test_github_push_without_event_path_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)
            (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
            commit_all(fixture, "clean")

            errors = git_whitespace.check(
                fixture, env={"GITHUB_EVENT_NAME": "push"}
            )
            self.assertTrue(errors)
            joined = "\n".join(errors)
            self.assertIn("cannot determine GitHub push whitespace range", joined)
            self.assertIn("GITHUB_EVENT_PATH is missing", joined)

    def test_github_push_with_nonexistent_event_file_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)

            (fixture / "clean.txt").write_text(
                "clean\n",
                encoding="utf-8",
            )

            commit_all(fixture, "clean")

            event_path = fixture.parent / "does-not-exist-event.json"
            self.assertFalse(event_path.exists())

            errors = git_whitespace.check(
                fixture,
                env={
                    "GITHUB_EVENT_NAME": "push",
                    "GITHUB_EVENT_PATH": str(event_path),
                },
            )

            joined = "\n".join(errors)
            self.assertIn(
                "cannot determine GitHub push whitespace range",
                joined,
            )

            self.assertIn(
                "does-not-exist-event.json",
                joined,
            )

    def test_github_pull_request_without_event_path_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)
            (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
            commit_all(fixture, "clean")

            errors = git_whitespace.check(
                fixture, env={"GITHUB_EVENT_NAME": "pull_request"}
            )
            self.assertTrue(errors)
            joined = "\n".join(errors)
            self.assertIn(
                "cannot determine GitHub pull_request whitespace range", joined
            )
            self.assertIn("GITHUB_EVENT_PATH is missing", joined)

    def test_github_push_with_malformed_event_json_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)
            (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
            commit_all(fixture, "clean")
            event_path = fixture.parent / "malformed-push-event.json"
            event_path.write_text("{not json", encoding="utf-8")

            errors = git_whitespace.check(
                fixture,
                env={
                    "GITHUB_EVENT_NAME": "push",
                    "GITHUB_EVENT_PATH": str(event_path),
                },
            )
            self.assertTrue(errors)
            joined = "\n".join(errors)
            self.assertIn("cannot determine GitHub push whitespace range", joined)
            self.assertIn("line 1", joined)

    def test_github_push_with_invalid_utf8_event_file_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)

            (fixture / "clean.txt").write_text(
                "clean\n",
                encoding="utf-8",
            )

            commit_all(fixture, "clean")

            event_path = fixture.parent / "invalid-utf8-event.json"
            event_path.write_bytes(
                b"\xff\xfe\xfa"
            )

            errors = git_whitespace.check(
                fixture,
                env={
                    "GITHUB_EVENT_NAME": "push",
                    "GITHUB_EVENT_PATH": str(event_path),
                },
            )

            joined = "\n".join(errors)
            self.assertIn(
                "cannot determine GitHub push whitespace range",
                joined,
            )

            self.assertIn(
                "utf-8",
                joined.lower(),
            )

    def test_github_pull_request_with_malformed_event_json_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)
            (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
            commit_all(fixture, "clean")
            event_path = fixture.parent / "malformed-pr-event.json"
            event_path.write_text("{not json", encoding="utf-8")

            errors = git_whitespace.check(
                fixture,
                env={
                    "GITHUB_EVENT_NAME": "pull_request",
                    "GITHUB_EVENT_PATH": str(event_path),
                },
            )
            self.assertTrue(errors)
            joined = "\n".join(errors)
            self.assertIn(
                "cannot determine GitHub pull_request whitespace range", joined
            )
            self.assertIn("line 1", joined)

    def test_github_event_json_must_be_object(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)
            (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
            commit_all(fixture, "clean")
            event_path = fixture.parent / "non-object-event.json"
            event_path.write_text("[]", encoding="utf-8")

            errors = git_whitespace.check(
                fixture,
                env={
                    "GITHUB_EVENT_NAME": "push",
                    "GITHUB_EVENT_PATH": str(event_path),
                },
            )
            joined = "\n".join(errors)
            self.assertIn("event JSON must be an object", joined)

    def test_github_pull_request_with_missing_sha_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)
            (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
            base = commit_all(fixture, "base")
            event_path = write_event(
                fixture,
                "missing-sha-pr-event.json",
                {"pull_request": {"base": {"sha": base}, "head": {}}},
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
                "cannot determine GitHub pull-request whitespace range", joined
            )
            self.assertIn("base.sha/head.sha missing", joined)

    def test_github_pull_request_with_missing_base_sha_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = make_fixture(temporary)

            (fixture / "clean.txt").write_text(
                "clean\n",
                encoding="utf-8",
            )

            head = commit_all(fixture, "head")

            event_path = write_event(
                fixture,
                "missing-base-sha-pr-event.json",
                {
                    "pull_request": {
                        "base": {},
                        "head": {
                            "sha": head,
                        },
                    },
                },
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
                "cannot determine GitHub pull-request whitespace range",
                joined,
            )

            self.assertIn(
                "base.sha/head.sha missing",
                joined,
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


if __name__ == "__main__":
    unittest.main()
