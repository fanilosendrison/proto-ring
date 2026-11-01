#![forbid(unsafe_code)]

#[rustfmt::skip]
mod projection {
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use proto_ring_engine::repository_integrity::{self, CommandBinding, ConsumerIntegrityProfile,
    EvaluationContext, EvaluationContextIdentity, IntegrityProfile, IntegrityResult,
    ProfileAuthority, ProfileIdentity, RepositoryPathSelector, RepositoryPathsMode,
    RuntimeCommandObligation, RuntimeValidation, SelectorKind, ValidationDefinition,
    ValidationEnvironmentRealization, ValidationInstances, conformance_internal};
use proto_ring_engine::repository_state::{self, StateCaptureError};
use serde::Deserialize;
use serde_json::{Map, Value};
use super::super::action_receipts::ActionReceiptChannel;
use super::super::transport::{fixture_failure, record, result, string_value};
use crate::fixture;
use crate::model::{Observation, RecordEntry, TransportValue};
type R<T> = Result<T, fixture::FixtureError>;

#[derive(Deserialize)]
struct Invocation { operation: String, #[serde(default)] actions: Vec<ActionFixture>,
    continue_after_non_satisfied: Option<bool>, marker_path: Option<String>,
    #[serde(default)] qualified_execution_observation: bool, profile: Option<ProfileFixture>,
    #[serde(default)] contexts: Vec<ContextFixture>, #[serde(default)] observe_paths: Vec<String>,
    #[serde(default)] ambient_environment: BTreeMap<String, String> }
#[derive(Deserialize)]
struct ActionFixture { id: String, action: String, #[serde(default)] undetermined_exit_codes: Vec<Value>,
    #[serde(flatten)] parameters: Map<String, Value> }
#[derive(Deserialize)]
struct ProfileFixture { authority: AuthorityFixture, carrier: String, continue_after_non_satisfied: bool,
    environments: Vec<String>, identity: String, order: Vec<String>, validations: Vec<ValidationFixture> }
#[derive(Deserialize)]
struct AuthorityFixture { responsibility: String, source: String }
#[derive(Deserialize)]
struct ValidationFixture { command: CommandFixture, id: String, instances: InstancesFixture,
    prerequisites: Vec<String>, responsibility: String }
#[derive(Deserialize)]
struct CommandFixture { arguments: Vec<String>, environment: String, undetermined_exit_codes: Vec<Value> }
#[derive(Deserialize)]
struct InstancesFixture { kind: String, mode: Option<String>, #[serde(default)] selectors: Vec<SelectorFixture> }
#[derive(Deserialize)]
struct SelectorFixture { kind: String, value: String }
#[derive(Deserialize)]
struct ContextFixture { name: String, realizations: BTreeMap<String, RealizationFixture> }
#[derive(Deserialize)]
struct RealizationFixture { action: Option<Value>, command_prefix: Option<Vec<String>>, identity: String,
    process_environment: BTreeMap<String, String> }

pub(crate) fn execute(value: &Value) -> R<Observation> {
    let object = value.as_object().ok_or_else(|| fixture_failure("Repository Integrity fixture must be an object"))?;
    let (base, fault) = match object.get("kind").and_then(Value::as_str) {
        Some("repository_plan") => (value, None),
        Some("procedural_requirement") => (
            object.get("base").ok_or_else(|| fixture_failure("procedural base is required"))?,
            Some(object.get("condition").and_then(Value::as_object)
                .ok_or_else(|| fixture_failure("procedural condition must be an object"))?)),
        _ => return Err(fixture_failure("unsupported Repository Integrity fixture kind")),
    };
    let materialized = fixture::materialize(base)?;
    let root = materialized.root().ok_or_else(|| fixture_failure("repository fixture is required"))?;
    let invocation: Invocation = serde_json::from_value(plain_json(materialized.arguments())?)
        .map_err(|error| fixture_failure(error.to_string()))?;
    match invocation.operation.as_str() {
        "evaluate_integrity" => evaluate_runtime(root, invocation, fault),
        "evaluate_consumer_profile" => evaluate_consumer(root, invocation, fault),
        _ => Err(fixture_failure("unsupported evaluation operation")),
    }
}

fn evaluate_runtime(root: &Path, invocation: Invocation, fault: Option<&Map<String, Value>>) -> R<Observation> {
    let mut receipts = ActionReceiptChannel::new(root)?;
    let validations = invocation.actions.iter().map(|action| {
        let mut command_action = action.parameters.clone();
        command_action.insert("action".to_owned(), Value::String(action.action.clone()));
        let id = action.id.clone();
        Ok(RuntimeValidation { id: id.clone(), prerequisites: BTreeSet::new(),
            obligations: vec![RuntimeCommandObligation { validation_id: id,
                command_prefix: action_command(Value::Object(command_action), &mut receipts)?, arguments: Vec::new(),
                undetermined_exit_codes: integer_set(&action.undetermined_exit_codes)?,
                process_environment: Some(BTreeMap::new()) }], resolution_failed: false })
    }).collect::<R<Vec<_>>>()?;
    let profile = IntegrityProfile { continue_after_non_satisfied: invocation.continue_after_non_satisfied.unwrap_or(true),
        validations, identity: ProfileIdentity(b"corpus runtime profile".to_vec()), governed_paths: Vec::new(),
        resolution_state_identity: None };
    let outcome = evaluate_with_fault(root, &profile, EvaluationContextIdentity(b"corpus".to_vec()),
        fault, &BTreeMap::new(), &mut receipts)?;
    receipts.validate(&profile, &outcome)?;
    let marker = invocation.marker_path.unwrap_or_else(|| "__none__".to_owned());
    let qualified = invocation.qualified_execution_observation;
    let mut values = vec![
        ("marker_absent", TransportValue::Boolean(!root.join(marker).exists())),
        ("obligations", TransportValue::Sequence(outcome.obligations.iter()
            .map(|item| obligation_value(item, qualified)).collect()))];
    if qualified { values.push(("state_coherence_halt", TransportValue::Boolean(outcome.state_coherence_halt))); }
    values.extend([("state_relation", string_value(outcome.state_relation.serialized())),
        ("verdict", string_value(outcome.verdict.serialized()))]);
    Ok(result(record(values)))
}

fn evaluate_consumer(root: &Path, invocation: Invocation, fault: Option<&Map<String, Value>>) -> R<Observation> {
    let profile = consumer_profile(root, invocation.profile
        .ok_or_else(|| fixture_failure("consumer profile is required"))?)?;
    let mut evaluations = Vec::new(); let mut identities = Vec::new(); let mut names = Vec::new();
    for context_fixture in invocation.contexts {
        let mut receipts = ActionReceiptChannel::new(root)?;
        let context = evaluation_context(context_fixture.realizations, &mut receipts)?;
        let realized = if fault_kind(fault) == Some("repository_path_discovery_unavailable") {
            conformance_internal::realize_with_discovery(root, &profile, &context, |_, _| {
                Err(StateCaptureError::new("fixture discovery unavailable")) })
        } else { repository_integrity::realize_consumer_profile(root, &profile, &context) }
            .map_err(|error| fixture_failure(error.to_string()))?;
        let outcome = evaluate_with_fault(root, &realized.profile, realized.evaluation_context_identity.clone(),
            fault, &invocation.ambient_environment, &mut receipts)?;
        receipts.validate(&realized.profile, &outcome)?;
        names.push(context_fixture.name); identities.push(outcome.evaluation_context_identity.clone());
        evaluations.push(consumer_value(root, &realized.profile, &outcome, &invocation.observe_paths)?);
    }
    if evaluations.len() == 1 { return Ok(result(evaluations.remove(0))); }
    let relations = (1..evaluations.len()).map(|index| record(vec![
        ("left", string_value(&names[0])),
        ("relation", string_value(if identities[0] == identities[index] { "EQUAL" } else { "NOT_EQUAL" })),
        ("right", string_value(&names[index]))
    ])).collect();
    Ok(result(record(vec![("evaluations", TransportValue::Sequence(evaluations)),
        ("identity_relations", TransportValue::Sequence(relations))])))
}

fn evaluate_with_fault(root: &Path, profile: &IntegrityProfile, identity: EvaluationContextIdentity,
    fault: Option<&Map<String, Value>>, ambient: &BTreeMap<String, String>,
    receipts: &mut ActionReceiptChannel) -> R<IntegrityResult>
{
    let kind = fault_kind(fault); let mutation = fault.and_then(|value| value.get("mutation")).cloned();
    Ok(conformance_internal::evaluate_with_execution_observer(root, profile, identity,
        |phase, repository, scope| {
            use conformance_internal::CapturePhase::{Baseline, PostObligation, PreObligation};
            if (kind == Some("repository_state_unavailable_before_obligation") && phase == PreObligation)
                || (kind == Some("repository_state_unavailable_after_obligation") && phase == PostObligation)
            { return Err(StateCaptureError::new("fixture state unavailable")); }
            let state = repository_state::capture(repository, scope)?;
            if kind == Some("repository_mutation_during_integrity_evaluation") && phase == Baseline {
                let mutation = mutation.as_ref().ok_or_else(|| StateCaptureError::new("fixture mutation absent"))?;
                apply_mutation(repository, mutation).map_err(|error| StateCaptureError::new(error.to_string()))?;
            }
            Ok(state)
        }, ambient, |obligation, started| receipts.observe_execution(obligation, started)))
}

fn consumer_profile(root: &Path, profile: ProfileFixture) -> R<ConsumerIntegrityProfile> {
    let validations = profile.validations.into_iter().map(|item| {
        let id = item.id.clone();
        let instances = match item.instances.kind.as_str() {
            "single" => ValidationInstances::Single,
            "repository_paths" => ValidationInstances::RepositoryPaths {
                mode: match item.instances.mode.as_deref() {
                    Some("append_all") => RepositoryPathsMode::AppendAll,
                    Some("for_each") => RepositoryPathsMode::ForEach,
                    _ => return Err(fixture_failure("invalid repository path mode")),
                },
                selectors: item.instances.selectors.into_iter().map(selector).collect::<R<_>>()?,
            },
            _ => return Err(fixture_failure("invalid instance kind")),
        };
        Ok((id.clone(), ValidationDefinition { id, responsibility: item.responsibility,
            prerequisites: item.prerequisites.into_iter().collect(), instances,
            command: CommandBinding { environment: item.command.environment, arguments: item.command.arguments,
                undetermined_exit_codes: integer_set(&item.command.undetermined_exit_codes)? }, target: None }))
    }).collect::<R<_>>()?;
    Ok(ConsumerIntegrityProfile {
        repository: root.canonicalize().map_err(|error| fixture_failure(error.to_string()))?,
        carrier: root.join(profile.carrier), model_version: 1,
        authority: ProfileAuthority { responsibility: profile.authority.responsibility, source: profile.authority.source },
        environments: profile.environments.into_iter().collect(),
        continue_after_non_satisfied: profile.continue_after_non_satisfied,
        validations, order: profile.order, identity: ProfileIdentity(profile.identity.into_bytes()),
    })
}

fn evaluation_context(values: BTreeMap<String, RealizationFixture>,
    receipts: &mut ActionReceiptChannel) -> R<EvaluationContext> {
    let realizations = values.into_iter().map(|(environment_id, item)| {
        let command_prefix = match (item.action, item.command_prefix) {
            (Some(action), None) => action_command(action, receipts)?, (None, Some(prefix)) => prefix,
            _ => return Err(fixture_failure("realization requires one command form")),
        };
        Ok((environment_id.clone(), ValidationEnvironmentRealization { environment_id,
            identity: item.identity.into_bytes(), command_prefix, process_environment: item.process_environment }))
    }).collect::<R<_>>()?;
    Ok(EvaluationContext { realizations })
}
fn selector(value: SelectorFixture) -> R<RepositoryPathSelector> {
    Ok(RepositoryPathSelector { kind: match value.kind.as_str() {
        "path" => SelectorKind::Path, "glob" => SelectorKind::Glob,
        _ => return Err(fixture_failure("invalid selector kind")), }, value: value.value })
}

fn consumer_value(root: &Path, profile: &IntegrityProfile, outcome: &IntegrityResult,
    observe_paths: &[String]) -> R<TransportValue>
{
    let executed_arguments = profile.validations.iter().zip(&outcome.validations)
        .flat_map(|(runtime, result)| runtime.obligations.iter().zip(&result.obligations))
        .filter(|(_, result)| result.executed).map(|(runtime, _)| strings(runtime.arguments.iter())).collect();
    let mut ordered_paths = observe_paths.to_vec(); ordered_paths.sort();
    let side_effects = ordered_paths.iter().map(|path| {
        let absolute = root.join(path);
        let text = absolute.is_file().then(|| fs::read_to_string(&absolute)).transpose()
            .map_err(|error| fixture_failure(error.to_string()))?
            .map_or(TransportValue::Null(()), |value| string_value(&value));
        Ok(RecordEntry { name: path.clone(), value: record(vec![
            ("present", TransportValue::Boolean(absolute.exists())), ("text", text)]) })
    }).collect::<R<Vec<_>>>()?;
    Ok(record(vec![
        ("executed_arguments", TransportValue::Sequence(executed_arguments)),
        ("obligations", TransportValue::Sequence(outcome.obligations.iter()
            .map(|item| obligation_value(item, false)).collect())),
        ("side_effects", TransportValue::Record(side_effects)),
        ("state_relation", string_value(outcome.state_relation.serialized())),
        ("validations", TransportValue::Sequence(outcome.validations.iter().map(validation_value).collect())),
        ("verdict", string_value(outcome.verdict.serialized())),
    ]))
}
fn obligation_value(item: &repository_integrity::ObligationResult, include_executed: bool) -> TransportValue {
    if include_executed { return record(vec![("executed", TransportValue::Boolean(item.executed)),
        ("id", string_value(&item.id)), ("status", string_value(item.status.serialized()))]); }
    record(vec![("id", string_value(&item.id)), ("status", string_value(item.status.serialized()))])
}
fn validation_value(item: &repository_integrity::ValidationResult) -> TransportValue {
    record(vec![("id", string_value(&item.id)),
        ("obligations", TransportValue::Sequence(item.obligations.iter()
            .map(|value| obligation_value(value, false)).collect())),
        ("status", string_value(item.status.serialized()))])
}

fn action_command(mut action: Value, receipts: &mut ActionReceiptChannel) -> R<Vec<String>> {
    if action.get("action").and_then(Value::as_str) == Some("git") {
        let object = action.as_object_mut()
            .ok_or_else(|| fixture_failure("command action must be an object"))?;
        object.insert("_fixture_git_executable".to_owned(), Value::String(resolve_program("git")?));
        let mut environment = Map::new();
        for name in ["PATH", "SYSTEMROOT", "WINDIR", "PATHEXT", "COMSPEC"] {
            if let Some(value) = std::env::var_os(name) {
                environment.insert(name.to_owned(), Value::String(value.into_string()
                    .map_err(|_| fixture_failure("trusted fixture environment is not UTF-8"))?));
            }
        }
        object.insert("_fixture_git_environment".to_owned(), Value::Object(environment));
    }
    receipts.command(
        &action,
        python_executable()?,
        &repository_root().join("tests/conformance_fixture_runner.py"),
    )
}
fn python_executable() -> R<String> {
    let configured = std::env::var_os("PROTO_RING_PYTHON").unwrap_or_else(|| "python3".into());
    let path = PathBuf::from(&configured);
    let resolved = if path.components().count() > 1 { path } else {
        std::env::var_os("PATH").into_iter().flat_map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
            .map(|directory| directory.join(&path)).find(|candidate| candidate.is_file()).unwrap_or(path) };
    resolved.into_os_string().into_string().map_err(|_| fixture_failure("Python executable path is not UTF-8"))
}
fn resolve_program(name: &str) -> R<String> {
    let path = std::env::var_os("PATH").ok_or_else(|| fixture_failure("trusted fixture PATH is unavailable"))?;
    let names = vec![name.to_owned()];
    #[cfg(windows)]
    let names = {
        let mut names = names;
        if Path::new(name).extension().is_none() {
            names.extend(std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".to_owned())
                .split(';').filter(|extension| !extension.is_empty())
                .map(|extension| format!("{name}{extension}")));
        }
        names
    };
    for directory in std::env::split_paths(&path) {
        for candidate_name in &names {
            let candidate = directory.join(candidate_name);
            if candidate.is_file() {
                return candidate.canonicalize().map_err(|error| fixture_failure(error.to_string()))?
                    .into_os_string().into_string()
                    .map_err(|_| fixture_failure("Git executable path is not UTF-8"));
            }
        }
    }
    Err(fixture_failure("Git executable is unavailable"))
}
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent)
        .expect("workspace root exists").to_path_buf()
}
fn apply_mutation(root: &Path, value: &Value) -> R<()> {
    let mut object = value.as_object().ok_or_else(|| fixture_failure("mutation must be an object"))?.clone();
    let action = object.remove("action").and_then(|value| value.as_str().map(str::to_owned))
        .ok_or_else(|| fixture_failure("mutation action must be a string"))?;
    object.insert("op".to_owned(), Value::String(action));
    fixture::apply_step(root, &Value::Object(object), &mut HashMap::new())
}
fn plain_json(value: &TransportValue) -> R<Value> {
    Ok(match value {
        TransportValue::Null(()) => Value::Null, TransportValue::Boolean(value) => Value::Bool(*value),
        TransportValue::Integer(value) => serde_json::from_str(value).map_err(|error| fixture_failure(error.to_string()))?,
        TransportValue::String(value) => Value::String(value.clone()),
        TransportValue::Bytes(_) => return Err(fixture_failure("bytes are not valid invocation data")),
        TransportValue::Sequence(values) => Value::Array(values.iter().map(plain_json).collect::<R<_>>()?),
        TransportValue::Record(entries) => Value::Object(entries.iter()
            .map(|entry| Ok((entry.name.clone(), plain_json(&entry.value)?))).collect::<R<_>>()?),
    })
}
fn integer_set(values: &[Value]) -> R<BTreeSet<String>> {
    values.iter().map(|value| value.as_number().map(ToString::to_string)
        .ok_or_else(|| fixture_failure("exit code must be integer"))).collect()
}
fn strings<'a>(values: impl IntoIterator<Item = &'a String>) -> TransportValue {
    TransportValue::Sequence(values.into_iter().map(|value| string_value(value)).collect())
}
fn fault_kind(value: Option<&Map<String, Value>>) -> Option<&str> {
    value.and_then(|object| object.get("kind")).and_then(Value::as_str)
}
}

pub(super) use projection::execute;
