use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use proto_ring_engine::governance_bootstrap::{self, GovernanceBootstrapErrorKind};
use proto_ring_engine::repository_governance_state::conformance_internal;
use proto_ring_engine::repository_state::StateCaptureError;
use serde_json::{Map, Value};

use super::super::transport::{fixture_failure, record, result, string_value};
use super::observe::{self, InvocationResult};
use crate::fixture;
use crate::model::Observation;

pub(crate) fn execute(fixture_value: &Value) -> Result<Observation, fixture::FixtureError> {
    let object = fixture_value
        .as_object()
        .ok_or_else(|| fixture_failure("RepositoryGovernanceState fixture must be an object"))?;
    match string_field(object, "kind")? {
        "repository_plan" => execute_repository_plan(object),
        "procedural_requirement" => execute_procedural(object),
        _ => Err(fixture_failure(
            "unsupported RepositoryGovernanceState fixture kind",
        )),
    }
}

fn execute_repository_plan(
    plan: &Map<String, Value>,
) -> Result<Observation, fixture::FixtureError> {
    let temporary = tempfile::tempdir()
        .map_err(|error| fixture_failure(format!("cannot create fixture root: {error}")))?;
    let root = temporary.path().join("repository");
    fs::create_dir(&root)
        .map_err(|error| fixture_failure(format!("cannot create repository root: {error}")))?;
    let mut captures = HashMap::new();
    let mut invocations = Vec::new();
    for raw_step in array_field(plan, "steps")? {
        let step = fixture::interpolate(raw_step, &captures)?;
        if step.get("op").and_then(Value::as_str) == Some("invoke") {
            invocations.push(execute_invoke(&root, &step)?);
        } else {
            fixture::apply_step(&root, &step, &mut captures)?;
        }
    }
    emit_bootstrap_symlink_diagnostics(temporary.path(), &root, plan);
    if !invocations.is_empty() {
        return identity_relation(invocations);
    }
    let invocation = fixture::interpolate(required(plan, "invocation")?, &captures)?;
    let arguments = fixture::decode_transport(&invocation)?;
    observe::execute(&root, &arguments).map(|result| result.observation)
}

