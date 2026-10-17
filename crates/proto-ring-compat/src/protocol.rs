use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

mod terminology;
pub(crate) use terminology::*;

pub(crate) const TRANSPORT_VERSION: u8 = 1;

#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct RawRequest { transport_version: u8, operation: String, input: Value }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct EmptyInput {}
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct ComposeInput { pub repository: String, #[serde(default)] pub binding_ids: Vec<String> }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct RepositoryInput { pub repository: String }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct EvidenceRegistryInput { pub repository: String, #[serde(default)] pub requirement_ids: Vec<String> }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct GitWhitespaceInput { pub repository: String, pub environment: BTreeMap<String, String> }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct GithubArmInput { pub binding_path: String }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct IntegrityInput { pub repository: String, pub evaluation_context: EvaluationContextInput }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct EvaluationContextInput { pub realizations: Vec<RealizationInput> }
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

#[derive(Deserialize, Serialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "lowercase",
    deny_unknown_fields
)]
pub(crate) enum CompatStructuredValue {
    Null(()),
    Boolean(bool),
    Integer(String),
    String(String),
    Sequence(Vec<CompatStructuredValue>),
    Record(Vec<CompatRecordEntry>),
}
#[rustfmt::skip]
#[derive(Deserialize, Serialize)] #[serde(deny_unknown_fields)]
pub(crate) struct CompatRecordEntry { pub name: String, pub value: CompatStructuredValue }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum AdrParseInput {
    LoadYaml { bytes_hex: String }, LoadJson { bytes_hex: String }, ParseAdr { bytes_hex: String },
    DecisionBody { bytes_hex: String }, H1 { bytes_hex: String }, PreservedPayload { bytes_hex: String }, Sha256 { bytes_hex: String },
}
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct RepositoryPathInput { pub repository: String, pub relative: Value }
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum AdrValidationInput {
    RequireMapping { value: CompatStructuredValue }, RequireString { value: CompatStructuredValue }, RequireStringList { value: CompatStructuredValue },
    RelationTargetCategories { source_id: CompatStructuredValue, relation_type: CompatStructuredValue, targets: CompatStructuredValue, known_ids: CompatStructuredValue },
    RelationTargetErrors { source_id: CompatStructuredValue, relation_type: CompatStructuredValue, targets: CompatStructuredValue, known_ids: CompatStructuredValue },
    SchemaErrorLabels { value: CompatStructuredValue, schema: CompatStructuredValue, overlay_schema: CompatStructuredValue },
    SchemaErrorLocations { value: CompatStructuredValue, schema: CompatStructuredValue, overlay_schema: CompatStructuredValue },
    SchemaErrors { value: CompatStructuredValue, schema: CompatStructuredValue, overlay_schema: CompatStructuredValue },
}
#[rustfmt::skip]
#[derive(Deserialize)] #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum CanonicalAdrInput {
    ConfiguredProfilePath { repository: String },
    ResolveAdr { repository: String, adr_id: Value, #[serde(default)] include_decision_body: bool },
}

#[rustfmt::skip]
pub(crate) enum Request {
    EngineIdentity, Compose(ComposeInput), EvaluateIntegrity(IntegrityInput), CheckAcceptedAdrBody(RepositoryInput),
    ValidateGitWhitespace(GitWhitespaceInput), ObserveGithubArm(GithubArmInput), LoadEvidenceRegistry(EvidenceRegistryInput),
    EvaluateExactEvidence(ExactEvidenceInput), ParseAdrMetadata(AdrParseInput), ResolveAdrRepositoryPath(RepositoryPathInput),
    ValidateAdrMetadata(AdrValidationInput), ResolveCanonicalAdr(CanonicalAdrInput), Terminology(TerminologyRequest),
}

