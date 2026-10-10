use super::support::{support_authority, support_catalog};
use super::transport::{
    exact_record, fixture_failure, record, rejection, repository_snapshot, result, string_value,
    unique_dynamic_record,
};
use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, RecordEntry, TransportValue};
use proto_ring_engine::governance_routing::ResolvedGovernanceRoute;
use proto_ring_engine::repository_integrity::{
    self, ConsumerIntegrityProfile, ValidationDefinition, ValidationInstances,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path};

type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;
pub(super) fn execute_identity_plan(
    fixture_value: &serde_json::Value,
) -> Result<Option<Observation>, fixture::FixtureError> {
    let Some(steps) = fixture_value
        .get("steps")
        .and_then(serde_json::Value::as_array)
    else {
        return Ok(None);
    };
    if !steps
        .iter()
        .any(|step| step.get("op").and_then(serde_json::Value::as_str) == Some("invoke"))
    {
        return Ok(None);
    }
    let temporary = tempfile::tempdir().map_err(|error| fixture_failure(error.to_string()))?;
    let root = temporary.path();
    let mut identities = Vec::new();
    for step in steps {
        let operation = step
            .get("op")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| fixture_failure("identity plan step requires string op"))?;
        match operation {
            "write_utf8" => {
                let relative = step
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| fixture_failure("write_utf8 requires path"))?;
                let path = checked_path(root, relative)?;
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|error| fixture_failure(error.to_string()))?;
                }
                let text = step
                    .get("text")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| fixture_failure("write_utf8 requires text"))?;
                fs::write(path, text).map_err(|error| fixture_failure(error.to_string()))?;
            }
            "invoke" => {
                let id = step
                    .get("invocation_id")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| fixture_failure("invoke requires invocation_id"))?;
                let arguments: TransportValue = serde_json::from_value(
                    step.get("arguments")
                        .cloned()
                        .ok_or_else(|| fixture_failure("invoke requires arguments"))?,
                )
                .map_err(|error| fixture_failure(error.to_string()))?;
                identities.push((id.to_owned(), load_identity(root, &arguments)?));
            }
            _ => return Err(fixture_failure("unsupported identity plan operation")),
        }
    }
    if identities.len() != 2 {
        return Err(fixture_failure(
            "identity plan requires exactly two invocations",
        ));
    }
    let relation = if identities[0].1 == identities[1].1 {
        "EQUAL"
    } else {
        "NOT_EQUAL"
    };
    Ok(Some(result(record(vec![
        (
            "identity_relation",
            record(vec![
                ("left", string_value(&identities[0].0)),
                ("relation", string_value(relation)),
                ("right", string_value(&identities[1].0)),
            ]),
        ),
        ("profiles_valid", TransportValue::Boolean(true)),
    ]))))
}

fn load_identity(
    root: &Path,
    arguments: &TransportValue,
) -> Result<proto_ring_engine::repository_integrity::ProfileIdentity, fixture::FixtureError> {
    let fields = exact_record(
        arguments,
        "identity invocation",
        &[
            "identity_only",
            "operation",
            "path",
            "semantic_inputs",
            "with_catalog",
        ],
    )?;
    if !boolean(field(&fields, "identity_only"), "identity_only")?
        || string_field(&fields, "operation")? != "load_integrity"
    {
        return Err(fixture_failure("invalid identity invocation"));
    }
    let semantic = exact_record(
        field(&fields, "semantic_inputs"),
        "semantic_inputs",
        &["authority", "bindings", "catalog", "integrity_profile"],
    )?;
    let authority = support_authority(root, field(&semantic, "authority"))?;
    let catalog = support_catalog(root, field(&semantic, "catalog"))?;
    let with_catalog = boolean(field(&fields, "with_catalog"), "with_catalog")?;
    let path = string_field(&fields, "path")?;
    let route = ResolvedGovernanceRoute {
        route: vec![
            "capabilities".to_owned(),
            "repository_integrity".to_owned(),
            "routes".to_owned(),
            "profile".to_owned(),
        ],
        declared_path: path.to_owned(),
        target: root.join(path),
    };
    repository_integrity::load(root, &route, &authority, with_catalog.then_some(&catalog))
        .map(|profile| profile.identity)
        .map_err(|error| fixture_failure(format!("identity profile rejected: {error}")))
}

