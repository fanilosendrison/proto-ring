use std::collections::BTreeMap;

use proto_ring_engine::structured_data::StructuredValue as EngineValue;

use crate::{MappingEntries, MappingEntry, ModelError, SignedDecimal, StructuredValue};

pub(super) fn structured_value(value: &EngineValue) -> Result<StructuredValue, ModelError> {
    Ok(match value {
        EngineValue::Null => StructuredValue::Null {},
        EngineValue::Boolean(value) => StructuredValue::Boolean { value: *value },
        EngineValue::Integer(decimal) => StructuredValue::Integer {
            decimal: SignedDecimal::try_from(decimal.as_str())?,
        },
        EngineValue::String(value) => StructuredValue::String {
            value: value.clone(),
        },
        EngineValue::Sequence(items) => StructuredValue::Sequence {
            items: items
                .iter()
                .map(structured_value)
                .collect::<Result<_, _>>()?,
        },
        EngineValue::Mapping(entries) => return structured_mapping(entries),
    })
}

pub(super) fn structured_mapping(
    entries: &BTreeMap<String, EngineValue>,
) -> Result<StructuredValue, ModelError> {
    let entries = entries
        .iter()
        .map(|(key, value)| {
            Ok(MappingEntry {
                key: key.clone(),
                value: structured_value(value)?,
            })
        })
        .collect::<Result<Vec<_>, ModelError>>()?;
    Ok(StructuredValue::Mapping {
        entries: MappingEntries::try_from(entries)?,
    })
}
