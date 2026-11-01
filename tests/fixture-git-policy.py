"""Exact Git command surface required by the immutable conformance corpus."""
from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile


_FIXED = {
    ("-c", "protocol.file.allow=always", "submodule", "add", "-q", "../submodule-source", "submod"),
    ("-c", "core.autocrlf=true", "clone", "-q", ".", "external-bootstrap"),
    ("add", "-A"), ("add", ".gitignore"), ("add", "a.txt", "b.txt"),
    ("add", "clean.txt"), ("add", "global-ignore", "global-config"),
    ("add", "tracked.txt"), ("branch", "-m", "renamed"),
    ("checkout", "-q", "--detach", "HEAD"), ("checkout", "-q", "-b", "side"),
    ("checkout", "-q", "-b", "theirs"), ("checkout", "-q", "main"),
    ("clean", "-q", "-f", "-x", "--", "external-link"),
    ("clone", "-q", ".", "../alternate"), ("clone", "-q", ".", "../external-bootstrap"),
    ("clone", "-q", ".", "external-bootstrap"),
    ("clone", "-q", ".", "../submodule-source"),
    ("config", "user.email", "corpus@example.invalid"),
    ("config", "user.email", "compat@example.invalid"),
    ("config", "user.name", "Corpus Author"),
    ("config", "user.name", "Compatibility Test"), ("hash-object", "-w", "--stdin"),
    ("init", "--bare", "-q", "../object-store"), ("init", "--quiet"),
    ("init", "-q", "-b", "main"), ("init", "-q", "nested"),
    ("merge", "-q", "--no-ff", "-s", "ours", "side", "-m", "merge side"),
    ("mv", "docs/adr/adr-001-example.md", "docs/adr/adr-001-renamed.md"),
    ("rev-parse", "--show-toplevel"), ("rev-parse", ":a.txt"),
    ("rev-parse", ":b.txt"), ("rev-parse", "HEAD"),
    ("update-index", "--chmod=+x", "tracked.txt"),
    ("update-index", "-z", "--index-info"), ("write-tree",),
}
_COMMIT_MESSAGES = {
    "accepted", "accepted A", "accepted B", "accepted C", "accepted old", "bad", "base",
    "baseline", "changed B", "clean", "global config fixture", "governance fixture", "governed fixture",
    "head bad", "ignore", "ignore link", "ignore tracked", "ignore untracked", "impostor",
    "legacy superseded", "ours", "proposed A", "proposed B", "raw path", "renamed",
    "renamed changed", "restore", "restored A", "root bad", "side accepted", "submodule",
    "superseded A", "superseded B", "symlink", "theirs", "unstructured",
}


