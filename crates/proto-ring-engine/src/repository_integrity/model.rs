use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::governed_objects::GovernedObjectRef;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileAuthority {
    pub responsibility: String,
    pub source: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorKind {
    Path,
    Glob,
}
impl SelectorKind {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Glob => "glob",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryPathSelector {
    pub kind: SelectorKind,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepositoryPathsMode {
    AppendAll,
    ForEach,
}
impl RepositoryPathsMode {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::AppendAll => "append_all",
            Self::ForEach => "for_each",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationInstances {
    Single,
    RepositoryPaths {
        mode: RepositoryPathsMode,
        selectors: Vec<RepositoryPathSelector>,
    },
}
impl ValidationInstances {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::RepositoryPaths { .. } => "repository_paths",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandBinding {
    pub environment: String,
    pub arguments: Vec<String>,
    pub undetermined_exit_codes: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationDefinition {
    pub id: String,
    pub responsibility: String,
    pub prerequisites: BTreeSet<String>,
    pub instances: ValidationInstances,
    pub command: CommandBinding,
    pub target: Option<GovernedObjectRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileIdentity(pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct ConsumerIntegrityProfile {
    pub repository: PathBuf,
    pub carrier: PathBuf,
    pub model_version: u8,
    pub authority: ProfileAuthority,
    pub environments: BTreeSet<String>,
    pub continue_after_non_satisfied: bool,
    pub validations: BTreeMap<String, ValidationDefinition>,
    pub order: Vec<String>,
    pub identity: ProfileIdentity,
}
