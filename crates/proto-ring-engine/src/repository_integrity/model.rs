use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::governed_objects::GovernedObjectRef;
use crate::repository_state::RepositoryStateIdentity;

use super::{RepositoryIntegrityError, failure};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationEnvironmentRealization {
    pub environment_id: String,
    pub identity: Vec<u8>,
    pub command_prefix: Vec<String>,
    pub process_environment: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvaluationContext {
    pub realizations: BTreeMap<String, ValidationEnvironmentRealization>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationContextIdentity(pub Vec<u8>);

pub(super) fn evaluation_context_identity(
    profile: &ConsumerIntegrityProfile,
    context: &EvaluationContext,
) -> Result<EvaluationContextIdentity, RepositoryIntegrityError> {
    let required: BTreeSet<_> = profile
        .validations
        .values()
        .map(|definition| definition.command.environment.as_str())
        .collect();
    let mut identity = Vec::new();
    frame(
        &mut identity,
        b"proto-ring evaluation-context opaque identity",
    )?;
    for environment_id in required {
        frame(&mut identity, environment_id.as_bytes())?;
        match context.realizations.get(environment_id) {
            Some(realization) => {
                if realization.environment_id != environment_id {
                    return Err(failure(
                        "required environment realization identity is mismatched",
                    ));
                }
                identity.push(1);
                frame(&mut identity, &realization.identity)?;
            }
            None => identity.push(0),
        }
    }
    Ok(EvaluationContextIdentity(identity))
}

fn frame(output: &mut Vec<u8>, value: &[u8]) -> Result<(), RepositoryIntegrityError> {
    let length =
        u64::try_from(value.len()).map_err(|_| failure("identity component is too large"))?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value);
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) enum EvaluationCapturePhase {
    Baseline,
    PreObligation,
    PostObligation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObligationStatus {
    Satisfied,
    Violated,
    Undetermined,
}
impl ObligationStatus {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Satisfied => "SATISFIED",
            Self::Violated => "VIOLATED",
            Self::Undetermined => "UNDETERMINED",
        }
    }

    pub(super) fn aggregate(results: &[ObligationResult]) -> Self {
        if results.iter().any(|result| result.status == Self::Violated) {
            Self::Violated
        } else if results
            .iter()
            .any(|result| result.status == Self::Undetermined)
        {
            Self::Undetermined
        } else {
            Self::Satisfied
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityVerdict {
    Pass,
    NonPass,
}
impl IntegrityVerdict {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::NonPass => "NON_PASS",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateRelation {
    Equal,
    NotEqual,
    Unavailable,
}
impl StateRelation {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Equal => "EQUAL",
            Self::NotEqual => "NOT_EQUAL",
            Self::Unavailable => "UNAVAILABLE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeCommandObligation {
    pub validation_id: String,
    pub command_prefix: Vec<String>,
    pub arguments: Vec<String>,
    pub undetermined_exit_codes: BTreeSet<String>,
    pub process_environment: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeValidation {
    pub id: String,
    pub prerequisites: BTreeSet<String>,
    pub obligations: Vec<RuntimeCommandObligation>,
    pub resolution_failed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityProfile {
    pub continue_after_non_satisfied: bool,
    pub validations: Vec<RuntimeValidation>,
    pub identity: ProfileIdentity,
    pub governed_paths: Vec<PathBuf>,
    pub resolution_state_identity: Option<RepositoryStateIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealizedIntegrityProfile {
    pub profile: IntegrityProfile,
    pub evaluation_context_identity: EvaluationContextIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationResult {
    pub id: String,
    pub status: ObligationStatus,
    pub exit_code: Option<String>,
    pub executed: bool,
}
impl ObligationResult {
    pub(super) fn undetermined(obligation: &RuntimeCommandObligation) -> Self {
        Self {
            id: obligation.validation_id.clone(),
            status: ObligationStatus::Undetermined,
            exit_code: None,
            executed: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    pub id: String,
    pub status: ObligationStatus,
    pub obligations: Vec<ObligationResult>,
}
impl ValidationResult {
    pub(super) fn skipped(validation: &RuntimeValidation) -> Self {
        Self {
            id: validation.id.clone(),
            status: ObligationStatus::Undetermined,
            obligations: validation
                .obligations
                .iter()
                .map(ObligationResult::undetermined)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityResult {
    pub verdict: IntegrityVerdict,
    pub profile_identity: ProfileIdentity,
    pub evaluation_context_identity: EvaluationContextIdentity,
    pub baseline_state_identity: Option<RepositoryStateIdentity>,
    pub final_state_identity: Option<RepositoryStateIdentity>,
    pub state_relation: StateRelation,
    pub obligations: Vec<ObligationResult>,
    pub validations: Vec<ValidationResult>,
    pub state_coherence_halt: bool,
}
