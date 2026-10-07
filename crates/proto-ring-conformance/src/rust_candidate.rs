#![forbid(unsafe_code)]

mod authority_objects;
mod foundation;
mod transport;

use crate::fixture::{self, MaterializedFixture};
use crate::harness::{CandidateExecutor, CandidateRequest, CandidateState, HarnessError};
use crate::model::Observation;

pub struct ReferenceRustCandidate;

const IMPLEMENTED: &[&str] = &[
    "governance-authority.profile",
    "governance-bootstrap.root",
    "governance-routing.resolve",
    "governed-objects.catalog",
    "repository-governance-model.binding-compatibility",
    "repository-governance-model.load",
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
