//! Validation-only loading of persistent projection relations.
use crate::declaration_support::{
    exact_keys, mapping, model_version_one, required_mapping, string,
};
use crate::governance_authority::{GovernanceAuthorityProfile, RoleLookup, SourceRole, role_of};
use crate::governance_bindings::GovernanceBindingRegistry;
use crate::governance_routing::ResolvedGovernanceRoute;
use crate::governed_objects::{GovernedObjectCatalog, GovernedObjectRef};
use crate::repository_integrity::ConsumerIntegrityProfile;
use crate::structured_data::{self, StructuredValue};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProjectionMode {
    Reference,
    Generated,
    MechanicallyValidatedMaintained,
    BoundedHistoricalSnapshot,
}
impl ProjectionMode {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Generated => "generated",
            Self::MechanicallyValidatedMaintained => "mechanically_validated_maintained",
            Self::BoundedHistoricalSnapshot => "bounded_historical_snapshot",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryAuthority {
    pub responsibility: String,
    pub source: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection {
    pub id: String,
    pub responsibility: String,
    pub canonical_source: String,
    pub secondary_source: String,
    pub mode: ProjectionMode,
    pub validation: String,
    pub generator_source: Option<String>,
    pub boundary_source: Option<String>,
    pub target: Option<GovernedObjectRef>,
    pub binding: Option<String>,
}
#[derive(Clone, Debug)]
pub struct ProjectionRegistry {
    pub repository: PathBuf,
    pub carrier: PathBuf,
    pub model_version: u8,
    pub authority: RegistryAuthority,
    pub projections: BTreeMap<String, Projection>,
}
#[derive(Debug)]
pub struct ProjectionRegistryError(String);
impl Display for ProjectionRegistryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl Error for ProjectionRegistryError {}
fn failure(message: impl Into<String>) -> ProjectionRegistryError {
    ProjectionRegistryError(message.into())
}
type Result<T> = std::result::Result<T, ProjectionRegistryError>;

fn require_same_repository(canonical_root: &Path, peer: &Path, label: &str) -> Result<()> {
    let peer_root = peer
        .canonicalize()
        .map_err(|error| failure(error.to_string()))?;
    if canonical_root != peer_root {
        return Err(failure(format!("registry and {label} repositories differ")));
    }
    Ok(())
}

pub fn load(
    repository: &Path,
    route: &ResolvedGovernanceRoute,
    authority: &GovernanceAuthorityProfile,
    integrity: &ConsumerIntegrityProfile,
    catalog: Option<&GovernedObjectCatalog>,
    bindings: Option<&GovernanceBindingRegistry>,
) -> Result<ProjectionRegistry> {
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
    require_same_repository(&canonical_root, &integrity.repository, "integrity profile")?;
    if let Some(catalog) = catalog {
        require_same_repository(&canonical_root, &catalog.repository, "catalog")?;
    }
    if let Some(bindings) = bindings {
        require_same_repository(&canonical_root, &bindings.repository, "bindings")?;
    }
    let bytes = fs::read(&route.target).map_err(|error| failure(error.to_string()))?;
    let parsed = structured_data::parse_frontmatter_bytes(&bytes)
        .map_err(|error| failure(error.to_string()))?;
    let metadata = mapping(&parsed.metadata, "frontmatter").map_err(failure)?;
    let root = required_mapping(metadata, "projection_registry").map_err(failure)?;
    exact_keys(root, &["authority", "model_version", "projections"]).map_err(failure)?;
    model_version_one(root).map_err(failure)?;
    let registry_authority = parse_authority(
        required_mapping(root, "authority").map_err(failure)?,
        authority,
        route,
    )?;
    let projections = required_mapping(root, "projections")
        .map_err(failure)?
        .iter()
        .map(|(id, value)| parse_projection(id, value, authority, integrity, catalog, bindings))
        .collect::<Result<_>>()?;
    reject_active_ambiguity(&projections)?;
    Ok(ProjectionRegistry {
        repository: canonical_root,
        carrier: route.target.clone(),
        model_version: 1,
        authority: registry_authority,
        projections,
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
fn parse_projection(
    id: &str,
    value: &StructuredValue,
    authority: &GovernanceAuthorityProfile,
    integrity: &ConsumerIntegrityProfile,
    catalog: Option<&GovernedObjectCatalog>,
    bindings: Option<&GovernanceBindingRegistry>,
) -> Result<(String, Projection)> {
    if id.is_empty() {
        return Err(failure("projection ID must be nonempty"));
    }
    let value = mapping(value, "projection").map_err(failure)?;
    let allowed = [
        "binding",
        "boundary_source",
        "canonical_source",
        "generator_source",
        "mode",
        "responsibility",
        "secondary_source",
        "target",
        "validation",
    ];
    if value.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(failure("projection has unknown key"));
    }
    for required in [
        "canonical_source",
        "mode",
        "responsibility",
        "secondary_source",
        "validation",
    ] {
        if !value.contains_key(required) {
            return Err(failure(format!("projection is missing {required}")));
        }
    }
    let responsibility = string(value, "responsibility").map_err(failure)?.to_owned();
    if !authority.responsibilities.contains_key(&responsibility) {
        return Err(failure("unknown projection responsibility"));
    }
    let canonical_source = known_source(value, "canonical_source", authority)?;
    let secondary_source = known_source(value, "secondary_source", authority)?;
    if canonical_source == secondary_source {
        return Err(failure("projection sources must differ"));
    }
    if role_of(authority, &responsibility, &canonical_source)
        != RoleLookup::Role(SourceRole::Authority)
    {
        return Err(failure("canonical source is not authority"));
    }
    if role_of(authority, &responsibility, &secondary_source)
        != RoleLookup::Role(SourceRole::SecondaryRepresentation)
    {
        return Err(failure("secondary source has wrong role"));
    }
    let mode = match string(value, "mode").map_err(failure)? {
        "reference" => ProjectionMode::Reference,
        "generated" => ProjectionMode::Generated,
        "mechanically_validated_maintained" => ProjectionMode::MechanicallyValidatedMaintained,
        "bounded_historical_snapshot" => ProjectionMode::BoundedHistoricalSnapshot,
        _ => return Err(failure("unknown projection mode")),
    };
    let generator_source = value
        .contains_key("generator_source")
        .then(|| known_source(value, "generator_source", authority))
        .transpose()?;
    let boundary_source = value
        .contains_key("boundary_source")
        .then(|| known_source(value, "boundary_source", authority))
        .transpose()?;
    validate_mode(
        mode,
        generator_source.as_deref(),
        boundary_source.as_deref(),
        &responsibility,
        authority,
    )?;
    let validation = string(value, "validation").map_err(failure)?.to_owned();
    if !integrity.validations.contains_key(&validation) {
        return Err(failure("unknown validation ID"));
    }
    let target = value
        .get("target")
        .map(|value| parse_target(value, catalog))
        .transpose()?;
    let binding = value
        .get("binding")
        .map(|_| string(value, "binding").map(str::to_owned).map_err(failure))
        .transpose()?;
    if let Some(binding_id) = &binding {
        let binding = bindings
            .and_then(|registry| registry.bindings.get(binding_id))
            .ok_or_else(|| failure("unknown Governance Binding"))?;
        if binding.authority.responsibility != responsibility
            || binding.authority.source != canonical_source
        {
            return Err(failure("projection and binding authority differ"));
        }
    }
    let projection = Projection {
        id: id.to_owned(),
        responsibility,
        canonical_source,
        secondary_source,
        mode,
        validation,
        generator_source,
        boundary_source,
        target,
        binding,
    };
    Ok((id.to_owned(), projection))
}
fn validate_mode(
    mode: ProjectionMode,
    generator: Option<&str>,
    boundary: Option<&str>,
    responsibility: &str,
    authority: &GovernanceAuthorityProfile,
) -> Result<()> {
    match mode {
        ProjectionMode::Reference | ProjectionMode::MechanicallyValidatedMaintained
            if generator.is_none() && boundary.is_none() =>
        {
            Ok(())
        }
        ProjectionMode::Generated if generator.is_some() && boundary.is_none() => {
            if role_of(authority, responsibility, generator.expect("present"))
                == RoleLookup::Role(SourceRole::NonAuthoritative)
            {
                Ok(())
            } else {
                Err(failure("generator source has wrong role"))
            }
        }
        ProjectionMode::BoundedHistoricalSnapshot if generator.is_none() && boundary.is_some() => {
            if role_of(authority, responsibility, boundary.expect("present"))
                == RoleLookup::Role(SourceRole::Authority)
            {
                Ok(())
            } else {
                Err(failure("boundary source has wrong role"))
            }
        }
        _ => Err(failure("projection mode fields are invalid")),
    }
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
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<GovernedObjectRef> {
    let value = mapping(value, "target").map_err(failure)?;
    exact_keys(value, &["interface", "object"]).map_err(failure)?;
    let target = GovernedObjectRef {
        interface_id: string(value, "interface").map_err(failure)?.to_owned(),
        object_id: string(value, "object").map_err(failure)?.to_owned(),
    };
    catalog
        .and_then(|catalog| catalog.interfaces.get(&target.interface_id))
        .and_then(|interface| interface.objects.get(&target.object_id))
        .ok_or_else(|| failure("unknown governed object target"))?;
    Ok(target)
}
fn reject_active_ambiguity(projections: &BTreeMap<String, Projection>) -> Result<()> {
    let mut active = BTreeMap::new();
    for projection in projections.values() {
        if projection.mode == ProjectionMode::BoundedHistoricalSnapshot {
            continue;
        }
        let key = (
            projection.responsibility.clone(),
            projection.secondary_source.clone(),
            projection.target.clone(),
            projection.binding.clone(),
        );
        let declaration = (projection.canonical_source.clone(), projection.mode);
        if active
            .get(&key)
            .is_some_and(|existing| existing != &declaration)
        {
            return Err(failure("conflicting active projection declarations"));
        }
        active.insert(key, declaration);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn projection(id: &str) -> Projection {
        Projection {
            id: id.to_owned(),
            responsibility: "responsibility".to_owned(),
            canonical_source: "canonical".to_owned(),
            secondary_source: "secondary".to_owned(),
            mode: ProjectionMode::Reference,
            validation: "validation".to_owned(),
            generator_source: None,
            boundary_source: None,
            target: None,
            binding: None,
        }
    }
    fn compatible(left: Projection, right: Projection) -> bool {
        reject_active_ambiguity(&BTreeMap::from([
            (left.id.clone(), left),
            (right.id.clone(), right),
        ]))
        .is_ok()
    }
    #[test]
    fn effective_ambiguity_key_includes_target_and_binding() {
        let left = projection("left");
        let mut distinct_target = projection("target");
        distinct_target.canonical_source = "other".to_owned();
        distinct_target.target = Some(GovernedObjectRef {
            interface_id: "interface".to_owned(),
            object_id: "object".to_owned(),
        });
        assert!(compatible(left, distinct_target));
        let mut binding_one = projection("binding-one");
        binding_one.binding = Some("one".to_owned());
        let mut binding_two = projection("binding-two");
        binding_two.binding = Some("two".to_owned());
        binding_two.mode = ProjectionMode::MechanicallyValidatedMaintained;
        assert!(compatible(binding_one, binding_two));
    }
    #[test]
    fn historical_snapshot_is_excluded_from_active_collision() {
        let left = projection("left");
        let mut historical = projection("historical");
        historical.canonical_source = "other".to_owned();
        historical.mode = ProjectionMode::BoundedHistoricalSnapshot;
        historical.boundary_source = Some("boundary".to_owned());
        assert!(compatible(left, historical));
    }
}
