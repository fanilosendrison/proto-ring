#![forbid(unsafe_code)]

use std::path::{Component, Path};

use proto_ring_engine::adr_metadata::{self, RelationTargetError, SchemaValidationError};
use proto_ring_engine::structured_data::StructuredValue;

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, TransportValue};

use super::transport::{
    exact_record, fixture_failure, record, rejection, result, string_value, structured_transport,
    transport_to_structured,
};

pub(super) fn execute(
    responsibility: &str,
    fixture: &MaterializedFixture,
) -> Result<Observation, fixture::FixtureError> {
    match responsibility {
        "adr-metadata.parse" => parse(fixture.arguments()),
        "adr-metadata.repository-path" => repository_path(fixture),
        "adr-metadata.validation" => validation(fixture.arguments()),
        _ => unreachable!("ADR metadata responsibility inventory is closed"),
    }
}

fn parse(arguments: &TransportValue) -> Result<Observation, fixture::FixtureError> {
    const RESPONSIBILITY: &str = "adr-metadata.parse";
    let fields = exact_record(
        arguments,
        "ADR metadata parse invocation",
        &["bytes_hex", "operation"],
    )?;
    let operation = fixture::string(fields["operation"])?;
    let bytes = hex_decode(fixture::string(fields["bytes_hex"])?)?;
    let observation = match operation {
        "load_yaml" => adr_metadata::load_yaml(&bytes).map(|value| structured_transport(&value)),
        "load_json" => adr_metadata::load_json(&bytes).map(|value| structured_transport(&value)),
        "parse_adr" => adr_metadata::parse_adr(&bytes).map(|parsed| {
            result_record(vec![
                ("body_hex", string_value(&hex_encode(&parsed.decision_body))),
                (
                    "metadata",
                    structured_transport(&StructuredValue::Mapping(parsed.metadata)),
                ),
            ])
        }),
        "decision_body" => adr_metadata::decision_body(&bytes)
            .map(|body| result_record(vec![("body_hex", string_value(&hex_encode(&body)))])),
        "h1" => adr_metadata::h1_text(&bytes)
            .map(|text| result_record(vec![("text", string_value(&text))])),
        "preserved_payload" => adr_metadata::preserved_payload(&bytes).map(|payload| {
            result_record(vec![("payload_hex", string_value(&hex_encode(&payload)))])
        }),
        "sha256" => Ok(result_record(vec![(
            "sha256",
            string_value(&adr_metadata::sha256(&bytes)),
        )])),
        _ => return Err(fixture_failure("unsupported ADR metadata parse operation")),
    };
    Ok(match observation {
        Ok(value) => result(value),
        Err(_) => rejection(RESPONSIBILITY),
    })
}

fn repository_path(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    const RESPONSIBILITY: &str = "adr-metadata.repository-path";
    let fields = exact_record(
        fixture.arguments(),
        "ADR metadata repository-path invocation",
        &["operation", "relative"],
    )?;
    if fixture::string(fields["operation"])? != "repository_path" {
        return Err(fixture_failure("unsupported repository-path operation"));
    }
    let TransportValue::String(relative) = fields["relative"] else {
        return Ok(rejection(RESPONSIBILITY));
    };
    let root = fixture
        .root()
        .ok_or_else(|| fixture_failure("repository-path operation requires repository"))?;
    Ok(
        match adr_metadata::resolve_repository_path(root, relative) {
            Ok(path) => {
                let canonical_root = root
                    .canonicalize()
                    .map_err(|error| fixture_failure(error.to_string()))?;
                let relative = path
                    .strip_prefix(canonical_root)
                    .map_err(|error| fixture_failure(error.to_string()))?;
                result(result_record(vec![(
                    "relative",
                    string_value(&portable_path(relative)?),
                )]))
            }
            Err(_) => rejection(RESPONSIBILITY),
        },
    )
}

fn validation(arguments: &TransportValue) -> Result<Observation, fixture::FixtureError> {
    const RESPONSIBILITY: &str = "adr-metadata.validation";
    let operation = fixture::string(fixture::record_field(arguments, "operation")?)?;
    let outcome = match operation {
        "require_mapping" | "require_string" | "require_string_list" => {
            let fields = exact_record(
                arguments,
                "ADR metadata validation invocation",
                &["operation", "value"],
            )?;
            requirement(operation, fields["value"])
        }
        "relation_target_categories" | "relation_target_errors" => {
            let fields = exact_record(
                arguments,
                "ADR relation invocation",
                &[
                    "known_ids",
                    "operation",
                    "relation_type",
                    "source_id",
                    "targets",
                ],
            )?;
            relations(operation, &fields)
        }
        "schema_errors" | "schema_error_labels" | "schema_error_locations" => {
            let fields = exact_record(
                arguments,
                "ADR schema invocation",
                &["operation", "overlay_schema", "schema", "value"],
            )?;
            schemas(operation, &fields)
        }
        _ => {
            return Err(fixture_failure(
                "unsupported ADR metadata validation operation",
            ));
        }
    };
    Ok(outcome.unwrap_or_else(|_| rejection(RESPONSIBILITY)))
}