fn checked_path(root: &Path, relative: &str) -> Result<std::path::PathBuf, fixture::FixtureError> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        Err(fixture_failure("identity plan path escapes repository"))
    } else {
        Ok(root.join(path))
    }
}

pub(super) fn execute(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    let entries = unique_dynamic_record(fixture.arguments(), "Repository Integrity invocation")?;
    let allowed = [
        "foreign_support_repository",
        "observe_execution_sentinel",
        "observe_read_only",
        "observe_validation_id",
        "observe_validation_ids",
        "operation",
        "path",
        "semantic_inputs",
        "with_catalog",
    ];
    if entries.iter().any(|(name, _)| !allowed.contains(name)) {
        return Err(fixture_failure(
            "Repository Integrity invocation has unknown field",
        ));
    }
    let fields: Fields<'_> = entries.into_iter().collect();
    for required in ["operation", "path", "semantic_inputs", "with_catalog"] {
        if !fields.contains_key(required) {
            return Err(fixture_failure(format!("invocation is missing {required}")));
        }
    }
    if string_field(&fields, "operation")? != "load_integrity" {
        return Err(fixture_failure("unsupported operation"));
    }
    let root = fixture
        .root()
        .ok_or_else(|| fixture_failure("repository fixture required"))?;
    let semantic = exact_record(
        field(&fields, "semantic_inputs"),
        "semantic_inputs",
        &["authority", "bindings", "catalog", "integrity_profile"],
    )?;
    let foreign_support = optional_string(&fields, "foreign_support_repository")?;
    if foreign_support.is_some_and(|selected| selected != "catalog") {
        return Err(fixture_failure(
            "invalid foreign_support_repository transport value",
        ));
    }
    let catalog_root = foreign_support
        .map(|_| root.join("foreign"))
        .unwrap_or_else(|| root.to_path_buf());
    let authority = support_authority(root, field(&semantic, "authority"))?;
    let catalog = support_catalog(&catalog_root, field(&semantic, "catalog"))?;
    let with_catalog = boolean(field(&fields, "with_catalog"), "with_catalog")?;
    let observe_read_only = optional_boolean(&fields, "observe_read_only")?.unwrap_or(false);
    let before = observe_read_only
        .then(|| repository_snapshot(root))
        .transpose()?;
    let path = string_field(&fields, "path")?;
    let route = ResolvedGovernanceRoute {
        route: vec![
            "capabilities".to_owned(),
            "repository_integrity".to_owned(),
            "routes".to_owned(),
            "profile".to_owned(),
        ],
        declared_path: path.to_owned(),
        target: root.join(path),
    };
    Ok(
        match repository_integrity::load(root, &route, &authority, with_catalog.then_some(&catalog))
        {
            Ok(profile) => {
                let unchanged = before
                    .map(|state| repository_snapshot(root).map(|after| state == after))
                    .transpose()?;
                result(profile_transport(&profile, &fields, unchanged)?)
            }
            Err(_) => rejection("repository-integrity.profile"),
        },
    )
}

