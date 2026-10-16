use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) const TRANSPORT_VERSION: u8 = 1;

#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct RawRequest { transport_version: u8, operation: String, input: Value }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyInput {}

#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct ComposeInput { pub repository: String, #[serde(default)] pub binding_ids: Vec<String> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepositoryInput {
    pub repository: String,
}
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct EvidenceRegistryInput { pub repository: String, #[serde(default)] pub requirement_ids: Vec<String> }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct GitWhitespaceInput { pub repository: String, pub environment: BTreeMap<String, String> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GithubArmInput {
    pub binding_path: String,
}
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct IntegrityInput { pub repository: String, pub evaluation_context: EvaluationContextInput }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvaluationContextInput {
    pub realizations: Vec<RealizationInput>,
}
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct RealizationInput { pub environment_id: String, pub identity_utf8: String, pub command_prefix: Vec<String>, pub process_environment: BTreeMap<String, String> }
#[derive(Deserialize)]
#[serde(transparent)]
pub(crate) struct NullableString(pub Option<String>);
#[derive(Deserialize)]
#[serde(transparent)]
pub(crate) struct NullableBinding(pub Option<EvidenceBindingInput>);
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct EvidenceRequirementInput { pub admitted_classes: Vec<String>, pub subject_hex: NullableString, pub context_hex: NullableString, pub context_required: bool }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct EvidenceBindingInput { pub evidence_class: NullableString, pub subject_hex: NullableString, pub context_hex: NullableString }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct HistoryComparisonInput { pub label: String, pub historical_requirement: EvidenceRequirementInput, pub current_requirement: EvidenceRequirementInput, pub binding: NullableBinding }

#[rustfmt::skip]
#[derive(Deserialize)] #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ExactEvidenceInput {
    EvaluateBinding { requirement: EvidenceRequirementInput, binding: NullableBinding },
    EvaluateRequirementHistory { comparisons: Vec<HistoryComparisonInput> },
}

#[rustfmt::skip]
pub(crate) enum Request {
    EngineIdentity, Compose(ComposeInput), EvaluateIntegrity(IntegrityInput),
    CheckAcceptedAdrBody(RepositoryInput), ValidateGitWhitespace(GitWhitespaceInput),
    ObserveGithubArm(GithubArmInput), LoadEvidenceRegistry(EvidenceRegistryInput),
    EvaluateExactEvidence(ExactEvidenceInput),
}

pub(crate) fn parse_request(bytes: &[u8]) -> Result<Request, InvalidRequest> {
    let raw: RawRequest = serde_json::from_slice(bytes).map_err(|_| InvalidRequest)?;
    if raw.transport_version != TRANSPORT_VERSION {
        return Err(InvalidRequest);
    }
    match raw.operation.as_str() {
        "engine.identity" => decode::<EmptyInput>(raw.input).map(|_| Request::EngineIdentity),
        "repository-governance-state.compose" => {
            let input: ComposeInput = decode(raw.input)?;
            valid_ids(&input.repository, &input.binding_ids)?;
            Ok(Request::Compose(input))
        }
        "repository-integrity.evaluate" => {
            let input: IntegrityInput = decode(raw.input)?;
            if input.repository.is_empty()
                || input
                    .evaluation_context
                    .realizations
                    .iter()
                    .any(|item| item.environment_id.is_empty() || item.command_prefix.is_empty())
                || !all_unique(
                    input
                        .evaluation_context
                        .realizations
                        .iter()
                        .map(|item| &item.environment_id),
                )
            {
                return Err(InvalidRequest);
            }
            Ok(Request::EvaluateIntegrity(input))
        }
        "accepted-adr-body.immutability" => {
            let input: RepositoryInput = decode(raw.input)?;
            nonempty(&input.repository)?;
            Ok(Request::CheckAcceptedAdrBody(input))
        }
        "git-whitespace.validate" => {
            let input: GitWhitespaceInput = decode(raw.input)?;
            nonempty(&input.repository)?;
            Ok(Request::ValidateGitWhitespace(input))
        }
        "github-authoritative-ref-monotonicity.observe" => {
            let input: GithubArmInput = decode(raw.input)?;
            nonempty(&input.binding_path)?;
            Ok(Request::ObserveGithubArm(input))
        }
        "evidence-requirements.registry" => {
            let input: EvidenceRegistryInput = decode(raw.input)?;
            valid_ids(&input.repository, &input.requirement_ids)?;
            Ok(Request::LoadEvidenceRegistry(input))
        }
        "exact-evidence-binding.evaluate" => {
            let input: ExactEvidenceInput = decode(raw.input)?;
            if !input.valid_hex() {
                return Err(InvalidRequest);
            }
            Ok(Request::EvaluateExactEvidence(input))
        }
        _ => Err(InvalidRequest),
    }
}

fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, InvalidRequest> {
    serde_json::from_value(value).map_err(|_| InvalidRequest)
}

fn nonempty(value: &str) -> Result<(), InvalidRequest> {
    (!value.is_empty()).then_some(()).ok_or(InvalidRequest)
}

fn valid_ids(repository: &str, ids: &[String]) -> Result<(), InvalidRequest> {
    if repository.is_empty() || ids.iter().any(String::is_empty) || !all_unique(ids) {
        Err(InvalidRequest)
    } else {
        Ok(())
    }
}

fn all_unique<'a>(values: impl IntoIterator<Item = &'a String>) -> bool {
    let mut unique = BTreeSet::new();
    values.into_iter().all(|value| unique.insert(value))
}

impl ExactEvidenceInput {
    fn valid_hex(&self) -> bool {
        match self {
            Self::EvaluateBinding {
                requirement,
                binding,
            } => {
                requirement.valid_hex()
                    && binding
                        .0
                        .as_ref()
                        .is_none_or(EvidenceBindingInput::valid_hex)
            }
            Self::EvaluateRequirementHistory { comparisons } => comparisons.iter().all(|item| {
                item.historical_requirement.valid_hex()
                    && item.current_requirement.valid_hex()
                    && item
                        .binding
                        .0
                        .as_ref()
                        .is_none_or(EvidenceBindingInput::valid_hex)
            }),
        }
    }
}

impl EvidenceRequirementInput {
    fn valid_hex(&self) -> bool {
        valid_hex(&self.subject_hex.0) && valid_hex(&self.context_hex.0)
    }
}

impl EvidenceBindingInput {
    fn valid_hex(&self) -> bool {
        valid_hex(&self.subject_hex.0) && valid_hex(&self.context_hex.0)
    }
}

