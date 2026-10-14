#![forbid(unsafe_code)]

mod git;
mod history;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path};

use crate::adr_metadata;
use crate::canonical_adr;
use crate::portable_pattern::PortablePattern;
use crate::repository_governance_model;
use crate::structured_data::StructuredValue;

use git::GitRepository;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcceptedBodyDiagnosticKind {
    BodyChanged,
    HistoricalStateInvalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedBodyDiagnostic {
    pub kind: AcceptedBodyDiagnosticKind,
    pub detail: String,
}

impl AcceptedBodyDiagnostic {
    fn body_changed(detail: impl Into<String>) -> Self {
        Self {
            kind: AcceptedBodyDiagnosticKind::BodyChanged,
            detail: detail.into(),
        }
    }

    fn historical(detail: impl Into<String>) -> Self {
        Self {
            kind: AcceptedBodyDiagnosticKind::HistoricalStateInvalid,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedBodyReport {
    pub diagnostics: Vec<AcceptedBodyDiagnostic>,
}

pub fn check(repository: &Path) -> AcceptedBodyReport {
    match check_repository(repository) {
        Ok(diagnostics) => AcceptedBodyReport { diagnostics },
        Err(detail) => AcceptedBodyReport {
            diagnostics: vec![AcceptedBodyDiagnostic::historical(detail)],
        },
    }
}

fn check_repository(repository: &Path) -> Result<Vec<AcceptedBodyDiagnostic>, String> {
    let git = GitRepository::open(repository).map_err(|error| error.to_string())?;
    let config = load_discovery_config(git.root())?;
    let (seals, mut diagnostics) = history::inspect(&git, &config)?;
    for seal in seals.values() {
        match canonical_adr::resolve(git.root(), &seal.logical_id) {
            Ok(current) if current.decision_body != seal.decision_body => {
                diagnostics.push(AcceptedBodyDiagnostic::body_changed(format!(
                    "current body for {} differs from anchor at commit {}, path {}",
                    seal.logical_id, seal.acceptance_commit, seal.historical_path
                )));
            }
            Ok(_) => {}
            Err(error) => diagnostics.push(AcceptedBodyDiagnostic::historical(format!(
                "cannot resolve current sealed ADR {}: {error}",
                seal.logical_id
            ))),
        }
    }
    Ok(diagnostics)
}

pub(super) struct DiscoveryConfig {
    adr_directory: String,
    filename_pattern: PortablePattern,
}

fn load_discovery_config(repository: &Path) -> Result<DiscoveryConfig, String> {
    let model = repository_governance_model::load(repository)
        .map_err(|error| format!("repository governance model failed: {error}"))?;
    let profile_route = model
        .capabilities
        .get("architecture_decisions")
        .and_then(|capability| capability.routes.get("profile"))
        .ok_or_else(|| "architecture-decisions profile route is missing".to_owned())?;
    let profile_bytes = fs::read(&profile_route.target)
        .map_err(|error| format!("cannot read ADR profile: {error}"))?;
    let profile = adr_metadata::load_yaml(&profile_bytes)
        .map_err(|error| format!("cannot load ADR profile: {error}"))?;
    let root = adr_metadata::require_mapping(&profile)
        .map_err(|error| format!("ADR profile must be a mapping: {error}"))?;
    let repository_profile = required_mapping(root, "repository")?;
    let declared_directory = required_string(repository_profile, "adr_directory")?;
    let filename_source = required_string(repository_profile, "filename_pattern")?;

    let resolved_directory =
        adr_metadata::resolve_repository_path(&model.repository_root, declared_directory)
            .map_err(|error| error.to_string())?;
    if !fs::metadata(&resolved_directory)
        .map_err(|error| format!("cannot inspect ADR directory: {error}"))?
        .is_dir()
    {
        return Err("ADR corpus path is not a directory".to_owned());
    }
    let relative =
        adr_metadata::contained_relative_path(&model.repository_root, &resolved_directory)
            .map_err(|error| error.to_string())?;
    let adr_directory = portable_relative_path(&relative)?;

    let filename_pattern = PortablePattern::compile(filename_source)
        .map_err(|error| format!("invalid filename pattern: {error}"))?;
    if !filename_pattern
        .capture_names()
        .any(|(name, _)| name == "number")
    {
        return Err("filename pattern has no number capture".to_owned());
    }
    Ok(DiscoveryConfig {
        adr_directory,
        filename_pattern,
    })
}

fn required_mapping<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a BTreeMap<String, StructuredValue>, String> {
    mapping
        .get(key)
        .ok_or_else(|| format!("ADR profile field is missing: {key}"))
        .and_then(|value| adr_metadata::require_mapping(value).map_err(|error| error.to_string()))
}

fn required_string<'a>(
    mapping: &'a BTreeMap<String, StructuredValue>,
    key: &str,
) -> Result<&'a str, String> {
    mapping
        .get(key)
        .ok_or_else(|| format!("ADR profile field is missing: {key}"))
        .and_then(|value| adr_metadata::require_string(value).map_err(|error| error.to_string()))
}

fn portable_relative_path(path: &Path) -> Result<String, String> {
    if path.as_os_str().is_empty() {
        return Ok(".".to_owned());
    }
    path.components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| "configured ADR directory is not UTF-8".to_owned()),
            _ => Err("configured ADR directory is not repository-relative".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join("/"))
}