fn profile_transport(
    profile: &ConsumerIntegrityProfile,
    fields: &Fields<'_>,
    unchanged: Option<bool>,
) -> Result<TransportValue, fixture::FixtureError> {
    let mut values = vec![
        (
            "continue_after_non_satisfied",
            TransportValue::Boolean(profile.continue_after_non_satisfied),
        ),
        ("environments", strings(profile.environments.iter())),
    ];
    if optional_boolean(fields, "observe_execution_sentinel")?.is_some() {
        values.push(("execution_sentinel_absent", TransportValue::Boolean(true)));
    }
    values.push((
        "identity_present",
        TransportValue::Boolean(!profile.identity.0.is_empty()),
    ));
    if let Some(id) = optional_string(fields, "observe_validation_id")? {
        let validation = profile
            .validations
            .get(id)
            .ok_or_else(|| fixture_failure("observed validation is absent"))?;
        values.push(("loaded_validation", validation_transport(validation)));
    }
    if let Some(ids) = fields.get("observe_validation_ids") {
        let loaded = fixture::sequence(ids)?
            .iter()
            .map(|id| {
                let id = fixture::string(id)?;
                profile
                    .validations
                    .get(id)
                    .map(|validation| (id.to_owned(), validation_transport(validation)))
                    .ok_or_else(|| fixture_failure("observed validation is absent"))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        values.push((
            "loaded_validations",
            TransportValue::Record(
                loaded
                    .into_iter()
                    .map(|(name, value)| RecordEntry { name, value })
                    .collect(),
            ),
        ));
    }
    values.push((
        "model_version",
        TransportValue::Integer(profile.model_version.to_string()),
    ));
    values.push(("order", strings(profile.order.iter())));
    values.push((
        "profile_authority",
        record(vec![
            (
                "responsibility",
                string_value(&profile.authority.responsibility),
            ),
            ("source", string_value(&profile.authority.source)),
        ]),
    ));
    if let Some(unchanged) = unchanged {
        values.push(("repository_unchanged", TransportValue::Boolean(unchanged)));
    }
    values.push(("validations", strings(profile.validations.keys())));
    Ok(record(values))
}

pub(super) fn validation_transport(validation: &ValidationDefinition) -> TransportValue {
    let (mode, selectors) = match &validation.instances {
        ValidationInstances::Single => (TransportValue::Null(()), Vec::new()),
        ValidationInstances::RepositoryPaths { mode, selectors } => (
            string_value(mode.serialized()),
            selectors
                .iter()
                .map(|selector| {
                    record(vec![
                        ("kind", string_value(selector.kind.serialized())),
                        ("value", string_value(&selector.value)),
                    ])
                })
                .collect(),
        ),
    };
    record(vec![
        (
            "command",
            record(vec![
                ("arguments", strings(validation.command.arguments.iter())),
                ("environment", string_value(&validation.command.environment)),
                (
                    "undetermined_exit_codes",
                    TransportValue::Sequence(
                        validation
                            .command
                            .undetermined_exit_codes
                            .iter()
                            .map(|value| TransportValue::Integer(value.clone()))
                            .collect(),
                    ),
                ),
            ]),
        ),
        ("id", string_value(&validation.id)),
        (
            "instances",
            record(vec![
                ("kind", string_value(validation.instances.kind())),
                ("mode", mode),
                ("selectors", TransportValue::Sequence(selectors)),
            ]),
        ),
        ("prerequisites", strings(validation.prerequisites.iter())),
        ("responsibility", string_value(&validation.responsibility)),
        (
            "target",
            validation
                .target
                .as_ref()
                .map(|target| {
                    record(vec![
                        ("interface", string_value(&target.interface_id)),
                        ("object", string_value(&target.object_id)),
                    ])
                })
                .unwrap_or(TransportValue::Null(())),
        ),
    ])
}

fn field<'a>(fields: &Fields<'a>, name: &str) -> &'a TransportValue {
    fields.get(name).copied().expect("required field exists")
}
fn string_field<'a>(fields: &Fields<'a>, name: &str) -> Result<&'a str, fixture::FixtureError> {
    fixture::string(field(fields, name))
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
fn boolean(value: &TransportValue, label: &str) -> Result<bool, fixture::FixtureError> {
    match value {
        TransportValue::Boolean(value) => Ok(*value),
        _ => Err(fixture_failure(format!("{label} must be boolean"))),
    }
}
fn optional_boolean(
    fields: &Fields<'_>,
    name: &str,
) -> Result<Option<bool>, fixture::FixtureError> {
    fields
        .get(name)
        .map(|value| boolean(value, name))
        .transpose()
}
fn strings<'a>(values: impl IntoIterator<Item = &'a String>) -> TransportValue {
    TransportValue::Sequence(
        values
            .into_iter()
            .map(|value| string_value(value))
            .collect(),
    )
}
