#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use proto_ring_engine::governance_authority::{
    GovernanceAuthorityProfile, GovernedResponsibility, GovernedSource, PrecedenceEdge, SourceRole,
};

use crate::fixture;
use crate::model::TransportValue;

use super::{field, profile_route, require_nonempty, string_field};
use crate::rust_candidate::transport::{exact_record, fixture_failure, unique_dynamic_record};

pub(super) fn support_authority(
    root: &Path,
    value: &TransportValue,
) -> Result<GovernanceAuthorityProfile, fixture::FixtureError> {
    let authority = exact_record(
        value,
        "support authority",
        &["carrier", "responsibilities", "sources"],
    )?;
    let carrier = string_field(&authority, "carrier")?;
    let mut sources = BTreeMap::new();
    for (source_id, value) in
        unique_dynamic_record(field(&authority, "sources"), "support sources")?
    {
        require_nonempty(source_id, "support source ID")?;
        let declaration =
            exact_record(value, "support source declaration", &["repository_target"])?;
        // semantic_inputs.authority represents an already-loaded Governance Authority
        // profile. Rehydrate its resolved source bindings directly; do not rerun Canonical
        // Governance Routing or filesystem availability checks in the conformance adapter.
        let repository_target = match field(&declaration, "repository_target") {
            TransportValue::Null(()) => None,
            TransportValue::String(path) => Some(profile_route(root, path, source_id)),
            _ => {
                return Err(fixture_failure(
                    "support repository_target must be string or null",
                ));
            }
        };
        sources.insert(
            source_id.to_owned(),
            GovernedSource {
                id: source_id.to_owned(),
                repository_target,
            },
        );
    }
    let mut responsibilities = BTreeMap::new();
    for (responsibility_id, value) in unique_dynamic_record(
        field(&authority, "responsibilities"),
        "support responsibilities",
    )? {
        require_nonempty(responsibility_id, "support responsibility ID")?;
        let declaration = exact_record(value, "support responsibility", &["precedence", "roles"])?;
        responsibilities.insert(
            responsibility_id.to_owned(),
            GovernedResponsibility {
                id: responsibility_id.to_owned(),
                roles: support_roles(field(&declaration, "roles"))?,
                precedence: support_precedence(field(&declaration, "precedence"))?,
            },
        );
    }
    Ok(GovernanceAuthorityProfile {
        repository: root.to_path_buf(),
        carrier: root.join(carrier),
        model_version: 1,
        sources,
        responsibilities,
    })
}

fn support_roles(
    value: &TransportValue,
) -> Result<BTreeMap<String, SourceRole>, fixture::FixtureError> {
    unique_dynamic_record(value, "support roles")?
        .into_iter()
        .map(|(source_id, value)| {
            require_nonempty(source_id, "support role source ID")?;
            let role = match fixture::string(value)? {
                "authority" => SourceRole::Authority,
                "secondary_representation" => SourceRole::SecondaryRepresentation,
                "non_authoritative" => SourceRole::NonAuthoritative,
                _ => return Err(fixture_failure("unknown support source role")),
            };
            Ok((source_id.to_owned(), role))
        })
        .collect()
}

fn support_precedence(
    value: &TransportValue,
) -> Result<Vec<PrecedenceEdge>, fixture::FixtureError> {
    fixture::sequence(value)?
        .iter()
        .map(|edge| {
            let fields = exact_record(
                edge,
                "support precedence entry",
                &["higher_source", "lower_source"],
            )?;
            Ok(PrecedenceEdge {
                higher_source: string_field(&fields, "higher_source")?.to_owned(),
                lower_source: string_field(&fields, "lower_source")?.to_owned(),
            })
        })
        .collect()
}
