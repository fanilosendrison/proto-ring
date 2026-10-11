use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use proto_ring_engine::repository_state::{
    self, RepositoryState, StateCaptureError, conformance_internal,
};

use super::super::transport::{
    fixture_failure, record, rejection, result, string_value, unique_dynamic_record,
};
use crate::fixture;
use crate::model::{Observation, TransportValue};

pub(crate) struct InvocationResult {
    pub(crate) observation: Observation,
    pub(crate) state: Option<RepositoryState>,
}

pub(crate) fn execute(
    root: &Path,
    arguments: &TransportValue,
) -> Result<InvocationResult, fixture::FixtureError> {
    let fields = fields(arguments)?;
    let operation = required_string(&fields, "operation")?;
    match operation {
        "capture_state" => capture_state(root, &fields),
        "capture_failure_category" => capture_failure_category(root, &fields),
        "capture_with_ambient" => capture_with_ambient(root, &fields),
        "capture_read_only" => capture_read_only(root, &fields),
        _ => Err(fixture_failure("unsupported RepositoryState operation")),
    }
}

pub(crate) fn execute_procedural<F>(
    root: &Path,
    arguments: &TransportValue,
    hook: F,
) -> Result<Observation, fixture::FixtureError>
where
    F: FnOnce(&Path) -> Result<(), StateCaptureError>,
{
    let fields = fields(arguments)?;
    require_allowed(&fields, &["operation", "scope_paths"])?;
    if required_string(&fields, "operation")? != "capture_state" {
        return Err(fixture_failure(
            "procedural requirement requires capture_state",
        ));
    }
    let scope = scope_paths(&fields)?;
    let relation =
        if conformance_internal::capture_with_validation_hook(root, &scope, hook).is_err() {
            "CONTROLLED_FAILURE"
        } else {
            "NO_FAILURE"
        };
    Ok(result(record(vec![("relation", string_value(relation))])))
}

fn capture_state(
    root: &Path,
    fields: &Fields<'_>,
) -> Result<InvocationResult, fixture::FixtureError> {
    require_allowed(fields, &["operation", "repository_subpath", "scope_paths"])?;
    let scope = scope_paths(fields)?;
    let repository = optional_string(fields, "repository_subpath")?
        .map(|relative| root.join(relative))
        .unwrap_or_else(|| root.to_path_buf());
    Ok(match repository_state::capture(&repository, &scope) {
        Ok(state) => InvocationResult {
            observation: successful_capture(&state)?,
            state: Some(state),
        },
        Err(_) => InvocationResult {
            observation: rejection("repository-state.capture"),
            state: None,
        },
    })
}

fn capture_failure_category(
    root: &Path,
    fields: &Fields<'_>,
) -> Result<InvocationResult, fixture::FixtureError> {
    require_allowed(fields, &["operation", "scope_paths"])?;
    let scope = scope_paths(fields)?;
    let relation = if repository_state::capture(root, &scope).is_err() {
        "CONTROLLED_FAILURE"
    } else {
        "NO_FAILURE"
    };
    Ok(InvocationResult {
        observation: result(record(vec![("relation", string_value(relation))])),
        state: None,
    })
}

