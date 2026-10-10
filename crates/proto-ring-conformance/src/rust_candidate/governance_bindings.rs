use std::collections::BTreeMap;

use proto_ring_engine::governance_bindings::{
    self, GovernanceBinding, GovernanceBindingIdentity, GovernanceBindingRegistry,
};
use proto_ring_engine::governance_routing::ResolvedGovernanceRoute;

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, RecordEntry, TransportValue};

use super::support::{support_authority, support_catalog};
use super::transport::{
    exact_record, fixture_failure, record, rejection, repository_snapshot, result, string_value,
    unique_dynamic_record,
};

type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;

pub(super) fn execute(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    let entries = unique_dynamic_record(fixture.arguments(), "Governance Bindings invocation")?;
    let allowed = [
        "foreign_support_repository",
        "observe_binding_id",
        "observe_read_only",
        "operation",
        "path",
        "semantic_inputs",
        "with_catalog",
    ];
    if entries.iter().any(|(name, _)| !allowed.contains(name)) {
        return Err(fixture_failure(
            "Governance Bindings invocation has unknown field",
        ));
    }
    let fields: Fields<'_> = entries.into_iter().collect();
    for required in ["operation", "path", "semantic_inputs", "with_catalog"] {
        if !fields.contains_key(required) {
            return Err(fixture_failure(format!("invocation is missing {required}")));
        }
    }
    if string_field(&fields, "operation")? != "load_bindings" {
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
    let observe = optional_string(&fields, "observe_binding_id")?;
    let observe_read_only = optional_boolean(&fields, "observe_read_only")?.unwrap_or(false);
    let before = observe_read_only
        .then(|| repository_snapshot(root))
        .transpose()?;
    let path = string_field(&fields, "path")?;
    let route = ResolvedGovernanceRoute {
        route: vec![
            "capabilities".to_owned(),
            "shared_governance_provider".to_owned(),
            "routes".to_owned(),
            "registry".to_owned(),
        ],
        declared_path: path.to_owned(),
        target: root.join(path),
    };
    let loaded =
        governance_bindings::load(root, &route, &authority, with_catalog.then_some(&catalog));
    Ok(match loaded {
        Ok(registry) => {
            let unchanged = before
                .map(|state| repository_snapshot(root).map(|after| state == after))
                .transpose()?;
            result(registry_transport(&registry, observe, unchanged)?)
        }
        Err(_) => rejection("governance-bindings.registry"),
    })
}

fn registry_transport(
    registry: &GovernanceBindingRegistry,
    observe: Option<&str>,
    unchanged: Option<bool>,
) -> Result<TransportValue, fixture::FixtureError> {
    let mut values = vec![
        ("bindings", strings(registry.bindings.keys())),
        (
            "kinds",
            TransportValue::Record(
                registry
                    .bindings
                    .iter()
                    .map(|(id, binding)| RecordEntry {
                        name: id.clone(),
                        value: string_value(binding.kind.serialized()),
                    })
                    .collect(),
            ),
        ),
    ];
    if let Some(id) = observe {
        let binding = registry
            .bindings
            .get(id)
            .ok_or_else(|| fixture_failure("observed binding is absent"))?;
        values.push(("loaded_binding", binding_transport(binding)));
    }
    values.push((
        "model_version",
        TransportValue::Integer(registry.model_version.to_string()),
    ));
    if let Some(unchanged) = unchanged {
        values.push(("repository_unchanged", TransportValue::Boolean(unchanged)));
    }
    values.push(("source", string_value(&registry.source)));
    Ok(record(values))
}

pub(super) fn binding_transport(binding: &GovernanceBinding) -> TransportValue {
    let (repository, commit, path) = match &binding.identity {
        GovernanceBindingIdentity::ExecutableProvider(identity) => {
            (&identity.repository, &identity.commit, None)
        }
        GovernanceBindingIdentity::GovernanceContract(identity) => (
            &identity.repository,
            &identity.commit,
            Some(identity.path.as_str()),
        ),
    };
    record(vec![
        (
            "authority",
            record(vec![
                (
                    "responsibility",
                    string_value(&binding.authority.responsibility),
                ),
                ("source", string_value(&binding.authority.source)),
            ]),
        ),
        ("id", string_value(&binding.id)),
        (
            "identity",
            record(vec![
                ("commit", string_value(commit)),
                (
                    "path",
                    path.map(string_value).unwrap_or(TransportValue::Null(())),
                ),
                ("repository", string_value(repository)),
            ]),
        ),
        ("kind", string_value(binding.kind.serialized())),
        (
            "scope",
            record(vec![
                (
                    "capability",
                    binding
                        .scope
                        .capability
                        .as_deref()
                        .map(string_value)
                        .unwrap_or(TransportValue::Null(())),
                ),
                (
                    "interface",
                    binding
                        .scope
                        .interface
                        .as_deref()
                        .map(string_value)
                        .unwrap_or(TransportValue::Null(())),
                ),
                ("kind", string_value(binding.scope.kind.serialized())),
                (
                    "object",
                    binding
                        .scope
                        .object
                        .as_deref()
                        .map(string_value)
                        .unwrap_or(TransportValue::Null(())),
                ),
            ]),
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
