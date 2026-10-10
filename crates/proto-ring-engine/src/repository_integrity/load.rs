use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::declaration_support::{
    boolean, exact_keys, mapping, model_version_one, required_mapping, sequence, string,
};
use crate::governance_authority::{GovernanceAuthorityProfile, RoleLookup, SourceRole, role_of};
use crate::governance_routing::ResolvedGovernanceRoute;
use crate::governed_objects::{GovernedObjectCatalog, GovernedObjectRef};
use crate::structured_data::{self, StructuredValue};

use super::identity::profile_identity;
use super::model::*;
use super::selectors::validate_selector;
use super::{RepositoryIntegrityError, failure};

type Result<T> = std::result::Result<T, RepositoryIntegrityError>;

pub fn load(
    repository: &Path,
    route: &ResolvedGovernanceRoute,
    authority: &GovernanceAuthorityProfile,
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<ConsumerIntegrityProfile> {
    let canonical_root = repository
        .canonicalize()
        .map_err(|error| failure(error.to_string()))?;
    let authority_root = authority
        .repository
        .canonicalize()
        .map_err(|error| failure(error.to_string()))?;
    if canonical_root != authority_root {
        return Err(failure("profile and authority repositories differ"));
    }
    if let Some(catalog) = catalog {
        let catalog_root = catalog
            .repository
            .canonicalize()
            .map_err(|error| failure(error.to_string()))?;
        if canonical_root != catalog_root {
            return Err(failure("profile and catalog repositories differ"));
        }
    }
    let bytes = fs::read(&route.target).map_err(|error| failure(error.to_string()))?;
    let parsed = structured_data::parse_frontmatter_bytes(&bytes)
        .map_err(|error| failure(error.to_string()))?;
    let metadata = mapping(&parsed.metadata, "frontmatter").map_err(failure)?;
    let root = required_mapping(metadata, "repository_integrity").map_err(failure)?;
    exact_keys(
        root,
        &[
            "authority",
            "continue_after_non_satisfied",
            "environments",
            "model_version",
            "order",
            "validations",
        ],
    )
    .map_err(failure)?;
    model_version_one(root).map_err(failure)?;
    let profile_authority = parse_authority(
        required_mapping(root, "authority").map_err(failure)?,
        authority,
        route,
    )?;
    let environments = unique_strings(
        sequence(root, "environments").map_err(failure)?,
        "environment",
    )?;
    let continue_after_non_satisfied =
        boolean(root, "continue_after_non_satisfied").map_err(failure)?;
    let validations = parse_validations(
        required_mapping(root, "validations").map_err(failure)?,
        &environments,
        authority,
        catalog,
    )?;
    let order = ordered_strings(sequence(root, "order").map_err(failure)?, "order ID")?;
    validate_order(&validations, &order)?;
    validate_prerequisites(&validations, &order)?;
    let mut profile = ConsumerIntegrityProfile {
        repository: canonical_root,
        carrier: route.target.clone(),
        model_version: 1,
        authority: profile_authority,
        environments,
        continue_after_non_satisfied,
        validations,
        order,
        identity: ProfileIdentity(Vec::new()),
    };
    profile.identity = profile_identity(&profile);
    Ok(profile)
}

fn parse_authority(
    value: &BTreeMap<String, StructuredValue>,
    authority: &GovernanceAuthorityProfile,
    route: &ResolvedGovernanceRoute,
) -> Result<ProfileAuthority> {
    exact_keys(value, &["responsibility", "source"]).map_err(failure)?;
    let responsibility = string(value, "responsibility").map_err(failure)?.to_owned();
    let source = string(value, "source").map_err(failure)?.to_owned();
    if !authority.sources.contains_key(&source) {
        return Err(failure("unknown profile authority source"));
    }
    if role_of(authority, &responsibility, &source) != RoleLookup::Role(SourceRole::Authority) {
        return Err(failure("profile source is not authority"));
    }
    let target = authority.sources[&source]
        .repository_target
        .as_ref()
        .ok_or_else(|| failure("profile authority has no repository target"))?;
    if target.target != route.target {
        return Err(failure(
            "profile authority source does not resolve to carrier",
        ));
    }
    Ok(ProfileAuthority {
        responsibility,
        source,
    })
}

fn parse_validations(
    values: &BTreeMap<String, StructuredValue>,
    environments: &BTreeSet<String>,
    authority: &GovernanceAuthorityProfile,
    catalog: Option<&GovernedObjectCatalog>,
) -> Result<BTreeMap<String, ValidationDefinition>> {
    values
        .iter()
        .map(|(id, value)| {
            if id.is_empty() {
                return Err(failure("validation ID must be nonempty"));
            }
            let declaration = mapping(value, "validation").map_err(failure)?;
            let keys = if declaration.contains_key("target") {
                &[
                    "command",
                    "instances",
                    "prerequisites",
                    "responsibility",
                    "target",
                ][..]
            } else {
                &["command", "instances", "prerequisites", "responsibility"][..]
            };
            exact_keys(declaration, keys).map_err(failure)?;
            let responsibility = string(declaration, "responsibility")
                .map_err(failure)?
                .to_owned();
            if !authority.responsibilities.contains_key(&responsibility) {
                return Err(failure("unknown validation responsibility"));
            }
            let prerequisites = unique_strings(
                sequence(declaration, "prerequisites").map_err(failure)?,
                "prerequisite",
            )?;
            let instances =
                parse_instances(declaration.get("instances").expect("exact key exists"))?;
            let command = parse_command(
                required_mapping(declaration, "command").map_err(failure)?,
                environments,
            )?;
            let target = declaration
                .get("target")
                .map(|value| parse_target(value, &responsibility, catalog))
                .transpose()?;
            Ok((
                id.clone(),
                ValidationDefinition {
                    id: id.clone(),
                    responsibility,
                    prerequisites,
                    instances,
                    command,
                    target,
                },
            ))
        })
        .collect()
}

fn parse_instances(value: &StructuredValue) -> Result<ValidationInstances> {
    let value = mapping(value, "instances").map_err(failure)?;
    let kind = match value.get("kind") {
        Some(StructuredValue::String(value)) if !value.is_empty() => value.as_str(),
        _ => return Err(failure("instance kind must be nonempty string")),
    };
    match kind {
        "single" => {
            exact_keys(value, &["kind"]).map_err(failure)?;
            Ok(ValidationInstances::Single)
        }
        "repository_paths" => {
            exact_keys(value, &["kind", "mode", "selectors"]).map_err(failure)?;
            let mode = match string(value, "mode").map_err(failure)? {
                "append_all" => RepositoryPathsMode::AppendAll,
                "for_each" => RepositoryPathsMode::ForEach,
                _ => return Err(failure("unknown repository paths mode")),
            };
            let selectors = sequence(value, "selectors")
                .map_err(failure)?
                .iter()
                .map(parse_selector)
                .collect::<Result<_>>()?;
            Ok(ValidationInstances::RepositoryPaths { mode, selectors })
        }
        _ => Err(failure("unknown instance kind")),
    }
}

fn parse_selector(value: &StructuredValue) -> Result<RepositoryPathSelector> {
    let value = mapping(value, "selector").map_err(failure)?;
    let kind = match value.get("kind") {
        Some(StructuredValue::String(value)) if !value.is_empty() => value.as_str(),
        _ => return Err(failure("selector kind must be nonempty string")),
    };
    match kind {
        "path" => {
            exact_keys(value, &["kind", "path"]).map_err(failure)?;
            validate_selector(
                SelectorKind::Path,
                string(value, "path").map_err(failure)?.to_owned(),
            )
        }
        "glob" => {
            exact_keys(value, &["glob", "kind"]).map_err(failure)?;
            validate_selector(
                SelectorKind::Glob,
                string(value, "glob").map_err(failure)?.to_owned(),
            )
        }
        _ => Err(failure("unknown selector kind")),
    }
}

fn parse_command(
    value: &BTreeMap<String, StructuredValue>,
    environments: &BTreeSet<String>,
) -> Result<CommandBinding> {
    exact_keys(
        value,
        &[
            "arguments",
            "environment",
            "kind",
            "undetermined_exit_codes",
        ],
    )
    .map_err(failure)?;
    if string(value, "kind").map_err(failure)? != "command" {
        return Err(failure("unknown command kind"));
    }
    let environment = string(value, "environment").map_err(failure)?.to_owned();
    if !environments.contains(&environment) {
        return Err(failure("undeclared command environment"));
    }
    let arguments = sequence(value, "arguments")
        .map_err(failure)?
        .iter()
        .map(|value| match value {
            StructuredValue::String(value) => Ok(value.clone()),
            _ => Err(failure("command argument must be string")),
        })
        .collect::<Result<_>>()?;
    let mut undetermined_exit_codes = BTreeSet::new();
    for value in sequence(value, "undetermined_exit_codes").map_err(failure)? {
        let StructuredValue::Integer(code) = value else {
            return Err(failure("exit code must be integer"));
        };
        if code == "0" || !undetermined_exit_codes.insert(code.clone()) {
            return Err(failure("exit codes must be unique nonzero integers"));
        }
    }
    Ok(CommandBinding {
        environment,
        arguments,
        undetermined_exit_codes,
    })
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

fn unique_strings(values: &[StructuredValue], label: &str) -> Result<BTreeSet<String>> {
    let mut output = BTreeSet::new();
    for value in values {
        let StructuredValue::String(value) = value else {
            return Err(failure(format!("{label} must be string")));
        };
        if value.is_empty() || !output.insert(value.clone()) {
            return Err(failure(format!("{label} must be nonempty and unique")));
        }
    }
    Ok(output)
}
fn ordered_strings(values: &[StructuredValue], label: &str) -> Result<Vec<String>> {
    values
        .iter()
        .map(|value| match value {
            StructuredValue::String(value) if !value.is_empty() => Ok(value.clone()),
            _ => Err(failure(format!("{label} must be nonempty string"))),
        })
        .collect()
}
fn validate_order(
    validations: &BTreeMap<String, ValidationDefinition>,
    order: &[String],
) -> Result<()> {
    let identities: BTreeSet<_> = order.iter().collect();
    if identities.len() != order.len()
        || order.len() != validations.len()
        || validations.keys().any(|id| !identities.contains(id))
    {
        Err(failure("order must contain every validation exactly once"))
    } else {
        Ok(())
    }
}
fn validate_prerequisites(
    validations: &BTreeMap<String, ValidationDefinition>,
    order: &[String],
) -> Result<()> {
    let positions: BTreeMap<_, _> = order
        .iter()
        .enumerate()
        .map(|(index, id)| (id, index))
        .collect();
    for validation in validations.values() {
        for prerequisite in &validation.prerequisites {
            let Some(position) = positions.get(prerequisite) else {
                return Err(failure("unknown prerequisite"));
            };
            if prerequisite == &validation.id || *position >= positions[&validation.id] {
                return Err(failure("prerequisite must precede dependent"));
            }
        }
    }
    Ok(())
}
