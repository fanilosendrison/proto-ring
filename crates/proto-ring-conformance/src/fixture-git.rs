use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

use crate::fixture::{FixtureError, failure};

pub(super) fn validate(arguments: &[&str]) -> Result<(), FixtureError> {
    if allowed(arguments) {
        Ok(())
    } else {
        Err(failure(format!(
            "Git fixture arguments are not allowed by the corpus surface: {arguments:?}"
        )))
    }
}

pub(super) fn execute(
    root: &Path,
    arguments: &[&str],
    stdin: Option<&[u8]>,
) -> Result<Output, FixtureError> {
    validate(arguments)?;
    let control = ControlledGitEnvironment::new()?;
    validate_local_config(root, arguments, &control)?;
    let mut command = Command::new("git");
    apply_environment(&mut command, &control);
    command
        .current_dir(root)
        .arg("-c")
        .arg(format!("core.hooksPath={}", control.hooks.display()))
        .arg("-c")
        .arg("core.autocrlf=false")
        .args(arguments)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| failure(format!("git execution failed: {error}")))?;
    if let Some(bytes) = stdin {
        child
            .stdin
            .take()
            .ok_or_else(|| failure("git stdin was unavailable"))?
            .write_all(bytes)
            .map_err(|error| failure(format!("git stdin failed: {error}")))?;
    }
    child
        .wait_with_output()
        .map_err(|error| failure(format!("git execution failed: {error}")))
}

struct ControlledGitEnvironment {
    _temporary: TempDir,
    global_config: std::path::PathBuf,
    home: std::path::PathBuf,
    hooks: std::path::PathBuf,
    template: std::path::PathBuf,
}

impl ControlledGitEnvironment {
    fn new() -> Result<Self, FixtureError> {
        let temporary = tempfile::Builder::new()
            .prefix("proto-ring-fixture-git-")
            .tempdir()
            .map_err(|error| failure(format!("cannot create Git control root: {error}")))?;
        let home = temporary.path().join("home");
        let hooks = temporary.path().join("hooks");
        let template = temporary.path().join("template");
        fs::create_dir(&home)
            .and_then(|()| fs::create_dir(&hooks))
            .and_then(|()| fs::create_dir(&template))
            .map_err(|error| failure(format!("cannot create Git control directories: {error}")))?;
        let global_config = temporary.path().join("global.gitconfig");
        fs::write(&global_config, [])
            .map_err(|error| failure(format!("cannot create Git global config: {error}")))?;
        Ok(Self {
            _temporary: temporary,
            global_config,
            home,
            hooks,
            template,
        })
    }
}

