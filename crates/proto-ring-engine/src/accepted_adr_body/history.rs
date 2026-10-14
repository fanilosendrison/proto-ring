use std::collections::BTreeMap;

use crate::adr_metadata;
use crate::structured_data::StructuredValue;

use super::git::GitRepository;
use super::{AcceptedBodyDiagnostic, DiscoveryConfig};

const REGULAR_FILE_MODES: [&str; 2] = ["100644", "100755"];

pub(super) struct AcceptanceSeal {
    pub(super) logical_id: String,
    pub(super) acceptance_commit: String,
    pub(super) historical_path: String,
    pub(super) decision_body: Vec<u8>,
}

pub(super) fn inspect(
    git: &GitRepository,
    config: &DiscoveryConfig,
) -> Result<
    (
        BTreeMap<String, AcceptanceSeal>,
        Vec<AcceptedBodyDiagnostic>,
    ),
    String,
> {
    let mut seals: BTreeMap<String, AcceptanceSeal> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for commit in git
        .first_parent_commits()
        .map_err(|error| error.to_string())?
    {
        let parent = git
            .first_parent(&commit)
            .map_err(|error| error.to_string())?;
        let paths = git
            .changed_paths(&commit, parent.as_deref())
            .map_err(|error| error.to_string())?;
        for raw_path in paths {
            let Some(slot) = matching_slot(&raw_path, config)? else {
                continue;
            };
            let entry = git
                .tree_entry(&commit, &slot.path)
                .map_err(|error| error.to_string())?;
            let Some(entry) = entry else {
                continue;
            };
            if !REGULAR_FILE_MODES.contains(&entry.mode.as_str()) || entry.object_type != "blob" {
                if seals.contains_key(&slot.logical_id) {
                    diagnostics.push(AcceptedBodyDiagnostic::historical(format!(
                        "sealed ADR {} has nonregular historical slot at commit {commit}: {}",
                        slot.logical_id, slot.path
                    )));
                }
                continue;
            }
            let blob = git
                .blob(&entry.object_id)
                .map_err(|error| error.to_string())?;
            let parsed = match adr_metadata::parse_adr(&blob) {
                Ok(parsed) => parsed,
                Err(error) => {
                    if seals.contains_key(&slot.logical_id) {
                        diagnostics.push(AcceptedBodyDiagnostic::historical(format!(
                            "sealed ADR {} is unparseable at commit {commit}, path {}: {error}",
                            slot.logical_id, slot.path
                        )));
                    }
                    continue;
                }
            };
            let metadata_id = string_field(parsed.metadata.get("id"));
            if metadata_id != Some(slot.logical_id.as_str()) {
                let claims_sealed_identity =
                    metadata_id.is_some_and(|logical_id| seals.contains_key(logical_id));
                if seals.contains_key(&slot.logical_id) || claims_sealed_identity {
                    diagnostics.push(AcceptedBodyDiagnostic::historical(format!(
                        "historical ADR identity is inconsistent at commit {commit}, path {}: slot {}, metadata.id {:?}",
                        slot.path, slot.logical_id, metadata_id
                    )));
                }
                continue;
            }

            if let Some(seal) = seals.get(&slot.logical_id) {
                if parsed.decision_body != seal.decision_body {
                    diagnostics.push(AcceptedBodyDiagnostic::body_changed(format!(
                        "sealed ADR {} body changed at commit {commit}, path {}; anchor commit {}, anchor path {}",
                        seal.logical_id,
                        slot.path,
                        seal.acceptance_commit,
                        seal.historical_path
                    )));
                }
                continue;
            }
            if string_field(parsed.metadata.get("status")) == Some("accepted") {
                seals.insert(
                    slot.logical_id.clone(),
                    AcceptanceSeal {
                        logical_id: slot.logical_id,
                        acceptance_commit: commit.clone(),
                        historical_path: slot.path,
                        decision_body: parsed.decision_body,
                    },
                );
            }
        }
    }
    Ok((seals, diagnostics))
}

struct HistoricalSlot {
    logical_id: String,
    path: String,
}

fn matching_slot(
    raw_path: &[u8],
    config: &DiscoveryConfig,
) -> Result<Option<HistoricalSlot>, String> {
    let basename = if config.adr_directory == "." {
        if raw_path.is_empty() || raw_path.contains(&b'/') {
            return Ok(None);
        }
        raw_path
    } else {
        let directory = config.adr_directory.as_bytes();
        if !raw_path.starts_with(directory) || raw_path.get(directory.len()) != Some(&b'/') {
            return Ok(None);
        }
        let basename = &raw_path[directory.len() + 1..];
        if basename.is_empty() || basename.contains(&b'/') {
            return Ok(None);
        }
        basename
    };
    let Ok(basename) = std::str::from_utf8(basename) else {
        return Ok(None);
    };
    let Some(matched) = config
        .filename_pattern
        .full_match(basename)
        .map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };
    let number = matched
        .named_capture("number")
        .filter(|number| !number.is_empty())
        .ok_or_else(|| "matching historical filename has no nonempty number capture".to_owned())?;
    let path = if config.adr_directory == "." {
        basename.to_owned()
    } else {
        format!("{}/{basename}", config.adr_directory)
    };
    Ok(Some(HistoricalSlot {
        logical_id: format!("ADR-{number}"),
        path,
    }))
}

fn string_field(value: Option<&StructuredValue>) -> Option<&str> {
    match value {
        Some(StructuredValue::String(value)) => Some(value),
        _ => None,
    }
}
