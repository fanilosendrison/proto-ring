#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use proto_ring_engine::governance_authority::{
    self, GovernanceAuthorityProfile, ResponsibilityLookup, RoleLookup,
};
use proto_ring_engine::governance_routing::ResolvedGovernanceRoute;
use proto_ring_engine::governed_objects::{self, GovernedObjectCatalog};

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, RecordEntry, TransportValue};

use super::transport::{
    exact_record, fixture_failure, record, rejection, result, string_value, unique_dynamic_record,
};

mod support;

use support::support_authority;

type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;

enum AuthorityQuery<'a> {
    None,
    RoleOf {
        responsibility: &'a str,
        source: &'a str,
    },
    AuthoritySources {
        responsibility: &'a str,
    },
    Outranks {
        responsibility: &'a str,
        left: &'a str,
        right: &'a str,
    },
    SourceBinding {
        source: &'a str,
    },
}

enum ObjectsQuery<'a> {
    Responsibilities { interface: &'a str, object: &'a str },
    Relations { interface: &'a str, object: &'a str },
}
pub(super) fn execute(
    responsibility: &str,
    fixture: &MaterializedFixture,
) -> Result<Observation, fixture::FixtureError> {
    match responsibility {
        "governance-authority.profile" => authority(fixture),
        "governed-objects.catalog" => objects(fixture),
        _ => unreachable!("authority/object responsibility inventory is closed"),
    }
}
fn authority(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    let invocation = exact_record(
        fixture.arguments(),
        "Governance Authority invocation",
        &["operation", "path", "query"],
    )?;
    require_string(&invocation, "operation", "load_authority")?;
    let path = string_field(&invocation, "path")?;
    let query = parse_authority_query(field(&invocation, "query"))?;
    let root = repository_root(fixture)?;
    let route = profile_route(root, path, "governance_authority");
    Ok(match governance_authority::load(root, &route) {
        Ok(profile) => result(authority_transport(root, &profile, query)?),
        Err(_) => rejection("governance-authority.profile"),
    })
}
fn parse_authority_query(
    value: &TransportValue,
) -> Result<AuthorityQuery<'_>, fixture::FixtureError> {
    if matches!(value, TransportValue::Null(())) {
        return Ok(AuthorityQuery::None);
    }
    let operation = record_operation(value, "Governance Authority query")?;
    let (fields, query) = match operation {
        "role_of" => {
            let fields = exact_record(
                value,
                "role_of query",
                &["operation", "responsibility", "source"],
            )?;
            let query = AuthorityQuery::RoleOf {
                responsibility: string_field(&fields, "responsibility")?,
                source: string_field(&fields, "source")?,
            };
            (fields, query)
        }
        "authority_sources" => {
            let fields = exact_record(
                value,
                "authority_sources query",
                &["operation", "responsibility"],
            )?;
            let query = AuthorityQuery::AuthoritySources {
                responsibility: string_field(&fields, "responsibility")?,
            };
            (fields, query)
        }
        "outranks" => {
            let fields = exact_record(
                value,
                "outranks query",
                &["left", "operation", "responsibility", "right"],
            )?;
            let query = AuthorityQuery::Outranks {
                responsibility: string_field(&fields, "responsibility")?,
                left: string_field(&fields, "left")?,
                right: string_field(&fields, "right")?,
            };
            (fields, query)
        }
        "source_binding" => {
            let fields = exact_record(value, "source_binding query", &["operation", "source"])?;
            let query = AuthorityQuery::SourceBinding {
                source: string_field(&fields, "source")?,
            };
            (fields, query)
        }
        _ => return Err(fixture_failure("unsupported authority query operation")),
    };
    require_string(&fields, "operation", operation)?;
    Ok(query)
}
fn authority_transport(
    root: &Path,
    profile: &GovernanceAuthorityProfile,
    query: AuthorityQuery<'_>,
) -> Result<TransportValue, fixture::FixtureError> {
    let query_result = match query {
        AuthorityQuery::None => TransportValue::Null(()),
        AuthorityQuery::RoleOf {
            responsibility,
            source,
        } => {
            let value = match governance_authority::role_of(profile, responsibility, source) {
                RoleLookup::Role(role) => role.serialized(),
                RoleLookup::UnknownResponsibility => "unknown_responsibility",
                RoleLookup::Undeclared => "undeclared",
            };
            string_value(value)
        }
        AuthorityQuery::AuthoritySources { responsibility } => {
            match governance_authority::authority_sources(profile, responsibility) {
                ResponsibilityLookup::Value(values) => strings(values.iter()),
                ResponsibilityLookup::UnknownResponsibility => {
                    string_value("unknown_responsibility")
                }
            }
        }
        AuthorityQuery::Outranks {
            responsibility,
            left,
            right,
        } => match governance_authority::outranks(profile, responsibility, left, right) {
            ResponsibilityLookup::Value(value) => TransportValue::Boolean(value),
            ResponsibilityLookup::UnknownResponsibility => string_value("unknown_responsibility"),
        },
        AuthorityQuery::SourceBinding { source } => source_binding(root, profile, source)?,
    };
    Ok(record(vec![
        (
            "model_version",
            TransportValue::Integer(profile.model_version.to_string()),
        ),
        ("query_result", query_result),
        ("responsibilities", strings(profile.responsibilities.keys())),
        ("sources", strings(profile.sources.keys())),
    ]))
}
fn source_binding(
    root: &Path,
    profile: &GovernanceAuthorityProfile,
    source_id: &str,
) -> Result<TransportValue, fixture::FixtureError> {
    let binding = profile
        .sources
        .get(source_id)
        .ok_or_else(|| fixture_failure("query source is absent"))?
        .repository_target
        .as_ref();
    let Some(binding) = binding else {
        return Ok(TransportValue::Null(()));
    };
    let canonical_root = root
        .canonicalize()
        .map_err(|error| fixture_failure(error.to_string()))?;
    let relative = binding
        .target
        .strip_prefix(canonical_root)
        .map_err(|error| fixture_failure(error.to_string()))?;
    let resolved_target = relative
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .ok_or_else(|| fixture_failure("non-UTF-8 source binding"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("/");
    Ok(record(vec![
        ("declared_path", string_value(&binding.declared_path)),
        ("resolved_target", string_value(&resolved_target)),
        ("route", strings(binding.route.iter())),
    ]))
}
fn objects(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    let entries = unique_dynamic_record(fixture.arguments(), "Governed Objects invocation")?;
    let has_query = entries.iter().any(|(name, _)| *name == "query");
    let expected = if has_query {
        &[
            "foreign_authority",
            "operation",
            "path",
            "query",
            "semantic_inputs",
        ][..]
    } else {
        &["foreign_authority", "operation", "path", "semantic_inputs"][..]
    };
    let invocation = exact_record(fixture.arguments(), "Governed Objects invocation", expected)?;
    require_string(&invocation, "operation", "load_objects")?;
    let foreign_authority = boolean_field(&invocation, "foreign_authority")?;
    let semantic_inputs = exact_record(
        field(&invocation, "semantic_inputs"),
        "Governed Objects semantic_inputs",
        &["authority", "bindings", "catalog", "integrity_profile"],
    )?;
    let root = repository_root(fixture)?;
    let mut authority = support_authority(root, field(&semantic_inputs, "authority"))?;
    if foreign_authority {
        authority.repository = root
            .parent()
            .ok_or_else(|| fixture_failure("repository root has no parent"))?
            .join("foreign");
    }
    let query = invocation
        .get("query")
        .copied()
        .map(parse_objects_query)
        .transpose()?;
    let route = profile_route(root, string_field(&invocation, "path")?, "governed_objects");
    Ok(match governed_objects::load(root, &route, &authority) {
        Ok(catalog) => result(objects_transport(&catalog, query)?),
        Err(_) => rejection("governed-objects.catalog"),
    })
}
fn parse_objects_query(value: &TransportValue) -> Result<ObjectsQuery<'_>, fixture::FixtureError> {
    let operation = record_operation(value, "Governed Objects query")?;
    let fields = exact_record(
        value,
        "Governed Objects query",
        &["interface", "object", "operation"],
    )?;
    require_string(&fields, "operation", operation)?;
    let interface = string_field(&fields, "interface")?;
    let object = string_field(&fields, "object")?;
    match operation {
        "object_responsibilities" => Ok(ObjectsQuery::Responsibilities { interface, object }),
        "object_relations" => Ok(ObjectsQuery::Relations { interface, object }),
        _ => Err(fixture_failure(
            "unsupported governed-object query operation",
        )),
    }
}
fn objects_transport(
    catalog: &GovernedObjectCatalog,
    query: Option<ObjectsQuery<'_>>,
) -> Result<TransportValue, fixture::FixtureError> {
    let interfaces = TransportValue::Record(
        catalog
            .interfaces
            .iter()
            .map(|(interface_id, interface)| RecordEntry {
                name: interface_id.clone(),
                value: strings(interface.objects.keys()),
            })
            .collect(),
    );
    let mut entries = vec![
        ("interfaces", interfaces),
        (
            "model_version",
            TransportValue::Integer(catalog.model_version.to_string()),
        ),
    ];
    if let Some(query) = query {
        let (interface_id, object_id, relations) = match query {
            ObjectsQuery::Responsibilities { interface, object } => (interface, object, false),
            ObjectsQuery::Relations { interface, object } => (interface, object, true),
        };
        let governed_object = catalog
            .interfaces
            .get(interface_id)
            .and_then(|interface| interface.objects.get(object_id))
            .ok_or_else(|| fixture_failure("query object is absent"))?;
        let value = if relations {
            TransportValue::Sequence(
                governed_object
                    .relations
                    .iter()
                    .map(|relation| {
                        record(vec![
                            ("relation", string_value(&relation.relation)),
                            ("responsibility", string_value(&relation.responsibility)),
                            (
                                "target_interface",
                                string_value(&relation.target.interface_id),
                            ),
                            ("target_object", string_value(&relation.target.object_id)),
                        ])
                    })
                    .collect(),
            )
        } else {
            strings(governed_object.responsibilities.iter())
        };
        entries.push(("query_result", value));
    }
    Ok(record(entries))
}
fn profile_route(root: &Path, path: &str, responsibility: &str) -> ResolvedGovernanceRoute {
    ResolvedGovernanceRoute {
        route: vec![
            "capabilities".to_owned(),
            responsibility.to_owned(),
            "routes".to_owned(),
            "profile".to_owned(),
        ],
        declared_path: path.to_owned(),
        target: root.join(path),
    }
}
fn record_operation<'a>(
    value: &'a TransportValue,
    label: &str,
) -> Result<&'a str, fixture::FixtureError> {
    let entries = unique_dynamic_record(value, label)?;
    let operation = entries
        .iter()
        .find(|(name, _)| *name == "operation")
        .ok_or_else(|| fixture_failure(format!("{label} is missing field: operation")))?;
    fixture::string(operation.1)
}
fn field<'a>(fields: &Fields<'a>, name: &str) -> &'a TransportValue {
    fields
        .get(name)
        .copied()
        .expect("exact record contains required field")
}
fn string_field<'a>(fields: &Fields<'a>, name: &str) -> Result<&'a str, fixture::FixtureError> {
    fixture::string(field(fields, name))
}
fn boolean_field(fields: &Fields<'_>, name: &str) -> Result<bool, fixture::FixtureError> {
    match field(fields, name) {
        TransportValue::Boolean(value) => Ok(*value),
        _ => Err(fixture_failure(format!("field {name} must be boolean"))),
    }
}
fn require_string(
    fields: &Fields<'_>,
    name: &str,
    expected: &str,
) -> Result<(), fixture::FixtureError> {
    if string_field(fields, name)? == expected {
        Ok(())
    } else {
        Err(fixture_failure(format!("unsupported {name}")))
    }
}
fn require_nonempty(value: &str, label: &str) -> Result<(), fixture::FixtureError> {
    if value.is_empty() {
        Err(fixture_failure(format!("{label} must be nonempty")))
    } else {
        Ok(())
    }
}
fn strings<'a>(values: impl IntoIterator<Item = &'a String>) -> TransportValue {
    TransportValue::Sequence(
        values
            .into_iter()
            .map(|value| string_value(value))
            .collect(),
    )
}

fn repository_root(fixture: &MaterializedFixture) -> Result<&Path, fixture::FixtureError> {
    fixture
        .root()
        .ok_or_else(|| fixture_failure("responsibility requires repository"))
}

#[cfg(test)]
mod tests;