fn emit_bootstrap_symlink_diagnostics(temporary: &Path, root: &Path, plan: &Map<String, Value>) {
    let Some(steps) = plan.get("steps").and_then(Value::as_array) else {
        return;
    };
    if !steps.iter().any(|step| {
        step.get("op").and_then(Value::as_str) == Some("symlink")
            && step.get("path").and_then(Value::as_str) == Some("AGENTS.md")
            && step.get("target").and_then(Value::as_str) == Some("../external-bootstrap/AGENTS.md")
    }) {
        return;
    }
    let authored = steps
        .iter()
        .find(|step| {
            step.get("op").and_then(Value::as_str) == Some("write_utf8")
                && step.get("path").and_then(Value::as_str) == Some("AGENTS.md")
        })
        .and_then(|step| step.get("text").and_then(Value::as_str))
        .map(|text| text.as_bytes().to_vec())
        .ok_or_else(|| "authored AGENTS.md write_utf8 step is unavailable".to_owned());
    let external_path = temporary.join("external-bootstrap/AGENTS.md");
    let symlink_path = root.join("AGENTS.md");
    let mut lines = vec![
        "ISSUE68_ACTUAL_FIXTURE_DIAGNOSTIC_BEGIN".to_owned(),
        format!("TEMP_ROOT={temporary:?}"),
    ];
    let authored = observe_bytes(&mut lines, "AUTHORED", authored);
    let blob = observe_bytes(&mut lines, "GIT_BLOB", read_git_blob(root));
    let external = observe_bytes(
        &mut lines,
        "EXTERNAL_CLONE",
        fs::read(&external_path).map_err(|error| io_detail(&error)),
    );
    for (name, left, right) in [
        ("AUTHORED_EQ_GIT_BLOB", authored.as_deref(), blob.as_deref()),
        (
            "GIT_BLOB_EQ_EXTERNAL_CLONE",
            blob.as_deref(),
            external.as_deref(),
        ),
    ] {
        observe_relation(&mut lines, name, left, right);
    }
    match fs::symlink_metadata(&symlink_path) {
        Ok(metadata) => {
            let kind = metadata.file_type();
            lines.push(format!(
                "SYMLINK_METADATA=SUCCESS:is_symlink={};is_file={};is_dir={}",
                kind.is_symlink(),
                kind.is_file(),
                kind.is_dir()
            ));
        }
        Err(error) => lines.push(format!("SYMLINK_METADATA=FAILURE:{}", io_detail(&error))),
    }
    match fs::read_link(&symlink_path) {
        Ok(target) => lines.push(format!("SYMLINK_TARGET={target:?}")),
        Err(error) => lines.push(format!("SYMLINK_TARGET=ERROR:{}", io_detail(&error))),
    }
    match fs::canonicalize(&symlink_path) {
        Ok(target) => lines.push(format!("SYMLINK_RESOLVED_TARGET={target:?}")),
        Err(error) => lines.push(format!(
            "SYMLINK_RESOLVED_TARGET=ERROR:{}",
            io_detail(&error)
        )),
    }
    let symlink = observe_bytes(
        &mut lines,
        "SYMLINK_READ",
        fs::read(&symlink_path).map_err(|error| io_detail(&error)),
    );
    for (name, right) in [
        ("SYMLINK_READ_EQ_EXTERNAL_CLONE", external.as_deref()),
        ("SYMLINK_READ_EQ_GIT_BLOB", blob.as_deref()),
    ] {
        observe_relation(&mut lines, name, symlink.as_deref(), right);
    }
    match governance_bootstrap::load(root) {
        Ok(_) => lines.extend([
            "BOOTSTRAP_LOAD=SUCCESS".to_owned(),
            "BOOTSTRAP_ERROR_KIND=NONE".to_owned(),
            "BOOTSTRAP_ERROR_DETAIL=NONE".to_owned(),
        ]),
        Err(error) => {
            let kind = match error.kind {
                GovernanceBootstrapErrorKind::CarrierIo => "CarrierIo",
                GovernanceBootstrapErrorKind::StructuredData => "StructuredData",
                GovernanceBootstrapErrorKind::Structure => "Structure",
            };
            lines.extend([
                "BOOTSTRAP_LOAD=FAILURE".to_owned(),
                format!("BOOTSTRAP_ERROR_KIND={kind}"),
                format!("BOOTSTRAP_ERROR_DETAIL={error}"),
            ]);
        }
    }
    lines.push("ISSUE68_ACTUAL_FIXTURE_DIAGNOSTIC_END".to_owned());
    let report = format!("{}\n", lines.join("\n"));
    let _ = std::io::stderr().lock().write_all(report.as_bytes());
}

#[rustfmt::skip]
fn observe_bytes(lines: &mut Vec<String>, name: &str, observed: Result<Vec<u8>, String>) -> Option<Vec<u8>> {
    let read_key = if name.ends_with("_READ") { name.to_owned() } else { format!("{name}_READ") }; match observed {
        Ok(bytes) => {
            let cr = bytes.iter().filter(|value| **value == b'\r').count();
            let lf = bytes.iter().filter(|value| **value == b'\n').count();
            lines.extend([
                format!("{read_key}_STATUS=SUCCESS"), format!("{name}_BYTE_LENGTH={}", bytes.len()),
                format!("{name}_CR_COUNT={cr}"), format!("{name}_LF_COUNT={lf}"),
            ]);
            Some(bytes)
        }
        Err(error) => {
            lines.extend([format!("{read_key}_STATUS=FAILURE"), format!("{read_key}_ERROR={error}")]);
            None
        }
    }
}