fn capture_with_ambient(
    root: &Path,
    fields: &Fields<'_>,
) -> Result<InvocationResult, fixture::FixtureError> {
    require_allowed(fields, &["ambient", "operation", "scope_paths"])?;
    let scope = scope_paths(fields)?;
    let ambient = record_field(fields, "ambient")?
        .iter()
        .map(|entry| {
            fixture::string(&entry.value)
                .map(|value| (OsString::from(entry.name.as_str()), OsString::from(value)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let control = repository_state::capture(root, &scope)
        .map_err(|error| fixture_failure(format!("control capture failed: {error}")))?;
    let redirected = conformance_internal::capture_with_ambient_environment(root, &scope, &ambient)
        .map_err(|error| fixture_failure(format!("ambient capture failed: {error}")))?;
    let relation = if control.identity == redirected.identity {
        "EQUAL"
    } else {
        "NOT_EQUAL"
    };
    Ok(InvocationResult {
        observation: result(record(vec![("relation", string_value(relation))])),
        state: None,
    })
}

fn capture_read_only(
    root: &Path,
    fields: &Fields<'_>,
) -> Result<InvocationResult, fixture::FixtureError> {
    require_allowed(fields, &["operation", "participating_paths", "scope_paths"])?;
    let participating = sequence_strings(fields, "participating_paths")?;
    let before = ReadOnlyProbe::capture(root, &participating)?;
    let scope = scope_paths(fields)?;
    repository_state::capture(root, &scope)
        .map_err(|error| fixture_failure(format!("read-only capture failed: {error}")))?;
    let after = ReadOnlyProbe::capture(root, &participating)?;
    let relations = [
        ("bytes", before.bytes == after.bytes),
        ("config", before.config == after.config),
        ("head", before.head == after.head),
        ("index", before.index == after.index),
        ("status", before.status == after.status),
    ];
    Ok(InvocationResult {
        observation: result(record(vec![
            (
                "all_equal",
                TransportValue::Boolean(relations.iter().all(|(_, equal)| *equal)),
            ),
            (
                "relations",
                record(
                    relations
                        .into_iter()
                        .map(|(name, equal)| {
                            (
                                name,
                                string_value(if equal { "EQUAL" } else { "NOT_EQUAL" }),
                            )
                        })
                        .collect(),
                ),
            ),
        ])),
        state: None,
    })
}

fn successful_capture(state: &RepositoryState) -> Result<Observation, fixture::FixtureError> {
    let scope = state
        .scope_paths
        .iter()
        .map(|path| {
            path.to_str().map(string_value).ok_or_else(|| {
                fixture_failure("scope path is not representable by UTF-8 transport")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(result(record(vec![
        (
            "identity_present",
            TransportValue::Boolean(!state.identity.0.is_empty()),
        ),
        ("repository_is_root", TransportValue::Boolean(true)),
        ("scope_paths", TransportValue::Sequence(scope)),
    ])))
}

type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;

fn fields(value: &TransportValue) -> Result<Fields<'_>, fixture::FixtureError> {
    Ok(unique_dynamic_record(value, "RepositoryState invocation")?
        .into_iter()
        .collect())
}

fn require_allowed(fields: &Fields<'_>, allowed: &[&str]) -> Result<(), fixture::FixtureError> {
    if let Some(name) = fields.keys().find(|name| !allowed.contains(name)) {
        Err(fixture_failure(format!(
            "RepositoryState invocation has unknown field: {name}"
        )))
    } else {
        Ok(())
    }
}

fn required_string<'a>(fields: &Fields<'a>, name: &str) -> Result<&'a str, fixture::FixtureError> {
    fixture::string(
        fields
            .get(name)
            .copied()
            .ok_or_else(|| fixture_failure(format!("missing invocation field: {name}")))?,
    )
}

fn optional_string<'a>(
    fields: &Fields<'a>,
    name: &str,
) -> Result<Option<&'a str>, fixture::FixtureError> {
    fields
        .get(name)
        .map(|value| fixture::string(value))
        .transpose()
}

fn record_field<'a>(
    fields: &Fields<'a>,
    name: &str,
) -> Result<&'a [crate::model::RecordEntry], fixture::FixtureError> {
    fixture::record(
        fields
            .get(name)
            .copied()
            .ok_or_else(|| fixture_failure(format!("missing invocation field: {name}")))?,
    )
}

fn sequence_strings(
    fields: &Fields<'_>,
    name: &str,
) -> Result<Vec<PathBuf>, fixture::FixtureError> {
    fixture::sequence(
        fields
            .get(name)
            .copied()
            .ok_or_else(|| fixture_failure(format!("missing invocation field: {name}")))?,
    )?
    .iter()
    .map(|value| fixture::string(value).map(PathBuf::from))
    .collect()
}

fn scope_paths(fields: &Fields<'_>) -> Result<Vec<PathBuf>, fixture::FixtureError> {
    fields
        .contains_key("scope_paths")
        .then(|| sequence_strings(fields, "scope_paths"))
        .transpose()
        .map(Option::unwrap_or_default)
}

#[derive(Eq, PartialEq)]
struct ReadOnlyProbe {
    bytes: Vec<(PathBuf, Vec<u8>)>,
    config: Vec<u8>,
    head: Vec<u8>,
    index: Vec<u8>,
    status: Vec<u8>,
}

impl ReadOnlyProbe {
    fn capture(root: &Path, paths: &[PathBuf]) -> Result<Self, fixture::FixtureError> {
        let bytes = paths
            .iter()
            .map(|relative| {
                fs::read(root.join(relative))
                    .map(|bytes| (relative.clone(), bytes))
                    .map_err(|error| {
                        fixture_failure(format!("read-only byte probe failed: {error}"))
                    })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            bytes,
            config: test_git(root, &["config", "--local", "--list", "-z"])?,
            head: test_git(root, &["rev-parse", "HEAD"])?,
            index: test_git(root, &["ls-files", "--stage", "-z"])?,
            status: test_git(root, &["status", "--porcelain=v1", "-z"])?,
        })
    }
}

fn test_git(root: &Path, arguments: &[&str]) -> Result<Vec<u8>, fixture::FixtureError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .map_err(|error| fixture_failure(format!("read-only Git probe failed: {error}")))?;
    if !output.status.success() {
        return Err(fixture_failure(format!(
            "read-only Git probe exited {:?}",
            output.status.code()
        )));
    }
    Ok(output.stdout)
}
