use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const READ_ONLY_SUBCOMMANDS: [&str; 5] =
    ["rev-parse", "rev-list", "diff-tree", "ls-tree", "cat-file"];

pub(super) struct TreeEntry {
    pub(super) mode: String,
    pub(super) object_type: String,
    pub(super) object_id: String,
}

pub(super) struct GitRepository {
    root: PathBuf,
}

#[derive(Debug)]
pub(super) struct GitError(String);

impl Display for GitError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl GitRepository {
    pub(super) fn open(repository: &Path) -> Result<Self, GitError> {
        if !fs::metadata(repository)
            .map_err(|error| failure(format!("cannot inspect repository: {error}")))?
            .is_dir()
        {
            return Err(failure("repository must be an existing directory"));
        }
        let root = repository
            .canonicalize()
            .map_err(|error| failure(format!("cannot resolve repository: {error}")))?;
        let git = Self { root };

        let top = git.text_output(["rev-parse", "--show-toplevel"])?;
        let top = single_line(&top, "Git worktree root")?;
        let resolved_top = PathBuf::from(top)
            .canonicalize()
            .map_err(|error| failure(format!("cannot resolve Git worktree root: {error}")))?;
        if resolved_top != git.root {
            return Err(failure("repository must be the exact Git worktree root"));
        }

        let head = git.text_output(["rev-parse", "--verify", "HEAD^{commit}"])?;
        if single_line(&head, "committed HEAD")?.is_empty() {
            return Err(failure("repository must have a committed HEAD"));
        }

        match single_line(
            &git.text_output(["rev-parse", "--is-shallow-repository"])?,
            "shallow-history state",
        )? {
            "false" => {}
            "true" => {
                return Err(failure(
                    "accepted ADR body verification requires non-shallow history",
                ));
            }
            _ => return Err(failure("cannot determine whether Git history is shallow")),
        }
        Ok(git)
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn first_parent_commits(&self) -> Result<Vec<String>, GitError> {
        let output = self.text_output(["rev-list", "--first-parent", "--reverse", "HEAD"])?;
        let commits: Vec<_> = output.lines().map(str::to_owned).collect();
        if commits.is_empty() || commits.iter().any(|commit| !object_id(commit)) {
            return Err(failure("first-parent commit list is malformed"));
        }
        Ok(commits)
    }

    pub(super) fn first_parent(&self, commit: &str) -> Result<Option<String>, GitError> {
        let output = self.text_output(["rev-list", "--parents", "-n", "1", commit])?;
        let fields: Vec<_> = output.split_whitespace().collect();
        if fields.first() != Some(&commit) || fields.iter().any(|field| !object_id(field)) {
            return Err(failure(format!(
                "cannot determine first parent for commit {commit}"
            )));
        }
        Ok(fields.get(1).map(|parent| (*parent).to_owned()))
    }

    pub(super) fn changed_paths(
        &self,
        commit: &str,
        first_parent: Option<&str>,
    ) -> Result<Vec<Vec<u8>>, GitError> {
        let mut arguments = [
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "--no-renames",
            "-z",
        ]
        .into_iter()
        .map(OsString::from)
        .collect::<Vec<_>>();
        match first_parent {
            Some(parent) => {
                arguments.push(parent.into());
                arguments.push(commit.into());
            }
            None => {
                arguments.push("--root".into());
                arguments.push(commit.into());
            }
        }
        let mut paths = self
            .bytes_output(arguments)?
            .split(|byte| *byte == 0)
            .map(<[u8]>::to_vec)
            .collect::<Vec<_>>();
        if paths.last().is_some_and(Vec::is_empty) {
            paths.pop();
        }
        if paths.iter().any(Vec::is_empty) {
            return Err(failure("changed-path output contains an empty record"));
        }
        Ok(paths)
    }

    pub(super) fn tree_entry(
        &self,
        commit: &str,
        path: &str,
    ) -> Result<Option<TreeEntry>, GitError> {
        let output = self.bytes_output([
            OsString::from("ls-tree"),
            OsString::from("-z"),
            OsString::from(commit),
            OsString::from("--"),
            OsString::from(path),
        ])?;
        if output.is_empty() {
            return Ok(None);
        }
        let records: Vec<_> = output
            .strip_suffix(&[0])
            .ok_or_else(|| failure("historical tree output is not NUL-terminated"))?
            .split(|byte| *byte == 0)
            .collect();
        if records.len() != 1 {
            return Err(failure("historical path has multiple tree entries"));
        }
        let separator = records[0]
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or_else(|| failure("historical tree entry is malformed"))?;
        let (header, reported_path_with_tab) = records[0].split_at(separator);
        let reported_path = &reported_path_with_tab[1..];
        if reported_path != path.as_bytes() {
            return Err(failure("historical tree entry path does not match request"));
        }
        let fields: Vec<_> = header.split(|byte| *byte == b' ').collect();
        if fields.len() != 3 {
            return Err(failure("historical tree entry header is malformed"));
        }
        Ok(Some(TreeEntry {
            mode: ascii(fields[0], "historical tree mode")?,
            object_type: ascii(fields[1], "historical object type")?,
            object_id: ascii(fields[2], "historical object ID")?,
        }))
    }

    pub(super) fn blob(&self, object_id: &str) -> Result<Vec<u8>, GitError> {
        self.bytes_output(["cat-file", "blob", object_id])
    }

    fn text_output<I, S>(&self, arguments: I) -> Result<String, GitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        String::from_utf8(self.bytes_output(arguments)?)
            .map_err(|_| failure("Git textual output is not UTF-8"))
    }

