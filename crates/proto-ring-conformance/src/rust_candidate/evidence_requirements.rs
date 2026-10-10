use std::collections::BTreeMap;

use proto_ring_engine::evidence_requirements::{
    self, EvidenceRequirementRegistry, PersistentEvidenceRequirement,
};
use proto_ring_engine::exact_evidence_binding::{EvidenceBinding, EvidenceRequirement};
use proto_ring_engine::governance_routing::ResolvedGovernanceRoute;

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, TransportValue};

use super::support::{support_authority, support_catalog};
use super::transport::{
    exact_record, fixture_failure, record, rejection, repository_snapshot, result, string_value,
    unique_dynamic_record,
};

type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;

pub(super) fn execute(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    let entries = unique_dynamic_record(fixture.arguments(), "Evidence Requirements invocation")?;
    let allowed = [
        "consumer_resolution",
        "foreign_support_repository",
        "observe_read_only",
        "observe_requirement_id",
        "operation",
        "path",
        "requirement_id",
        "semantic_inputs",
        "with_catalog",
    ];
    if entries.iter().any(|(name, _)| !allowed.contains(name)) {
        return Err(fixture_failure(
            "Evidence Requirements invocation has unknown field",
        ));
    }
    let fields: Fields<'_> = entries.into_iter().collect();
    for required in ["operation", "path", "semantic_inputs", "with_catalog"] {
        if !fields.contains_key(required) {
            return Err(fixture_failure(format!("invocation is missing {required}")));
        }
    }
    let operation = string_field(&fields, "operation")?;
    if !["load_evidence", "load_and_evaluate_unknown_context"].contains(&operation) {
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
    let authority = support_authority(root, field(&semantic, "authority"))?;
    let foreign_support = optional_string(&fields, "foreign_support_repository")?;
    if foreign_support.is_some_and(|selected| selected != "catalog") {
        return Err(fixture_failure(
            "invalid foreign_support_repository transport value",
        ));
    }
    let catalog_root = foreign_support
        .map(|_| root.join("foreign"))
        .unwrap_or_else(|| root.to_path_buf());
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
            "evidence_requirements".to_owned(),
            "routes".to_owned(),
            "registry".to_owned(),
        ],
        declared_path: path.to_owned(),
        target: root.join(path),
    };
    Ok(
        match evidence_requirements::load(
            root,
            &route,
            &authority,
            with_catalog.then_some(&catalog),
        ) {
            Ok(registry) if operation == "load_and_evaluate_unknown_context" => {
                evaluate_resolution(&registry, &fields)?
            }
            Ok(registry) => {
                let unchanged = before
                    .map(|state| repository_snapshot(root).map(|after| state == after))
                    .transpose()?;
                result(registry_transport(
                    &registry,
                    optional_string(&fields, "observe_requirement_id")?,
                    unchanged,
                )?)
            }
            Err(_) => rejection("evidence-requirements.registry"),
        },
    )
}

fn registry_transport(
    registry: &EvidenceRequirementRegistry,
    observe: Option<&str>,
    unchanged: Option<bool>,
) -> Result<TransportValue, fixture::FixtureError> {
    let mut values = vec![("authority_source", string_value(&registry.authority.source))];
    if let Some(id) = observe {
        let requirement = registry
            .requirements
            .get(id)
            .ok_or_else(|| fixture_failure("observed requirement is absent"))?;
        values.push(("loaded_requirement", requirement_transport(requirement)));
    }
    values.push((
        "model_version",
        TransportValue::Integer(registry.model_version.to_string()),
    ));
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
    if let Some(unchanged) = unchanged {
        values.push(("repository_unchanged", TransportValue::Boolean(unchanged)));
    }
    values.push(("requirements", strings(registry.requirements.keys())));
    Ok(record(values))
}

