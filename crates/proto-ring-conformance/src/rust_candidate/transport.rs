#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_engine::structured_data::StructuredValue;

use crate::fixture;
use crate::model::{Observation, ObservationKind, RecordEntry, TransportValue};

pub(super) fn exact_record<'a>(
    value: &'a TransportValue,
    label: &str,
    expected_fields: &[&str],
) -> Result<BTreeMap<&'a str, &'a TransportValue>, fixture::FixtureError> {
    let entries = unique_dynamic_record(value, label)?;
    if let Some((unknown, _)) = entries
        .iter()
        .find(|(name, _)| !expected_fields.contains(name))
    {
        return Err(fixture_failure(format!(
            "{label} has unknown field: {unknown}"
        )));
    }
    if let Some(missing) = expected_fields
        .iter()
        .find(|name| !entries.iter().any(|(actual, _)| actual == *name))
    {
        return Err(fixture_failure(format!(
            "{label} is missing field: {missing}"
        )));
    }
    Ok(entries.into_iter().collect())
}

pub(super) fn unique_dynamic_record<'a>(
    value: &'a TransportValue,
    label: &str,
) -> Result<Vec<(&'a str, &'a TransportValue)>, fixture::FixtureError> {
    let TransportValue::Record(entries) = value else {
        return Err(fixture_failure(format!("{label} must be a record")));
    };
    let mut names = BTreeSet::new();
    let mut unique = Vec::with_capacity(entries.len());
    for entry in entries {
        if !names.insert(entry.name.as_str()) {
            return Err(fixture_failure(format!(
                "{label} has duplicate field: {}",
                entry.name
            )));
        }
        unique.push((entry.name.as_str(), &entry.value));
    }
    Ok(unique)
}

pub(super) fn structured_transport(value: &StructuredValue) -> TransportValue {
    match value {
        StructuredValue::Mapping(values) => TransportValue::Record(
            values
                .iter()
                .map(|(name, value)| RecordEntry {
                    name: name.clone(),
                    value: structured_transport(value),
                })
                .collect(),
        ),
        StructuredValue::Sequence(values) => {
            TransportValue::Sequence(values.iter().map(structured_transport).collect())
        }
        StructuredValue::String(value) => string_value(value),
        StructuredValue::Boolean(value) => TransportValue::Boolean(*value),
        StructuredValue::Integer(value) => TransportValue::Integer(value.clone()),
        StructuredValue::Null => TransportValue::Null(()),
    }
}

pub(super) fn transport_to_structured(
    value: &TransportValue,
) -> Result<StructuredValue, fixture::FixtureError> {
    Ok(match value {
        TransportValue::Record(entries) => StructuredValue::Mapping(
            entries
                .iter()
                .map(|entry| Ok((entry.name.clone(), transport_to_structured(&entry.value)?)))
                .collect::<Result<BTreeMap<_, _>, fixture::FixtureError>>()?,
        ),
        TransportValue::Sequence(values) => StructuredValue::Sequence(
            values
                .iter()
                .map(transport_to_structured)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        TransportValue::String(value) => StructuredValue::String(value.clone()),
        TransportValue::Boolean(value) => StructuredValue::Boolean(*value),
        TransportValue::Integer(value) => StructuredValue::Integer(value.clone()),
        TransportValue::Null(()) => StructuredValue::Null,
        TransportValue::Bytes(_) => return Err(fixture_failure("bytes are not structured values")),
    })
}

pub(super) fn result(value: TransportValue) -> Observation {
    Observation {
        kind: ObservationKind::Result,
        value,
    }
}

pub(super) fn rejection(responsibility: &str) -> Observation {
    Observation {
        kind: ObservationKind::ControlledRejection,
        value: record(vec![(
            "category",
            string_value(&format!("{responsibility}.rejected")),
        )]),
    }
}

pub(super) fn record(entries: Vec<(&str, TransportValue)>) -> TransportValue {
    TransportValue::Record(
        entries
            .into_iter()
            .map(|(name, value)| RecordEntry {
                name: name.to_owned(),
                value,
            })
            .collect(),
    )
}

pub(super) fn string_value(value: &str) -> TransportValue {
    TransportValue::String(value.to_owned())
}

pub(super) fn repository_snapshot(
    root: &Path,
) -> Result<Vec<(PathBuf, Vec<u8>)>, fixture::FixtureError> {
    fn visit(
        root: &Path,
        current: &Path,
        output: &mut Vec<(PathBuf, Vec<u8>)>,
    ) -> Result<(), fixture::FixtureError> {
        let mut entries = fs::read_dir(current)
            .map_err(|error| fixture_failure(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| fixture_failure(error.to_string()))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            if entry
                .file_type()
                .map_err(|error| fixture_failure(error.to_string()))?
                .is_dir()
            {
                visit(root, &path, output)?;
            } else {
                output.push((
                    path.strip_prefix(root)
                        .expect("visited path is contained")
                        .to_path_buf(),
                    fs::read(path).map_err(|error| fixture_failure(error.to_string()))?,
                ));
            }
        }
        Ok(())
    }
    let mut output = Vec::new();
    visit(root, root, &mut output)?;
    Ok(output)
}

pub(super) fn fixture_failure(message: impl Into<String>) -> fixture::FixtureError {
    fixture::FixtureError::new(message)
}
