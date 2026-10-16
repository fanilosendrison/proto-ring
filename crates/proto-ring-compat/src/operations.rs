use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use proto_ring_engine::accepted_adr_body::{self, AcceptedBodyDiagnosticKind};
use proto_ring_engine::evidence_requirements::{
    EvidenceRequirementRegistry, PersistentEvidenceRequirement,
};
use proto_ring_engine::exact_evidence_binding::{EvidenceBinding, EvidenceRequirement};
use proto_ring_engine::git_whitespace::{self, WhitespaceDomain};
use proto_ring_engine::github_authoritative_ref_monotonicity::{self, ArmObservation};
use proto_ring_engine::governance_bindings::{GovernanceBinding, GovernanceBindingIdentity};
use proto_ring_engine::repository_governance_state::{
    self, RepositoryGovernanceStage, RepositoryGovernanceStateError,
};
use proto_ring_engine::repository_integrity::{
    self, EvaluationContext, IntegrityResult, ObligationResult, ValidationEnvironmentRealization,
    ValidationResult,
};

use crate::protocol::{
    AcceptedAdrBodyValue, BindingAuthority, BindingIdentity, BindingProjection, BindingScope,
    ComponentPresence, ComposeInput, ComposeValue, ContextValue, DeclarationSourceValue,
    EngineIdentityValue, EvidenceBindingInput, EvidenceClassesValue, EvidenceRegistryInput,
    EvidenceRegistryValue, EvidenceRequirementInput, ExactBindingValue, ExactComparisonValue,
    ExactEvidenceInput, ExactHistoryValue, GitWhitespaceInput, GitWhitespaceValue, GithubArmInput,
    GithubArmValue, IntegrityInput, IntegrityValue, ObligationProjection,
    PersistentRequirementValue, PresentBinding, RegistryAuthorityValue, RepositoryInput, Request,
    Response, ResultValue, TRANSPORT_VERSION, TargetValue, ValidationProjection,
};

const SOURCE_REPOSITORY: &str = "fanilosendrison/proto-ring";
const SOURCE_COMMIT: &str = env!("PROTO_RING_SOURCE_COMMIT");
const EVIDENCE_OPERATION: &str = "exact-evidence-binding.evaluate";