    fn bytes_output<I, S>(&self, arguments: I) -> Result<Vec<u8>, GitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = self.run(arguments)?;
        if output.status.success() {
            return Ok(output.stdout);
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(failure(format!("Git read failed: {}", stderr.trim())))
    }

    fn run<I, S>(&self, arguments: I) -> Result<Output, GitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let arguments: Vec<OsString> = arguments
            .into_iter()
            .map(|argument| argument.as_ref().to_owned())
            .collect();
        let subcommand = arguments
            .first()
            .and_then(|argument| argument.to_str())
            .ok_or_else(|| failure("Git read operation has no textual subcommand"))?;
        if !READ_ONLY_SUBCOMMANDS.contains(&subcommand) {
            return Err(failure("unsupported Git read operation"));
        }
        let environment = env::vars_os().filter(|(name, _)| !git_environment_name(name));
        Command::new("git")
            .env_clear()
            .envs(environment)
            .args(&arguments)
            .current_dir(&self.root)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", platform_null_file())
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .output()
            .map_err(|error| failure(format!("cannot start Git read operation: {error}")))
    }
}

fn single_line<'a>(value: &'a str, label: &str) -> Result<&'a str, GitError> {
    let value = value.strip_suffix('\n').unwrap_or(value);
    let value = value.strip_suffix('\r').unwrap_or(value);
    if value.is_empty() || value.contains('\r') || value.contains('\n') {
        return Err(failure(format!("{label} is not one nonempty line")));
    }
    Ok(value)
}

fn object_id(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn ascii(value: &[u8], label: &str) -> Result<String, GitError> {
    if !value.is_ascii() {
        return Err(failure(format!("{label} is not ASCII")));
    }
    Ok(std::str::from_utf8(value)
        .expect("ASCII is valid UTF-8")
        .to_owned())
}

#[cfg(unix)]
fn git_environment_name(name: &OsStr) -> bool {
    name.as_encoded_bytes().starts_with(b"GIT_")
}

#[cfg(windows)]
fn git_environment_name(name: &OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let prefix = b"GIT_";
    name.encode_wide()
        .zip(prefix)
        .all(|(unit, expected)| windows_ascii_upper(unit) == u16::from(*expected))
        && name.encode_wide().count() >= prefix.len()
}

#[cfg(windows)]
fn windows_ascii_upper(unit: u16) -> u16 {
    if unit >= u16::from(b'a') && unit <= u16::from(b'z') {
        unit - u16::from(b'a' - b'A')
    } else {
        unit
    }
}

#[cfg(unix)]
fn platform_null_file() -> &'static str {
    "/dev/null"
}

#[cfg(windows)]
fn platform_null_file() -> &'static str {
    "NUL"
}

fn failure(message: impl Into<String>) -> GitError {
    GitError(message.into())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::git_environment_name;

    #[test]
    fn ambient_git_environment_is_removed_at_the_process_boundary() {
        for name in [
            "GIT_DIR",
            "GIT_OBJECT_DIRECTORY",
            "GIT_GRAFT_FILE",
            "GIT_SHALLOW_FILE",
            "GIT_CONFIG_KEY_0",
            "GIT_TRACE2",
        ] {
            assert!(git_environment_name(OsStr::new(name)), "{name}");
        }
        for name in ["PATH", "HOME", "SYSTEMROOT", "PROTO_RING_EXAMPLE"] {
            assert!(!git_environment_name(OsStr::new(name)), "{name}");
        }
    }
}
