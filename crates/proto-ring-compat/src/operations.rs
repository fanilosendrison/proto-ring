use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use proto_ring_engine::accepted_adr_body::{self, AcceptedBodyDiagnosticKind};
use proto_ring_engine::adr_metadata::{self, RelationTargetError, SchemaValidationError};
use proto_ring_engine::canonical_adr;
use proto_ring_engine::evidence_requirements::{
    EvidenceRequirementRegistry, PersistentEvidenceRequirement,
};
use proto_ring_engine::exact_evidence_binding::{EvidenceBinding, EvidenceRequirement};
use proto_ring_engine::git_whitespace::{self, WhitespaceDomain};
use proto_ring_engine::github_authoritative_ref_monotonicity::{self, ArmObservation};
use proto_ring_engine::governance_bindings::{GovernanceBinding, GovernanceBindingIdentity};
use proto_ring_engine::repository_governance_model;
use proto_ring_engine::repository_governance_state::{
    self, RepositoryGovernanceStage, RepositoryGovernanceStateError,
};
use proto_ring_engine::repository_integrity::{
    self, EvaluationContext, IntegrityResult, ObligationResult, ValidationEnvironmentRealization,
    ValidationResult,
};
use proto_ring_engine::structured_data::StructuredValue;

use crate::protocol::*;

const SOURCE_REPOSITORY: &str = "fanilosendrison/proto-ring";
const SOURCE_COMMIT: &str = env!("PROTO_RING_SOURCE_COMMIT");
const EVIDENCE_OPERATION: &str = "exact-evidence-binding.evaluate";
const ADR_PARSE_OPERATION: &str = "adr-metadata.parse";
const ADR_PATH_OPERATION: &str = "adr-metadata.repository-path";
const ADR_VALIDATION_OPERATION: &str = "adr-metadata.validation";
const CANONICAL_ADR_OPERATION: &str = "canonical-adr.resolve";

#[rustfmt::skip]
pub(crate) fn execute(request: Request) -> Result<Response, InternalFailure> {
    Ok(match request {
        Request::EngineIdentity => Response::result("engine.identity", ResultValue::EngineIdentity(EngineIdentityValue { source_repository: SOURCE_REPOSITORY, source_commit: SOURCE_COMMIT, transport_version: TRANSPORT_VERSION })),
        Request::Compose(input) => compose(input)?, Request::EvaluateIntegrity(input) => evaluate_integrity(input)?,
        Request::CheckAcceptedAdrBody(input) => check_accepted_adr_body(input), Request::ValidateGitWhitespace(input) => validate_git_whitespace(input),
        Request::ObserveGithubArm(input) => observe_github_arm(input), Request::LoadEvidenceRegistry(input) => load_evidence_registry(input),
        Request::EvaluateExactEvidence(input) => evaluate_exact_evidence(input), Request::ParseAdrMetadata(input) => parse_adr_metadata(input),
        Request::ResolveAdrRepositoryPath(input) => resolve_adr_repository_path(input), Request::ValidateAdrMetadata(input) => validate_adr_metadata(input),
        Request::ResolveCanonicalAdr(input) => resolve_canonical_adr(input),
    })
}

fn rgs_failure(operation: &'static str, error: RepositoryGovernanceStateError) -> Response {
    Response::controlled_failure(
        operation,
        "repository-governance-state",
        Some(error.diagnostic.stage.serialized()),
        error.diagnostic.detail,
    )
}

