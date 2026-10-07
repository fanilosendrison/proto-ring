#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::governance_authority::GovernanceAuthorityProfile;
use crate::governance_routing::ResolvedGovernanceRoute;
use crate::structured_data::{self, StructuredValue};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GovernedObjectRef {
    pub interface_id: String,
    pub object_id: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GovernedRelation {
    pub relation: String,
    pub responsibility: String,
    pub target: GovernedObjectRef,
}

#[derive(Clone, Debug)]
pub struct GovernedObject {
    pub id: String,
    pub responsibilities: BTreeSet<String>,
    pub relations: BTreeSet<GovernedRelation>,
}

#[derive(Clone, Debug)]
pub struct GovernedInterface {
    pub id: String,
    pub objects: BTreeMap<String, GovernedObject>,
}

#[derive(Clone, Debug)]
pub struct GovernedObjectCatalog {
    pub repository: PathBuf,
    pub carrier: PathBuf,
    pub model_version: u8,
    pub interfaces: BTreeMap<String, GovernedInterface>,
}

#[derive(Debug)]
pub struct GovernedObjectsError(String);

impl Display for GovernedObjectsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for GovernedObjectsError {}

pub fn load(
    repository_root: &Path,
    profile_route: &ResolvedGovernanceRoute,
    authority_profile: &GovernanceAuthorityProfile,
) -> Result<GovernedObjectCatalog, GovernedObjectsError> {
    let canonical_root = repository_root
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve repository root: {error}")))?;
    let authority_root = authority_profile
        .repository
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve authority repository: {error}")))?;
    if canonical_root != authority_root {
        return Err(failure("catalog and authority repositories differ"));
    }
    let bytes = fs::read(&profile_route.target)
        .map_err(|error| failure(format!("cannot read governed-object profile: {error}")))?;
    let parsed = structured_data::parse_frontmatter_bytes(&bytes)
        .map_err(|error| failure(format!("cannot parse governed-object profile: {error}")))?;
    let metadata = value_mapping(&parsed.metadata, "frontmatter must be a mapping")?;
    let profile = required_mapping(metadata, "governed_objects")?;
    require_keys(profile, &["interfaces", "model_version"])?;
    match profile.get("model_version") {
        Some(StructuredValue::Integer(value)) if value == "1" => {}
        _ => return Err(failure("unsupported governed-object model version")),
    }
    let interfaces = load_interfaces(required_mapping(profile, "interfaces")?, authority_profile)?;
    validate_targets(&interfaces)?;
    Ok(GovernedObjectCatalog {
        repository: canonical_root,
        carrier: profile_route.target.clone(),
        model_version: 1,
        interfaces,
    })
}

fn load_interfaces(
    declarations: &BTreeMap<String, StructuredValue>,
    authority: &GovernanceAuthorityProfile,
) -> Result<BTreeMap<String, GovernedInterface>, GovernedObjectsError> {
    declarations
        .iter()
        .map(|(interface_id, value)| {
            require_identifier(interface_id, "interface ID")?;
            let declaration = value_mapping(value, "interface must be a mapping")?;
            require_keys(declaration, &["objects"])?;
            let objects = load_objects(required_mapping(declaration, "objects")?, authority)?;
            Ok((
                interface_id.clone(),
                GovernedInterface {
                    id: interface_id.clone(),
                    objects,
                },
            ))
        })
        .collect()
}

fn load_objects(
    declarations: &BTreeMap<String, StructuredValue>,
    authority: &GovernanceAuthorityProfile,
) -> Result<BTreeMap<String, GovernedObject>, GovernedObjectsError> {
    declarations
        .iter()
        .map(|(object_id, value)| {
            require_identifier(object_id, "object ID")?;
            let declaration = value_mapping(value, "object must be a mapping")?;
            require_keys(declaration, &["relations", "responsibilities"])?;
            let responsibilities = load_responsibilities(
                declaration
                    .get("responsibilities")
                    .ok_or_else(|| failure("responsibilities are missing"))?,
                authority,
            )?;
            let relations = load_relations(
                declaration
                    .get("relations")
                    .ok_or_else(|| failure("relations are missing"))?,
                &responsibilities,
            )?;
            Ok((
                object_id.clone(),
                GovernedObject {
                    id: object_id.clone(),
                    responsibilities,
                    relations,
                },
            ))
        })
        .collect()
}

fn load_responsibilities(
    value: &StructuredValue,
    authority: &GovernanceAuthorityProfile,
) -> Result<BTreeSet<String>, GovernedObjectsError> {
    let StructuredValue::Sequence(values) = value else {
        return Err(failure("object responsibilities must be a sequence"));
    };
    if values.is_empty() {
        return Err(failure("object responsibilities must not be empty"));
    }
    let mut responsibilities = BTreeSet::new();
    for value in values {
        let responsibility = value_string(value, "responsibility must be a nonempty string")?;
        if !authority.responsibilities.contains_key(responsibility) {
            return Err(failure("unknown governed responsibility"));
        }
        if !responsibilities.insert(responsibility.to_owned()) {
            return Err(failure("duplicate object responsibility"));
        }
    }
    Ok(responsibilities)
}

fn load_relations(
    value: &StructuredValue,
    source_responsibilities: &BTreeSet<String>,
) -> Result<BTreeSet<GovernedRelation>, GovernedObjectsError> {
    let StructuredValue::Sequence(values) = value else {
        return Err(failure("object relations must be a sequence"));
    };
    let mut relations = BTreeSet::new();
    for value in values {
        let declaration = value_mapping(value, "relation must be a mapping")?;
        require_keys(declaration, &["relation", "responsibility", "target"])?;
        let relation = required_string(declaration, "relation")?.to_owned();
        let responsibility = required_string(declaration, "responsibility")?.to_owned();
        if !source_responsibilities.contains(&responsibility) {
            return Err(failure(
                "relation responsibility is not declared by source object",
            ));
        }
        let target = required_mapping(declaration, "target")?;
        require_keys(target, &["interface", "object"])?;
        let relation = GovernedRelation {
            relation,
            responsibility,
            target: GovernedObjectRef {
                interface_id: required_string(target, "interface")?.to_owned(),
                object_id: required_string(target, "object")?.to_owned(),
            },
        };
        if !relations.insert(relation) {
            return Err(failure("duplicate governed relation"));
        }
    }
    Ok(relations)
}

fn validate_targets(
    interfaces: &BTreeMap<String, GovernedInterface>,
) -> Result<(), GovernedObjectsError> {
    for relation in interfaces
        .values()
        .flat_map(|interface| interface.objects.values())
        .flat_map(|object| &object.relations)
    {
        let Some(interface) = interfaces.get(&relation.target.interface_id) else {
            return Err(failure("relation target interface does not exist"));
        };
        if !interface.objects.contains_key(&relation.target.object_id) {
            return Err(failure("relation target object does not exist"));
        }
    }
    Ok(())
}

fn require_keys(
    mapping: &BTreeMap<String, StructuredValue>,
    expected: &[&str],
) -> Result<(), GovernedObjectsError> {
    if mapping.len() != expected.len() || expected.iter().any(|key| !mapping.contains_key(*key)) {
        return Err(failure("mapping has missing or unknown direct keys"));
    }
    Ok(())
}

fn required_mapping<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a BTreeMap<String, StructuredValue>, GovernedObjectsError> {
    mapping
        .get(key)
        .ok_or_else(|| failure(format!("missing {key}")))
        .and_then(|value| value_mapping(value, &format!("{key} must be a mapping")))
}

fn value_mapping<'a>(
    value: &'a StructuredValue,
    message: &str,
) -> Result<&'a BTreeMap<String, StructuredValue>, GovernedObjectsError> {
    match value {
        StructuredValue::Mapping(mapping) => Ok(mapping),
        _ => Err(failure(message)),
    }
}

fn required_string<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a str, GovernedObjectsError> {
    match mapping.get(key) {
        Some(value) => value_string(value, &format!("{key} must be a nonempty string")),
        None => Err(failure(format!("missing {key}"))),
    }
}

fn value_string<'a>(
    value: &'a StructuredValue,
    message: &str,
) -> Result<&'a str, GovernedObjectsError> {
    match value {
        StructuredValue::String(value) => {
            require_identifier(value, message)?;
            Ok(value)
        }
        _ => Err(failure(message)),
    }
}

fn require_identifier(value: &str, label: &str) -> Result<(), GovernedObjectsError> {
    if value.is_empty() {
        Err(failure(format!("{label} must be nonempty")))
    } else {
        Ok(())
    }
}

fn failure(message: impl Into<String>) -> GovernedObjectsError {
    GovernedObjectsError(message.into())
}