def run(
    arguments: list[str],
    *,
    cwd: Path,
    input_bytes: bytes | None = None,
    git_executable: Path | None = None,
    trusted_environment: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[bytes]:
    if not allowed(arguments):
        raise ValueError(
            f"Git fixture arguments are not allowed by the corpus surface: {tuple(arguments)!r}"
        )
    with tempfile.TemporaryDirectory(prefix="proto-ring-fixture-git-") as directory:
        control = Path(directory)
        home = control / "home"
        hooks = control / "hooks"
        template = control / "template"
        for path in (home, hooks, template):
            path.mkdir()
        global_config = control / "global.gitconfig"
        global_config.write_bytes(b"")
        if git_executable is None:
            resolved = shutil.which("git")
            if resolved is None:
                raise ValueError("Git executable is unavailable")
            git_executable = Path(resolved).resolve()
        if not git_executable.is_absolute() or not git_executable.is_file():
            raise ValueError("Git executable identity is invalid")
        if trusted_environment is None:
            trusted_environment = {
                name: os.environ[name]
                for name in ("PATH", "SYSTEMROOT", "WINDIR", "PATHEXT", "COMSPEC")
                if name in os.environ
            }
        allowed_environment = {"PATH", "SYSTEMROOT", "WINDIR", "PATHEXT", "COMSPEC"}
        if not set(trusted_environment).issubset(allowed_environment):
            raise ValueError("trusted Git environment contains an unsupported key")
        environment = dict(trusted_environment)
        environment.update({
            "HOME": str(home), "XDG_CONFIG_HOME": str(home),
            "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": str(global_config),
            "GIT_TEMPLATE_DIR": str(template), "GIT_TERMINAL_PROMPT": "0",
            "GIT_EDITOR": "false", "GIT_SEQUENCE_EDITOR": "false",
            "GIT_MERGE_AUTOEDIT": "no", "GIT_PAGER": "", "PAGER": "",
            "GIT_AUTHOR_NAME": "Corpus Author", "GIT_AUTHOR_EMAIL": "corpus@example.invalid",
            "GIT_COMMITTER_NAME": "Corpus Committer", "GIT_COMMITTER_EMAIL": "corpus@example.invalid",
            "GIT_AUTHOR_DATE": "2001-02-03T04:05:06+0000",
            "GIT_COMMITTER_DATE": "2001-02-03T04:05:06+0000",
        })
        _validate_local_config(cwd, arguments, environment, git_executable)
        return subprocess.run(
            [str(git_executable), "-c", f"core.hooksPath={hooks}", "-c", "core.autocrlf=false", *arguments],
            cwd=cwd, input=input_bytes, env=environment, capture_output=True, check=False,
        )


def _validate_local_config(
    cwd: Path,
    arguments: list[str],
    environment: dict[str, str],
    git_executable: Path | None = None,
) -> None:
    effective = cwd / arguments[1] if arguments[:1] == ["-C"] else cwd
    executable = str(git_executable or shutil.which("git") or "git")
    completed = subprocess.run(
        [executable, "config", "--local", "--no-includes", "--name-only", "--null", "--list"],
        cwd=effective, env=environment, capture_output=True, check=False,
    )
    if completed.returncode:
        if (effective / ".git").exists() or ((effective / "HEAD").exists() and (effective / "config").exists()):
            raise ValueError("Git local configuration is unavailable")
        return
    for raw in filter(None, completed.stdout.split(b"\0")):
        try:
            key = raw.decode("utf-8")
        except UnicodeDecodeError as error:
            raise ValueError("Git local configuration key is not UTF-8") from error
        if not _safe_local_key(key):
            raise ValueError(f"Git local configuration key is outside the fixture surface: {key}")


def _safe_local_key(key: str) -> bool:
    fixed = {
        "core.repositoryformatversion", "core.filemode", "core.bare", "core.logallrefupdates",
        "core.ignorecase", "core.precomposeunicode", "core.symlinks", "core.worktree",
        "core.hookspath", "extensions.objectformat", "extensions.refstorage",
        "extensions.worktreeconfig", "user.name", "user.email",
    }
    return (
        key in fixed
        or key.startswith("remote.") and key.endswith((".url", ".fetch"))
        or key.startswith("branch.") and key.endswith((".remote", ".merge"))
        or key.startswith("submodule.") and key.endswith((".url", ".active"))
    )


def allowed(arguments: list[str]) -> bool:
    values = tuple(arguments)
    object_id = lambda value: len(value) == 40 and all(char in "0123456789abcdefABCDEF" for char in value)
    normal_commit = (len(values) == 4 and values[:3] == ("commit", "-q", "-m")
                     and values[3] in _COMMIT_MESSAGES)
    special_commit = values in {
        ("commit", "--allow-empty", "-q", "-m", "probe"),
        ("-C", "../alternate", "commit", "--allow-empty", "-q", "-m", "alternate"),
    }
    dynamic = (
        len(values) == 5 and values[:2] == ("read-tree", "-m") and all(map(object_id, values[2:]))
    ) or (
        len(values) == 5 and values[:3] == ("update-index", "--cacheinfo", "100644")
        and object_id(values[3]) and values[4] in {"a.txt", "b.txt"}
    ) or (
        len(values) == 4 and values[0] == "commit-tree" and object_id(values[1])
        and values[2:] == ("-m", "replacement")
    ) or (
        len(values) == 3 and values[0] == "replace" and object_id(values[1]) and object_id(values[2])
    )
    return values in _FIXED or normal_commit or special_commit or dynamic
