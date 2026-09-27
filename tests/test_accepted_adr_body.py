from __future__ import annotations

import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from proto_ring.accepted_adr_body import check


ADR_NAME = "adr-001-example.md"
ADR_PATH = Path("docs/adr") / ADR_NAME


class GitRepository:
    def __init__(self, root: Path) -> None:
        self.root = root
        root.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "Test Author")
        self.git("config", "user.email", "test@example.invalid")
        (root / "AGENTS.md").write_text(
            "---\n"
            "repository_governance:\n"
            "  architecture_decisions:\n"
            "    profile_path: config/adr-profile.yaml\n"
            "---\n"
            "# Test directives\n",
            encoding="utf-8",
        )
        profile = root / "config/adr-profile.yaml"
        profile.parent.mkdir()
        profile.write_text(
            "repository:\n"
            "  adr_directory: docs/adr\n"
            "  filename_pattern: '^adr-(?P<number>[0-9]{3})-"
            "[a-z0-9]+(?:-[a-z0-9]+)*\\.md$'\n"
            "  id_pattern: '^ADR-[0-9]{3}$'\n"
            "  id_width: 3\n",
            encoding="utf-8",
        )

    def git(self, *args: str, input_bytes: bytes | None = None) -> bytes:
        result = subprocess.run(
            ["git", *args],
            cwd=self.root,
            input=input_bytes,
            capture_output=True,
            check=False,
        )
        if result.returncode != 0:
            raise AssertionError(result.stderr.decode(errors="replace"))
        return result.stdout

    def write_adr(
        self,
        *,
        status: str,
        body: bytes,
        name: str = ADR_NAME,
        adr_id: str = "ADR-001",
        directory: str = "docs/adr",
    ) -> Path:
        path = self.root / directory / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(
            b"---\n"
            + f'id: "{adr_id}"\nstatus: "{status}"\n'.encode()
            + b"---\n\n# Decision\n\n## Context\n\n"
            + body
            + b"\n"
        )
        return path

    def commit(self, message: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-qm", message)
        return self.git("rev-parse", "HEAD").decode().strip()


class AcceptedAdrBodyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory(prefix="proto-ring-accepted-body-")
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.repository = GitRepository(self.base / "repository")

    def assert_passes(self, repository: Path | None = None) -> None:
        self.assertEqual([], check(repository or self.repository.root))

    def assert_fails(self, fragment: str, repository: Path | None = None) -> None:
        errors = check(repository or self.repository.root)
        self.assertTrue(any(fragment in error for error in errors), errors)

    def test_non_git_repository_fails(self) -> None:
        non_git = self.base / "non-git"
        non_git.mkdir()
        self.assert_fails("Git worktree", non_git)

    def test_shallow_repository_fails(self) -> None:
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted")
        shallow = self.base / "shallow"
        result = subprocess.run(
            [
                "git",
                "clone",
                "-q",
                "--depth",
                "1",
                self.repository.root.resolve().as_uri(),
                str(shallow),
            ],
            capture_output=True,
            check=False,
        )
        self.assertEqual(0, result.returncode, result.stderr)
        self.assert_fails("complete non-shallow Git history", shallow)

    def test_direct_accepted_creation_unchanged_passes(self) -> None:
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted")
        self.assert_passes()

    def test_proposed_body_may_change_before_acceptance(self) -> None:
        self.repository.write_adr(status="proposed", body=b"A")
        self.repository.commit("proposed A")
        self.repository.write_adr(status="proposed", body=b"B")
        self.repository.commit("proposed B")
        self.repository.write_adr(status="accepted", body=b"C")
        self.repository.commit("accepted C")
        self.assert_passes()

    def test_post_acceptance_committed_rewrite_fails(self) -> None:
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted A")
        self.repository.write_adr(status="accepted", body=b"B")
        self.repository.commit("accepted B")
        self.assert_fails("accepted ADR body changed")

    def test_rewrite_then_revert_still_fails(self) -> None:
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted A")
        self.repository.write_adr(status="accepted", body=b"B")
        self.repository.commit("changed B")
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("restored A")
        self.assert_fails("accepted ADR body changed")

    def test_current_uncommitted_rewrite_fails(self) -> None:
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted A")
        self.repository.write_adr(status="accepted", body=b"B")
        self.assert_fails("current accepted ADR body changed")

    def test_first_uncommitted_acceptance_is_allowed(self) -> None:
        self.repository.write_adr(status="proposed", body=b"A")
        self.repository.commit("proposed A")
        self.repository.write_adr(status="accepted", body=b"B")
        self.assert_passes()

    def test_accepted_to_superseded_same_body_passes(self) -> None:
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted A")
        self.repository.write_adr(status="superseded", body=b"A")
        self.repository.commit("superseded A")
        self.assert_passes()

    def test_accepted_to_superseded_changed_body_fails(self) -> None:
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted A")
        self.repository.write_adr(status="superseded", body=b"B")
        self.repository.commit("superseded B")
        self.assert_fails("accepted ADR body changed")

    def test_first_structured_superseded_state_creates_no_seal(self) -> None:
        self.repository.write_adr(status="superseded", body=b"A")
        self.repository.commit("legacy superseded")
        self.assert_passes()

    def test_slug_rename_preserving_id_and_body_passes(self) -> None:
        old = self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted old")
        old.rename(old.with_name("adr-001-renamed.md"))
        self.repository.commit("renamed")
        self.assert_passes()

    def test_slug_rename_changing_body_fails(self) -> None:
        old = self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted old")
        old.unlink()
        self.repository.write_adr(
            name="adr-001-renamed.md", status="accepted", body=b"B"
        )
        self.repository.commit("renamed and changed")
        self.assert_fails("accepted ADR body changed")

    def test_post_acceptance_symlink_state_fails_even_after_restore(self) -> None:
        path = self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted regular")
        path.unlink()
        path.symlink_to("../outside.md")
        self.repository.commit("symlink state")
        path.unlink()
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("restored regular")
        self.assert_fails(
            "historical ADR slot ADR-001 is not a regular file after acceptance"
        )

    def test_post_acceptance_unstructured_state_fails_after_restore(self) -> None:
        path = self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted structured")
        path.write_bytes(b"# Legacy\n\n## Context\n\nB\n")
        self.repository.commit("unstructured state")
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("restored structured")
        self.assert_fails("historical ADR ADR-001 is not parseable after acceptance")

    def test_git_replace_refs_are_ignored(self) -> None:
        path = self.repository.write_adr(status="accepted", body=b"A")
        accepted_commit = self.repository.commit("accepted A")
        original = path.read_bytes()
        self.repository.write_adr(status="accepted", body=b"B")
        self.repository.git("add", "-A")
        replacement_tree = self.repository.git("write-tree").decode().strip()
        path.write_bytes(original)
        self.repository.git("add", "-A")
        replacement = self.repository.git(
            "commit-tree", replacement_tree, "-m", "replacement"
        ).decode().strip()
        self.repository.git("replace", accepted_commit, replacement)
        visible = self.repository.git("show", f"{accepted_commit}:{ADR_PATH}")
        self.assertIn(b"\nB\n", visible)
        self.assert_passes()

    def test_same_id_impostor_outside_corpus_is_ignored(self) -> None:
        self.repository.write_adr(status="accepted", body=b"A")
        self.repository.commit("accepted canonical")
        self.repository.write_adr(
            name="adr-001-impostor.md",
            status="accepted",
            body=b"B",
            directory="outside",
        )
        self.repository.commit("outside impostor")
        self.assert_passes()


if __name__ == "__main__":
    unittest.main()