fn valid_hex(value: &Option<String>) -> bool {
    value.as_ref().is_none_or(|value| {
        value.len() % 2 == 0
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}

#[derive(Debug)]
pub(crate) struct InvalidRequest;

#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct EngineIdentityValue { pub source_repository: &'static str, pub source_commit: &'static str, pub transport_version: u8 }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct ComposeValue { pub stage: &'static str, pub identity_present: bool, pub scope_paths: Vec<String>, pub components: ComponentPresence, pub bindings: BTreeMap<String, BindingProjection> }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct ComponentPresence { pub repository_governance_model: bool, pub governance_authority: bool, pub governed_objects: bool, pub governance_bindings: bool, pub repository_integrity: bool, pub projection_registry: bool, pub evidence_requirements: bool }
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum BindingProjection {
    Absent { present: bool },
    Present(Box<PresentBinding>),
}
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct PresentBinding { pub present: bool, pub kind: &'static str, pub scope: BindingScope, pub identity: BindingIdentity, pub authority: BindingAuthority }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct BindingScope { pub kind: &'static str, pub capability: Option<String>, pub interface: Option<String>, pub object: Option<String> }
#[rustfmt::skip]
#[derive(Serialize)] #[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum BindingIdentity { ExecutableProvider { repository: String, commit: String }, GovernanceContract { repository: String, commit: String, path: String } }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct BindingAuthority { pub responsibility: String, pub source: String }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct AcceptedAdrBodyValue { pub status: &'static str, pub category: Option<&'static str> }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct GitWhitespaceValue { pub status: &'static str, pub domains: Vec<&'static str> }
#[derive(Serialize)]
pub(crate) struct GithubArmValue {
    pub status: &'static str,
}
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct EvidenceRegistryValue { pub authority_source: String, pub model_version: u8, pub registry_authority: RegistryAuthorityValue, pub requirement_ids: Vec<String>, pub requirements: BTreeMap<String, PersistentRequirementValue> }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct RegistryAuthorityValue { pub responsibility: String, pub source: String }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct PersistentRequirementValue { pub id: String, pub responsibility: String, pub instances: DeclarationSourceValue, pub evidence_classes: EvidenceClassesValue, pub subject_source: String, pub context: ContextValue, pub candidate_source: String, pub target: Option<TargetValue> }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct DeclarationSourceValue { pub kind: &'static str, pub source: Option<String> }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct EvidenceClassesValue { pub kind: &'static str, pub classes: Option<Vec<String>>, pub source: Option<String> }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct ContextValue { pub required: bool, pub source: Option<String> }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct TargetValue { pub interface: String, pub object: String }
#[derive(Serialize)]
pub(crate) struct ExactBindingValue {
    pub status: &'static str,
}
#[derive(Serialize)]
pub(crate) struct ExactHistoryValue {
    pub comparisons: Vec<ExactComparisonValue>,
}
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct ExactComparisonValue { pub label: String, pub historical: &'static str, pub current: &'static str }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct IntegrityValue { pub verdict: &'static str, pub state_relation: &'static str, pub state_coherence_halt: bool, pub validations: Vec<ValidationProjection>, pub obligations: Vec<ObligationProjection> }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct ValidationProjection { pub id: String, pub status: &'static str, pub obligations: Vec<ObligationProjection> }
#[rustfmt::skip]
#[derive(Clone, Serialize)]
pub(crate) struct ObligationProjection { pub id: String, pub status: &'static str, pub executed: bool, pub exit_code: Option<String> }

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum ResultValue {
    EngineIdentity(EngineIdentityValue),
    Compose(ComposeValue),
    Integrity(IntegrityValue),
    AcceptedAdrBody(AcceptedAdrBodyValue),
    GitWhitespace(GitWhitespaceValue),
    GithubArm(GithubArmValue),
    EvidenceRegistry(EvidenceRegistryValue),
    ExactBinding(ExactBindingValue),
    ExactHistory(ExactHistoryValue),
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Response {
    Result(ResultResponse),
    ControlledFailure(ControlledFailureResponse),
    TransportError(TransportErrorResponse),
}
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct ResultResponse { transport_version: u8, operation: &'static str, outcome: &'static str, value: ResultValue }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct ControlledFailureResponse { transport_version: u8, operation: &'static str, outcome: &'static str, error: ControlledFailureError }
#[rustfmt::skip]
#[derive(Serialize)]
struct ControlledFailureError { category: &'static str, stage: Option<&'static str>, detail: String }
#[rustfmt::skip]
#[derive(Serialize)]
pub(crate) struct TransportErrorResponse { transport_version: u8, outcome: &'static str, error: TransportError }
#[derive(Serialize)]
struct TransportError {
    category: &'static str,
}

impl Response {
    pub fn result(operation: &'static str, value: ResultValue) -> Self {
        Self::Result(ResultResponse {
            transport_version: 1,
            operation,
            outcome: "result",
            value,
        })
    }

    pub fn controlled_failure(
        operation: &'static str,
        category: &'static str,
        stage: Option<&'static str>,
        detail: String,
    ) -> Self {
        Self::ControlledFailure(ControlledFailureResponse {
            transport_version: 1,
            operation,
            outcome: "controlled_failure",
            error: ControlledFailureError {
                category,
                stage,
                detail,
            },
        })
    }

    pub fn transport_error() -> Self {
        Self::TransportError(TransportErrorResponse {
            transport_version: 1,
            outcome: "transport_error",
            error: TransportError {
                category: "invalid_request",
            },
        })
    }
}
