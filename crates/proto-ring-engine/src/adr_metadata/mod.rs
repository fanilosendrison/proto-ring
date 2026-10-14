#![forbid(unsafe_code)]

mod exact_text;
mod repository_path;
mod validation;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::structured_data::{self, StructuredValue};

pub use repository_path::resolve_repository_path;
pub use validation::{SchemaLabel, SchemaValidationError, validate_schemas};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedAdr {
    pub metadata: BTreeMap<String, StructuredValue>,
    pub decision_body: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationTargetError {
    SelfReference,
    MissingTarget(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrMetadataError(String);

impl Display for AdrMetadataError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for AdrMetadataError {}

pub fn load_yaml(bytes: &[u8]) -> Result<StructuredValue, AdrMetadataError> {
    structured_data::parse_document_bytes(bytes).map_err(|error| failure(error.to_string()))
}

pub fn load_json(bytes: &[u8]) -> Result<StructuredValue, AdrMetadataError> {
    let text = std::str::from_utf8(bytes).map_err(|_| failure("JSON is not valid UTF-8"))?;
    let value =
        serde_json::from_str(text).map_err(|error| failure(format!("invalid JSON: {error}")))?;
    validation::from_json(&value)
}

pub fn parse_adr(bytes: &[u8]) -> Result<ParsedAdr, AdrMetadataError> {
    exact_text::checked_adr_text(bytes)?;
    let parsed = structured_data::parse_frontmatter_bytes(bytes)
        .map_err(|error| failure(error.to_string()))?;
    let metadata = match parsed.metadata {
        StructuredValue::Mapping(mapping) => mapping,
        _ => return Err(failure("ADR frontmatter must be a mapping")),
    };
    Ok(ParsedAdr {
        metadata,
        decision_body: exact_text::decision_body(bytes)?,
    })
}

pub fn parse_adr_file(path: &Path) -> Result<ParsedAdr, AdrMetadataError> {
    let bytes = fs::read(path).map_err(|error| failure(format!("cannot read ADR: {error}")))?;
    parse_adr(&bytes)
}

pub fn decision_body(bytes: &[u8]) -> Result<Vec<u8>, AdrMetadataError> {
    exact_text::decision_body(bytes)
}

pub fn h1_text(bytes: &[u8]) -> Result<String, AdrMetadataError> {
    exact_text::h1_text(bytes)
}

pub fn preserved_payload(bytes: &[u8]) -> Result<Vec<u8>, AdrMetadataError> {
    exact_text::preserved_payload(bytes)
}

pub fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

pub fn require_mapping(
    value: &StructuredValue,
) -> Result<&BTreeMap<String, StructuredValue>, AdrMetadataError> {
    match value {
        StructuredValue::Mapping(mapping) => Ok(mapping),
        _ => Err(failure("value must be a mapping")),
    }
}

pub fn require_string(value: &StructuredValue) -> Result<&str, AdrMetadataError> {
    match value {
        StructuredValue::String(value) if !value.is_empty() => Ok(value),
        _ => Err(failure("value must be a nonempty string")),
    }
}

pub fn require_string_list(value: &StructuredValue) -> Result<Vec<String>, AdrMetadataError> {
    let StructuredValue::Sequence(values) = value else {
        return Err(failure("value must be a sequence"));
    };
    let mut unique = BTreeSet::new();
    let mut strings = Vec::with_capacity(values.len());
    for value in values {
        let value = require_string(value)?.to_owned();
        if !unique.insert(value.clone()) {
            return Err(failure("string list contains a duplicate"));
        }
        strings.push(value);
    }
    Ok(strings)
}

pub fn relation_target_errors(
    source_id: &str,
    _relation_type: &str,
    targets: &[String],
    known_ids: &[String],
) -> Vec<RelationTargetError> {
    let known: BTreeSet<&str> = known_ids.iter().map(String::as_str).collect();
    let mut errors = Vec::new();
    for target in targets {
        if target == source_id {
            errors.push(RelationTargetError::SelfReference);
        }
        if !known.contains(target.as_str()) {
            errors.push(RelationTargetError::MissingTarget(target.clone()));
        }
    }
    errors
}

pub(crate) fn failure(message: impl Into<String>) -> AdrMetadataError {
    AdrMetadataError(message.into())
}

pub(crate) fn contained_relative_path(
    root: &Path,
    target: &Path,
) -> Result<PathBuf, AdrMetadataError> {
    let resolved_root = root
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve repository root: {error}")))?;
    target
        .strip_prefix(&resolved_root)
        .map(Path::to_path_buf)
        .map_err(|_| failure("path is outside repository root"))
}
