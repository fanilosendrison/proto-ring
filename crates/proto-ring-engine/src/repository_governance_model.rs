#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use crate::governance_bootstrap;
use crate::governance_routing::{self, ResolvedGovernanceRoute};
use crate::structured_data::StructuredValue;

#[derive(Clone, Debug)]
pub struct RepositoryGovernanceModel {
    pub repository_root: PathBuf,
    pub model_version: u8,
    pub provider: LogicalProvider,
    pub capabilities: BTreeMap<String, Capability>,
}

#[derive(Clone, Debug)]
pub struct LogicalProvider {
    pub id: String,
    pub binding: ProviderBindingRef,
}

#[derive(Clone, Debug)]
pub struct ProviderBindingRef {
    pub capability: String,
    pub route: String,
}

#[derive(Clone, Debug)]
pub struct Capability {
    pub configuration: BTreeMap<String, StructuredValue>,
    pub routes: BTreeMap<String, ResolvedGovernanceRoute>,
}

#[derive(Debug)]
pub struct RepositoryGovernanceModelError(String);

impl Display for RepositoryGovernanceModelError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for RepositoryGovernanceModelError {}

pub fn load(
    repository_root: &Path,
) -> Result<RepositoryGovernanceModel, RepositoryGovernanceModelError> {
    let bootstrap = governance_bootstrap::load(repository_root)
        .map_err(|error| failure(format!("bootstrap failed: {error}")))?;
    let root = &bootstrap.repository_governance;
    require_keys(root, &["capabilities", "model_version", "provider"])?;
    let model_version = match root.get("model_version") {
        Some(StructuredValue::Integer(value)) if value == "1" => 1,
        Some(StructuredValue::Integer(value)) if value == "2" => 2,
        _ => return Err(failure("unsupported model version")),
    };
    let provider = parse_provider(required_mapping(root, "provider")?, model_version)?;
    let declarations = required_mapping(root, "capabilities")?;
    validate_capability_vocabulary(declarations, model_version)?;
    let mut capabilities = BTreeMap::new();
    for (capability_id, value) in declarations {
        let declaration = value_mapping(value, "capability must be a mapping")?;
        require_keys(declaration, &["configuration", "routes"])?;
        let configuration = required_mapping(declaration, "configuration")?.clone();
        let routes = required_mapping(declaration, "routes")?;
        validate_required_route(capability_id, routes, model_version)?;
        let mut resolved_routes = BTreeMap::new();
        for (route_id, route_value) in routes {
            if route_id.is_empty() {
                return Err(failure("route ID must be nonempty"));
            }
            match route_value {
                StructuredValue::String(value) if !value.is_empty() => {}
                _ => return Err(failure("route value must be a nonempty string")),
            }
            let segments = vec![
                "repository_governance".to_owned(),
                "capabilities".to_owned(),
                capability_id.clone(),
                "routes".to_owned(),
                route_id.clone(),
            ];
            let resolved =
                governance_routing::resolve(repository_root, &bootstrap.metadata, &segments)
                    .map_err(|error| failure(format!("route resolution failed: {error}")))?;
            resolved_routes.insert(route_id.clone(), resolved);
        }
        capabilities.insert(
            capability_id.clone(),
            Capability {
                configuration,
                routes: resolved_routes,
            },
        );
    }
    validate_cross_capabilities(&capabilities, model_version)?;
    Ok(RepositoryGovernanceModel {
        repository_root: repository_root
            .canonicalize()
            .map_err(|error| failure(format!("root resolution failed: {error}")))?,
        model_version,
        provider,
        capabilities,
    })
}

pub fn validate_binding_capabilities(
    model: &RepositoryGovernanceModel,
    capability_scopes: &[String],
) -> Result<(), RepositoryGovernanceModelError> {
    if let Some(missing) = capability_scopes
        .iter()
        .find(|capability| !model.capabilities.contains_key(*capability))
    {
        return Err(failure(format!(
            "binding references undeclared capability: {missing}"
        )));
    }
    Ok(())
}