fn validate_local_config(
    root: &Path,
    arguments: &[&str],
    control: &ControlledGitEnvironment,
) -> Result<(), FixtureError> {
    let effective_root = match arguments {
        ["-C", relative, ..] => root.join(relative),
        _ => root.to_path_buf(),
    };
    let mut command = Command::new("git");
    apply_environment(&mut command, control);
    let output = command
        .current_dir(&effective_root)
        .args([
            "config",
            "--local",
            "--no-includes",
            "--name-only",
            "--null",
            "--list",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| failure(format!("Git config inspection failed: {error}")))?;
    if !output.status.success() {
        if effective_root.join(".git").exists()
            || (effective_root.join("HEAD").exists() && effective_root.join("config").exists())
        {
            return Err(failure("Git local configuration is unavailable"));
        }
        return Ok(());
    }
    for raw in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
    {
        let key = std::str::from_utf8(raw)
            .map_err(|_| failure("Git local configuration key is not UTF-8"))?;
        if !safe_local_key(key) {
            return Err(failure(format!(
                "Git local configuration key is outside the fixture surface: {key}"
            )));
        }
    }
    Ok(())
}

fn safe_local_key(key: &str) -> bool {
    matches!(
        key,
        "core.repositoryformatversion"
            | "core.filemode"
            | "core.bare"
            | "core.logallrefupdates"
            | "core.ignorecase"
            | "core.precomposeunicode"
            | "core.symlinks"
            | "core.worktree"
            | "core.hookspath"
            | "extensions.objectformat"
            | "extensions.refstorage"
            | "extensions.worktreeconfig"
            | "user.name"
            | "user.email"
    ) || key.starts_with("remote.") && (key.ends_with(".url") || key.ends_with(".fetch"))
        || key.starts_with("branch.") && (key.ends_with(".remote") || key.ends_with(".merge"))
        || key.starts_with("submodule.") && (key.ends_with(".url") || key.ends_with(".active"))
}

fn apply_environment(command: &mut Command, control: &ControlledGitEnvironment) {
    command.env_clear();
    for name in ["PATH", "SYSTEMROOT", "WINDIR", "PATHEXT", "COMSPEC"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    for (name, value) in git_environment() {
        command.env(name, value);
    }
    command
        .env("HOME", &control.home)
        .env("XDG_CONFIG_HOME", &control.home)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", &control.global_config)
        .env("GIT_TEMPLATE_DIR", &control.template)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "false")
        .env("GIT_SEQUENCE_EDITOR", "false")
        .env("GIT_MERGE_AUTOEDIT", "no")
        .env("GIT_PAGER", "")
        .env("PAGER", "");
}

fn git_environment() -> [(&'static str, &'static str); 6] {
    [
        ("GIT_AUTHOR_NAME", "Corpus Author"),
        ("GIT_AUTHOR_EMAIL", "corpus@example.invalid"),
        ("GIT_COMMITTER_NAME", "Corpus Committer"),
        ("GIT_COMMITTER_EMAIL", "corpus@example.invalid"),
        ("GIT_AUTHOR_DATE", "2001-02-03T04:05:06+0000"),
        ("GIT_COMMITTER_DATE", "2001-02-03T04:05:06+0000"),
    ]
}

fn allowed(arguments: &[&str]) -> bool {
    match arguments {
        [
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            "../submodule-source",
            "submod",
        ]
        | [
            "-c",
            "core.autocrlf=true",
            "clone",
            "-q",
            ".",
            "external-bootstrap",
        ]
        | ["add", "-A"]
        | ["add", ".gitignore"]
        | ["add", "a.txt", "b.txt"]
        | ["add", "clean.txt"]
        | ["add", "global-ignore", "global-config"]
        | ["add", "tracked.txt"]
        | ["branch", "-m", "renamed"]
        | ["checkout", "-q", "--detach", "HEAD"]
        | ["checkout", "-q", "-b", "side"]
        | ["checkout", "-q", "-b", "theirs"]
        | ["checkout", "-q", "main"]
        | ["clean", "-q", "-f", "-x", "--", "external-link"]
        | ["clone", "-q", ".", "../alternate"]
        | ["clone", "-q", ".", "../external-bootstrap"]
        | ["clone", "-q", ".", "external-bootstrap"]
        | ["clone", "-q", ".", "../submodule-source"]
        | ["config", "user.email", "corpus@example.invalid"]
        | ["config", "user.email", "compat@example.invalid"]
        | ["config", "user.name", "Corpus Author"]
        | ["config", "user.name", "Compatibility Test"]
        | ["hash-object", "-w", "--stdin"]
        | ["init", "--bare", "-q", "../object-store"]
        | ["init", "--quiet"]
        | ["init", "-q", "-b", "main"]
        | ["init", "-q", "nested"]
        | [
            "merge",
            "-q",
            "--no-ff",
            "-s",
            "ours",
            "side",
            "-m",
            "merge side",
        ]
        | [
            "mv",
            "docs/adr/adr-001-example.md",
            "docs/adr/adr-001-renamed.md",
        ]
        | ["rev-parse", "--show-toplevel"]
        | ["rev-parse", ":a.txt"]
        | ["rev-parse", ":b.txt"]
        | ["rev-parse", "HEAD"]
        | ["update-index", "--chmod=+x", "tracked.txt"]
        | ["update-index", "-z", "--index-info"]
        | ["write-tree"] => true,
        ["commit", "-q", "-m", message] => COMMIT_MESSAGES.contains(message),
        ["commit", "--allow-empty", "-q", "-m", "probe"]
        | [
            "-C",
            "../alternate",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            "alternate",
        ] => true,
        ["read-tree", "-m", base, ours, theirs] => {
            [base, ours, theirs].iter().all(|value| object_id(value))
        }
        ["update-index", "--cacheinfo", "100644", oid, path] => {
            object_id(oid) && matches!(*path, "a.txt" | "b.txt")
        }
        ["commit-tree", tree, "-m", "replacement"] => object_id(tree),
        ["replace", accepted, replacement] => object_id(accepted) && object_id(replacement),
        _ => false,
    }
}

const COMMIT_MESSAGES: &[&str] = &[
    "accepted",
    "accepted A",
    "accepted B",
    "accepted C",
    "accepted old",
    "bad",
    "base",
    "baseline",
    "changed B",
    "clean",
    "global config fixture",
    "governance fixture",
    "governed fixture",
    "head bad",
    "ignore",
    "ignore link",
    "ignore tracked",
    "ignore untracked",
    "impostor",
    "legacy superseded",
    "ours",
    "proposed A",
    "proposed B",
    "raw path",
    "renamed",
    "renamed changed",
    "restore",
    "restored A",
    "root bad",
    "side accepted",
    "submodule",
    "superseded A",
    "superseded B",
    "symlink",
    "theirs",
    "unstructured",
];

fn object_id(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
