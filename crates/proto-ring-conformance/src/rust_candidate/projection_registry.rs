use std::collections::BTreeMap;

use proto_ring_engine::governance_routing::ResolvedGovernanceRoute;
use proto_ring_engine::projection_registry::{self, Projection, ProjectionRegistry};

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, RecordEntry, TransportValue};

use super::support::{
    support_authority, support_bindings, support_catalog, support_integrity_profile,
};
use super::transport::{
    exact_record, fixture_failure, record, rejection, repository_snapshot, result, string_value,
    unique_dynamic_record,
};

type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;

pub(super) fn execute(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    let entries = unique_dynamic_record(fixture.arguments(), "Projection Registry invocation")?;
    let allowed = [
        "foreign_support_repository",
        "observe_projection_id",
        "observe_read_only",
        "observe_registry_authority",
        "operation",
        "path",
        "qualified_observation",
        "semantic_inputs",
        "with_bindings",
        "with_catalog",
    ];
    if entries.iter().any(|(name, _)| !allowed.contains(name)) {
        return Err(fixture_failure(
            "Projection Registry invocation has unknown field",
        ));
    }
    let fields: Fields<'_> = entries.into_iter().collect();
    for required in [
        "operation",
        "path",
        "semantic_inputs",
        "with_bindings",
        "with_catalog",
    ] {
        if !fields.contains_key(required) {
            return Err(fixture_failure(format!("invocation is missing {required}")));
        }
    }
    if string_field(&fields, "operation")? != "load_projections" {
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
    if foreign_support
        .is_some_and(|selected| !matches!(selected, "integrity_profile" | "catalog" | "bindings"))
    {
        return Err(fixture_failure(
            "invalid foreign_support_repository transport value",
        ));
    }
    let support_root = |name: &str| {
        if foreign_support == Some(name) {
            root.join("foreign")
        } else {
            root.to_path_buf()
        }
    };
    let authority = support_authority(root, field(&semantic, "authority"))?;
    let catalog = support_catalog(&support_root("catalog"), field(&semantic, "catalog"))?;
    let integrity = support_integrity_profile(
        &support_root("integrity_profile"),
        field(&semantic, "integrity_profile"),
    )?;
    let bindings = support_bindings(&support_root("bindings"), field(&semantic, "bindings"))?;
    let with_catalog = boolean(field(&fields, "with_catalog"), "with_catalog")?;
    let with_bindings = boolean(field(&fields, "with_bindings"), "with_bindings")?;
    let observe_read_only = optional_boolean(&fields, "observe_read_only")?.unwrap_or(false);
    let before = observe_read_only
        .then(|| repository_snapshot(root))
        .transpose()?;
    let path = string_field(&fields, "path")?;
    let route = ResolvedGovernanceRoute {
        route: vec![
            "capabilities".to_owned(),
            "projection_integrity".to_owned(),
            "routes".to_owned(),
            "registry".to_owned(),
        ],
        declared_path: path.to_owned(),
        target: root.join(path),
    };
    Ok(
        match projection_registry::load(
            root,
            &route,
            &authority,
            &integrity,
            with_catalog.then_some(&catalog),
            with_bindings.then_some(&bindings),
        ) {
            Ok(registry) => {
                let unchanged = before
                    .map(|state| repository_snapshot(root).map(|after| state == after))
                    .transpose()?;
                result(registry_transport(&registry, &fields, unchanged)?)
            }
            Err(_) => rejection("projection-registry.registry"),
        },
    )
}

fn registry_transport(
    registry: &ProjectionRegistry,
    fields: &Fields<'_>,
    unchanged: Option<bool>,
) -> Result<TransportValue, fixture::FixtureError> {
    let qualified = optional_boolean(fields, "qualified_observation")?.unwrap_or(false);
    let mut values = Vec::new();
    if qualified {
        values.push((
            "loaded",
            TransportValue::Record(
                registry
                    .projections
                    .iter()
                    .map(|(id, projection)| RecordEntry {
                        name: id.clone(),
                        value: record(vec![
                            (
                                "canonical_source",
                                string_value(&projection.canonical_source),
                            ),
                            ("mode", string_value(projection.mode.serialized())),
                            (
                                "secondary_source",
                                string_value(&projection.secondary_source),
                            ),
                        ]),
                    })
                    .collect(),
            ),
        ));
    }
    if let Some(id) = optional_string(fields, "observe_projection_id")? {
        let projection = registry
            .projections
            .get(id)
            .ok_or_else(|| fixture_failure("observed projection is absent"))?;
        values.push(("loaded_projection", projection_transport(projection)));
    }
    values.push((
        "model_version",
        TransportValue::Integer(registry.model_version.to_string()),
    ));
    values.push((
        "modes",
        TransportValue::Record(
            registry
                .projections
                .iter()
                .map(|(id, projection)| RecordEntry {
                    name: id.clone(),
                    value: string_value(projection.mode.serialized()),
                })
                .collect(),
        ),
    ));
    values.push(("projections", strings(registry.projections.keys())));
    if qualified {
        values.push(("registry", string_value("ACCEPTED")));
    }
    if optional_boolean(fields, "observe_registry_authority")?.is_some() {
        values.push((
            "registry_authority",
            record(vec![
                (
                    "responsibility",
                    string_value(&registry.authority.responsibility),
                ),
                ("source", string_value(&registry.authority.source)),
            ]),
        ));
    }
    if let Some(unchanged) = unchanged {
        values.push(("repository_unchanged", TransportValue::Boolean(unchanged)));
    }
    Ok(record(values))
}

fn projection_transport(projection: &Projection) -> TransportValue {
    record(vec![
        ("binding", optional(&projection.binding)),
        ("boundary_source", optional(&projection.boundary_source)),
        (
            "canonical_source",
            string_value(&projection.canonical_source),
        ),
        ("generator_source", optional(&projection.generator_source)),
        ("id", string_value(&projection.id)),
        ("mode", string_value(projection.mode.serialized())),
        ("responsibility", string_value(&projection.responsibility)),
        (
            "secondary_source",
            string_value(&projection.secondary_source),
        ),
        (
            "target",
            projection
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
        ("validation", string_value(&projection.validation)),
    ])
}

fn optional(value: &Option<String>) -> TransportValue {
    value
        .as_deref()
        .map(string_value)
        .unwrap_or(TransportValue::Null(()))
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