#[rustfmt::skip]
fn compose(input: ComposeInput) -> Result<Response, InternalFailure> {
    let state = match repository_governance_state::load(Path::new(&input.repository)) { Ok(state) => state, Err(error) => return Ok(rgs_failure("repository-governance-state.compose", error)) };
    let scope_paths = state.observed_state.scope_paths.iter().map(|path| path.to_str().map(str::to_owned).ok_or(InternalFailure)).collect::<Result<Vec<_>, _>>()?;
    let bindings = input.binding_ids.into_iter().map(|id| { let value = state.governance_bindings.bindings.get(&id).map(project_binding).unwrap_or(BindingProjection::Absent { present: false }); (id, value) }).collect();
    let components = ComponentPresence { repository_governance_model: true, governance_authority: true, governed_objects: state.governed_objects.is_some(), governance_bindings: true, repository_integrity: state.repository_integrity.is_some(), projection_registry: state.projection_registry.is_some(), evidence_requirements: state.evidence_requirements.is_some() };
    Ok(Response::result("repository-governance-state.compose", ResultValue::Compose(ComposeValue { stage: RepositoryGovernanceStage::Composition.serialized(), identity_present: !state.observed_state.identity.0.is_empty(), scope_paths, components, bindings })))
}
#[rustfmt::skip]
fn project_binding(binding: &GovernanceBinding) -> BindingProjection {
    let identity = match &binding.identity { GovernanceBindingIdentity::ExecutableProvider(value) => BindingIdentity::ExecutableProvider { repository: value.repository.clone(), commit: value.commit.clone() }, GovernanceBindingIdentity::GovernanceContract(value) => BindingIdentity::GovernanceContract { repository: value.repository.clone(), commit: value.commit.clone(), path: value.path.clone() } };
    BindingProjection::Present(Box::new(PresentBinding { present: true, kind: binding.kind.serialized(), identity, scope: BindingScope { kind: binding.scope.kind.serialized(), capability: binding.scope.capability.clone(), interface: binding.scope.interface.clone(), object: binding.scope.object.clone() }, authority: BindingAuthority { responsibility: binding.authority.responsibility.clone(), source: binding.authority.source.clone() } }))
}

#[rustfmt::skip]
fn evaluate_integrity(input: IntegrityInput) -> Result<Response, InternalFailure> {
    let repository = Path::new(&input.repository); let state = match repository_governance_state::load(repository) { Ok(state) => state, Err(error) => return Ok(rgs_failure("repository-integrity.evaluate", error)) };
    let Some(profile) = state.repository_integrity.as_ref() else { return Ok(Response::controlled_failure("repository-integrity.evaluate", "repository-integrity-profile-absent", None, "repository governance state has no Repository Integrity profile".to_owned())); };
    let realizations = input.evaluation_context.realizations.into_iter().map(|item| { let environment_id = item.environment_id; (environment_id.clone(), ValidationEnvironmentRealization { environment_id, identity: item.identity_utf8.into_bytes(), command_prefix: item.command_prefix, process_environment: item.process_environment }) }).collect();
    let context = EvaluationContext { realizations };
    let realized = match repository_integrity::realize_consumer_profile(repository, profile, &context) { Ok(realized) => realized, Err(error) => return Ok(Response::controlled_failure("repository-integrity.evaluate", "repository-integrity-realization", None, error.to_string())) };
    let result = repository_integrity::evaluate(repository, &realized.profile, realized.evaluation_context_identity);
    Ok(Response::result("repository-integrity.evaluate", ResultValue::Integrity(project_integrity_result(result))))
}
#[rustfmt::skip]
fn project_integrity_result(result: IntegrityResult) -> IntegrityValue { IntegrityValue { verdict: result.verdict.serialized(), state_relation: result.state_relation.serialized(), state_coherence_halt: result.state_coherence_halt, validations: result.validations.iter().map(project_validation).collect(), obligations: result.obligations.iter().map(project_obligation).collect() } }
#[rustfmt::skip]
fn project_validation(value: &ValidationResult) -> ValidationProjection { ValidationProjection { id: value.id.clone(), status: value.status.serialized(), obligations: value.obligations.iter().map(project_obligation).collect() } }
#[rustfmt::skip]
fn project_obligation(value: &ObligationResult) -> ObligationProjection { ObligationProjection { id: value.id.clone(), status: value.status.serialized(), executed: value.executed, exit_code: value.exit_code.clone() } }

