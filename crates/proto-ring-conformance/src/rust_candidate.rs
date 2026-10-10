#![forbid(unsafe_code)]

mod authority_objects;
mod evidence_requirements;
mod exact_evidence_binding;
mod foundation;
mod governance_bindings;
mod projection_registry;
mod repository_integrity;
mod support;
mod transport;

use crate::fixture::{self, MaterializedFixture};
use crate::harness::{CandidateExecutor, CandidateRequest, CandidateState, HarnessError};
use crate::model::Observation;

pub struct ReferenceRustCandidate;

const IMPLEMENTED: &[&str] = &[
    "evidence-requirements.registry",
    "exact-evidence-binding.evaluate",
    "governance-authority.profile",
    "governance-bindings.registry",
    "governance-bootstrap.root",
    "governance-routing.resolve",
    "governed-objects.catalog",
    "projection-registry.registry",
    "repository-governance-model.binding-compatibility",
    "repository-governance-model.load",
    "repository-integrity.profile",
    "structured-data.document",
    "structured-data.frontmatter",
];

impl CandidateExecutor for ReferenceRustCandidate {
    fn execute(&self, request: &CandidateRequest) -> Result<CandidateState, HarnessError> {
        if !IMPLEMENTED.contains(&request.responsibility_id.as_str()) {
            return Ok(CandidateState::Unimplemented);
        }
        let fixture_error =
            |error| HarnessError::new(format!("fixture failed for {}: {error}", request.vector_id));
        if request.responsibility_id == "repository-integrity.profile"
            && let Some(observation) = repository_integrity::execute_identity_plan(&request.fixture)
                .map_err(fixture_error)?
        {
            return Ok(CandidateState::Observation(observation));
        }
        let realized = fixture::materialize(&request.fixture).map_err(fixture_error)?;
        execute_responsibility(&request.responsibility_id, &realized)
            .map(CandidateState::Observation)
            .map_err(fixture_error)
    }
}

fn execute_responsibility(
    responsibility: &str,
    fixture: &MaterializedFixture,
) -> Result<Observation, fixture::FixtureError> {
    match responsibility {
        "evidence-requirements.registry" => evidence_requirements::execute(fixture),
        "exact-evidence-binding.evaluate" => exact_evidence_binding::execute(fixture.arguments()),
        "governance-bindings.registry" => governance_bindings::execute(fixture),
        "projection-registry.registry" => projection_registry::execute(fixture),
        "repository-integrity.profile" => repository_integrity::execute(fixture),
        "governance-authority.profile" | "governed-objects.catalog" => {
            authority_objects::execute(responsibility, fixture)
        }
        "structured-data.document"
        | "structured-data.frontmatter"
        | "governance-bootstrap.root"
        | "governance-routing.resolve"
        | "repository-governance-model.load"
        | "repository-governance-model.binding-compatibility" => {
            foundation::execute(responsibility, fixture)
        }
        _ => unreachable!("implemented responsibility inventory is closed"),
    }
}
