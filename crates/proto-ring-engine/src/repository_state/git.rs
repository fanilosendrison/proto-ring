use super::{
    AmbientEnvironment, StateCaptureError,
    git_discovery::{DiscoveredPath, DiscoverySource, parse_discovered_paths},
    git_environment::removed_git_name,
    path,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepositoryStructure {
    pub(crate) symbolic_head: Option<Vec<u8>>,
    pub(crate) head_oid: Option<Vec<u8>>,
    pub(crate) index_raw: Vec<u8>,
    pub(crate) paths: Vec<PathBuf>,
}
#[derive(Clone, Debug)]
struct IgnoreMatch {
    source: Vec<u8>,
    pattern: Vec<u8>,
}
pub fn resolve_worktree_root(repository: &Path) -> Result<PathBuf, StateCaptureError> {
    resolve_worktree_root_with_environment(repository, &[])
}
pub(crate) fn resolve_worktree_root_with_environment(
    repository: &Path,
    ambient: &AmbientEnvironment,
) -> Result<PathBuf, StateCaptureError> {
    let requested = repository.canonicalize().map_err(|error| {
        StateCaptureError::new(format!("repository cannot be resolved: {error}"))
    })?;
    if !requested.is_dir() {
        return Err(StateCaptureError::new("repository is not a directory"));
    }
    let output = run_git(&requested, &["rev-parse", "--show-toplevel"], ambient)?;
    require_success(&output, "worktree root")?;
    let bytes = line_value(output.stdout, "worktree root")?;
    let reported = bytes_to_path(bytes)?.canonicalize().map_err(|error| {
        StateCaptureError::new(format!("reported root cannot be resolved: {error}"))
    })?;
    if reported != requested {
        return Err(StateCaptureError::new(
            "repository path is not the exact Git worktree root",
        ));
    }
    Ok(requested)
}
pub(crate) fn capture_structure(
    root: &Path,
    explicit_scope: &BTreeSet<PathBuf>,
    ambient: &AmbientEnvironment,
) -> Result<RepositoryStructure, StateCaptureError> {
    let symbolic = run_git(root, &["symbolic-ref", "-q", "HEAD"], ambient)?;
    let symbolic_head = optional_line(symbolic, "symbolic HEAD")?;
    let oid = run_git(root, &["rev-parse", "--verify", "--quiet", "HEAD"], ambient)?;
    let head_oid = optional_line(oid, "HEAD object")?;
    let index = run_git(root, &["ls-files", "--stage", "-z"], ambient)?;
    require_success(&index, "Git index")?;
    let mut paths = discover_governed_paths(root, ambient)?;
    paths.extend(explicit_scope.iter().cloned());
    Ok(RepositoryStructure {
        symbolic_head,
        head_oid,
        index_raw: index.stdout,
        paths: paths.into_iter().collect(),
    })
}
fn discover_governed_paths(
    root: &Path,
    ambient: &AmbientEnvironment,
) -> Result<BTreeSet<PathBuf>, StateCaptureError> {
    let tracked_output = run_git(root, &["ls-files", "-z", "--cached"], ambient)?;
    require_success(&tracked_output, "tracked path discovery")?;
    let tracked = parse_discovered_paths(&tracked_output.stdout, DiscoverySource::Tracked)?;
    reject_discovered_git_paths(&tracked)?;
    let tracked_sources: BTreeSet<Vec<u8>> = tracked
        .iter()
        .map(|path| path.identity_bytes.clone())
        .collect();
    let untracked_output = run_git(root, &["ls-files", "-z", "--others"], ambient)?;
    require_success(&untracked_output, "untracked path discovery")?;
    let untracked = parse_discovered_paths(&untracked_output.stdout, DiscoverySource::Untracked)?;
    reject_discovered_git_paths(&untracked)?;
    let mut query = Vec::new();
    for path in &untracked {
        query.extend_from_slice(&path.git_bytes);
        query.push(0);
    }
    let ignored = classify_ignored_paths(root, ambient, &query, &untracked)?;
    let mut governed = BTreeSet::new();
    for path in tracked {
        governed.insert(bytes_to_path(path.identity_bytes)?);
    }
    for path in untracked {
        match ignored.get(&path.git_bytes) {
            None => {
                governed.insert(bytes_to_path(path.identity_bytes)?);
            }
            Some(rule) if rule.source == b".git/info/exclude" => {
                governed.insert(bytes_to_path(path.identity_bytes)?);
            }
            Some(rule) if repository_gitignore_source(&rule.source)? => {
                if !tracked_sources.contains(&rule.source) {
                    return Err(StateCaptureError::new(
                        "untracked .gitignore affected governed path discovery",
                    ));
                }
                if rule.pattern.first() == Some(&b'!') {
                    governed.insert(bytes_to_path(path.identity_bytes)?);
                }
            }
            Some(_) => {
                governed.insert(bytes_to_path(path.identity_bytes)?);
            }
        }
    }
    Ok(governed)
}
fn reject_discovered_git_paths(paths: &[DiscoveredPath]) -> Result<(), StateCaptureError> {
    for discovered in paths {
        path::reject_git_implementation_path(&bytes_to_path(discovered.identity_bytes.clone())?)?;
    }
    Ok(())
}
fn classify_ignored_paths(
    root: &Path,
    ambient: &AmbientEnvironment,
    query: &[u8],
    untracked: &[DiscoveredPath],
) -> Result<BTreeMap<Vec<u8>, IgnoreMatch>, StateCaptureError> {
    if query.is_empty() {
        return Ok(BTreeMap::new());
    }
    let output = run_git_with_input(
        root,
        &["check-ignore", "--no-index", "--verbose", "-z", "--stdin"],
        ambient,
        query,
    )?;
    if !matches!(output.status.code(), Some(0 | 1)) {
        return Err(StateCaptureError::new(format!(
            "Git ignore classification failed with status {:?}: {:?}",
            output.status.code(),
            output.stderr
        )));
    }
    let fields = nul_fields(&output.stdout, "Git ignore classification")?;
    let (quadruples, remainder) = fields.as_chunks::<4>();
    if !remainder.is_empty() {
        return Err(StateCaptureError::new(
            "Git ignore classification did not return source/line/pattern/path quadruples",
        ));
    }
    let queried: BTreeSet<&[u8]> = untracked
        .iter()
        .map(|path| path.git_bytes.as_slice())
        .collect();
    let mut matches = BTreeMap::new();
    for [source, line, pattern, pathname] in quadruples {
        if !line.iter().all(u8::is_ascii_digit) || !queried.contains(pathname) {
            return Err(StateCaptureError::new(
                "Git ignore classification returned an invalid line or unknown path",
            ));
        }
        if matches
            .insert(
                pathname.to_vec(),
                IgnoreMatch {
                    source: source.to_vec(),
                    pattern: pattern.to_vec(),
                },
            )
            .is_some()
        {
            return Err(StateCaptureError::new(
                "Git ignore classification returned duplicate paths",
            ));
        }
    }
    Ok(matches)
}
fn repository_gitignore_source(source: &[u8]) -> Result<bool, StateCaptureError> {
    let path = bytes_to_path(source.to_vec())?;
    Ok(!path.is_absolute()
        && path.file_name().is_some_and(|name| {
            exact_os_bytes(name).is_ok_and(|bytes| bytes.as_slice() == b".gitignore")
        }))
}
fn run_git(
    root: &Path,
    arguments: &[&str],
    ambient: &AmbientEnvironment,
) -> Result<Output, StateCaptureError> {
    run_git_with_optional_input(root, arguments, ambient, None)
}
fn run_git_with_input(
    root: &Path,
    arguments: &[&str],
    ambient: &AmbientEnvironment,
    input: &[u8],
) -> Result<Output, StateCaptureError> {
    run_git_with_optional_input(root, arguments, ambient, Some(input))
}
fn run_git_with_optional_input(
    root: &Path,
    arguments: &[&str],
    ambient: &AmbientEnvironment,
    input: Option<&[u8]>,
) -> Result<Output, StateCaptureError> {
    let mut environment: BTreeMap<OsString, OsString> = env::vars_os().collect();
    environment.extend(ambient.iter().cloned());
    environment.retain(|name, _| !removed_git_name(name));
    environment.insert(OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1"));
    environment.insert(
        OsString::from("GIT_CONFIG_GLOBAL"),
        OsString::from(platform_null_file()),
    );
    let mut command = Command::new("git");
    command
        .env_clear()
        .envs(environment)
        .arg("-C")
        .arg(root)
        .args(arguments)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| StateCaptureError::new(format!("Git execution failed: {error}")))?;
    if let Some(input) = input {
        child
            .stdin
            .take()
            .ok_or_else(|| StateCaptureError::new("Git stdin was unavailable"))?
            .write_all(input)
            .map_err(|error| StateCaptureError::new(format!("Git stdin failed: {error}")))?;
    }
    child
        .wait_with_output()
        .map_err(|error| StateCaptureError::new(format!("Git execution failed: {error}")))
}
fn require_success(output: &Output, label: &str) -> Result<(), StateCaptureError> {
    if output.status.success() {
        Ok(())
    } else {
        Err(StateCaptureError::new(format!(
            "{label} observation failed with status {:?}: {:?}",
            output.status.code(),
            output.stderr
        )))
    }
}
fn optional_line(output: Output, label: &str) -> Result<Option<Vec<u8>>, StateCaptureError> {
    match output.status.code() {
        Some(0) => line_value(output.stdout, label).map(Some),
        Some(1) => Ok(None),
        status => Err(StateCaptureError::new(format!(
            "{label} observation failed with status {status:?}: {:?}",
            output.stderr
        ))),
    }
}
#[cfg(unix)]
fn line_value(mut bytes: Vec<u8>, label: &str) -> Result<Vec<u8>, StateCaptureError> {
    if bytes.last() != Some(&b'\n') {
        return Err(StateCaptureError::new(format!(
            "{label} did not end with the Git LF terminator"
        )));
    }
    bytes.pop();
    validate_line_value(bytes, label)
}
#[cfg(windows)]
fn line_value(mut bytes: Vec<u8>, label: &str) -> Result<Vec<u8>, StateCaptureError> {
    if bytes.ends_with(b"\r\n") {
        bytes.truncate(bytes.len() - 2);
    } else if bytes.last() == Some(&b'\n') {
        bytes.pop();
    } else {
        return Err(StateCaptureError::new(format!(
            "{label} did not end with a Git line terminator"
        )));
    }
    validate_line_value(bytes, label)
}
fn validate_line_value(bytes: Vec<u8>, label: &str) -> Result<Vec<u8>, StateCaptureError> {
    if bytes.is_empty() || bytes.contains(&0) {
        return Err(StateCaptureError::new(format!(
            "{label} returned an invalid empty or NUL-containing value"
        )));
    }
    Ok(bytes)
}
fn nul_fields<'a>(bytes: &'a [u8], label: &str) -> Result<Vec<&'a [u8]>, StateCaptureError> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if bytes.last() != Some(&0) {
        return Err(StateCaptureError::new(format!(
            "{label} was not NUL terminated"
        )));
    }
    let fields: Vec<_> = bytes[..bytes.len() - 1].split(|byte| *byte == 0).collect();
    if fields.iter().any(|field| field.is_empty()) {
        return Err(StateCaptureError::new(format!(
            "{label} returned an empty interior field"
        )));
    }
    Ok(fields)
}

