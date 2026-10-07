#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use crate::structured_data::{self, StructuredValue};

#[derive(Clone, Debug)]
pub struct GovernanceBootstrap {
    pub metadata: StructuredValue,
    pub repository_governance: BTreeMap<String, StructuredValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GovernanceBootstrapErrorKind {
    CarrierIo,
    StructuredData,
    Structure,
}

#[derive(Debug)]
pub struct GovernanceBootstrapError {
    pub kind: GovernanceBootstrapErrorKind,
    message: String,
}

impl Display for GovernanceBootstrapError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for GovernanceBootstrapError {}

pub fn load(repository_root: &Path) -> Result<GovernanceBootstrap, GovernanceBootstrapError> {
    let bytes = fs::read(repository_root.join("AGENTS.md"))
        .map_err(|error| failure(GovernanceBootstrapErrorKind::CarrierIo, error.to_string()))?;
    let parsed = structured_data::parse_frontmatter_bytes(&bytes).map_err(|error| {
        failure(
            GovernanceBootstrapErrorKind::StructuredData,
            error.to_string(),
        )
    })?;
    let metadata = parsed.metadata;
    let root = match &metadata {
        StructuredValue::Mapping(root) => root,
        _ => {
            return Err(failure(
                GovernanceBootstrapErrorKind::Structure,
                "frontmatter root must be a mapping",
            ));
        }
    };
    let repository_governance = match root.get("repository_governance") {
        Some(StructuredValue::Mapping(value)) => value.clone(),
        _ => {
            return Err(failure(
                GovernanceBootstrapErrorKind::Structure,
                "repository_governance must be a mapping",
            ));
        }
    };
    Ok(GovernanceBootstrap {
        metadata,
        repository_governance,
    })
}

fn failure(
    kind: GovernanceBootstrapErrorKind,
    message: impl Into<String>,
) -> GovernanceBootstrapError {
    GovernanceBootstrapError {
        kind,
        message: message.into(),
    }
}
