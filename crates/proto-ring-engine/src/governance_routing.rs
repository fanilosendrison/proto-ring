#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::structured_data::StructuredValue;

#[derive(Clone, Debug)]
pub struct ResolvedGovernanceRoute {
    pub route: Vec<String>,
    pub declared_path: String,
    pub target: PathBuf,
}

#[derive(Debug)]
pub struct GovernanceRoutingError(String);

impl Display for GovernanceRoutingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for GovernanceRoutingError {}

pub fn resolve(
    repository_root: &Path,
    routing: &StructuredValue,
    route: &[String],
) -> Result<ResolvedGovernanceRoute, GovernanceRoutingError> {
    if route.is_empty() || route.iter().any(String::is_empty) {
        return Err(failure("route segments must be nonempty"));
    }
    let mut current = routing;
    for segment in &route[..route.len() - 1] {
        current = match current {
            StructuredValue::Mapping(mapping) => mapping
                .get(segment)
                .ok_or_else(|| failure(format!("missing route segment: {segment}")))?,
            _ => return Err(failure("intermediate route value must be a mapping")),
        };
    }
    let leaf = route.last().expect("nonempty route checked");
    let declared_path = match current {
        StructuredValue::Mapping(mapping) => match mapping.get(leaf) {
            Some(StructuredValue::String(value)) if !value.is_empty() => value.clone(),
            _ => return Err(failure("route leaf must be a nonempty string")),
        },
        _ => return Err(failure("route parent must be a mapping")),
    };
    let resolved_root = repository_root
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve repository root: {error}")))?;
    let target = resolve_filesystem_path(&resolved_root, &declared_path)?;
    if !target.starts_with(&resolved_root) {
        return Err(failure("resolved target is outside repository root"));
    }
    Ok(ResolvedGovernanceRoute {
        route: route.to_vec(),
        declared_path,
        target,
    })
}

fn resolve_filesystem_path(
    resolved_root: &Path,
    declared_path: &str,
) -> Result<PathBuf, GovernanceRoutingError> {
    let declared = Path::new(declared_path);
    if declared.is_absolute() {
        return Err(failure("declared path must be repository-relative"));
    }
    let mut current = resolved_root.to_path_buf();
    for component in declared.components() {
        match component {
            Component::Prefix(prefix) => current.push(prefix.as_os_str()),
            Component::RootDir => current.push(Path::new("/")),
            Component::Normal(value) => {
                current.push(value);
                current = current.canonicalize().map_err(|error| {
                    failure(format!("cannot resolve declared path component: {error}"))
                })?;
            }
            Component::CurDir => require_directory(&current)?,
            Component::ParentDir => {
                require_directory(&current)?;
                current.pop();
            }
        }
    }
    if declared_path.ends_with('/') || declared_path.ends_with("/.") {
        require_directory(&current)?;
    }
    Ok(current)
}

fn require_directory(path: &Path) -> Result<(), GovernanceRoutingError> {
    let metadata = fs::metadata(path)
        .map_err(|error| failure(format!("cannot inspect path component: {error}")))?;
    if !metadata.is_dir() {
        return Err(failure("path traversal requires a directory"));
    }
    Ok(())
}

fn failure(message: impl Into<String>) -> GovernanceRoutingError {
    GovernanceRoutingError(message.into())
}
