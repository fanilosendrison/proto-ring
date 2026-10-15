//! Read-only Git whitespace validation over explicit repository and event inputs.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

const EVENT_NAME: &str = "GITHUB_EVENT_NAME";
const EVENT_PATH: &str = "GITHUB_EVENT_PATH";
const ROOT_PUSH_BEFORE: &str = "0000000000000000000000000000000000000000";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhitespaceDomain {
    Unstaged,
    Staged,
    PushCommittedRange,
    PullRequestCommittedRange,
    RangeDetermination,
    GitObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticKind {
    GitReportedViolationOrFailure,
    GitProcessStartFailure,
    RangeDeterminationFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhitespaceDiagnostic {
    pub domain: WhitespaceDomain,
    pub kind: DiagnosticKind,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WhitespaceReport {
    pub diagnostics: Vec<WhitespaceDiagnostic>,
}

impl WhitespaceReport {
    pub fn passes(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub fn check(repository: &Path, environment: &BTreeMap<String, String>) -> WhitespaceReport {
    check_with_runner(repository, environment, &mut RealGitRunner)
}

fn check_with_runner(
    repository: &Path,
    environment: &BTreeMap<String, String>,
    runner: &mut impl GitRunner,
) -> WhitespaceReport {
    let mut diagnostics = Vec::new();
    observe_diff(
        runner,
        repository,
        &["diff", "--check"],
        WhitespaceDomain::Unstaged,
        &mut diagnostics,
    );
    observe_diff(
        runner,
        repository,
        &["diff", "--cached", "--check"],
        WhitespaceDomain::Staged,
        &mut diagnostics,
    );

    match committed_range(environment) {
        Ok(None) => {}
        Ok(Some(CommittedRange::Push { before, after })) => {
            let before = if before == ROOT_PUSH_BEFORE {
                match empty_tree(runner, repository) {
                    Ok(object_id) => object_id,
                    Err(diagnostic) => {
                        diagnostics.push(diagnostic);
                        return WhitespaceReport { diagnostics };
                    }
                }
            } else {
                before
            };
            let range = format!("{before}..{after}");
            observe_diff(
                runner,
                repository,
                &["diff", "--check", &range],
                WhitespaceDomain::PushCommittedRange,
                &mut diagnostics,
            );
        }
        Ok(Some(CommittedRange::PullRequest { base, head })) => {
            let range = format!("{base}...{head}");
            observe_diff(
                runner,
                repository,
                &["diff", "--check", &range],
                WhitespaceDomain::PullRequestCommittedRange,
                &mut diagnostics,
            );
        }
        Err(()) => diagnostics.push(WhitespaceDiagnostic {
            domain: WhitespaceDomain::RangeDetermination,
            kind: DiagnosticKind::RangeDeterminationFailure,
        }),
    }

    WhitespaceReport { diagnostics }
}

fn observe_diff(
    runner: &mut impl GitRunner,
    repository: &Path,
    arguments: &[&str],
    domain: WhitespaceDomain,
    diagnostics: &mut Vec<WhitespaceDiagnostic>,
) {
    match runner.run(repository, arguments, None) {
        Ok(output) if output.success => {}
        Ok(_) | Err(GitExecutionFailure::Observation) => {
            diagnostics.push(WhitespaceDiagnostic {
                domain,
                kind: DiagnosticKind::GitReportedViolationOrFailure,
            });
        }
        Err(GitExecutionFailure::ProcessStart) => {
            diagnostics.push(WhitespaceDiagnostic {
                domain: WhitespaceDomain::GitObservation,
                kind: DiagnosticKind::GitProcessStartFailure,
            });
        }
    }
}

fn empty_tree(
    runner: &mut impl GitRunner,
    repository: &Path,
) -> Result<String, WhitespaceDiagnostic> {
    let failure = || WhitespaceDiagnostic {
        domain: WhitespaceDomain::RangeDetermination,
        kind: DiagnosticKind::RangeDeterminationFailure,
    };
    let output = runner
        .run(
            repository,
            &["hash-object", "-t", "tree", "--stdin"],
            Some(&[]),
        )
        .map_err(|error| match error {
            GitExecutionFailure::ProcessStart => WhitespaceDiagnostic {
                domain: WhitespaceDomain::GitObservation,
                kind: DiagnosticKind::GitProcessStartFailure,
            },
            GitExecutionFailure::Observation => failure(),
        })?;
    if !output.success {
        return Err(failure());
    }
    let object_id = std::str::from_utf8(&output.stdout)
        .map_err(|_| failure())?
        .trim();
    if object_id.is_empty() {
        return Err(failure());
    }
    Ok(object_id.to_owned())
}

enum CommittedRange {
    Push { before: String, after: String },
    PullRequest { base: String, head: String },
}

fn committed_range(environment: &BTreeMap<String, String>) -> Result<Option<CommittedRange>, ()> {
    match environment.get(EVENT_NAME).map(String::as_str) {
        Some("push") => {
            let payload = event_payload(environment)?;
            Ok(Some(CommittedRange::Push {
                before: required_string(&payload, &["before"])?.to_owned(),
                after: required_string(&payload, &["after"])?.to_owned(),
            }))
        }
        Some("pull_request") => {
            let payload = event_payload(environment)?;
            Ok(Some(CommittedRange::PullRequest {
                base: required_string(&payload, &["pull_request", "base", "sha"])?.to_owned(),
                head: required_string(&payload, &["pull_request", "head", "sha"])?.to_owned(),
            }))
        }
        _ => Ok(None),
    }
}

fn event_payload(environment: &BTreeMap<String, String>) -> Result<serde_json::Value, ()> {
    let path = environment.get(EVENT_PATH).ok_or(())?;
    let bytes = fs::read(Path::new(path)).map_err(|_| ())?;
    let text = std::str::from_utf8(&bytes).map_err(|_| ())?;
    let value: serde_json::Value = serde_json::from_str(text).map_err(|_| ())?;
    value.as_object().ok_or(())?;
    Ok(value)
}

fn required_string<'a>(value: &'a serde_json::Value, path: &[&str]) -> Result<&'a str, ()> {
    let mut current = value;
    for name in path {
        current = current
            .as_object()
            .and_then(|object| object.get(*name))
            .ok_or(())?;
    }
    current.as_str().filter(|text| !text.is_empty()).ok_or(())
}

trait GitRunner {
    fn run(
        &mut self,
        repository: &Path,
        arguments: &[&str],
        stdin: Option<&[u8]>,
    ) -> Result<GitOutput, GitExecutionFailure>;
}

struct GitOutput {
    success: bool,
    stdout: Vec<u8>,
}

enum GitExecutionFailure {
    ProcessStart,
    Observation,
}

struct RealGitRunner;

impl GitRunner for RealGitRunner {
    fn run(
        &mut self,
        repository: &Path,
        arguments: &[&str],
        stdin: Option<&[u8]>,
    ) -> Result<GitOutput, GitExecutionFailure> {
        execute_git(repository, arguments, stdin)
    }
}

fn execute_git(
    repository: &Path,
    arguments: &[&str],
    stdin: Option<&[u8]>,
) -> Result<GitOutput, GitExecutionFailure> {
    let mut command = Command::new("git");
    command
        .current_dir(repository)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
    let mut child = command
        .spawn()
        .map_err(|_| GitExecutionFailure::ProcessStart)?;
    if let Some(input) = stdin {
        child
            .stdin
            .take()
            .ok_or(GitExecutionFailure::Observation)?
            .write_all(input)
            .map_err(|_| GitExecutionFailure::Observation)?;
    }
    let output = child
        .wait_with_output()
        .map_err(|_| GitExecutionFailure::Observation)?;
    Ok(GitOutput {
        success: output.status.success(),
        stdout: output.stdout,
    })
}

#[cfg(feature = "conformance-internal")]
pub mod conformance_internal {
    use super::*;

    pub fn check_with_unavailable_git(
        repository: &Path,
        environment: &BTreeMap<String, String>,
    ) -> WhitespaceReport {
        check_with_runner(repository, environment, &mut UnavailableGitRunner)
    }

    struct UnavailableGitRunner;

    impl GitRunner for UnavailableGitRunner {
        fn run(
            &mut self,
            _repository: &Path,
            _arguments: &[&str],
            _stdin: Option<&[u8]>,
        ) -> Result<GitOutput, GitExecutionFailure> {
            Err(GitExecutionFailure::ProcessStart)
        }
    }
}
