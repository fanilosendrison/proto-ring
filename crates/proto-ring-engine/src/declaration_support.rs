use std::collections::BTreeMap;

use crate::structured_data::StructuredValue;

pub(crate) type DeclarationResult<T> = Result<T, String>;

pub(crate) fn mapping<'a>(
    value: &'a StructuredValue,
    label: &str,
) -> DeclarationResult<&'a BTreeMap<String, StructuredValue>> {
    match value {
        StructuredValue::Mapping(value) => Ok(value),
        _ => Err(format!("{label} must be a mapping")),
    }
}

pub(crate) fn required_mapping<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> DeclarationResult<&'a BTreeMap<String, StructuredValue>> {
    mapping
        .get(key)
        .ok_or_else(|| format!("missing {key}"))
        .and_then(|value| self::mapping(value, key))
}

pub(crate) fn exact_keys(
    mapping: &BTreeMap<String, StructuredValue>,
    keys: &[&str],
) -> DeclarationResult<()> {
    if mapping.len() == keys.len() && keys.iter().all(|key| mapping.contains_key(*key)) {
        Ok(())
    } else {
        Err("mapping has missing or unknown keys".to_owned())
    }
}

pub(crate) fn string<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> DeclarationResult<&'a str> {
    match mapping.get(key) {
        Some(StructuredValue::String(value)) if !value.is_empty() => Ok(value),
        _ => Err(format!("{key} must be a nonempty string")),
    }
}

pub(crate) fn sequence<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> DeclarationResult<&'a [StructuredValue]> {
    match mapping.get(key) {
        Some(StructuredValue::Sequence(values)) => Ok(values),
        _ => Err(format!("{key} must be a sequence")),
    }
}

pub(crate) fn boolean(
    mapping: &BTreeMap<String, StructuredValue>,
    key: &str,
) -> DeclarationResult<bool> {
    match mapping.get(key) {
        Some(StructuredValue::Boolean(value)) => Ok(*value),
        _ => Err(format!("{key} must be boolean")),
    }
}

pub(crate) fn model_version_one(
    mapping: &BTreeMap<String, StructuredValue>,
) -> DeclarationResult<()> {
    match mapping.get("model_version") {
        Some(StructuredValue::Integer(value)) if value == "1" => Ok(()),
        _ => Err("model_version must be integer 1".to_owned()),
    }
}
