#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::governance_routing::{self, ResolvedGovernanceRoute};
use crate::structured_data::{self, StructuredValue};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceRole {
    Authority,
    SecondaryRepresentation,
    NonAuthoritative,
}

impl SourceRole {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Authority => "authority",
            Self::SecondaryRepresentation => "secondary_representation",
            Self::NonAuthoritative => "non_authoritative",
        }
    }
}
#[derive(Clone, Debug)]
pub struct GovernedSource {
    pub id: String,
    pub repository_target: Option<ResolvedGovernanceRoute>,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PrecedenceEdge {
    pub higher_source: String,
    pub lower_source: String,
}
#[derive(Clone, Debug)]
pub struct GovernedResponsibility {
    pub id: String,
    pub roles: BTreeMap<String, SourceRole>,
    pub precedence: Vec<PrecedenceEdge>,
}

#[derive(Clone, Debug)]
pub struct GovernanceAuthorityProfile {
    pub repository: PathBuf,
    pub carrier: PathBuf,
    pub model_version: u8,
    pub sources: BTreeMap<String, GovernedSource>,
    pub responsibilities: BTreeMap<String, GovernedResponsibility>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoleLookup {
    Role(SourceRole),
    UnknownResponsibility,
    Undeclared,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResponsibilityLookup<T> {
    Value(T),
    UnknownResponsibility,
}

#[derive(Debug)]
pub struct GovernanceAuthorityError(String);

impl Display for GovernanceAuthorityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for GovernanceAuthorityError {}

pub fn load(
    repository_root: &Path,
    profile_route: &ResolvedGovernanceRoute,
) -> Result<GovernanceAuthorityProfile, GovernanceAuthorityError> {
    let canonical_root = repository_root
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve repository root: {error}")))?;
    let bytes = fs::read(&profile_route.target)
        .map_err(|error| failure(format!("cannot read authority profile: {error}")))?;
    let parsed = structured_data::parse_frontmatter_bytes(&bytes)
        .map_err(|error| failure(format!("cannot parse authority profile: {error}")))?;
    let metadata = value_mapping(&parsed.metadata, "frontmatter must be a mapping")?;
    let profile = required_mapping(metadata, "governance_authority")?;
    require_keys(profile, &["model_version", "responsibilities", "sources"])?;
    match profile.get("model_version") {
        Some(StructuredValue::Integer(value)) if value == "1" => {}
        _ => return Err(failure("unsupported authority model version")),
    }
    let source_declarations = required_mapping(profile, "sources")?;
    let sources = load_sources(&canonical_root, source_declarations)?;
    let responsibilities =
        load_responsibilities(required_mapping(profile, "responsibilities")?, &sources)?;
    let used: BTreeSet<_> = responsibilities
        .values()
        .flat_map(|responsibility| responsibility.roles.keys().cloned())
        .collect();
    if sources.keys().any(|source| !used.contains(source)) {
        return Err(failure("declared source is unused"));
    }
    Ok(GovernanceAuthorityProfile {
        repository: canonical_root,
        carrier: profile_route.target.clone(),
        model_version: 1,
        sources,
        responsibilities,
    })
}

pub fn role_of(
    profile: &GovernanceAuthorityProfile,
    responsibility_id: &str,
    source_id: &str,
) -> RoleLookup {
    let Some(responsibility) = profile.responsibilities.get(responsibility_id) else {
        return RoleLookup::UnknownResponsibility;
    };
    responsibility
        .roles
        .get(source_id)
        .copied()
        .map(RoleLookup::Role)
        .unwrap_or(RoleLookup::Undeclared)
}

pub fn sources_with_role(
    profile: &GovernanceAuthorityProfile,
    responsibility_id: &str,
    role: SourceRole,
) -> ResponsibilityLookup<BTreeSet<String>> {
    let Some(responsibility) = profile.responsibilities.get(responsibility_id) else {
        return ResponsibilityLookup::UnknownResponsibility;
    };
    ResponsibilityLookup::Value(
        responsibility
            .roles
            .iter()
            .filter_map(|(source, declared)| (*declared == role).then_some(source.clone()))
            .collect(),
    )
}

pub fn authority_sources(
    profile: &GovernanceAuthorityProfile,
    responsibility_id: &str,
) -> ResponsibilityLookup<BTreeSet<String>> {
    sources_with_role(profile, responsibility_id, SourceRole::Authority)
}

pub fn outranks(
    profile: &GovernanceAuthorityProfile,
    responsibility_id: &str,
    higher_source: &str,
    lower_source: &str,
) -> ResponsibilityLookup<bool> {
    let Some(responsibility) = profile.responsibilities.get(responsibility_id) else {
        return ResponsibilityLookup::UnknownResponsibility;
    };
    let mut successors: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &responsibility.precedence {
        successors
            .entry(&edge.higher_source)
            .or_default()
            .push(&edge.lower_source);
    }
    let mut pending = successors.get(higher_source).cloned().unwrap_or_default();
    let mut visited = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if current == lower_source {
            return ResponsibilityLookup::Value(true);
        }
        if visited.insert(current) {
            pending.extend(successors.get(current).into_iter().flatten().copied());
        }
    }
    ResponsibilityLookup::Value(false)
}

fn load_sources(
    repository_root: &Path,
    declarations: &BTreeMap<String, StructuredValue>,
) -> Result<BTreeMap<String, GovernedSource>, GovernanceAuthorityError> {
    let routing = StructuredValue::Mapping(declarations.clone());
    declarations
        .iter()
        .map(|(source_id, value)| {
            require_identifier(source_id, "source ID")?;
            let declaration = value_mapping(value, "source declaration must be a mapping")?;
            let repository_target = if declaration.is_empty() {
                None
            } else {
                require_keys(declaration, &["repository_target"])?;
                required_string(declaration, "repository_target")?;
                Some(
                    governance_routing::resolve(
                        repository_root,
                        &routing,
                        &[source_id.clone(), "repository_target".to_owned()],
                    )
                    .map_err(|error| failure(format!("source routing failed: {error}")))?,
                )
            };
            Ok((
                source_id.clone(),
                GovernedSource {
                    id: source_id.clone(),
                    repository_target,
                },
            ))
        })
        .collect()
}

fn load_responsibilities(
    declarations: &BTreeMap<String, StructuredValue>,
    sources: &BTreeMap<String, GovernedSource>,
) -> Result<BTreeMap<String, GovernedResponsibility>, GovernanceAuthorityError> {
    declarations
        .iter()
        .map(|(responsibility_id, value)| {
            require_identifier(responsibility_id, "responsibility ID")?;
            let declaration = value_mapping(value, "responsibility must be a mapping")?;
            require_keys(declaration, &["precedence", "roles"])?;
            let roles = load_roles(required_mapping(declaration, "roles")?, sources)?;
            let precedence = load_precedence(
                declaration
                    .get("precedence")
                    .ok_or_else(|| failure("precedence is missing"))?,
                &roles,
            )?;
            if roles
                .values()
                .any(|role| *role == SourceRole::SecondaryRepresentation)
                && !roles.values().any(|role| *role == SourceRole::Authority)
            {
                return Err(failure("secondary representation requires authority"));
            }
            Ok((
                responsibility_id.clone(),
                GovernedResponsibility {
                    id: responsibility_id.clone(),
                    roles,
                    precedence,
                },
            ))
        })
        .collect()
}

fn load_roles(
    declarations: &BTreeMap<String, StructuredValue>,
    sources: &BTreeMap<String, GovernedSource>,
) -> Result<BTreeMap<String, SourceRole>, GovernanceAuthorityError> {
    declarations
        .iter()
        .map(|(source_id, value)| {
            require_identifier(source_id, "role source ID")?;
            if !sources.contains_key(source_id) {
                return Err(failure("role source is not declared"));
            }
            let StructuredValue::String(role) = value else {
                return Err(failure("source role must be a string"));
            };
            let role = match role.as_str() {
                "authority" => SourceRole::Authority,
                "secondary_representation" => SourceRole::SecondaryRepresentation,
                "non_authoritative" => SourceRole::NonAuthoritative,
                _ => return Err(failure("unknown source role")),
            };
            Ok((source_id.clone(), role))
        })
        .collect()
}

fn load_precedence(
    value: &StructuredValue,
    roles: &BTreeMap<String, SourceRole>,
) -> Result<Vec<PrecedenceEdge>, GovernanceAuthorityError> {
    let StructuredValue::Sequence(declarations) = value else {
        return Err(failure("precedence must be a sequence"));
    };
    let mut edges = Vec::new();
    let mut identities = BTreeSet::new();
    for value in declarations {
        let declaration = value_mapping(value, "precedence edge must be a mapping")?;
        require_keys(declaration, &["higher_source", "lower_source"])?;
        let higher_source = required_string(declaration, "higher_source")?.to_owned();
        let lower_source = required_string(declaration, "lower_source")?.to_owned();
        if higher_source == lower_source {
            return Err(failure("precedence self edge is invalid"));
        }
        for source in [&higher_source, &lower_source] {
            if roles.get(source.as_str()) != Some(&SourceRole::Authority) {
                return Err(failure("precedence endpoint is not an authority"));
            }
        }
        if !identities.insert((higher_source.clone(), lower_source.clone())) {
            return Err(failure("duplicate precedence edge"));
        }
        edges.push(PrecedenceEdge {
            higher_source,
            lower_source,
        });
    }
    reject_cycle(&edges)?;
    Ok(edges)
}

fn reject_cycle(edges: &[PrecedenceEdge]) -> Result<(), GovernanceAuthorityError> {
    let mut successors: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut indegree = BTreeMap::new();
    for edge in edges {
        successors
            .entry(&edge.higher_source)
            .or_default()
            .insert(&edge.lower_source);
        indegree
            .entry(edge.higher_source.as_str())
            .or_insert(0_usize);
        *indegree.entry(&edge.lower_source).or_insert(0) += 1;
    }
    let mut pending: Vec<_> = indegree
        .iter()
        .filter_map(|(source, degree)| (*degree == 0).then_some(*source))
        .collect();
    let mut visited = 0;
    while let Some(source) = pending.pop() {
        visited += 1;
        for successor in successors.get(source).into_iter().flatten() {
            let degree = indegree.get_mut(successor).expect("edge endpoint exists");
            *degree -= 1;
            if *degree == 0 {
                pending.push(successor);
            }
        }
    }
    if visited != indegree.len() {
        return Err(failure("precedence cycle is invalid"));
    }
    Ok(())
}

fn require_keys(
    mapping: &BTreeMap<String, StructuredValue>,
    expected: &[&str],
) -> Result<(), GovernanceAuthorityError> {
    if mapping.len() != expected.len() || expected.iter().any(|key| !mapping.contains_key(*key)) {
        return Err(failure("mapping has missing or unknown direct keys"));
    }
    Ok(())
}

fn required_mapping<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a BTreeMap<String, StructuredValue>, GovernanceAuthorityError> {
    mapping
        .get(key)
        .ok_or_else(|| failure(format!("missing {key}")))
        .and_then(|value| value_mapping(value, &format!("{key} must be a mapping")))
}

fn value_mapping<'a>(
    value: &'a StructuredValue,
    message: &str,
) -> Result<&'a BTreeMap<String, StructuredValue>, GovernanceAuthorityError> {
    match value {
        StructuredValue::Mapping(mapping) => Ok(mapping),
        _ => Err(failure(message)),
    }
}

fn required_string<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a str, GovernanceAuthorityError> {
    match mapping.get(key) {
        Some(StructuredValue::String(value)) => {
            require_identifier(value, key)?;
            Ok(value)
        }
        _ => Err(failure(format!("{key} must be a nonempty string"))),
    }
}

fn require_identifier(value: &str, label: &str) -> Result<(), GovernanceAuthorityError> {
    if value.is_empty() {
        Err(failure(format!("{label} must be nonempty")))
    } else {
        Ok(())
    }
}

fn failure(message: impl Into<String>) -> GovernanceAuthorityError {
    GovernanceAuthorityError(message.into())
}