#[rustfmt::skip]
fn check_accepted_adr_body(input: RepositoryInput) -> Response {
    let report = accepted_adr_body::check(Path::new(&input.repository));
    let category = if report.diagnostics.is_empty() { None } else if report.diagnostics.iter().any(|item| item.kind == AcceptedBodyDiagnosticKind::BodyChanged) { Some("body_changed") } else { Some("historical_state_invalid") };
    let status = if category.is_none() { "PASS" } else { "NON_PASS" };
    Response::result("accepted-adr-body.immutability", ResultValue::AcceptedAdrBody(AcceptedAdrBodyValue { status, category }))
}
#[rustfmt::skip]
fn validate_git_whitespace(input: GitWhitespaceInput) -> Response {
    let report = git_whitespace::check(Path::new(&input.repository), &input.environment); let mut domains = Vec::new();
    for item in &report.diagnostics { let domain = match item.domain { WhitespaceDomain::Unstaged => "unstaged", WhitespaceDomain::Staged => "staged", WhitespaceDomain::PushCommittedRange => "push_committed_range", WhitespaceDomain::PullRequestCommittedRange => "pull_request_committed_range", WhitespaceDomain::RangeDetermination => "range_determination", WhitespaceDomain::GitObservation => "git_observation" }; if !domains.contains(&domain) { domains.push(domain); } }
    let status = if report.passes() { "PASS" } else { "NON_PASS" }; Response::result("git-whitespace.validate", ResultValue::GitWhitespace(GitWhitespaceValue { status, domains }))
}
fn observe_github_arm(input: GithubArmInput) -> Response {
    let observation =
        github_authoritative_ref_monotonicity::observe(Path::new(&input.binding_path));
    Response::result(
        "github-authoritative-ref-monotonicity.observe",
        ResultValue::GithubArm(project_github_arm(&observation)),
    )
}
fn project_github_arm(observation: &ArmObservation) -> GithubArmValue {
    GithubArmValue {
        status: observation.status.serialized(),
    }
}

#[rustfmt::skip]
fn load_evidence_registry(input: EvidenceRegistryInput) -> Response {
    const OPERATION: &str = "evidence-requirements.registry"; let state = match repository_governance_state::load(Path::new(&input.repository)) { Ok(state) => state, Err(error) => return rgs_failure(OPERATION, error) };
    let Some(registry) = state.evidence_requirements.as_ref() else { return Response::controlled_failure(OPERATION, "evidence-requirements-absent", None, "repository governance state has no Evidence Requirements registry".to_owned()); };
    if input.requirement_ids.iter().any(|id| !registry.requirements.contains_key(id)) { return Response::controlled_failure(OPERATION, "evidence-requirement-not-found", None, "a requested evidence requirement is absent".to_owned()); }
    let requirements = input.requirement_ids.into_iter().map(|id| (id.clone(), project_requirement(&registry.requirements[&id]))).collect();
    Response::result(OPERATION, ResultValue::EvidenceRegistry(project_registry(registry, requirements)))
}
#[rustfmt::skip]
fn project_registry(registry: &EvidenceRequirementRegistry, requirements: BTreeMap<String, PersistentRequirementValue>) -> EvidenceRegistryValue { EvidenceRegistryValue { authority_source: registry.authority.source.clone(), model_version: registry.model_version, registry_authority: RegistryAuthorityValue { responsibility: registry.authority.responsibility.clone(), source: registry.authority.source.clone() }, requirement_ids: registry.requirements.keys().cloned().collect(), requirements } }
#[rustfmt::skip]
fn project_requirement(value: &PersistentEvidenceRequirement) -> PersistentRequirementValue { PersistentRequirementValue { id: value.id.clone(), responsibility: value.responsibility.clone(), instances: DeclarationSourceValue { kind: value.instances.kind.serialized(), source: value.instances.source.clone() }, evidence_classes: EvidenceClassesValue { kind: value.evidence_classes.kind.serialized(), classes: value.evidence_classes.source.is_none().then(|| value.evidence_classes.classes.iter().cloned().collect()), source: value.evidence_classes.source.clone() }, subject_source: value.subject_source.clone(), context: ContextValue { required: value.context.required, source: value.context.source.clone() }, candidate_source: value.candidate_source.clone(), target: value.target.as_ref().map(|target| TargetValue { interface: target.interface_id.clone(), object: target.object_id.clone() }) } }

