use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) const TRANSPORT_VERSION: u8 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequest {
    transport_version: u8,
    operation: String,
    input: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyInput {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ComposeInput {
    pub(crate) repository: String,
    #[serde(default)]
    pub(crate) binding_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AcceptedAdrBodyInput {
    pub(crate) repository: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GitWhitespaceInput {
    pub(crate) repository: String,
    pub(crate) environment: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GithubArmInput {
    pub(crate) binding_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IntegrityInput {
    pub(crate) repository: String,
    pub(crate) evaluation_context: EvaluationContextInput,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvaluationContextInput {
    pub(crate) realizations: Vec<RealizationInput>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RealizationInput {
    pub(crate) environment_id: String,
    pub(crate) identity_utf8: String,
    pub(crate) command_prefix: Vec<String>,
    pub(crate) process_environment: BTreeMap<String, String>,
}

pub(crate) enum Request {
    EngineIdentity,
    Compose(ComposeInput),
    EvaluateIntegrity(IntegrityInput),
    CheckAcceptedAdrBody(AcceptedAdrBodyInput),
    ValidateGitWhitespace(GitWhitespaceInput),
    ObserveGithubArm(GithubArmInput),
}

pub(crate) fn parse_request(bytes: &[u8]) -> Result<Request, InvalidRequest> {
    let raw: RawRequest = serde_json::from_slice(bytes).map_err(|_| InvalidRequest)?;
    if raw.transport_version != TRANSPORT_VERSION {
        return Err(InvalidRequest);
    }
    match raw.operation.as_str() {
        "engine.identity" => {
            serde_json::from_value::<EmptyInput>(raw.input).map_err(|_| InvalidRequest)?;
            Ok(Request::EngineIdentity)
        }
        "repository-governance-state.compose" => {
            let input: ComposeInput =
                serde_json::from_value(raw.input).map_err(|_| InvalidRequest)?;
            if input.repository.is_empty()
                || input.binding_ids.iter().any(String::is_empty)
                || !all_unique(input.binding_ids.iter())
            {
                return Err(InvalidRequest);
            }
            Ok(Request::Compose(input))
        }
        "repository-integrity.evaluate" => {
            let input: IntegrityInput =
                serde_json::from_value(raw.input).map_err(|_| InvalidRequest)?;
            if input.repository.is_empty()
                || input
                    .evaluation_context
                    .realizations
                    .iter()
                    .any(|realization| {
                        realization.environment_id.is_empty()
                            || realization.command_prefix.is_empty()
                    })
                || !all_unique(
                    input
                        .evaluation_context
                        .realizations
                        .iter()
                        .map(|realization| &realization.environment_id),
                )
            {
                return Err(InvalidRequest);
            }
            Ok(Request::EvaluateIntegrity(input))
        }
        "accepted-adr-body.immutability" => {
            let input: AcceptedAdrBodyInput =
                serde_json::from_value(raw.input).map_err(|_| InvalidRequest)?;
            if input.repository.is_empty() {
                return Err(InvalidRequest);
            }
            Ok(Request::CheckAcceptedAdrBody(input))
        }
        "git-whitespace.validate" => {
            let input: GitWhitespaceInput =
                serde_json::from_value(raw.input).map_err(|_| InvalidRequest)?;
            if input.repository.is_empty() {
                return Err(InvalidRequest);
            }
            Ok(Request::ValidateGitWhitespace(input))
        }
        "github-authoritative-ref-monotonicity.observe" => {
            let input: GithubArmInput =
                serde_json::from_value(raw.input).map_err(|_| InvalidRequest)?;
            if input.binding_path.is_empty() {
                return Err(InvalidRequest);
            }
            Ok(Request::ObserveGithubArm(input))
        }
        _ => Err(InvalidRequest),
    }
}

fn all_unique<'a>(values: impl IntoIterator<Item = &'a String>) -> bool {
    let mut unique = BTreeSet::new();
    values.into_iter().all(|value| unique.insert(value))
}

#[derive(Debug)]
pub(crate) struct InvalidRequest;

#[derive(Serialize)]
pub(crate) struct EngineIdentityValue {
    pub(crate) source_repository: &'static str,
    pub(crate) source_commit: &'static str,
    pub(crate) transport_version: u8,
}

#[derive(Serialize)]
pub(crate) struct ComposeValue {
    pub(crate) stage: &'static str,
    pub(crate) identity_present: bool,
    pub(crate) scope_paths: Vec<String>,
    pub(crate) components: ComponentPresence,
    pub(crate) bindings: BTreeMap<String, BindingProjection>,
}

#[derive(Serialize)]
pub(crate) struct ComponentPresence {
    pub(crate) repository_governance_model: bool,
    pub(crate) governance_authority: bool,
    pub(crate) governed_objects: bool,
    pub(crate) governance_bindings: bool,
    pub(crate) repository_integrity: bool,
    pub(crate) projection_registry: bool,
    pub(crate) evidence_requirements: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum BindingProjection {
    Absent { present: bool },
    Present(Box<PresentBinding>),
}

#[derive(Serialize)]
pub(crate) struct PresentBinding {
    pub(crate) present: bool,
    pub(crate) kind: &'static str,
    pub(crate) scope: BindingScope,
    pub(crate) identity: BindingIdentity,
    pub(crate) authority: BindingAuthority,
}

#[derive(Serialize)]
pub(crate) struct BindingScope {
    pub(crate) kind: &'static str,
    pub(crate) capability: Option<String>,
    pub(crate) interface: Option<String>,
    pub(crate) object: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum BindingIdentity {
    ExecutableProvider {
        repository: String,
        commit: String,
    },
    GovernanceContract {
        repository: String,
        commit: String,
        path: String,
    },
}

#[derive(Serialize)]
pub(crate) struct BindingAuthority {
    pub(crate) responsibility: String,
    pub(crate) source: String,
}

#[derive(Serialize)]
pub(crate) struct AcceptedAdrBodyValue {
    pub(crate) status: &'static str,
    pub(crate) category: Option<&'static str>,
}

#[derive(Serialize)]
pub(crate) struct GitWhitespaceValue {
    pub(crate) status: &'static str,
    pub(crate) domains: Vec<&'static str>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct GithubArmValue {
    pub(crate) status: &'static str,
}

#[derive(Serialize)]
pub(crate) struct IntegrityValue {
    pub(crate) verdict: &'static str,
    pub(crate) state_relation: &'static str,
    pub(crate) state_coherence_halt: bool,
    pub(crate) validations: Vec<ValidationProjection>,
    pub(crate) obligations: Vec<ObligationProjection>,
}

#[derive(Serialize)]
pub(crate) struct ValidationProjection {
    pub(crate) id: String,
    pub(crate) status: &'static str,
    pub(crate) obligations: Vec<ObligationProjection>,
}

#[derive(Clone, Serialize)]
pub(crate) struct ObligationProjection {
    pub(crate) id: String,
    pub(crate) status: &'static str,
    pub(crate) executed: bool,
    pub(crate) exit_code: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum ResultValue {
    EngineIdentity(EngineIdentityValue),
    Compose(ComposeValue),
    Integrity(IntegrityValue),
    AcceptedAdrBody(AcceptedAdrBodyValue),
    GitWhitespace(GitWhitespaceValue),
    GithubArm(GithubArmValue),
}

#[derive(Serialize)]
pub(crate) struct ResultResponse {
    transport_version: u8,
    operation: &'static str,
    outcome: &'static str,
    value: ResultValue,
}

#[derive(Serialize)]
pub(crate) struct ControlledFailureResponse {
    transport_version: u8,
    operation: &'static str,
    outcome: &'static str,
    error: ControlledFailureError,
}

#[derive(Serialize)]
pub(crate) struct ControlledFailureError {
    category: &'static str,
    stage: Option<&'static str>,
    detail: String,
}

#[derive(Serialize)]
pub(crate) struct TransportErrorResponse {
    transport_version: u8,
    outcome: &'static str,
    error: TransportError,
}

#[derive(Serialize)]
pub(crate) struct TransportError {
    category: &'static str,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Response {
    Result(ResultResponse),
    ControlledFailure(ControlledFailureResponse),
    TransportError(TransportErrorResponse),
}

impl Response {
    pub(crate) fn result(operation: &'static str, value: ResultValue) -> Self {
        Self::Result(ResultResponse {
            transport_version: TRANSPORT_VERSION,
            operation,
            outcome: "result",
            value,
        })
    }

    pub(crate) fn controlled_failure(
        operation: &'static str,
        category: &'static str,
        stage: Option<&'static str>,
        detail: String,
    ) -> Self {
        Self::ControlledFailure(ControlledFailureResponse {
            transport_version: TRANSPORT_VERSION,
            operation,
            outcome: "controlled_failure",
            error: ControlledFailureError {
                category,
                stage,
                detail,
            },
        })
    }

    pub(crate) fn transport_error() -> Self {
        Self::TransportError(TransportErrorResponse {
            transport_version: TRANSPORT_VERSION,
            outcome: "transport_error",
            error: TransportError {
                category: "invalid_request",
            },
        })
    }
}
