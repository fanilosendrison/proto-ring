#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::adr_metadata::{self, ParsedAdr};
use crate::portable_pattern::PortablePattern;
use crate::repository_governance_model;
use crate::structured_data::StructuredValue;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalAdr {
    pub adr_id: String,
    pub relative_path: PathBuf,
    pub metadata: BTreeMap<String, StructuredValue>,
    pub decision_body: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalAdrError(String);

impl Display for CanonicalAdrError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CanonicalAdrError {}

pub fn resolve(repository_root: &Path, adr_id: &str) -> Result<CanonicalAdr, CanonicalAdrError> {
    let model = repository_governance_model::load(repository_root)
        .map_err(|error| failure(format!("repository governance model failed: {error}")))?;
    let profile_route = model
        .capabilities
        .get("architecture_decisions")
        .and_then(|capability| capability.routes.get("profile"))
        .ok_or_else(|| failure("architecture-decisions profile route is missing"))?;
    let profile_bytes = fs::read(&profile_route.target)
        .map_err(|error| failure(format!("cannot read ADR profile: {error}")))?;
    let profile = adr_metadata::load_yaml(&profile_bytes)
        .map_err(|error| failure(format!("cannot load ADR profile: {error}")))?;
    let repository = repository_profile(&profile)?;
    let directory = required_string(repository, "adr_directory")?;
    let filename_source = required_string(repository, "filename_pattern")?;
    let id_source = required_string(repository, "id_pattern")?;
    let id_width = required_positive_integer(repository, "id_width")?;

    let filename_pattern = PortablePattern::compile(filename_source)
        .map_err(|error| failure(format!("invalid filename pattern: {error}")))?;
    if !filename_pattern
        .capture_names()
        .any(|(name, _)| name == "number")
    {
        return Err(failure("filename pattern has no number capture"));
    }
    let id_pattern = PortablePattern::compile(id_source)
        .map_err(|error| failure(format!("invalid ID pattern: {error}")))?;
    if id_pattern
        .full_match(adr_id)
        .map_err(|error| failure(error.to_string()))?
        .is_none()
    {
        return Err(failure(
            "requested ADR identity does not match the ID pattern",
        ));
    }
    let number = requested_number(adr_id)?;
    if !decimal_equals_usize(id_width, number.len()) {
        return Err(failure("requested ADR identity has the wrong width"));
    }

    let resolved_directory =
        adr_metadata::resolve_repository_path(&model.repository_root, directory)
            .map_err(|error| failure(error.to_string()))?;
    if !fs::metadata(&resolved_directory)
        .map_err(|error| failure(format!("cannot inspect ADR directory: {error}")))?
        .is_dir()
    {
        return Err(failure("ADR corpus path is not a directory"));
    }
    let candidate = unique_candidate(&resolved_directory, &filename_pattern, number)?;
    let relative = adr_metadata::contained_relative_path(&model.repository_root, &candidate)
        .map_err(|error| failure(error.to_string()))?;
    let declared = native_relative_text(&relative)?;
    let artifact = adr_metadata::resolve_repository_path(&model.repository_root, &declared)
        .map_err(|error| failure(error.to_string()))?;
    let ParsedAdr {
        metadata,
        decision_body,
    } = adr_metadata::parse_adr_file(&artifact)
        .map_err(|error| failure(format!("cannot parse canonical ADR: {error}")))?;
    match metadata.get("id") {
        Some(StructuredValue::String(value)) if value == adr_id => {}
        _ => return Err(failure("canonical ADR metadata ID does not match")),
    }
    Ok(CanonicalAdr {
        adr_id: adr_id.to_owned(),
        relative_path: relative,
        metadata,
        decision_body,
    })
}

fn repository_profile(
    profile: &StructuredValue,
) -> Result<&BTreeMap<String, StructuredValue>, CanonicalAdrError> {
    let root =
        adr_metadata::require_mapping(profile).map_err(|error| failure(error.to_string()))?;
    root.get("repository")
        .ok_or_else(|| failure("ADR profile repository mapping is missing"))
        .and_then(|value| {
            adr_metadata::require_mapping(value).map_err(|error| failure(error.to_string()))
        })
}

fn required_string<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a str, CanonicalAdrError> {
    mapping
        .get(key)
        .ok_or_else(|| failure(format!("ADR profile field is missing: {key}")))
        .and_then(|value| {
            adr_metadata::require_string(value).map_err(|error| failure(error.to_string()))
        })
}

fn required_positive_integer<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a str, CanonicalAdrError> {
    match mapping.get(key) {
        Some(StructuredValue::Integer(value)) if positive_decimal(value) => Ok(value),
        _ => Err(failure(format!("{key} must be a positive integer"))),
    }
}

fn positive_decimal(value: &str) -> bool {
    matches!(value.as_bytes(), [b'1'..=b'9', rest @ ..] if rest.iter().all(u8::is_ascii_digit))
}

fn requested_number(adr_id: &str) -> Result<&str, CanonicalAdrError> {
    let number = adr_id
        .strip_prefix("ADR-")
        .ok_or_else(|| failure("requested ADR identity has the wrong shape"))?;
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(failure("requested ADR identity has the wrong shape"));
    }
    Ok(number)
}

fn decimal_equals_usize(decimal: &str, value: usize) -> bool {
    decimal.as_bytes() == value.to_string().as_bytes()
}

fn unique_candidate(
    directory: &Path,
    pattern: &PortablePattern,
    requested_number: &str,
) -> Result<PathBuf, CanonicalAdrError> {
    let entries = fs::read_dir(directory)
        .map_err(|error| failure(format!("cannot read ADR directory: {error}")))?;
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| failure(format!("cannot read ADR entry: {error}")))?;
        let filename = entry
            .file_name()
            .into_string()
            .map_err(|_| failure("ADR corpus filename is not UTF-8"))?;
        let Some(matched) = pattern
            .full_match(&filename)
            .map_err(|error| failure(error.to_string()))?
        else {
            continue;
        };
        let number = matched
            .named_capture("number")
            .filter(|capture| !capture.is_empty())
            .ok_or_else(|| failure("matching filename has no nonempty number capture"))?;
        if number != requested_number {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|error| failure(format!("cannot inspect ADR entry: {error}")))?;
        if file_type.is_symlink() {
            return Err(failure("matching ADR entry is a symbolic link"));
        }
        if file_type.is_file() {
            candidates.push(entry.path());
        }
    }
    match candidates.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => Err(failure("no canonical ADR candidate")),
        _ => Err(failure("multiple canonical ADR candidates")),
    }
}

fn native_relative_text(path: &Path) -> Result<String, CanonicalAdrError> {
    path.components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| failure("canonical ADR path is not UTF-8")),
            _ => Err(failure("canonical ADR path is not repository-relative")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join(std::path::MAIN_SEPARATOR_STR))
}

fn failure(message: impl Into<String>) -> CanonicalAdrError {
    CanonicalAdrError(message.into())
}

#[cfg(test)]
mod tests {
    use super::decimal_equals_usize;

    #[test]
    fn width_comparison_never_parses_the_profile_integer() {
        assert!(decimal_equals_usize("3", 3));
        assert!(!decimal_equals_usize(&"9".repeat(4301), 3));
    }
}
