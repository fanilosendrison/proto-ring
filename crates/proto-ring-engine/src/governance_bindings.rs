//! Validation-only loading of immutable governance binding declarations.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::declaration_support::{
    exact_keys, mapping, model_version_one, required_mapping, string,
};
use crate::governance_authority::{GovernanceAuthorityProfile, RoleLookup, SourceRole, role_of};
use crate::governance_routing::ResolvedGovernanceRoute;
use crate::governed_objects::GovernedObjectCatalog;
use crate::structured_data::{self, StructuredValue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingKind {
    ExecutableProvider,
    GovernanceContract,
}
impl BindingKind {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::ExecutableProvider => "executable_provider",
            Self::GovernanceContract => "governance_contract",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScopeKind {
    LogicalProvider,
    Capability,
    GovernedObject,
}
impl ScopeKind {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::LogicalProvider => "logical_provider",
            Self::Capability => "capability",
            Self::GovernedObject => "governed_object",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GovernanceBindingScope {
    pub kind: ScopeKind,
    pub capability: Option<String>,
    pub interface: Option<String>,
    pub object: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableProviderIdentity {
    pub repository: String,
    pub commit: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceContractIdentity {
    pub repository: String,
    pub commit: String,
    pub path: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GovernanceBindingIdentity {
    ExecutableProvider(ExecutableProviderIdentity),
    GovernanceContract(GovernanceContractIdentity),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingAuthority {
    pub responsibility: String,
    pub source: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceBinding {
    pub id: String,
    pub kind: BindingKind,
    pub scope: GovernanceBindingScope,
    pub identity: GovernanceBindingIdentity,
    pub authority: BindingAuthority,
}
#[derive(Clone, Debug)]
pub struct GovernanceBindingRegistry {
    pub repository: PathBuf,
    pub carrier: PathBuf,
    pub model_version: u8,
    pub source: String,
    pub bindings: BTreeMap<String, GovernanceBinding>,
}

#[derive(Debug)]
pub struct GovernanceBindingsError(String);
impl Display for GovernanceBindingsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl Error for GovernanceBindingsError {}
fn failure(message: impl Into<String>) -> GovernanceBindingsError {
    GovernanceBindingsError(message.into())
}

type Result<T> = std::result::Result<T, GovernanceBindingsError>;

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
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<GovernanceBindingRegistry> {
    let canonical_root = repository
        .canonicalize()
        .map_err(|error| failure(error.to_string()))?;
    require_same_repository(&canonical_root, &authority.repository, "authority")?;
    if let Some(catalog) = catalog {
        require_same_repository(&canonical_root, &catalog.repository, "catalog")?;
    }
    let bytes = fs::read(&route.target).map_err(|error| failure(error.to_string()))?;
    let parsed = structured_data::parse_frontmatter_bytes(&bytes)
        .map_err(|error| failure(error.to_string()))?;
    let metadata = mapping(&parsed.metadata, "frontmatter").map_err(failure)?;
    let root = required_mapping(metadata, "governance_bindings").map_err(failure)?;
    exact_keys(root, &["model_version", "source", "bindings"]).map_err(failure)?;
    model_version_one(root).map_err(failure)?;
    let source = string(root, "source").map_err(failure)?.to_owned();
    let governed_source = authority
        .sources
        .get(&source)
        .ok_or_else(|| failure("unknown registry source"))?;
    let source_target = governed_source
        .repository_target
        .as_ref()
        .ok_or_else(|| failure("registry source has no repository target"))?;
    if source_target.target != route.target {
        return Err(failure("registry source does not resolve to carrier"));
    }
    let declarations = required_mapping(root, "bindings").map_err(failure)?;
    let mut bindings = BTreeMap::new();
    for (id, value) in declarations {
        if id.is_empty() {
            return Err(failure("binding ID must be nonempty"));
        }
        let binding = parse_binding(id, value, &source, authority, catalog)?;
        bindings.insert(id.clone(), binding);
    }
    if bindings
        .values()
        .filter(|binding| binding.kind == BindingKind::ExecutableProvider)
        .count()
        != 1
    {
        return Err(failure("exactly one executable provider is required"));
    }
    reject_contract_ambiguity(&bindings)?;
    Ok(GovernanceBindingRegistry {
        repository: canonical_root,
        carrier: route.target.clone(),
        model_version: 1,
        source,
        bindings,
    })
}

fn parse_binding(
    id: &str,
    value: &StructuredValue,
    registry_source: &str,
    authority: &GovernanceAuthorityProfile,
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<GovernanceBinding> {
    let value = mapping(value, "binding").map_err(failure)?;
    exact_keys(value, &["authority", "identity", "kind", "scope"]).map_err(failure)?;
    let kind = match string(value, "kind").map_err(failure)? {
        "executable_provider" => BindingKind::ExecutableProvider,
        "governance_contract" => BindingKind::GovernanceContract,
        _ => return Err(failure("unknown binding kind")),
    };
    let scope = parse_scope(value.get("scope").expect("exact key exists"), catalog)?;
    if kind == BindingKind::ExecutableProvider && scope.kind != ScopeKind::LogicalProvider {
        return Err(failure(
            "executable provider requires logical provider scope",
        ));
    }
    let identity = parse_identity(kind, value.get("identity").expect("exact key exists"))?;
    let authority_declaration = required_mapping(value, "authority").map_err(failure)?;
    exact_keys(authority_declaration, &["responsibility", "source"]).map_err(failure)?;
    let binding_authority = BindingAuthority {
        responsibility: string(authority_declaration, "responsibility")
            .map_err(failure)?
            .to_owned(),
        source: string(authority_declaration, "source")
            .map_err(failure)?
            .to_owned(),
    };
    if !authority.sources.contains_key(&binding_authority.source) {
        return Err(failure("unknown authority source"));
    }
    if role_of(
        authority,
        &binding_authority.responsibility,
        &binding_authority.source,
    ) != RoleLookup::Role(SourceRole::Authority)
    {
        return Err(failure("binding source is not authority"));
    }
    if binding_authority.source != registry_source
        && role_of(
            authority,
            &binding_authority.responsibility,
            registry_source,
        ) != RoleLookup::Role(SourceRole::SecondaryRepresentation)
    {
        return Err(failure("registry source is not secondary representation"));
    }
    Ok(GovernanceBinding {
        id: id.to_owned(),
        kind,
        scope,
        identity,
        authority: binding_authority,
    })
}

fn parse_scope(
    value: &StructuredValue,
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<GovernanceBindingScope> {
    let value = mapping(value, "scope").map_err(failure)?;
    let kind = match value.get("kind") {
        Some(StructuredValue::String(kind)) if !kind.is_empty() => kind.as_str(),
        _ => return Err(failure("scope kind must be nonempty string")),
    };
    match kind {
        "logical_provider" => {
            exact_keys(value, &["kind"]).map_err(failure)?;
            Ok(GovernanceBindingScope {
                kind: ScopeKind::LogicalProvider,
                capability: None,
                interface: None,
                object: None,
            })
        }
        "capability" => {
            exact_keys(value, &["capability", "kind"]).map_err(failure)?;
            Ok(GovernanceBindingScope {
                kind: ScopeKind::Capability,
                capability: Some(string(value, "capability").map_err(failure)?.to_owned()),
                interface: None,
                object: None,
            })
        }
        "governed_object" => {
            exact_keys(value, &["interface", "kind", "object"]).map_err(failure)?;
            let interface = string(value, "interface").map_err(failure)?.to_owned();
            let object = string(value, "object").map_err(failure)?.to_owned();
            let catalog =
                catalog.ok_or_else(|| failure("governed object scope requires catalog"))?;
            if !catalog
                .interfaces
                .get(&interface)
                .is_some_and(|entry| entry.objects.contains_key(&object))
            {
                return Err(failure("unknown governed object"));
            }
            Ok(GovernanceBindingScope {
                kind: ScopeKind::GovernedObject,
                capability: None,
                interface: Some(interface),
                object: Some(object),
            })
        }
        _ => Err(failure("unknown scope kind")),
    }
}

fn parse_identity(kind: BindingKind, value: &StructuredValue) -> Result<GovernanceBindingIdentity> {
    let value = mapping(value, "identity").map_err(failure)?;
    let expected = if kind == BindingKind::ExecutableProvider {
        &["commit", "repository"][..]
    } else {
        &["commit", "path", "repository"][..]
    };
    exact_keys(value, expected).map_err(failure)?;
    let repository = string(value, "repository").map_err(failure)?.to_owned();
    let commit = string(value, "commit").map_err(failure)?.to_owned();
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(failure(
            "commit must be 40 lowercase hexadecimal characters",
        ));
    }
    Ok(match kind {
        BindingKind::ExecutableProvider => {
            GovernanceBindingIdentity::ExecutableProvider(ExecutableProviderIdentity {
                repository,
                commit,
            })
        }
        BindingKind::GovernanceContract => {
            GovernanceBindingIdentity::GovernanceContract(GovernanceContractIdentity {
                repository,
                commit,
                path: string(value, "path").map_err(failure)?.to_owned(),
            })
        }
    })
}

fn reject_contract_ambiguity(bindings: &BTreeMap<String, GovernanceBinding>) -> Result<()> {
    let mut commits = BTreeMap::new();
    for binding in bindings.values() {
        let GovernanceBindingIdentity::GovernanceContract(identity) = &binding.identity else {
            continue;
        };
        let key = (
            binding.scope.clone(),
            identity.repository.clone(),
            identity.path.clone(),
        );
        if commits
            .get(&key)
            .is_some_and(|commit| commit != &identity.commit)
        {
            return Err(failure("ambiguous governance contract commits"));
        }
        commits.insert(key, identity.commit.clone());
    }
    Ok(())
}