pub(crate) fn execute(request: Request) -> Result<Response, InternalFailure> {
    Ok(match request {
        Request::EngineIdentity => Response::result(
            "engine.identity",
            ResultValue::EngineIdentity(EngineIdentityValue {
                source_repository: SOURCE_REPOSITORY,
                source_commit: SOURCE_COMMIT,
                transport_version: TRANSPORT_VERSION,
            }),
        ),
        Request::Compose(input) => compose(input)?,
        Request::EvaluateIntegrity(input) => evaluate_integrity(input)?,
        Request::CheckAcceptedAdrBody(input) => check_accepted_adr_body(input),
        Request::ValidateGitWhitespace(input) => validate_git_whitespace(input),
        Request::ObserveGithubArm(input) => observe_github_arm(input),
        Request::LoadEvidenceRegistry(input) => load_evidence_registry(input),
        Request::EvaluateExactEvidence(input) => evaluate_exact_evidence(input),
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
    let bindings = input.binding_ids.into_iter().map(|id| {
        let value = state.governance_bindings.bindings.get(&id).map(project_binding).unwrap_or(BindingProjection::Absent { present: false });
        (id, value)
    }).collect();
    let components = ComponentPresence {
        repository_governance_model: true, governance_authority: true,
        governed_objects: state.governed_objects.is_some(), governance_bindings: true,
        repository_integrity: state.repository_integrity.is_some(), projection_registry: state.projection_registry.is_some(),
        evidence_requirements: state.evidence_requirements.is_some(),
    };
    Ok(Response::result("repository-governance-state.compose", ResultValue::Compose(ComposeValue {
        stage: RepositoryGovernanceStage::Composition.serialized(), identity_present: !state.observed_state.identity.0.is_empty(), scope_paths, components, bindings,
    })))
}

#[rustfmt::skip]
fn project_binding(binding: &GovernanceBinding) -> BindingProjection {
    let identity = match &binding.identity {
        GovernanceBindingIdentity::ExecutableProvider(value) => BindingIdentity::ExecutableProvider { repository: value.repository.clone(), commit: value.commit.clone() },
        GovernanceBindingIdentity::GovernanceContract(value) => BindingIdentity::GovernanceContract { repository: value.repository.clone(), commit: value.commit.clone(), path: value.path.clone() },
    };
    BindingProjection::Present(Box::new(PresentBinding {
        present: true, kind: binding.kind.serialized(), identity,
        scope: BindingScope { kind: binding.scope.kind.serialized(), capability: binding.scope.capability.clone(), interface: binding.scope.interface.clone(), object: binding.scope.object.clone() },
        authority: BindingAuthority { responsibility: binding.authority.responsibility.clone(), source: binding.authority.source.clone() },
    }))
}

#[rustfmt::skip]
fn evaluate_integrity(input: IntegrityInput) -> Result<Response, InternalFailure> {
    let repository = Path::new(&input.repository);
    let state = match repository_governance_state::load(repository) { Ok(state) => state, Err(error) => return Ok(rgs_failure("repository-integrity.evaluate", error)) };
    let Some(profile) = state.repository_integrity.as_ref() else {
        return Ok(Response::controlled_failure("repository-integrity.evaluate", "repository-integrity-profile-absent", None,
            "repository governance state has no Repository Integrity profile".to_owned()));
    };
    let realizations = input.evaluation_context.realizations.into_iter().map(|item| {
        let environment_id = item.environment_id;
        (environment_id.clone(), ValidationEnvironmentRealization { environment_id, identity: item.identity_utf8.into_bytes(),
            command_prefix: item.command_prefix, process_environment: item.process_environment })
    }).collect();
    let context = EvaluationContext { realizations };
    let realized = match repository_integrity::realize_consumer_profile(repository, profile, &context) {
        Ok(realized) => realized,
        Err(error) => return Ok(Response::controlled_failure("repository-integrity.evaluate", "repository-integrity-realization", None, error.to_string())),
    };
    let result = repository_integrity::evaluate(repository, &realized.profile, realized.evaluation_context_identity);
    Ok(Response::result("repository-integrity.evaluate", ResultValue::Integrity(project_integrity_result(result))))
}

#[rustfmt::skip]
fn project_integrity_result(result: IntegrityResult) -> IntegrityValue {
    IntegrityValue { verdict: result.verdict.serialized(), state_relation: result.state_relation.serialized(), state_coherence_halt: result.state_coherence_halt,
        validations: result.validations.iter().map(project_validation).collect(), obligations: result.obligations.iter().map(project_obligation).collect() }
}
#[rustfmt::skip]
fn project_validation(value: &ValidationResult) -> ValidationProjection {
    ValidationProjection { id: value.id.clone(), status: value.status.serialized(), obligations: value.obligations.iter().map(project_obligation).collect() }
}
#[rustfmt::skip]
fn project_obligation(value: &ObligationResult) -> ObligationProjection {
    ObligationProjection { id: value.id.clone(), status: value.status.serialized(), executed: value.executed, exit_code: value.exit_code.clone() }
}

#[rustfmt::skip]
fn check_accepted_adr_body(input: RepositoryInput) -> Response {
    let report = accepted_adr_body::check(Path::new(&input.repository));
    let category = if report.diagnostics.is_empty() { None }
        else if report.diagnostics.iter().any(|item| item.kind == AcceptedBodyDiagnosticKind::BodyChanged) { Some("body_changed") }
        else { Some("historical_state_invalid") };
    let status = if category.is_none() { "PASS" } else { "NON_PASS" };
    Response::result("accepted-adr-body.immutability", ResultValue::AcceptedAdrBody(AcceptedAdrBodyValue { status, category }))
}

fn validate_git_whitespace(input: GitWhitespaceInput) -> Response {
    let report = git_whitespace::check(Path::new(&input.repository), &input.environment);
    let mut domains = Vec::new();
    for item in &report.diagnostics {
        let domain = match item.domain {
            WhitespaceDomain::Unstaged => "unstaged",
            WhitespaceDomain::Staged => "staged",
            WhitespaceDomain::PushCommittedRange => "push_committed_range",
            WhitespaceDomain::PullRequestCommittedRange => "pull_request_committed_range",
            WhitespaceDomain::RangeDetermination => "range_determination",
            WhitespaceDomain::GitObservation => "git_observation",
        };
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    let status = if report.passes() { "PASS" } else { "NON_PASS" };
    Response::result(
        "git-whitespace.validate",
        ResultValue::GitWhitespace(GitWhitespaceValue { status, domains }),
    )
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
    const OPERATION: &str = "evidence-requirements.registry";
    let state = match repository_governance_state::load(Path::new(&input.repository)) { Ok(state) => state, Err(error) => return rgs_failure(OPERATION, error) };
    let Some(registry) = state.evidence_requirements.as_ref() else {
        return Response::controlled_failure(OPERATION, "evidence-requirements-absent", None,
            "repository governance state has no Evidence Requirements registry".to_owned());
    };
    if input.requirement_ids.iter().any(|id| !registry.requirements.contains_key(id)) {
        return Response::controlled_failure(OPERATION, "evidence-requirement-not-found", None,
            "a requested evidence requirement is absent".to_owned());
    }
    let requirements = input.requirement_ids.into_iter().map(|id| (id.clone(), project_requirement(&registry.requirements[&id]))).collect();
    Response::result(OPERATION, ResultValue::EvidenceRegistry(project_registry(registry, requirements)))
}

#[rustfmt::skip]
fn project_registry(registry: &EvidenceRequirementRegistry, requirements: BTreeMap<String, PersistentRequirementValue>) -> EvidenceRegistryValue {
    EvidenceRegistryValue {
        authority_source: registry.authority.source.clone(), model_version: registry.model_version,
        registry_authority: RegistryAuthorityValue { responsibility: registry.authority.responsibility.clone(), source: registry.authority.source.clone() },
        requirement_ids: registry.requirements.keys().cloned().collect(), requirements,
    }
}

#[rustfmt::skip]
fn project_requirement(value: &PersistentEvidenceRequirement) -> PersistentRequirementValue {
    PersistentRequirementValue {
        id: value.id.clone(), responsibility: value.responsibility.clone(),
        instances: DeclarationSourceValue { kind: value.instances.kind.serialized(), source: value.instances.source.clone() },
        evidence_classes: EvidenceClassesValue { kind: value.evidence_classes.kind.serialized(),
            classes: value.evidence_classes.source.is_none().then(|| value.evidence_classes.classes.iter().cloned().collect()),
            source: value.evidence_classes.source.clone() },
        subject_source: value.subject_source.clone(),
        context: ContextValue { required: value.context.required, source: value.context.source.clone() },
        candidate_source: value.candidate_source.clone(),
        target: value.target.as_ref().map(|target| TargetValue { interface: target.interface_id.clone(), object: target.object_id.clone() }),
    }
}

#[rustfmt::skip]
fn evaluate_exact_evidence(input: ExactEvidenceInput) -> Response {
    match input {
        ExactEvidenceInput::EvaluateBinding { requirement, binding } => {
            let Ok(requirement) = build_requirement(requirement) else { return exact_rejection(); };
            let candidate = binding.0.map(build_binding);
            let status = requirement.evaluate(candidate.as_ref()).serialized();
            Response::result(EVIDENCE_OPERATION, ResultValue::ExactBinding(ExactBindingValue { status }))
        }
        ExactEvidenceInput::EvaluateRequirementHistory { comparisons } => {
            let prepared = comparisons.into_iter().map(|item| Ok((item.label,
                build_requirement(item.historical_requirement)?, build_requirement(item.current_requirement)?,
                item.binding.0.map(build_binding)))).collect::<Result<Vec<_>, ()>>();
            let Ok(prepared) = prepared else { return exact_rejection(); };
            let comparisons = prepared.into_iter().map(|(label, historical, current, binding)| ExactComparisonValue {
                label, historical: historical.evaluate(binding.as_ref()).serialized(), current: current.evaluate(binding.as_ref()).serialized(),
            }).collect();
            Response::result(EVIDENCE_OPERATION, ResultValue::ExactHistory(ExactHistoryValue { comparisons }))
        }
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
        decode_hex(input.subject_hex.0),
        decode_hex(input.context_hex.0),
        input.context_required,
    )
    .map_err(|_| ())
}

#[rustfmt::skip]
fn build_binding(input: EvidenceBindingInput) -> EvidenceBinding {
    EvidenceBinding { evidence_class: input.evidence_class.0, subject_identity: decode_hex(input.subject_hex.0), context_identity: decode_hex(input.context_hex.0) }
}

fn decode_hex(value: Option<String>) -> Option<Vec<u8>> {
    value.map(|value| {
        value
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    })
}

fn exact_rejection() -> Response {
    Response::controlled_failure(
        EVIDENCE_OPERATION,
        "exact-evidence-binding.evaluate.rejected",
        None,
        "evidence requirement is structurally invalid".to_owned(),
    )
}

#[derive(Debug)]
pub(crate) struct InternalFailure;

#[cfg(test)]
mod tests {
    use proto_ring_engine::evidence_requirements::{
        ContextBinding, EvidenceClassAdmission, EvidenceClassKind, InstantiationKind,
        RequirementInstantiation,
    };
    use proto_ring_engine::github_authoritative_ref_monotonicity::{
        ArmDiagnostic, ArmDiagnosticKind, ArmStatus,
    };
    use proto_ring_engine::governed_objects::GovernedObjectRef;
    use serde_json::json;

    use super::*;

    #[rustfmt::skip]
    fn persistent(shape: u8) -> PersistentEvidenceRequirement {
        PersistentEvidenceRequirement {
            id: format!("shape-{shape}"), responsibility: "evidence-binding".to_owned(),
            instances: RequirementInstantiation { kind: if shape == 3 { InstantiationKind::Source } else { InstantiationKind::Single }, source: (shape == 3).then(|| "instances".to_owned()) },
            evidence_classes: EvidenceClassAdmission { kind: if shape == 3 { EvidenceClassKind::Source } else { EvidenceClassKind::Explicit },
                classes: if shape != 3 { BTreeSet::from(["check".to_owned()]) } else { BTreeSet::new() }, source: (shape == 3).then(|| "classes".to_owned()) },
            subject_source: "subject".to_owned(), context: ContextBinding { required: shape == 2, source: (shape == 2).then(|| "context".to_owned()) },
            candidate_source: "candidates".to_owned(), target: (shape == 3).then(|| GovernedObjectRef { interface_id: "qualification".to_owned(), object_id: "artifact".to_owned() }),
        }
    }

    #[rustfmt::skip]
    #[test]
    fn evidence_requirement_projection_covers_three_typed_shapes() {
        let values = (1..=3).map(|shape| serde_json::to_value(project_requirement(&persistent(shape))).unwrap()).collect::<Vec<_>>();
        assert_eq!(values[0]["instances"], json!({"kind":"single","source":null}));
        assert_eq!(values[0]["evidence_classes"], json!({"kind":"explicit","classes":["check"],"source":null}));
        assert_eq!(values[0]["context"], json!({"required":false,"source":null}));
        assert_eq!(values[1]["context"], json!({"required":true,"source":"context"}));
        assert_eq!(values[2]["instances"], json!({"kind":"source","source":"instances"}));
        assert_eq!(values[2]["evidence_classes"], json!({"kind":"source","classes":null,"source":"classes"}));
        assert_eq!(values[2]["target"], json!({"interface":"qualification","object":"artifact"}));
    }

    #[rustfmt::skip]
    #[test]
    fn arm_projection_uses_only_typed_status() {
        for (status, expected) in [(ArmStatus::Satisfied, "SATISFIED"), (ArmStatus::Violated, "VIOLATED"), (ArmStatus::Undetermined, "UNDETERMINED")] {
            let observation = ArmObservation { status, diagnostics: vec![ArmDiagnostic { kind: ArmDiagnosticKind::ProviderObservationUnavailable }] };
            assert_eq!(project_github_arm(&observation).status, expected);
        }
    }
}