#[rustfmt::skip]
fn evaluate_exact_evidence(input: ExactEvidenceInput) -> Response {
    match input {
        ExactEvidenceInput::EvaluateBinding { requirement, binding } => { let Ok(requirement) = build_requirement(requirement) else { return exact_rejection(); }; let candidate = binding.0.map(build_binding); let status = requirement.evaluate(candidate.as_ref()).serialized(); Response::result(EVIDENCE_OPERATION, ResultValue::ExactBinding(ExactBindingValue { status })) }
        ExactEvidenceInput::EvaluateRequirementHistory { comparisons } => { let prepared = comparisons.into_iter().map(|item| Ok((item.label, build_requirement(item.historical_requirement)?, build_requirement(item.current_requirement)?, item.binding.0.map(build_binding)))).collect::<Result<Vec<_>, ()>>(); let Ok(prepared) = prepared else { return exact_rejection(); }; let comparisons = prepared.into_iter().map(|(label, historical, current, binding)| ExactComparisonValue { label, historical: historical.evaluate(binding.as_ref()).serialized(), current: current.evaluate(binding.as_ref()).serialized() }).collect(); Response::result(EVIDENCE_OPERATION, ResultValue::ExactHistory(ExactHistoryValue { comparisons })) }
    }
}
fn build_requirement(input: EvidenceRequirementInput) -> Result<EvidenceRequirement, ()> {
    let mut admitted_classes = BTreeSet::new();
    for value in input.admitted_classes {
        if !admitted_classes.insert(value) {
            return Err(());
        }
    }
    EvidenceRequirement::new(
        admitted_classes,
        decode_optional_hex(input.subject_hex.0),
        decode_optional_hex(input.context_hex.0),
        input.context_required,
    )
    .map_err(|_| ())
}
#[rustfmt::skip]
fn build_binding(input: EvidenceBindingInput) -> EvidenceBinding { EvidenceBinding { evidence_class: input.evidence_class.0, subject_identity: decode_optional_hex(input.subject_hex.0), context_identity: decode_optional_hex(input.context_hex.0) } }
fn exact_rejection() -> Response {
    Response::controlled_failure(
        EVIDENCE_OPERATION,
        "exact-evidence-binding.evaluate.rejected",
        None,
        "evidence requirement is structurally invalid".to_owned(),
    )
}

#[rustfmt::skip]
fn parse_adr_metadata(input: AdrParseInput) -> Response {
    let result = match input {
        AdrParseInput::LoadYaml { bytes_hex } => adr_metadata::load_yaml(&decode_bytes(&bytes_hex)).map(|value| AdrParseValue::Structured { structured: project_structured(&value) }).map_err(|error| error.to_string()),
        AdrParseInput::LoadJson { bytes_hex } => adr_metadata::load_json(&decode_bytes(&bytes_hex)).map(|value| AdrParseValue::Structured { structured: project_structured(&value) }).map_err(|error| error.to_string()),
        AdrParseInput::ParseAdr { bytes_hex } => adr_metadata::parse_adr(&decode_bytes(&bytes_hex)).map(|value| AdrParseValue::Parsed { metadata: project_structured(&StructuredValue::Mapping(value.metadata)), body_hex: hex_encode(&value.decision_body) }).map_err(|error| error.to_string()),
        AdrParseInput::DecisionBody { bytes_hex } => adr_metadata::decision_body(&decode_bytes(&bytes_hex)).map(|body| AdrParseValue::Body { body_hex: hex_encode(&body) }).map_err(|error| error.to_string()),
        AdrParseInput::H1 { bytes_hex } => adr_metadata::h1_text(&decode_bytes(&bytes_hex)).map(|text| AdrParseValue::Heading { text }).map_err(|error| error.to_string()),
        AdrParseInput::PreservedPayload { bytes_hex } => adr_metadata::preserved_payload(&decode_bytes(&bytes_hex)).map(|payload| AdrParseValue::Payload { payload_hex: hex_encode(&payload) }).map_err(|error| error.to_string()),
        AdrParseInput::Sha256 { bytes_hex } => Ok(AdrParseValue::Digest { sha256: adr_metadata::sha256(&decode_bytes(&bytes_hex)) }),
    };
    finish(ADR_PARSE_OPERATION, result, ResultValue::AdrParse)
}