fn parse_provider(
    provider: &BTreeMap<String, StructuredValue>,
    version: u8,
) -> Result<LogicalProvider, RepositoryGovernanceModelError> {
    require_keys(provider, &["binding", "id"])?;
    let id = required_string(provider, "id")?;
    if id != "proto-ring" {
        return Err(failure("provider ID must be proto-ring"));
    }
    let binding = required_mapping(provider, "binding")?;
    require_keys(binding, &["capability", "route"])?;
    let capability = required_string(binding, "capability")?;
    let route = required_string(binding, "route")?;
    let expected_route = if version == 1 { "binding" } else { "registry" };
    if capability != "shared_governance_provider" || route != expected_route {
        return Err(failure("provider binding does not match model version"));
    }
    Ok(LogicalProvider {
        id: id.to_owned(),
        binding: ProviderBindingRef {
            capability: capability.to_owned(),
            route: route.to_owned(),
        },
    })
}

fn validate_capability_vocabulary(
    declarations: &BTreeMap<String, StructuredValue>,
    version: u8,
) -> Result<(), RepositoryGovernanceModelError> {
    let allowed_v1 = [
        "architecture_decisions",
        "governance_authority",
        "governed_objects",
        "shared_governance_provider",
    ];
    let allowed_v2 = [
        "architecture_decisions",
        "authoritative_ref_monotonicity",
        "evidence_requirements",
        "governance_authority",
        "governed_objects",
        "projection_integrity",
        "repository_integrity",
        "shared_governance_provider",
    ];
    let allowed = if version == 1 {
        &allowed_v1[..]
    } else {
        &allowed_v2[..]
    };
    if let Some(unknown) = declarations
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
    {
        return Err(failure(format!("unknown capability: {unknown}")));
    }
    let mandatory = if version == 1 {
        &["shared_governance_provider"][..]
    } else {
        &["governance_authority", "shared_governance_provider"][..]
    };
    if mandatory.iter().any(|key| !declarations.contains_key(*key)) {
        return Err(failure("mandatory capability is missing"));
    }
    Ok(())
}

fn validate_required_route(
    capability: &str,
    routes: &BTreeMap<String, StructuredValue>,
    version: u8,
) -> Result<(), RepositoryGovernanceModelError> {
    let required = match (version, capability) {
        (1, "shared_governance_provider") => Some("binding"),
        (2, "shared_governance_provider") => Some("registry"),
        (_, "architecture_decisions" | "governance_authority" | "governed_objects") => {
            Some("profile")
        }
        (2, "projection_integrity" | "evidence_requirements") => Some("registry"),
        (2, "repository_integrity") => Some("profile"),
        (2, "authoritative_ref_monotonicity") => Some("binding"),
        _ => None,
    };
    if required.is_some_and(|route| !routes.contains_key(route)) {
        return Err(failure(format!("required route missing for {capability}")));
    }
    Ok(())
}

fn validate_cross_capabilities(
    capabilities: &BTreeMap<String, Capability>,
    version: u8,
) -> Result<(), RepositoryGovernanceModelError> {
    let ids: BTreeSet<_> = capabilities.keys().map(String::as_str).collect();
    if ids.contains("governed_objects") && !ids.contains("governance_authority") {
        return Err(failure("governed_objects requires governance_authority"));
    }
    if version == 2 && ids.contains("projection_integrity") && !ids.contains("repository_integrity")
    {
        return Err(failure(
            "projection_integrity requires repository_integrity",
        ));
    }
    Ok(())
}

fn require_keys(
    mapping: &BTreeMap<String, StructuredValue>,
    expected: &[&str],
) -> Result<(), RepositoryGovernanceModelError> {
    if mapping.len() != expected.len() || expected.iter().any(|key| !mapping.contains_key(*key)) {
        return Err(failure("mapping has missing or unknown direct keys"));
    }
    Ok(())
}

fn required_mapping<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a BTreeMap<String, StructuredValue>, RepositoryGovernanceModelError> {
    mapping
        .get(key)
        .ok_or_else(|| failure(format!("missing {key}")))
        .and_then(|value| value_mapping(value, &format!("{key} must be a mapping")))
}

fn value_mapping<'a>(
    value: &'a StructuredValue,
    message: &str,
) -> Result<&'a BTreeMap<String, StructuredValue>, RepositoryGovernanceModelError> {
    match value {
        StructuredValue::Mapping(mapping) => Ok(mapping),
        _ => Err(failure(message)),
    }
}

fn required_string<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a str, RepositoryGovernanceModelError> {
    match mapping.get(key) {
        Some(StructuredValue::String(value)) if !value.is_empty() => Ok(value),
        _ => Err(failure(format!("{key} must be a nonempty string"))),
    }
}

fn failure(message: impl Into<String>) -> RepositoryGovernanceModelError {
    RepositoryGovernanceModelError(message.into())
}
