use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use proto_ring_engine::repository_state::StateCaptureError;
use serde_json::{Map, Value};

use super::super::transport::{fixture_failure, record, result, string_value};
use super::observe::{self, InvocationResult};
use crate::fixture;
use crate::model::Observation;

pub(crate) fn execute(fixture_value: &Value) -> Result<Observation, fixture::FixtureError> {
    let object = fixture_value
        .as_object()
        .ok_or_else(|| fixture_failure("RepositoryState fixture must be an object"))?;
    match string_field(object, "kind")? {
        "repository_plan" => execute_repository_plan(object),
        "procedural_requirement" => execute_procedural(object),
        _ => Err(fixture_failure("unsupported RepositoryState fixture kind")),
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
    if !invocations.is_empty() {
        return identity_relation(invocations);
    }
    let invocation = fixture::interpolate(required(plan, "invocation")?, &captures)?;
    let arguments = fixture::decode_transport(&invocation)?;
    observe::execute(&root, &arguments).map(|result| result.observation)
}

fn execute_procedural(
    fixture_value: &Map<String, Value>,
) -> Result<Observation, fixture::FixtureError> {
    let base = required(fixture_value, "base")?
        .as_object()
        .ok_or_else(|| fixture_failure("procedural base must be an object"))?;
    if string_field(base, "kind")? != "repository_plan" {
        return Err(fixture_failure(
            "RepositoryState procedural base must be repository_plan",
        ));
    }
    let condition = required(fixture_value, "condition")?
        .as_object()
        .ok_or_else(|| fixture_failure("procedural condition must be an object"))?;
    if string_field(condition, "kind")? != "repository_mutation_during_capture" {
        return Err(fixture_failure(
            "unsupported RepositoryState procedural condition",
        ));
    }
    let mutation = required(condition, "mutation")?.clone();
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
                "procedural base cannot invoke before capture checkpoint",
            ));
        }
        fixture::apply_step(&root, &step, &mut captures)?;
    }
    let invocation = fixture::interpolate(required(base, "invocation")?, &captures)?;
    let arguments = fixture::decode_transport(&invocation)?;
    observe::execute_procedural(&root, &arguments, |root| {
        apply_mutation(root, &mutation).map_err(|error| StateCaptureError::new(error.to_string()))
    })
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
            "RepositoryState relation plan requires exactly two invocations",
        ));
    }
    let (left_id, left) = &invocations[0];
    let (right_id, right) = &invocations[1];
    let left_state = left
        .state
        .as_ref()
        .ok_or_else(|| fixture_failure("left relation capture did not produce identity"))?;
    let right_state = right
        .state
        .as_ref()
        .ok_or_else(|| fixture_failure("right relation capture did not produce identity"))?;
    let relation = if left_state.identity == right_state.identity {
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
    match action {
        "append_utf8" => {
            let path = mutation_path(root, string_field(object, "path")?)?;
            let text = string_field(object, "text")?;
            OpenOptions::new()
                .append(true)
                .open(path)
                .and_then(|mut file| file.write_all(text.as_bytes()))
                .map_err(|error| fixture_failure(format!("mutation append failed: {error}")))
        }
        "touch" => {
            let path = mutation_path(root, string_field(object, "path")?)?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| fixture_failure(format!("touch parent failed: {error}")))?;
            }
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|error| fixture_failure(format!("touch failed: {error}")))?;
            file.set_modified(SystemTime::now())
                .map_err(|error| fixture_failure(format!("touch timestamp failed: {error}")))
        }
        "write_utf8" | "remove" | "chmod" | "git" => {
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
        _ => Err(fixture_failure("unsupported procedural mutation action")),
    }
}

fn mutation_path(root: &Path, relative: &str) -> Result<PathBuf, fixture::FixtureError> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(fixture_failure("mutation path escapes fixture root"));
    }
    Ok(root.join(path))
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