fn requirement_transport(requirement: &PersistentEvidenceRequirement) -> TransportValue {
    record(vec![
        (
            "candidate_source",
            string_value(&requirement.candidate_source),
        ),
        (
            "context",
            record(vec![
                (
                    "required",
                    TransportValue::Boolean(requirement.context.required),
                ),
                ("source", optional(&requirement.context.source)),
            ]),
        ),
        (
            "evidence_classes",
            record(vec![
                (
                    "classes",
                    if requirement.evidence_classes.source.is_some() {
                        TransportValue::Null(())
                    } else {
                        strings(requirement.evidence_classes.classes.iter())
                    },
                ),
                (
                    "kind",
                    string_value(requirement.evidence_classes.kind.serialized()),
                ),
                ("source", optional(&requirement.evidence_classes.source)),
            ]),
        ),
        ("id", string_value(&requirement.id)),
        (
            "instances",
            record(vec![
                (
                    "kind",
                    string_value(requirement.instances.kind.serialized()),
                ),
                ("source", optional(&requirement.instances.source)),
            ]),
        ),
        ("responsibility", string_value(&requirement.responsibility)),
        ("subject_source", string_value(&requirement.subject_source)),
        (
            "target",
            requirement
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

fn evaluate_resolution(
    registry: &EvidenceRequirementRegistry,
    fields: &Fields<'_>,
) -> Result<Observation, fixture::FixtureError> {
    let requirement_id = string_field(fields, "requirement_id")?;
    let persistent = registry
        .requirements
        .get(requirement_id)
        .ok_or_else(|| fixture_failure("requirement is absent"))?;
    let resolution = exact_record(
        field(fields, "consumer_resolution"),
        "consumer_resolution",
        &["candidate", "context", "subject"],
    )?;
    let subject = exact_record(
        field(&resolution, "subject"),
        "resolved subject",
        &["identity_hex", "known"],
    )?;
    let context = exact_record(
        field(&resolution, "context"),
        "resolved context",
        &["identity_hex", "known", "source_id"],
    )?;
    let candidate = exact_record(
        field(&resolution, "candidate"),
        "resolved candidate",
        &["context_hex", "evidence_class", "subject_hex"],
    )?;
    let resolved_context_source = string_field(&context, "source_id")?;
    let context_source = if persistent.context.required {
        let declared = persistent
            .context
            .source
            .as_deref()
            .ok_or_else(|| fixture_failure("required context source is absent"))?;
        if resolved_context_source != declared {
            return Err(fixture_failure(
                "resolved context source differs from persistent requirement",
            ));
        }
        declared
    } else {
        resolved_context_source
    };
    let subject_known = boolean(field(&subject, "known"), "subject known")?;
    let context_known = boolean(field(&context, "known"), "context known")?;
    let requirement = EvidenceRequirement::new(
        persistent.evidence_classes.classes.clone(),
        subject_known
            .then(|| decode_hex(field(&subject, "identity_hex")))
            .transpose()?,
        context_known
            .then(|| decode_hex(field(&context, "identity_hex")))
            .transpose()?,
        persistent.context.required,
    )
    .map_err(|_| fixture_failure("resolved requirement is structurally invalid"))?;
    let candidate = EvidenceBinding {
        evidence_class: Some(string_field(&candidate, "evidence_class")?.to_owned()),
        subject_identity: Some(decode_hex(field(&candidate, "subject_hex"))?),
        context_identity: Some(decode_hex(field(&candidate, "context_hex"))?),
    };
    Ok(result(record(vec![
        (
            "binding_status",
            string_value(requirement.evaluate(Some(&candidate)).serialized()),
        ),
        (
            "context_required",
            TransportValue::Boolean(persistent.context.required),
        ),
        ("context_source_id", string_value(context_source)),
        ("registry", string_value("ACCEPTED")),
        (
            "resolved_context_known",
            TransportValue::Boolean(context_known),
        ),
    ])))
}

fn decode_hex(value: &TransportValue) -> Result<Vec<u8>, fixture::FixtureError> {
    let value = fixture::string(value)?;
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(fixture_failure("hex has odd length"));
    }
    pairs
        .iter()
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|text| u8::from_str_radix(text, 16).ok())
                .ok_or_else(|| fixture_failure("invalid hex"))
        })
        .collect()
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