#[cfg(unix)]
fn bytes_to_path(bytes: Vec<u8>) -> Result<PathBuf, StateCaptureError> {
    use std::os::unix::ffi::OsStringExt;
    Ok(PathBuf::from(OsString::from_vec(bytes)))
}

#[cfg(windows)]
fn bytes_to_path(bytes: Vec<u8>) -> Result<PathBuf, StateCaptureError> {
    String::from_utf8(bytes)
        .map(PathBuf::from)
        .map_err(|_| StateCaptureError::new("Git path output is not exact UTF-8"))
}

#[cfg(unix)]
fn exact_os_bytes(value: &OsStr) -> Result<Vec<u8>, StateCaptureError> {
    use std::os::unix::ffi::OsStrExt;
    Ok(value.as_bytes().to_vec())
}

#[cfg(windows)]
fn exact_os_bytes(value: &OsStr) -> Result<Vec<u8>, StateCaptureError> {
    value
        .to_str()
        .map(|value| value.as_bytes().to_vec())
        .ok_or_else(|| StateCaptureError::new("Git ignore source is not exact UTF-8"))
}

#[cfg(unix)]
fn platform_null_file() -> &'static str {
    "/dev/null"
}

#[cfg(windows)]
fn platform_null_file() -> &'static str {
    "NUL"
}
