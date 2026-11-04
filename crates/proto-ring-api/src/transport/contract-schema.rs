use super::failure::internal;
use crate::{ApiError, Response};
use serde_json::{Map, Value};
use std::{collections::BTreeSet, sync::OnceLock};

pub(super) fn canonical() -> Result<&'static Value, ApiError> {
    static API: OnceLock<Result<Value, ApiError>> = OnceLock::new();
    API.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../docs/api/v1/schemas/api.schema.json"
        ))
        .map_err(|_| internal("embedded canonical schema is invalid"))
    })
    .as_ref()
    .map_err(Clone::clone)
}

pub(super) fn bundle(fragment: &Value) -> Result<Value, ApiError> {
    let api = canonical()?;
    let mut needed = BTreeSet::new();
    visit(fragment, &api["$defs"], &mut needed)?;
    let mut result = fragment
        .as_object()
        .cloned()
        .ok_or_else(|| internal("invalid schema fragment"))?;
    result.insert("$schema".into(), api["$schema"].clone());
    let definitions: Map<_, _> = needed
        .into_iter()
        .map(|name| {
            let value = api["$defs"][&name].clone();
            (name, value)
        })
        .collect();
    result.insert("$defs".into(), Value::Object(definitions));
    Ok(Value::Object(result))
}

fn visit(
    value: &Value,
    definitions: &Value,
    needed: &mut BTreeSet<String>,
) -> Result<(), ApiError> {
    match value {
        Value::Object(map) => {
            if let Some(reference) = map.get("$ref") {
                let name = reference
                    .as_str()
                    .and_then(|r| r.strip_prefix("#/$defs/"))
                    .filter(|name| !name.contains('/') && definitions.get(*name).is_some())
                    .ok_or_else(|| internal("nonlocal or unknown canonical schema reference"))?;
                if needed.insert(name.into()) {
                    visit(&definitions[name], definitions, needed)?;
                }
            }
            for child in map.values() {
                visit(child, definitions, needed)?;
            }
        }
        Value::Array(items) => {
            for child in items {
                visit(child, definitions, needed)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn response_validator() -> Result<&'static jsonschema::Validator, ApiError> {
    static VALIDATOR: OnceLock<Result<jsonschema::Validator, ApiError>> = OnceLock::new();
    VALIDATOR
        .get_or_init(|| {
            let schema = bundle(&canonical()?["$defs"]["response"])?;
            // bundle rejects every nonlocal reference; retrieval is never required.
            jsonschema::validator_for(&schema)
                .map_err(|_| internal("cannot compile embedded response schema"))
        })
        .as_ref()
        .map_err(Clone::clone)
}

pub(super) fn valid_request(bytes: &[u8]) -> Result<bool, ApiError> {
    static VALIDATOR: OnceLock<Result<jsonschema::Validator, ApiError>> = OnceLock::new();
    let validator = VALIDATOR
        .get_or_init(|| {
            let schema = bundle(&canonical()?["$defs"]["request"])?;
            jsonschema::validator_for(&schema)
                .map_err(|_| internal("cannot compile embedded request schema"))
        })
        .as_ref()
        .map_err(Clone::clone)?;
    // Bounded wire-syntax validation only, after duplicate rejection. Governance
    // declarations and the dispatched request still use their shared typed DTOs.
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| internal("validated request is not JSON"))?;
    Ok(validator.is_valid(&value))
}

pub(super) fn valid_response(response: &Response) -> Result<bool, ApiError> {
    // The encoder establishes the byte budget first. Convert that exact typed
    // candidate, not its bytes: recursive tags can exceed a parser's depth limit.
    let value = serde_json::to_value(response)
        .map_err(|_| internal("response cannot be represented as JSON"))?;
    Ok(response_validator()?.is_valid(&value))
}