fn resolve_adr_repository_path(input: RepositoryPathInput) -> Response {
    let result = (|| {
        let relative = input
            .relative
            .as_str()
            .ok_or_else(|| "relative must be a string".to_owned())?;
        let root = Path::new(&input.repository);
        let resolved = adr_metadata::resolve_repository_path(root, relative)
            .map_err(|error| error.to_string())?;
        let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
        let contained = resolved
            .strip_prefix(canonical_root)
            .map_err(|error| error.to_string())?;
        Ok(RepositoryPathValue {
            relative: portable_path(contained)?,
        })
    })();
    finish(ADR_PATH_OPERATION, result, ResultValue::RepositoryPath)
}

#[rustfmt::skip]
fn validate_adr_metadata(input: AdrValidationInput) -> Response {
    let result = match input {
        AdrValidationInput::RequireMapping { value } => require_mapping(value), AdrValidationInput::RequireString { value } => require_string(value), AdrValidationInput::RequireStringList { value } => require_string_list(value),
        AdrValidationInput::RelationTargetCategories { source_id, relation_type, targets, known_ids } | AdrValidationInput::RelationTargetErrors { source_id, relation_type, targets, known_ids } => relation_errors(source_id, relation_type, targets, known_ids),
        AdrValidationInput::SchemaErrorLabels { value, schema, overlay_schema } => schema_errors(0, value, schema, overlay_schema),
        AdrValidationInput::SchemaErrorLocations { value, schema, overlay_schema } => schema_errors(1, value, schema, overlay_schema),
        AdrValidationInput::SchemaErrors { value, schema, overlay_schema } => schema_errors(2, value, schema, overlay_schema),
    };
    finish(ADR_VALIDATION_OPERATION, result, ResultValue::AdrValidation)
}
fn require_mapping(value: CompatStructuredValue) -> Result<AdrValidationValue, String> {
    let value = into_structured(value);
    let mapping = adr_metadata::require_mapping(&value).map_err(|error| error.to_string())?;
    Ok(AdrValidationValue::Structured {
        value: project_structured(&StructuredValue::Mapping(mapping.clone())),
    })
}
fn require_string(value: CompatStructuredValue) -> Result<AdrValidationValue, String> {
    let value = into_structured(value);
    Ok(AdrValidationValue::Text {
        text: adr_metadata::require_string(&value)
            .map_err(|error| error.to_string())?
            .to_owned(),
    })
}
fn require_string_list(value: CompatStructuredValue) -> Result<AdrValidationValue, String> {
    let value = into_structured(value);
    Ok(AdrValidationValue::Items {
        items: adr_metadata::require_string_list(&value).map_err(|error| error.to_string())?,
    })
}
#[rustfmt::skip]
fn relation_errors(source: CompatStructuredValue, relation_type: CompatStructuredValue, targets: CompatStructuredValue, known: CompatStructuredValue) -> Result<AdrValidationValue, String> {
    let source = into_structured(source); let relation_type = into_structured(relation_type); let targets = into_structured(targets); let known = into_structured(known);
    let source = adr_metadata::require_string(&source).map_err(|error| error.to_string())?; let relation_type = adr_metadata::require_string(&relation_type).map_err(|error| error.to_string())?;
    let targets = adr_metadata::require_string_list(&targets).map_err(|error| error.to_string())?; let known = adr_metadata::require_string_list(&known).map_err(|error| error.to_string())?;
    let errors = adr_metadata::relation_target_errors(source, relation_type, &targets, &known).into_iter().map(project_relation_error).collect(); Ok(AdrValidationValue::Relations { errors })
}
#[rustfmt::skip]
fn schema_errors(mode: u8, value: CompatStructuredValue, schema: CompatStructuredValue, overlay: CompatStructuredValue) -> Result<AdrValidationValue, String> {
    let errors = adr_metadata::validate_schemas(&into_structured(value), &into_structured(schema), &into_structured(overlay)).map_err(|error| error.to_string())?;
    Ok(match mode { 0 => AdrValidationValue::SchemaLabels { base_has_error: errors.iter().any(|error| error.schema.as_str() == "base"), overlay_has_error: errors.iter().any(|error| error.schema.as_str() == "overlay") }, 1 => { let rejected = !errors.is_empty(); AdrValidationValue::SchemaLocations { errors: errors.iter().map(project_schema_error).collect(), rejected } }, _ => AdrValidationValue::SchemaErrors { errors: errors.iter().map(project_schema_error).collect() } })
}
fn project_relation_error(error: RelationTargetError) -> RelationErrorValue {
    match error {
        RelationTargetError::SelfReference => RelationErrorValue::SelfReference,
        RelationTargetError::MissingTarget(target) => RelationErrorValue::MissingTarget { target },
    }
}
fn project_schema_error(error: &SchemaValidationError) -> SchemaErrorValue {
    SchemaErrorValue {
        path: error.path.clone(),
        schema: error.schema.as_str(),
    }
}

