use std::collections::BTreeMap;
use std::path::Path;

use proto_ring_engine::accepted_adr_body::{self, AcceptedBodyDiagnosticKind};
use proto_ring_engine::git_whitespace::{self, WhitespaceDomain};
use proto_ring_engine::github_authoritative_ref_monotonicity::{self, ArmObservation};
use proto_ring_engine::governance_bindings::{GovernanceBinding, GovernanceBindingIdentity};
use proto_ring_engine::repository_governance_state::{self, RepositoryGovernanceStage};
use proto_ring_engine::repository_integrity::{
    self, EvaluationContext, IntegrityResult, ObligationResult, ValidationEnvironmentRealization,
    ValidationResult,
};

use crate::protocol::{
    AcceptedAdrBodyInput, AcceptedAdrBodyValue, BindingAuthority, BindingIdentity,
    BindingProjection, BindingScope, ComponentPresence, ComposeInput, ComposeValue,
    EngineIdentityValue, GitWhitespaceInput, GitWhitespaceValue, GithubArmInput, GithubArmValue,
    IntegrityInput, IntegrityValue, ObligationProjection, PresentBinding, Request, Response,
    ResultValue, TRANSPORT_VERSION, ValidationProjection,
};

const SOURCE_REPOSITORY: &str = "fanilosendrison/proto-ring";
const SOURCE_COMMIT: &str = env!("PROTO_RING_SOURCE_COMMIT");

pub(crate) fn execute(request: Request) -> Result<Response, InternalFailure> {
    match request {
        Request::EngineIdentity => Ok(Response::result(
            "engine.identity",
            ResultValue::EngineIdentity(EngineIdentityValue {
                source_repository: SOURCE_REPOSITORY,
                source_commit: SOURCE_COMMIT,
                transport_version: TRANSPORT_VERSION,
            }),
        )),
        Request::Compose(input) => compose(input),
        Request::EvaluateIntegrity(input) => evaluate_integrity(input),
        Request::CheckAcceptedAdrBody(input) => Ok(check_accepted_adr_body(input)),
        Request::ValidateGitWhitespace(input) => Ok(validate_git_whitespace(input)),
        Request::ObserveGithubArm(input) => Ok(observe_github_arm(input)),
    }
}

fn compose(input: ComposeInput) -> Result<Response, InternalFailure> {
    let state = match repository_governance_state::load(Path::new(&input.repository)) {
        Ok(state) => state,
        Err(error) => {
            return Ok(Response::controlled_failure(
                "repository-governance-state.compose",
                "repository-governance-state",
                Some(error.diagnostic.stage.serialized()),
                error.diagnostic.detail,
            ));
        }
    };
    let scope_paths = state
        .observed_state
        .scope_paths
        .iter()
        .map(|path| path.to_str().map(str::to_owned).ok_or(InternalFailure))
        .collect::<Result<Vec<_>, _>>()?;
    let bindings = input
        .binding_ids
        .into_iter()
        .map(|binding_id| {
            let projection = state
                .governance_bindings
                .bindings
                .get(&binding_id)
                .map(project_binding)
                .unwrap_or(BindingProjection::Absent { present: false });
            (binding_id, projection)
        })
        .collect();
    Ok(Response::result(
        "repository-governance-state.compose",
        ResultValue::Compose(ComposeValue {
            stage: RepositoryGovernanceStage::Composition.serialized(),
            identity_present: !state.observed_state.identity.0.is_empty(),
            scope_paths,
            components: ComponentPresence {
                repository_governance_model: true,
                governance_authority: true,
                governed_objects: state.governed_objects.is_some(),
                governance_bindings: true,
                repository_integrity: state.repository_integrity.is_some(),
                projection_registry: state.projection_registry.is_some(),
                evidence_requirements: state.evidence_requirements.is_some(),
            },
            bindings,
        }),
    ))
}

fn project_binding(binding: &GovernanceBinding) -> BindingProjection {
    let identity = match &binding.identity {
        GovernanceBindingIdentity::ExecutableProvider(identity) => {
            BindingIdentity::ExecutableProvider {
                repository: identity.repository.clone(),
                commit: identity.commit.clone(),
            }
        }
        GovernanceBindingIdentity::GovernanceContract(identity) => {
            BindingIdentity::GovernanceContract {
                repository: identity.repository.clone(),
                commit: identity.commit.clone(),
                path: identity.path.clone(),
            }
        }
    };
    BindingProjection::Present(Box::new(PresentBinding {
        present: true,
        kind: binding.kind.serialized(),
        scope: BindingScope {
            kind: binding.scope.kind.serialized(),
            capability: binding.scope.capability.clone(),
            interface: binding.scope.interface.clone(),
            object: binding.scope.object.clone(),
        },
        identity,
        authority: BindingAuthority {
            responsibility: binding.authority.responsibility.clone(),
            source: binding.authority.source.clone(),
        },
    }))
}