#[rustfmt::skip]
fn observe_relation(lines: &mut Vec<String>, name: &str, left: Option<&[u8]>, right: Option<&[u8]>) {
    let (Some(left), Some(right)) = (left, right) else {
        lines.push(format!("{name}=UNAVAILABLE"));
        return;
    };
    lines.push(format!("{name}={}", if left == right { "YES" } else { "NO" }));
    let difference = left.iter().zip(right).position(|(left, right)| left != right)
        .or((left.len() != right.len()).then_some(left.len().min(right.len())));
    lines.push(format!("{name}_FIRST_DIFFERING_OFFSET={}",
        difference.map_or_else(|| "NONE".to_owned(), |index| index.to_string())));
}

#[rustfmt::skip]
fn read_git_blob(root: &Path) -> Result<Vec<u8>, String> {
    let output = Command::new("git").args(["show", "HEAD:AGENTS.md"]).current_dir(root)
        .env("GIT_OPTIONAL_LOCKS", "0").output()
        .map_err(|error| format!("git show execution failed: {error}"))?;
    if output.status.success() { Ok(output.stdout) } else {
        Err(format!("git show exited {:?}; stderr={:?}", output.status.code(),
            String::from_utf8_lossy(&output.stderr)))
    }
}

#[rustfmt::skip]
fn io_detail(error: &std::io::Error) -> String {
    format!("kind={:?};raw_os_error={:?};detail={error}", error.kind(), error.raw_os_error())
}

fn execute_procedural(
    fixture_value: &Map<String, Value>,
) -> Result<Observation, fixture::FixtureError> {
    let base = required(fixture_value, "base")?
        .as_object()
        .ok_or_else(|| fixture_failure("procedural base must be an object"))?;
    if string_field(base, "kind")? != "repository_plan" {
        return Err(fixture_failure(
            "RepositoryGovernanceState procedural base must be repository_plan",
        ));
    }
    let condition = required(fixture_value, "condition")?
        .as_object()
        .ok_or_else(|| fixture_failure("procedural condition must be an object"))?;
    let (temporary, root, captures) = materialize_base(base)?;
    let invocation = fixture::interpolate(required(fixture_value, "invocation")?, &captures)?;
    let arguments = fixture::decode_transport(&invocation)?;
    let observation = match string_field(condition, "kind")? {
        "provider_network_access_forbidden" => observe::execute(&root, &arguments)?.observation,
        "repository_mutation_during_governance_state_construction" => {
            execute_repository_drift(&root, condition)?
        }
        "observation_scope_mutation_during_governance_state_construction" => {
            execute_scope_drift(&root, condition)?
        }
        _ => {
            return Err(fixture_failure(
                "unsupported RepositoryGovernanceState procedural condition",
            ));
        }
    };
    drop(temporary);
    Ok(observation)
}

fn materialize_base(
    base: &Map<String, Value>,
) -> Result<(tempfile::TempDir, PathBuf, HashMap<String, String>), fixture::FixtureError> {
    let temporary = tempfile::tempdir()
        .map_err(|error| fixture_failure(format!("cannot create fixture root: {error}")))?;
    let root = temporary.path().join("repository");
    fs::create_dir(&root)
        .map_err(|error| fixture_failure(format!("cannot create repository root: {error}")))?;
    let mut captures = HashMap::new();
    for raw_step in array_field(base, "steps")? {
        let step = fixture::interpolate(raw_step, &captures)?;
        if step.get("op").and_then(Value::as_str) == Some("invoke") {
            return Err(fixture_failure(
                "procedural base cannot invoke before construction checkpoint",
            ));
        }
        fixture::apply_step(&root, &step, &mut captures)?;
    }
    Ok((temporary, root, captures))
}

fn execute_repository_drift(
    root: &Path,
    condition: &Map<String, Value>,
) -> Result<Observation, fixture::FixtureError> {
    let mutation = required(condition, "mutation")?.clone();
    let outcome = conformance_internal::load_with_after_initial_capture_hook(root, |root| {
        apply_mutation(root, &mutation).map_err(|error| StateCaptureError::new(error.to_string()))
    });
    let relation = if outcome.is_err() {
        "CONTROLLED_FAILURE"
    } else {
        "NO_FAILURE"
    };
    Ok(result(record(vec![("relation", string_value(relation))])))
}

