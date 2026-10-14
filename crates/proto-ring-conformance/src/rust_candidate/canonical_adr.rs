#![forbid(unsafe_code)]

use std::path::{Component, Path};

use proto_ring_engine::canonical_adr;

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, TransportValue};

use super::transport::{
    fixture_failure, record, rejection, result, string_value, unique_dynamic_record,
};

const RESPONSIBILITY: &str = "canonical-adr.resolve";

pub(super) fn execute(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    let entries = unique_dynamic_record(fixture.arguments(), "Canonical ADR invocation")?;
    for (name, _) in &entries {
        if !["adr_id", "include_decision_body", "operation"].contains(name) {
            return Err(fixture_failure(format!(
                "Canonical ADR invocation has unknown field: {name}"
            )));
        }
    }
    let operation = required(&entries, "operation")?;
    if fixture::string(operation)? != "resolve_adr" {
        return Err(fixture_failure("unsupported Canonical ADR operation"));
    }
    let Some(adr_id) = entries
        .iter()
        .find(|(name, _)| *name == "adr_id")
        .map(|(_, value)| *value)
    else {
        return Err(fixture_failure(
            "Canonical ADR invocation is missing adr_id",
        ));
    };
    let TransportValue::String(adr_id) = adr_id else {
        return Ok(rejection(RESPONSIBILITY));
    };
    let include_body = match entries
        .iter()
        .find(|(name, _)| *name == "include_decision_body")
        .map(|(_, value)| *value)
    {
        None => false,
        Some(TransportValue::Boolean(value)) => *value,
        Some(_) => {
            return Err(fixture_failure("include_decision_body must be a boolean"));
        }
    };
    let root = fixture
        .root()
        .ok_or_else(|| fixture_failure("Canonical ADR resolution requires repository"))?;
    Ok(match canonical_adr::resolve(root, adr_id) {
        Ok(resolved) => {
            let relative = portable_path(&resolved.relative_path)?;
            let entries = if include_body {
                vec![
                    ("adr_id", string_value(&resolved.adr_id)),
                    (
                        "decision_body",
                        TransportValue::Bytes(hex_encode(&resolved.decision_body)),
                    ),
                    ("relative_path", string_value(&relative)),
                ]
            } else {
                vec![
                    ("adr_id", string_value(&resolved.adr_id)),
                    ("relative_path", string_value(&relative)),
                ]
            };
            result(record(entries))
        }
        Err(_) => rejection(RESPONSIBILITY),
    })
}

fn required<'a>(
    entries: &[(&'a str, &'a TransportValue)],
    name: &str,
) -> Result<&'a TransportValue, fixture::FixtureError> {
    entries
        .iter()
        .find(|(actual, _)| *actual == name)
        .map(|(_, value)| *value)
        .ok_or_else(|| fixture_failure(format!("Canonical ADR invocation is missing {name}")))
}

fn portable_path(path: &Path) -> Result<String, fixture::FixtureError> {
    path.components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| fixture_failure("Canonical ADR path is not UTF-8")),
            _ => Err(fixture_failure("Canonical ADR path is not relative")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join("/"))
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
