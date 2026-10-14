use std::collections::BTreeMap;

use jsonschema::draft202012;
use serde_json::{Map, Number, Value};

use super::{AdrMetadataError, failure};
use crate::structured_data::StructuredValue;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SchemaLabel {
    Base,
    Overlay,
}

impl SchemaLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Overlay => "overlay",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaValidationError {
    pub schema: SchemaLabel,
    pub path: String,
}

pub fn validate_schemas(
    value: &StructuredValue,
    base_schema: &StructuredValue,
    overlay_schema: &StructuredValue,
) -> Result<Vec<SchemaValidationError>, AdrMetadataError> {
    let instance = to_json(value)?;
    let base = to_json(base_schema)?;
    let overlay = to_json(overlay_schema)?;
    let mut errors = validate_one(SchemaLabel::Base, &base, &instance);
    errors.extend(validate_one(SchemaLabel::Overlay, &overlay, &instance));
    errors.sort_by(|left, right| {
        left.schema
            .cmp(&right.schema)
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(errors)
}

pub(crate) fn from_json(value: &Value) -> Result<StructuredValue, AdrMetadataError> {
    Ok(match value {
        Value::Null => StructuredValue::Null,
        Value::Bool(value) => StructuredValue::Boolean(*value),
        Value::Number(value) => {
            let text = value.to_string();
            if !is_canonical_integer(&text) {
                return Err(failure(
                    "JSON number is outside the structured integer domain",
                ));
            }
            StructuredValue::Integer(text)
        }
        Value::String(value) => StructuredValue::String(value.clone()),
        Value::Array(values) => StructuredValue::Sequence(
            values
                .iter()
                .map(from_json)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Object(values) => StructuredValue::Mapping(
            values
                .iter()
                .map(|(name, value)| Ok((name.clone(), from_json(value)?)))
                .collect::<Result<BTreeMap<_, _>, AdrMetadataError>>()?,
        ),
    })
}

fn validate_one(
    label: SchemaLabel,
    schema: &Value,
    instance: &Value,
) -> Vec<SchemaValidationError> {
    if draft202012::meta::validate(schema).is_err() {
        return vec![schema_error(label, "$".to_owned())];
    }
    let validator = match draft202012::options()
        .should_validate_formats(true)
        .offline()
        .build(schema)
    {
        Ok(validator) => validator,
        Err(_) => return vec![schema_error(label, "$".to_owned())],
    };
    validator
        .iter_errors(instance)
        .map(|error| schema_error(label, json_path(error.instance_path().as_str())))
        .collect()
}

fn schema_error(schema: SchemaLabel, path: String) -> SchemaValidationError {
    SchemaValidationError { schema, path }
}

fn to_json(value: &StructuredValue) -> Result<Value, AdrMetadataError> {
    Ok(match value {
        StructuredValue::Null => Value::Null,
        StructuredValue::Boolean(value) => Value::Bool(*value),
        StructuredValue::String(value) => Value::String(value.clone()),
        StructuredValue::Integer(value) => Value::Number(parse_integer(value)?),
        StructuredValue::Sequence(values) => {
            Value::Array(values.iter().map(to_json).collect::<Result<Vec<_>, _>>()?)
        }
        StructuredValue::Mapping(values) => Value::Object(
            values
                .iter()
                .map(|(name, value)| Ok((name.clone(), to_json(value)?)))
                .collect::<Result<Map<_, _>, AdrMetadataError>>()?,
        ),
    })
}

fn parse_integer(value: &str) -> Result<Number, AdrMetadataError> {
    let parsed: Value =
        serde_json::from_str(value).map_err(|_| failure("structured integer is not valid JSON"))?;
    match parsed {
        Value::Number(number) if is_canonical_integer(value) => Ok(number),
        _ => Err(failure("structured integer is not canonical")),
    }
}

fn is_canonical_integer(value: &str) -> bool {
    let bytes = value.as_bytes();
    match bytes {
        [b'0'] => true,
        [b'-', first, rest @ ..] => {
            matches!(first, b'1'..=b'9') && rest.iter().all(u8::is_ascii_digit)
        }
        [first, rest @ ..] => matches!(first, b'1'..=b'9') && rest.iter().all(u8::is_ascii_digit),
        _ => false,
    }
}

fn json_path(pointer: &str) -> String {
    if pointer.is_empty() {
        return "$".to_owned();
    }
    let mut path = "$".to_owned();
    for encoded in pointer.trim_start_matches('/').split('/') {
        let segment = encoded.replace("~1", "/").replace("~0", "~");
        if !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_digit()) {
            path.push('[');
            path.push_str(&segment);
            path.push(']');
        } else if segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && !segment.is_empty()
        {
            path.push('.');
            path.push_str(&segment);
        } else {
            path.push_str("['");
            path.push_str(&segment.replace('\\', "\\\\").replace('\'', "\\'"));
            path.push_str("']");
        }
    }
    path
}