fn execute_scope_drift(
    root: &Path,
    condition: &Map<String, Value>,
) -> Result<Observation, fixture::FixtureError> {
    let before = path_sequence(condition, "before_scope")?;
    let after = path_sequence(condition, "after_scope")?;
    match conformance_internal::load_with_scope_overrides(root, &before, &after) {
        Ok(_) => Ok(result(record(vec![
            ("stage", string_value("composition")),
            ("status", string_value("SUCCESS")),
        ]))),
        Err(error) => {
            Ok(observe::controlled_failure(error.diagnostic.stage.serialized()).observation)
        }
    }
}

fn execute_invoke(
    root: &Path,
    step: &Value,
) -> Result<(String, InvocationResult), fixture::FixtureError> {
    let object = step
        .as_object()
        .ok_or_else(|| fixture_failure("invoke step must be an object"))?;
    let id = string_field(object, "invocation_id")?;
    if id.is_empty() {
        return Err(fixture_failure("invocation_id must be nonempty"));
    }
    let arguments = fixture::decode_transport(required(object, "arguments")?)?;
    observe::execute(root, &arguments).map(|result| (id.to_owned(), result))
}

fn identity_relation(
    invocations: Vec<(String, InvocationResult)>,
) -> Result<Observation, fixture::FixtureError> {
    if invocations.len() != 2 {
        return Err(fixture_failure(
            "RepositoryGovernanceState relation plan requires exactly two invocations",
        ));
    }
    let (left_id, left) = &invocations[0];
    let (right_id, right) = &invocations[1];
    let left_state = left
        .state
        .as_ref()
        .ok_or_else(|| fixture_failure("left invocation did not produce exact state"))?;
    let right_state = right
        .state
        .as_ref()
        .ok_or_else(|| fixture_failure("right invocation did not produce exact state"))?;
    let relation = if left_state.observed_state.identity == right_state.observed_state.identity {
        "EQUAL"
    } else {
        "NOT_EQUAL"
    };
    Ok(result(record(vec![
        ("left", string_value(left_id)),
        ("relation", string_value(relation)),
        ("right", string_value(right_id)),
    ])))
}

fn apply_mutation(root: &Path, mutation: &Value) -> Result<(), fixture::FixtureError> {
    let object = mutation
        .as_object()
        .ok_or_else(|| fixture_failure("mutation must be an object"))?;
    let action = string_field(object, "action")?;
    let mut step = object.clone();
    step.remove("action");
    step.insert("op".to_owned(), Value::String(action.to_owned()));
    if action == "git" {
        step.entry("environment")
            .or_insert_with(|| Value::Object(Map::new()));
        step.entry("stdin_hex").or_insert(Value::Null);
        step.entry("capture_stdout_as").or_insert(Value::Null);
    }
    fixture::apply_step(root, &Value::Object(step), &mut HashMap::new())
}

fn path_sequence(
    object: &Map<String, Value>,
    name: &str,
) -> Result<Vec<PathBuf>, fixture::FixtureError> {
    required(object, name)?
        .as_array()
        .ok_or_else(|| fixture_failure(format!("{name} must be an array")))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| fixture_failure(format!("{name} values must be strings")))
        })
        .collect()
}

fn required<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a Value, fixture::FixtureError> {
    object
        .get(name)
        .ok_or_else(|| fixture_failure(format!("missing fixture property: {name}")))
}

fn string_field<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, fixture::FixtureError> {
    required(object, name)?
        .as_str()
        .ok_or_else(|| fixture_failure(format!("fixture property {name} must be a string")))
}

fn array_field<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a [Value], fixture::FixtureError> {
    required(object, name)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| fixture_failure(format!("fixture property {name} must be an array")))
}