fn requirement(operation: &str, value: &TransportValue) -> Result<Observation, ()> {
    let structured = transport_to_structured(value).map_err(|_| ())?;
    let projected = match operation {
        "require_mapping" => structured_transport(&StructuredValue::Mapping(
            adr_metadata::require_mapping(&structured)
                .map_err(|_| ())?
                .clone(),
        )),
        "require_string" => result_record(vec![(
            "text",
            string_value(adr_metadata::require_string(&structured).map_err(|_| ())?),
        )]),
        "require_string_list" => result_record(vec![(
            "items",
            TransportValue::Sequence(
                adr_metadata::require_string_list(&structured)
                    .map_err(|_| ())?
                    .iter()
                    .map(|value| string_value(value))
                    .collect(),
            ),
        )]),
        _ => unreachable!("requirement operation checked"),
    };
    Ok(result(projected))
}

fn relations(
    operation: &str,
    fields: &std::collections::BTreeMap<&str, &TransportValue>,
) -> Result<Observation, ()> {
    let known = structured_strings(fields["known_ids"])?;
    let targets = structured_strings(fields["targets"])?;
    let source = structured_string(fields["source_id"])?;
    let relation_type = structured_string(fields["relation_type"])?;
    let errors = adr_metadata::relation_target_errors(source, relation_type, &targets, &known);
    let values = errors.iter().map(relation_error_transport).collect();
    let _ = operation;
    Ok(result(result_record(vec![(
        "errors",
        TransportValue::Sequence(values),
    )])))
}

fn schemas(
    operation: &str,
    fields: &std::collections::BTreeMap<&str, &TransportValue>,
) -> Result<Observation, ()> {
    let value = structured_value(fields["value"])?;
    let base = structured_value(fields["schema"])?;
    let overlay = structured_value(fields["overlay_schema"])?;
    let errors = adr_metadata::validate_schemas(&value, &base, &overlay).map_err(|_| ())?;
    let projected = match operation {
        "schema_error_labels" => result_record(vec![
            (
                "base_has_error",
                TransportValue::Boolean(has_schema(&errors, "base")),
            ),
            (
                "overlay_has_error",
                TransportValue::Boolean(has_schema(&errors, "overlay")),
            ),
        ]),
        "schema_errors" => result_record(vec![(
            "errors",
            TransportValue::Sequence(errors.iter().map(schema_error_transport).collect()),
        )]),
        "schema_error_locations" => result_record(vec![
            (
                "errors",
                TransportValue::Sequence(errors.iter().map(schema_error_transport).collect()),
            ),
            ("rejected", TransportValue::Boolean(!errors.is_empty())),
        ]),
        _ => unreachable!("schema operation checked"),
    };
    Ok(result(projected))
}

fn structured_value(value: &TransportValue) -> Result<StructuredValue, ()> {
    transport_to_structured(value).map_err(|_| ())
}

fn structured_string(value: &TransportValue) -> Result<&str, ()> {
    let TransportValue::String(value) = value else {
        return Err(());
    };
    if value.is_empty() {
        return Err(());
    }
    Ok(value)
}

fn structured_strings(value: &TransportValue) -> Result<Vec<String>, ()> {
    adr_metadata::require_string_list(&structured_value(value)?).map_err(|_| ())
}

fn relation_error_transport(error: &RelationTargetError) -> TransportValue {
    match error {
        RelationTargetError::SelfReference => {
            result_record(vec![("category", string_value("self_reference"))])
        }
        RelationTargetError::MissingTarget(target) => result_record(vec![
            ("category", string_value("missing_target")),
            ("target", string_value(target)),
        ]),
    }
}

fn schema_error_transport(error: &SchemaValidationError) -> TransportValue {
    result_record(vec![
        ("path", string_value(&error.path)),
        ("schema", string_value(error.schema.as_str())),
    ])
}

fn has_schema(errors: &[SchemaValidationError], label: &str) -> bool {
    errors.iter().any(|error| error.schema.as_str() == label)
}

fn result_record(entries: Vec<(&str, TransportValue)>) -> TransportValue {
    record(entries)
}

fn portable_path(path: &Path) -> Result<String, fixture::FixtureError> {
    path.components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| fixture_failure("repository path is not UTF-8")),
            _ => Err(fixture_failure("repository path is not relative")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join("/"))
}

fn hex_decode(value: &str) -> Result<Vec<u8>, fixture::FixtureError> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(fixture_failure("hex input has odd length"));
    }
    pairs
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|_| fixture_failure("invalid hex"))?;
            u8::from_str_radix(text, 16).map_err(|_| fixture_failure("invalid hex"))
        })
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}