#[rustfmt::skip]
fn resolve_canonical_adr(input: CanonicalAdrInput) -> Response {
    let result = match input {
        CanonicalAdrInput::ConfiguredProfilePath { repository } => configured_profile_path(&repository),
        CanonicalAdrInput::ResolveAdr { repository, adr_id, include_decision_body } => (|| { let adr_id = adr_id.as_str().ok_or_else(|| "adr_id must be a string".to_owned())?; let resolved = canonical_adr::resolve(Path::new(&repository), adr_id).map_err(|error| error.to_string())?; Ok(CanonicalAdrValue::Adr { adr_id: resolved.adr_id, relative_path: portable_path(&resolved.relative_path)?, decision_body_hex: include_decision_body.then(|| hex_encode(&resolved.decision_body)) }) })(),
    };
    finish(CANONICAL_ADR_OPERATION, result, ResultValue::CanonicalAdr)
}
fn configured_profile_path(repository: &str) -> Result<CanonicalAdrValue, String> {
    let model = repository_governance_model::load(Path::new(repository))
        .map_err(|error| error.to_string())?;
    let target = &model
        .capabilities
        .get("architecture_decisions")
        .and_then(|capability| capability.routes.get("profile"))
        .ok_or_else(|| "architecture_decisions profile route is missing".to_owned())?
        .target;
    let relative = target
        .strip_prefix(&model.repository_root)
        .map_err(|error| error.to_string())?;
    Ok(CanonicalAdrValue::Profile {
        relative_path: portable_path(relative)?,
    })
}

fn finish<T>(
    operation: &'static str,
    result: Result<T, String>,
    wrap: fn(T) -> ResultValue,
) -> Response {
    match result {
        Ok(value) => Response::result(operation, wrap(value)),
        Err(detail) => rejected(operation, detail),
    }
}
fn rejected(operation: &'static str, detail: String) -> Response {
    let category = match operation {
        ADR_PARSE_OPERATION => "adr-metadata.parse.rejected",
        ADR_PATH_OPERATION => "adr-metadata.repository-path.rejected",
        ADR_VALIDATION_OPERATION => "adr-metadata.validation.rejected",
        CANONICAL_ADR_OPERATION => "canonical-adr.resolve.rejected",
        _ => unreachable!("rejection operation is closed"),
    };
    Response::controlled_failure(operation, category, None, detail)
}
#[rustfmt::skip]
fn into_structured(value: CompatStructuredValue) -> StructuredValue { match value { CompatStructuredValue::Null(()) => StructuredValue::Null, CompatStructuredValue::Boolean(value) => StructuredValue::Boolean(value), CompatStructuredValue::Integer(value) => StructuredValue::Integer(value), CompatStructuredValue::String(value) => StructuredValue::String(value), CompatStructuredValue::Sequence(values) => StructuredValue::Sequence(values.into_iter().map(into_structured).collect()), CompatStructuredValue::Record(entries) => StructuredValue::Mapping(entries.into_iter().map(|entry| (entry.name, into_structured(entry.value))).collect()) } }
#[rustfmt::skip]
fn project_structured(value: &StructuredValue) -> CompatStructuredValue { match value { StructuredValue::Null => CompatStructuredValue::Null(()), StructuredValue::Boolean(value) => CompatStructuredValue::Boolean(*value), StructuredValue::Integer(value) => CompatStructuredValue::Integer(value.clone()), StructuredValue::String(value) => CompatStructuredValue::String(value.clone()), StructuredValue::Sequence(values) => CompatStructuredValue::Sequence(values.iter().map(project_structured).collect()), StructuredValue::Mapping(values) => CompatStructuredValue::Record(values.iter().map(|(name, value)| CompatRecordEntry { name: name.clone(), value: project_structured(value) }).collect()) } }
fn portable_path(path: &Path) -> Result<String, String> {
    path.components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| "path is not UTF-8".to_owned()),
            _ => Err("path is not repository-relative".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join("/"))
}
fn decode_bytes(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn decode_optional_hex(value: Option<String>) -> Option<Vec<u8>> {
    value.map(|value| decode_bytes(&value))
}
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}

