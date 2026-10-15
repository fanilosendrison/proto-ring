use std::collections::BTreeMap;
use std::path::Path;

use proto_ring_engine::governance_bindings::{GovernanceBinding, GovernanceBindingIdentity};
use proto_ring_engine::repository_governance_state::{self, RepositoryGovernanceStage};
use proto_ring_engine::repository_integrity::{
    self, EvaluationContext, IntegrityResult, ObligationResult, ValidationEnvironmentRealization,
    ValidationResult,
};

use crate::protocol::{
    BindingAuthority, BindingIdentity, BindingProjection, BindingScope, ComponentPresence,
    ComposeInput, ComposeValue, EngineIdentityValue, IntegrityInput, IntegrityValue,
    ObligationProjection, PresentBinding, Request, Response, ResultValue, TRANSPORT_VERSION,
    ValidationProjection,
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

#[derive(Debug)]
pub(crate) struct InternalFailure;
