use super::super::transport::{fixture_failure, record, string_value};
use super::{Fields, optional_string, strings, validation_transport};
use crate::fixture;
use crate::model::{RecordEntry, TransportValue};
use proto_ring_engine::repository_integrity::ConsumerIntegrityProfile;
use std::collections::BTreeMap;
use std::fs::{self, Metadata};
use std::io;
use std::path::Path;

// Kept separate from the syscall so controlled errors exercise the same
// classification as real metadata results without permission-dependent tests.
pub(crate) fn sentinel_absent(
    metadata: io::Result<Metadata>,
) -> Result<bool, fixture::FixtureError> {
    match metadata {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(fixture_failure(format!(
            "execution sentinel metadata inspection failed: {error}"
        ))),
    }
}

pub(super) fn profile_transport(
    root: &Path,
    profile: &ConsumerIntegrityProfile,
    fields: &Fields<'_>,
    unchanged: Option<bool>,
    observe_execution_sentinel: bool,
) -> Result<TransportValue, fixture::FixtureError> {
    let mut values = vec![
        (
            "continue_after_non_satisfied",
            TransportValue::Boolean(profile.continue_after_non_satisfied),
        ),
        ("environments", strings(profile.environments.iter())),
    ];
    if observe_execution_sentinel {
        let absent = sentinel_absent(fs::symlink_metadata(root.join("execution-sentinel.txt")))?;
        values.push(("execution_sentinel_absent", TransportValue::Boolean(absent)));
    }
    values.push((
        "identity_present",
        TransportValue::Boolean(!profile.identity.0.is_empty()),
    ));
    if let Some(id) = optional_string(fields, "observe_validation_id")? {
        let validation = profile
            .validations
            .get(id)
            .ok_or_else(|| fixture_failure("observed validation is absent"))?;
        values.push(("loaded_validation", validation_transport(validation)));
    }
    if let Some(ids) = fields.get("observe_validation_ids") {
        let loaded = fixture::sequence(ids)?
            .iter()
            .map(|id| {
                let id = fixture::string(id)?;
                profile
                    .validations
                    .get(id)
                    .map(|validation| (id.to_owned(), validation_transport(validation)))
                    .ok_or_else(|| fixture_failure("observed validation is absent"))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        values.push((
            "loaded_validations",
            TransportValue::Record(
                loaded
                    .into_iter()
                    .map(|(name, value)| RecordEntry { name, value })
                    .collect(),
            ),
        ));
    }
    values.push((
        "model_version",
        TransportValue::Integer(profile.model_version.to_string()),
    ));
    values.push(("order", strings(profile.order.iter())));
    values.push((
        "profile_authority",
        record(vec![
            (
                "responsibility",
                string_value(&profile.authority.responsibility),
            ),
            ("source", string_value(&profile.authority.source)),
        ]),
    ));
    if let Some(unchanged) = unchanged {
        values.push(("repository_unchanged", TransportValue::Boolean(unchanged)));
    }
    values.push(("validations", strings(profile.validations.keys())));
    Ok(record(values))
}
