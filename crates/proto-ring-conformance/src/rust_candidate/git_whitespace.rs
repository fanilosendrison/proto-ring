#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use proto_ring_engine::git_whitespace::{
    self, DiagnosticKind, WhitespaceDiagnostic, WhitespaceDomain, WhitespaceReport,
    conformance_internal,
};
use serde_json::Value;

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, TransportValue};

use super::transport::{
    exact_record, fixture_failure, record, result, string_value, unique_dynamic_record,
};

pub(super) fn execute(value: &Value) -> Result<Observation, fixture::FixtureError> {
    let object = value
        .as_object()
        .ok_or_else(|| fixture_failure("Git Whitespace fixture must be an object"))?;
    match object.get("kind").and_then(Value::as_str) {
        Some("repository_plan") => {
            let materialized = fixture::materialize(value)?;
            execute_materialized(&materialized, false)
        }
        Some("procedural_requirement") => {
            let condition = object
                .get("condition")
                .and_then(Value::as_object)
                .ok_or_else(|| fixture_failure("procedural condition must be an object"))?;
            if condition.get("kind").and_then(Value::as_str)
                != Some("git_process_start_unavailable")
            {
                return Err(fixture_failure(
                    "unsupported Git Whitespace procedural condition",
                ));
            }
            let base = object
                .get("base")
                .ok_or_else(|| fixture_failure("procedural base is required"))?;
            let materialized = fixture::materialize(base)?;
            execute_materialized(&materialized, true)
        }
        _ => Err(fixture_failure("unsupported Git Whitespace fixture kind")),
    }
}

fn execute_materialized(
    fixture: &MaterializedFixture,
    git_unavailable: bool,
) -> Result<Observation, fixture::FixtureError> {
    let repository = fixture
        .root()
        .ok_or_else(|| fixture_failure("Git Whitespace check requires repository plan"))?;
    let fields = exact_record(
        fixture.arguments(),
        "Git Whitespace invocation",
        &["environment", "operation"],
    )?;
    if fixture::string(fields["operation"])? != "check_whitespace" {
        return Err(fixture_failure("unsupported Git Whitespace operation"));
    }
    let mut environment = decode_environment(fields["environment"])?;
    resolve_fixture_event_path(repository, &mut environment)?;
    let report = if git_unavailable {
        conformance_internal::check_with_unavailable_git(repository, &environment)
    } else {
        git_whitespace::check(repository, &environment)
    };
    Ok(project(&report))
}

fn decode_environment(
    value: &TransportValue,
) -> Result<BTreeMap<String, String>, fixture::FixtureError> {
    unique_dynamic_record(value, "Git Whitespace environment")?
        .into_iter()
        .map(|(name, value)| Ok((name.to_owned(), fixture::string(value)?.to_owned())))
        .collect()
}

fn resolve_fixture_event_path(
    repository: &Path,
    environment: &mut BTreeMap<String, String>,
) -> Result<(), fixture::FixtureError> {
    let Some(value) = environment.get_mut("GITHUB_EVENT_PATH") else {
        return Ok(());
    };
    let supplied = PathBuf::from(&*value);
    if supplied.is_relative() {
        *value = repository
            .join(supplied)
            .into_os_string()
            .into_string()
            .map_err(|_| fixture_failure("fixture event path is not UTF-8"))?;
    }
    Ok(())
}

fn project(report: &WhitespaceReport) -> Observation {
    let mut domains = Vec::new();
    for diagnostic in &report.diagnostics {
        let domain = projected_domain(diagnostic);
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    result(record(vec![
        (
            "domains",
            TransportValue::Sequence(domains.into_iter().map(string_value).collect()),
        ),
        (
            "status",
            string_value(if report.passes() { "PASS" } else { "NON_PASS" }),
        ),
    ]))
}

fn projected_domain(diagnostic: &WhitespaceDiagnostic) -> &'static str {
    if diagnostic.kind == DiagnosticKind::GitProcessStartFailure {
        return "git_observation";
    }
    match diagnostic.domain {
        WhitespaceDomain::Unstaged => "unstaged",
        WhitespaceDomain::Staged => "staged",
        WhitespaceDomain::PushCommittedRange => "push_committed_range",
        WhitespaceDomain::PullRequestCommittedRange => "pull_request_committed_range",
        WhitespaceDomain::RangeDetermination => "range_determination",
        WhitespaceDomain::GitObservation => "git_observation",
    }
}
