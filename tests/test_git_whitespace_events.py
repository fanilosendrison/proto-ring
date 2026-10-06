from __future__ import annotations

import os
import tempfile
import unittest
from unittest import mock

from proto_ring import git_whitespace

from git_whitespace_test_support import (
    commit_all,
    make_fixture,
    write_event,
)


class GitWhitespaceEventTests(unittest.TestCase):
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

    def test_supplied_environment_ignores_ambient_github_event(self) -> None:
            with tempfile.TemporaryDirectory() as temporary:
                fixture = make_fixture(temporary)
                (fixture / "clean.txt").write_text("clean\n", encoding="utf-8")
                commit_all(fixture, "clean")
                with mock.patch.dict(
                    os.environ,
                    {
                        "GITHUB_EVENT_NAME": "push",
                        "GITHUB_EVENT_PATH": str(fixture / "missing.json"),
                    },
                ):
                    self.assertEqual([], git_whitespace.check(fixture, env={}))


if __name__ == "__main__":
    unittest.main()