fn evaluate_integrity(input: IntegrityInput) -> Result<Response, InternalFailure> {
    let repository = Path::new(&input.repository);
    let state = match repository_governance_state::load(repository) {
        Ok(state) => state,
        Err(error) => {
            return Ok(Response::controlled_failure(
                "repository-integrity.evaluate",
                "repository-governance-state",
                Some(error.diagnostic.stage.serialized()),
                error.diagnostic.detail,
            ));
        }
    };
    let Some(profile) = state.repository_integrity.as_ref() else {
        return Ok(Response::controlled_failure(
            "repository-integrity.evaluate",
            "repository-integrity-profile-absent",
            None,
            "repository governance state has no Repository Integrity profile".to_owned(),
        ));
    };
    let context = EvaluationContext {
        realizations: input
            .evaluation_context
            .realizations
            .into_iter()
            .map(|realization| {
                let environment_id = realization.environment_id;
                (
                    environment_id.clone(),
                    ValidationEnvironmentRealization {
                        environment_id,
                        identity: realization.identity_utf8.into_bytes(),
                        command_prefix: realization.command_prefix,
                        process_environment: realization.process_environment,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
    };
    let realized =
        match repository_integrity::realize_consumer_profile(repository, profile, &context) {
            Ok(realized) => realized,
            Err(error) => {
                return Ok(Response::controlled_failure(
                    "repository-integrity.evaluate",
                    "repository-integrity-realization",
                    None,
                    error.to_string(),
                ));
            }
        };
    let result = repository_integrity::evaluate(
        repository,
        &realized.profile,
        realized.evaluation_context_identity,
    );
    Ok(Response::result(
        "repository-integrity.evaluate",
        ResultValue::Integrity(project_integrity_result(result)),
    ))
}

fn project_integrity_result(result: IntegrityResult) -> IntegrityValue {
    IntegrityValue {
        verdict: result.verdict.serialized(),
        state_relation: result.state_relation.serialized(),
        state_coherence_halt: result.state_coherence_halt,
        validations: result.validations.iter().map(project_validation).collect(),
        obligations: result.obligations.iter().map(project_obligation).collect(),
    }
}

fn project_validation(validation: &ValidationResult) -> ValidationProjection {
    ValidationProjection {
        id: validation.id.clone(),
        status: validation.status.serialized(),
        obligations: validation
            .obligations
            .iter()
            .map(project_obligation)
            .collect(),
    }
}

fn project_obligation(obligation: &ObligationResult) -> ObligationProjection {
    ObligationProjection {
        id: obligation.id.clone(),
        status: obligation.status.serialized(),
        executed: obligation.executed,
        exit_code: obligation.exit_code.clone(),
    }
}

fn check_accepted_adr_body(input: AcceptedAdrBodyInput) -> Response {
    let report = accepted_adr_body::check(Path::new(&input.repository));
    let category = if report.diagnostics.is_empty() {
        None
    } else if report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == AcceptedBodyDiagnosticKind::BodyChanged)
    {
        Some("body_changed")
    } else {
        Some("historical_state_invalid")
    };
    Response::result(
        "accepted-adr-body.immutability",
        ResultValue::AcceptedAdrBody(AcceptedAdrBodyValue {
            status: if report.diagnostics.is_empty() {
                "PASS"
            } else {
                "NON_PASS"
            },
            category,
        }),
    )
}

fn validate_git_whitespace(input: GitWhitespaceInput) -> Response {
    let report = git_whitespace::check(Path::new(&input.repository), &input.environment);
    let mut domains = Vec::new();
    for diagnostic in &report.diagnostics {
        let domain = match diagnostic.domain {
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
    Response::result(
        "git-whitespace.validate",
        ResultValue::GitWhitespace(GitWhitespaceValue {
            status: if report.passes() { "PASS" } else { "NON_PASS" },
            domains,
        }),
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

#[derive(Debug)]
pub(crate) struct InternalFailure;

#[cfg(test)]
mod tests {
    use proto_ring_engine::github_authoritative_ref_monotonicity::{
        ArmDiagnostic, ArmDiagnosticKind, ArmObservation, ArmStatus,
    };

    use super::project_github_arm;

    #[test]
    fn arm_projection_uses_only_typed_status() {
        for (status, expected) in [
            (ArmStatus::Satisfied, "SATISFIED"),
            (ArmStatus::Violated, "VIOLATED"),
            (ArmStatus::Undetermined, "UNDETERMINED"),
        ] {
            let observation = ArmObservation {
                status,
                diagnostics: vec![ArmDiagnostic {
                    kind: ArmDiagnosticKind::ProviderObservationUnavailable,
                }],
            };
            assert_eq!(project_github_arm(&observation).status, expected);
        }
    }
}