#[rustfmt::skip]
pub(crate) fn parse_request(bytes: &[u8]) -> Result<Request, InvalidRequest> {
    let raw: RawRequest = serde_json::from_slice(bytes).map_err(|_| InvalidRequest)?;
    if raw.transport_version != TRANSPORT_VERSION { return Err(InvalidRequest); }
    match raw.operation.as_str() {
        "engine.identity" => decode::<EmptyInput>(raw.input).map(|_| Request::EngineIdentity),
        "repository-governance-state.compose" => { let input: ComposeInput = decode(raw.input)?; valid_ids(&input.repository, &input.binding_ids)?; Ok(Request::Compose(input)) }
        "repository-integrity.evaluate" => { let input: IntegrityInput = decode(raw.input)?; if input.repository.is_empty() || input.evaluation_context.realizations.iter().any(|item| item.environment_id.is_empty() || item.command_prefix.is_empty()) || !all_unique(input.evaluation_context.realizations.iter().map(|item| &item.environment_id)) { return Err(InvalidRequest); } Ok(Request::EvaluateIntegrity(input)) }
        "accepted-adr-body.immutability" => { let input: RepositoryInput = decode(raw.input)?; nonempty(&input.repository)?; Ok(Request::CheckAcceptedAdrBody(input)) }
        "git-whitespace.validate" => { let input: GitWhitespaceInput = decode(raw.input)?; nonempty(&input.repository)?; Ok(Request::ValidateGitWhitespace(input)) }
        "github-authoritative-ref-monotonicity.observe" => { let input: GithubArmInput = decode(raw.input)?; nonempty(&input.binding_path)?; Ok(Request::ObserveGithubArm(input)) }
        "evidence-requirements.registry" => { let input: EvidenceRegistryInput = decode(raw.input)?; valid_ids(&input.repository, &input.requirement_ids)?; Ok(Request::LoadEvidenceRegistry(input)) }
        "exact-evidence-binding.evaluate" => { let input: ExactEvidenceInput = decode(raw.input)?; if !input.valid_hex() { return Err(InvalidRequest); } Ok(Request::EvaluateExactEvidence(input)) }
        "adr-metadata.parse" => { let input: AdrParseInput = decode(raw.input)?; if !valid_hex_text(input.bytes_hex()) { return Err(InvalidRequest); } Ok(Request::ParseAdrMetadata(input)) }
        "adr-metadata.repository-path" => { let input: RepositoryPathInput = decode(raw.input)?; nonempty(&input.repository)?; Ok(Request::ResolveAdrRepositoryPath(input)) }
        "adr-metadata.validation" => { let input: AdrValidationInput = decode(raw.input)?; if !input.valid_structure() { return Err(InvalidRequest); } Ok(Request::ValidateAdrMetadata(input)) }
        "canonical-adr.resolve" => { let input: CanonicalAdrInput = decode(raw.input)?; nonempty(input.repository())?; Ok(Request::ResolveCanonicalAdr(input)) }
        operation @ ("normative-terminology.fingerprint" | "normative-terminology.registry" | "normative-terminology.markdown" | "normative-terminology.inventory") => terminology::parse_operation(operation, raw.input).map(Request::Terminology),
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
fn valid_hex_text(value: &str) -> bool {
    value.len().is_multiple_of(2)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
fn valid_hex(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(valid_hex_text)
}

#[rustfmt::skip]
impl ExactEvidenceInput {
    fn valid_hex(&self) -> bool { match self {
        Self::EvaluateBinding { requirement, binding } => requirement.valid_hex() && binding.0.as_ref().is_none_or(EvidenceBindingInput::valid_hex),
        Self::EvaluateRequirementHistory { comparisons } => comparisons.iter().all(|item| item.historical_requirement.valid_hex() && item.current_requirement.valid_hex() && item.binding.0.as_ref().is_none_or(EvidenceBindingInput::valid_hex)),
    }}
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
impl AdrParseInput {
    pub fn bytes_hex(&self) -> &str {
        match self {
            Self::LoadYaml { bytes_hex }
            | Self::LoadJson { bytes_hex }
            | Self::ParseAdr { bytes_hex }
            | Self::DecisionBody { bytes_hex }
            | Self::H1 { bytes_hex }
            | Self::PreservedPayload { bytes_hex }
            | Self::Sha256 { bytes_hex } => bytes_hex,
        }
    }
}
impl CanonicalAdrInput {
    fn repository(&self) -> &str {
        match self {
            Self::ConfiguredProfilePath { repository } | Self::ResolveAdr { repository, .. } => {
                repository
            }
        }
    }
}
#[rustfmt::skip]
impl CompatStructuredValue {
    fn valid(&self) -> bool { match self {
        Self::Sequence(values) => values.iter().all(Self::valid),
        Self::Record(entries) => { let mut names = BTreeSet::new(); entries.iter().all(|entry| names.insert(&entry.name) && entry.value.valid()) },
        _ => true,
    }}
}
#[rustfmt::skip]
impl AdrValidationInput {
    fn valid_structure(&self) -> bool { match self {
        Self::RequireMapping { value } | Self::RequireString { value } | Self::RequireStringList { value } => value.valid(),
        Self::RelationTargetCategories { source_id, relation_type, targets, known_ids } | Self::RelationTargetErrors { source_id, relation_type, targets, known_ids } => [source_id, relation_type, targets, known_ids].into_iter().all(CompatStructuredValue::valid),
        Self::SchemaErrorLabels { value, schema, overlay_schema } | Self::SchemaErrorLocations { value, schema, overlay_schema } | Self::SchemaErrors { value, schema, overlay_schema } => [value, schema, overlay_schema].into_iter().all(CompatStructuredValue::valid),
    }}
}

#[derive(Debug)]
pub(crate) struct InvalidRequest;

#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct EngineIdentityValue { pub source_repository: &'static str, pub source_commit: &'static str, pub transport_version: u8 }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct ComposeValue { pub stage: &'static str, pub identity_present: bool, pub scope_paths: Vec<String>, pub components: ComponentPresence, pub bindings: BTreeMap<String, BindingProjection> }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct ComponentPresence { pub repository_governance_model: bool, pub governance_authority: bool, pub governed_objects: bool, pub governance_bindings: bool, pub repository_integrity: bool, pub projection_registry: bool, pub evidence_requirements: bool }
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum BindingProjection {
    Absent { present: bool },
    Present(Box<PresentBinding>),
}
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct PresentBinding { pub present: bool, pub kind: &'static str, pub scope: BindingScope, pub identity: BindingIdentity, pub authority: BindingAuthority }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct BindingScope { pub kind: &'static str, pub capability: Option<String>, pub interface: Option<String>, pub object: Option<String> }
#[rustfmt::skip]
#[derive(Serialize)] #[serde(tag = "kind", rename_all = "snake_case")] pub(crate) enum BindingIdentity { ExecutableProvider { repository: String, commit: String }, GovernanceContract { repository: String, commit: String, path: String } }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct BindingAuthority { pub responsibility: String, pub source: String }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct AcceptedAdrBodyValue { pub status: &'static str, pub category: Option<&'static str> }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct GitWhitespaceValue { pub status: &'static str, pub domains: Vec<&'static str> }
#[derive(Serialize)]
pub(crate) struct GithubArmValue {
    pub status: &'static str,
}
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct EvidenceRegistryValue { pub authority_source: String, pub model_version: u8, pub registry_authority: RegistryAuthorityValue, pub requirement_ids: Vec<String>, pub requirements: BTreeMap<String, PersistentRequirementValue> }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct RegistryAuthorityValue { pub responsibility: String, pub source: String }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct PersistentRequirementValue { pub id: String, pub responsibility: String, pub instances: DeclarationSourceValue, pub evidence_classes: EvidenceClassesValue, pub subject_source: String, pub context: ContextValue, pub candidate_source: String, pub target: Option<TargetValue> }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct DeclarationSourceValue { pub kind: &'static str, pub source: Option<String> }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct EvidenceClassesValue { pub kind: &'static str, pub classes: Option<Vec<String>>, pub source: Option<String> }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct ContextValue { pub required: bool, pub source: Option<String> }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct TargetValue { pub interface: String, pub object: String }
#[derive(Serialize)]
pub(crate) struct ExactBindingValue {
    pub status: &'static str,
}
#[derive(Serialize)]
pub(crate) struct ExactHistoryValue {
    pub comparisons: Vec<ExactComparisonValue>,
}
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct ExactComparisonValue { pub label: String, pub historical: &'static str, pub current: &'static str }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct IntegrityValue { pub verdict: &'static str, pub state_relation: &'static str, pub state_coherence_halt: bool, pub validations: Vec<ValidationProjection>, pub obligations: Vec<ObligationProjection> }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct ValidationProjection { pub id: String, pub status: &'static str, pub obligations: Vec<ObligationProjection> }
#[rustfmt::skip]
#[derive(Clone, Serialize)] pub(crate) struct ObligationProjection { pub id: String, pub status: &'static str, pub executed: bool, pub exit_code: Option<String> }

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum AdrParseValue {
    Structured {
        structured: CompatStructuredValue,
    },
    Parsed {
        metadata: CompatStructuredValue,
        body_hex: String,
    },
    Body {
        body_hex: String,
    },
    Heading {
        text: String,
    },
    Payload {
        payload_hex: String,
    },
    Digest {
        sha256: String,
    },
}
#[derive(Serialize)]
pub(crate) struct RepositoryPathValue {
    pub relative: String,
}
#[rustfmt::skip]
#[derive(Serialize)] #[serde(tag = "category", rename_all = "snake_case")]
pub(crate) enum RelationErrorValue { SelfReference, MissingTarget { target: String } }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct SchemaErrorValue { pub path: String, pub schema: &'static str }
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum AdrValidationValue {
    Structured {
        value: CompatStructuredValue,
    },
    Text {
        text: String,
    },
    Items {
        items: Vec<String>,
    },
    Relations {
        errors: Vec<RelationErrorValue>,
    },
    SchemaErrors {
        errors: Vec<SchemaErrorValue>,
    },
    SchemaLabels {
        base_has_error: bool,
        overlay_has_error: bool,
    },
    SchemaLocations {
        errors: Vec<SchemaErrorValue>,
        rejected: bool,
    },
}
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum CanonicalAdrValue {
    Profile {
        relative_path: String,
    },
    Adr {
        adr_id: String,
        relative_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        decision_body_hex: Option<String>,
    },
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
    EvidenceRegistry(EvidenceRegistryValue),
    ExactBinding(ExactBindingValue),
    ExactHistory(ExactHistoryValue),
    AdrParse(AdrParseValue),
    RepositoryPath(RepositoryPathValue),
    AdrValidation(AdrValidationValue),
    CanonicalAdr(CanonicalAdrValue),
    Terminology(TerminologyValue),
}
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Response {
    Result(ResultResponse),
    ControlledFailure(ControlledFailureResponse),
    TransportError(TransportErrorResponse),
}
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct ResultResponse { transport_version: u8, operation: &'static str, outcome: &'static str, value: ResultValue }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct ControlledFailureResponse { transport_version: u8, operation: &'static str, outcome: &'static str, error: ControlledFailureError }
#[rustfmt::skip]
#[derive(Serialize)] struct ControlledFailureError { category: &'static str, stage: Option<&'static str>, detail: String }
#[rustfmt::skip]
#[derive(Serialize)] pub(crate) struct TransportErrorResponse { transport_version: u8, outcome: &'static str, error: TransportError }
#[derive(Serialize)]
struct TransportError {
    category: &'static str,
}

#[rustfmt::skip]
impl Response {
    pub fn result(operation: &'static str, value: ResultValue) -> Self { Self::Result(ResultResponse { transport_version: 1, operation, outcome: "result", value }) }
    pub fn controlled_failure(operation: &'static str, category: &'static str, stage: Option<&'static str>, detail: String) -> Self { Self::ControlledFailure(ControlledFailureResponse { transport_version: 1, operation, outcome: "controlled_failure", error: ControlledFailureError { category, stage, detail } }) }
    pub fn transport_error() -> Self { Self::TransportError(TransportErrorResponse { transport_version: 1, outcome: "transport_error", error: TransportError { category: "invalid_request" } }) }
}
