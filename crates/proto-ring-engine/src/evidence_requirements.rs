//! Validation-only loading of persistent evidence requirement declarations.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::declaration_support::{
    boolean, exact_keys, mapping, model_version_one, required_mapping, sequence, string,
};
use crate::governance_authority::{GovernanceAuthorityProfile, RoleLookup, SourceRole, role_of};
use crate::governance_routing::ResolvedGovernanceRoute;
use crate::governed_objects::{GovernedObjectCatalog, GovernedObjectRef};
use crate::structured_data::{self, StructuredValue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstantiationKind {
    Single,
    Source,
}
impl InstantiationKind {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Source => "source",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceClassKind {
    Explicit,
    Source,
}
impl EvidenceClassKind {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Source => "source",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryAuthority {
    pub responsibility: String,
    pub source: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequirementInstantiation {
    pub kind: InstantiationKind,
    pub source: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceClassAdmission {
    pub kind: EvidenceClassKind,
    pub classes: BTreeSet<String>,
    pub source: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextBinding {
    pub required: bool,
    pub source: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistentEvidenceRequirement {
    pub id: String,
    pub responsibility: String,
    pub instances: RequirementInstantiation,
    pub evidence_classes: EvidenceClassAdmission,
    pub subject_source: String,
    pub context: ContextBinding,
    pub candidate_source: String,
    pub target: Option<GovernedObjectRef>,
}
#[derive(Clone, Debug)]
pub struct EvidenceRequirementRegistry {
    pub repository: PathBuf,
    pub carrier: PathBuf,
    pub model_version: u8,
    pub authority: RegistryAuthority,
    pub requirements: BTreeMap<String, PersistentEvidenceRequirement>,
}
#[derive(Debug)]
pub struct EvidenceRequirementsError(String);
impl Display for EvidenceRequirementsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl Error for EvidenceRequirementsError {}
fn failure(message: impl Into<String>) -> EvidenceRequirementsError {
    EvidenceRequirementsError(message.into())
}
type Result<T> = std::result::Result<T, EvidenceRequirementsError>;

pub fn load(
    repository: &Path,
    route: &ResolvedGovernanceRoute,
    authority: &GovernanceAuthorityProfile,
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<EvidenceRequirementRegistry> {
    let canonical_root = repository
        .canonicalize()
        .map_err(|error| failure(error.to_string()))?;
    if canonical_root
        != authority
            .repository
            .canonicalize()
            .map_err(|error| failure(error.to_string()))?
    {
        return Err(failure("registry and authority repositories differ"));
    }
    if let Some(catalog) = catalog
        && canonical_root
            != catalog
                .repository
                .canonicalize()
                .map_err(|error| failure(error.to_string()))?
    {
        return Err(failure("registry and catalog repositories differ"));
    }
    let bytes = fs::read(&route.target).map_err(|error| failure(error.to_string()))?;
    let parsed = structured_data::parse_frontmatter_bytes(&bytes)
        .map_err(|error| failure(error.to_string()))?;
    let metadata = mapping(&parsed.metadata, "frontmatter").map_err(failure)?;
    let root = required_mapping(metadata, "evidence_requirements").map_err(failure)?;
    exact_keys(root, &["authority", "model_version", "requirements"]).map_err(failure)?;
    model_version_one(root).map_err(failure)?;
    let registry_authority = parse_authority(
        required_mapping(root, "authority").map_err(failure)?,
        authority,
        route,
    )?;
    let requirements = required_mapping(root, "requirements")
        .map_err(failure)?
        .iter()
        .map(|(id, value)| parse_requirement(id, value, authority, catalog))
        .collect::<Result<_>>()?;
    Ok(EvidenceRequirementRegistry {
        repository: canonical_root,
        carrier: route.target.clone(),
        model_version: 1,
        authority: registry_authority,
        requirements,
    })
}

fn parse_authority(
    value: &BTreeMap<String, StructuredValue>,
    authority: &GovernanceAuthorityProfile,
    route: &ResolvedGovernanceRoute,
) -> Result<RegistryAuthority> {
    exact_keys(value, &["responsibility", "source"]).map_err(failure)?;
    let responsibility = string(value, "responsibility").map_err(failure)?.to_owned();
    let source = string(value, "source").map_err(failure)?.to_owned();
    if !authority.sources.contains_key(&source) {
        return Err(failure("unknown registry authority source"));
    }
    if role_of(authority, &responsibility, &source) != RoleLookup::Role(SourceRole::Authority) {
        return Err(failure("registry source is not authority"));
    }
    let target = authority.sources[&source]
        .repository_target
        .as_ref()
        .ok_or_else(|| failure("registry source has no repository target"))?;
    if target.target != route.target {
        return Err(failure("registry source does not resolve to carrier"));
    }
    Ok(RegistryAuthority {
        responsibility,
        source,
    })
}

fn parse_requirement(
    id: &str,
    value: &StructuredValue,
    authority: &GovernanceAuthorityProfile,
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<(String, PersistentEvidenceRequirement)> {
    if id.is_empty() {
        return Err(failure("requirement ID must be nonempty"));
    }
    let value = mapping(value, "requirement").map_err(failure)?;
    let keys = if value.contains_key("target") {
        &[
            "candidates",
            "context",
            "evidence_classes",
            "instances",
            "responsibility",
            "subject",
            "target",
        ][..]
    } else {
        &[
            "candidates",
            "context",
            "evidence_classes",
            "instances",
            "responsibility",
            "subject",
        ][..]
    };
    exact_keys(value, keys).map_err(failure)?;
    let responsibility = string(value, "responsibility").map_err(failure)?.to_owned();
    if !authority.responsibilities.contains_key(&responsibility) {
        return Err(failure("unknown requirement responsibility"));
    }
    let instances = parse_instances(value.get("instances").expect("exact key exists"), authority)?;
    let evidence_classes = parse_classes(
        value.get("evidence_classes").expect("exact key exists"),
        authority,
    )?;
    let subject_source =
        parse_source_ref(value.get("subject").expect("exact key exists"), authority)?;
    let context = parse_context(value.get("context").expect("exact key exists"), authority)?;
    let candidate_source = parse_source_ref(
        value.get("candidates").expect("exact key exists"),
        authority,
    )?;
    let target = value
        .get("target")
        .map(|value| parse_target(value, &responsibility, catalog))
        .transpose()?;
    let requirement = PersistentEvidenceRequirement {
        id: id.to_owned(),
        responsibility,
        instances,
        evidence_classes,
        subject_source,
        context,
        candidate_source,
        target,
    };
    Ok((id.to_owned(), requirement))
}

fn parse_instances(
    value: &StructuredValue,
    authority: &GovernanceAuthorityProfile,
) -> Result<RequirementInstantiation> {
    let value = mapping(value, "instances").map_err(failure)?;
    let kind = match value.get("kind") {
        Some(StructuredValue::String(value)) if !value.is_empty() => value.as_str(),
        _ => return Err(failure("instance kind must be nonempty string")),
    };
    match kind {
        "single" => {
            exact_keys(value, &["kind"]).map_err(failure)?;
            Ok(RequirementInstantiation {
                kind: InstantiationKind::Single,
                source: None,
            })
        }
        "source" => {
            exact_keys(value, &["kind", "source"]).map_err(failure)?;
            Ok(RequirementInstantiation {
                kind: InstantiationKind::Source,
                source: Some(known_source(value, "source", authority)?),
            })
        }
        _ => Err(failure("unknown instance kind")),
    }
}

fn parse_classes(
    value: &StructuredValue,
    authority: &GovernanceAuthorityProfile,
) -> Result<EvidenceClassAdmission> {
    let value = mapping(value, "evidence classes").map_err(failure)?;
    let kind = match value.get("kind") {
        Some(StructuredValue::String(value)) if !value.is_empty() => value.as_str(),
        _ => return Err(failure("evidence class kind must be nonempty string")),
    };
    match kind {
        "explicit" => {
            exact_keys(value, &["classes", "kind"]).map_err(failure)?;
            let mut classes = BTreeSet::new();
            for value in sequence(value, "classes").map_err(failure)? {
                let StructuredValue::String(value) = value else {
                    return Err(failure("evidence class must be string"));
                };
                if value.is_empty() || !classes.insert(value.clone()) {
                    return Err(failure("explicit classes must be nonempty and unique"));
                }
            }
            if classes.is_empty() {
                return Err(failure("explicit classes must not be empty"));
            }
            Ok(EvidenceClassAdmission {
                kind: EvidenceClassKind::Explicit,
                classes,
                source: None,
            })
        }
        "source" => {
            exact_keys(value, &["kind", "source"]).map_err(failure)?;
            Ok(EvidenceClassAdmission {
                kind: EvidenceClassKind::Source,
                classes: BTreeSet::new(),
                source: Some(known_source(value, "source", authority)?),
            })
        }
        _ => Err(failure("unknown evidence class kind")),
    }
}

fn parse_context(
    value: &StructuredValue,
    authority: &GovernanceAuthorityProfile,
) -> Result<ContextBinding> {
    let value = mapping(value, "context").map_err(failure)?;
    let required = boolean(value, "required").map_err(failure)?;
    if required {
        exact_keys(value, &["required", "source"]).map_err(failure)?;
        Ok(ContextBinding {
            required,
            source: Some(known_source(value, "source", authority)?),
        })
    } else {
        exact_keys(value, &["required"]).map_err(failure)?;
        Ok(ContextBinding {
            required,
            source: None,
        })
    }
}

fn parse_source_ref(
    value: &StructuredValue,
    authority: &GovernanceAuthorityProfile,
) -> Result<String> {
    let value = mapping(value, "source reference").map_err(failure)?;
    exact_keys(value, &["source"]).map_err(failure)?;
    known_source(value, "source", authority)
}
fn known_source(
    value: &BTreeMap<String, StructuredValue>,
    key: &str,
    authority: &GovernanceAuthorityProfile,
) -> Result<String> {
    let source = string(value, key).map_err(failure)?.to_owned();
    if authority.sources.contains_key(&source) {
        Ok(source)
    } else {
        Err(failure("unknown governed source"))
    }
}
fn parse_target(
    value: &StructuredValue,
    responsibility: &str,
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<GovernedObjectRef> {
    let value = mapping(value, "target").map_err(failure)?;
    exact_keys(value, &["interface", "object"]).map_err(failure)?;
    let target = GovernedObjectRef {
        interface_id: string(value, "interface").map_err(failure)?.to_owned(),
        object_id: string(value, "object").map_err(failure)?.to_owned(),
    };
    let object = catalog
        .and_then(|catalog| catalog.interfaces.get(&target.interface_id))
        .and_then(|interface| interface.objects.get(&target.object_id))
        .ok_or_else(|| failure("unknown governed object target"))?;
    if !object.responsibilities.contains(responsibility) {
        return Err(failure("target does not participate in responsibility"));
    }
    Ok(target)
}