#[derive(Debug)]
pub(crate) struct InternalFailure;

#[cfg(test)]
mod tests {
    use super::*;
    use proto_ring_engine::evidence_requirements::{
        ContextBinding, EvidenceClassAdmission, EvidenceClassKind, InstantiationKind,
        RequirementInstantiation,
    };
    use proto_ring_engine::github_authoritative_ref_monotonicity::{
        ArmDiagnostic, ArmDiagnosticKind, ArmStatus,
    };
    use proto_ring_engine::governed_objects::GovernedObjectRef;
    use serde_json::json;

    #[rustfmt::skip]
    fn persistent(shape: u8) -> PersistentEvidenceRequirement { PersistentEvidenceRequirement { id: format!("shape-{shape}"), responsibility: "evidence-binding".to_owned(), instances: RequirementInstantiation { kind: if shape == 3 { InstantiationKind::Source } else { InstantiationKind::Single }, source: (shape == 3).then(|| "instances".to_owned()) }, evidence_classes: EvidenceClassAdmission { kind: if shape == 3 { EvidenceClassKind::Source } else { EvidenceClassKind::Explicit }, classes: if shape != 3 { BTreeSet::from(["check".to_owned()]) } else { BTreeSet::new() }, source: (shape == 3).then(|| "classes".to_owned()) }, subject_source: "subject".to_owned(), context: ContextBinding { required: shape == 2, source: (shape == 2).then(|| "context".to_owned()) }, candidate_source: "candidates".to_owned(), target: (shape == 3).then(|| GovernedObjectRef { interface_id: "qualification".to_owned(), object_id: "artifact".to_owned() }) } }
    #[rustfmt::skip]
    #[test]
    fn evidence_requirement_projection_covers_three_typed_shapes() { let values = (1..=3).map(|shape| serde_json::to_value(project_requirement(&persistent(shape))).unwrap()).collect::<Vec<_>>(); assert_eq!(values[0]["instances"], json!({"kind":"single","source":null})); assert_eq!(values[0]["evidence_classes"], json!({"kind":"explicit","classes":["check"],"source":null})); assert_eq!(values[0]["context"], json!({"required":false,"source":null})); assert_eq!(values[1]["context"], json!({"required":true,"source":"context"})); assert_eq!(values[2]["instances"], json!({"kind":"source","source":"instances"})); assert_eq!(values[2]["evidence_classes"], json!({"kind":"source","classes":null,"source":"classes"})); assert_eq!(values[2]["target"], json!({"interface":"qualification","object":"artifact"})); }
    #[rustfmt::skip]
    #[test]
    fn arm_projection_uses_only_typed_status() { for (status, expected) in [(ArmStatus::Satisfied, "SATISFIED"), (ArmStatus::Violated, "VIOLATED"), (ArmStatus::Undetermined, "UNDETERMINED")] { let observation = ArmObservation { status, diagnostics: vec![ArmDiagnostic { kind: ArmDiagnosticKind::ProviderObservationUnavailable }] }; assert_eq!(project_github_arm(&observation).status, expected); } }
}
