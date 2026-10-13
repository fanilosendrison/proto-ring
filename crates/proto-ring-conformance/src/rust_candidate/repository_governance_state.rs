#![forbid(unsafe_code)]

mod observe;
mod plan;

use crate::fixture;
use crate::model::Observation;

pub(super) fn execute(fixture: &serde_json::Value) -> Result<Observation, fixture::FixtureError> {
    plan::execute(fixture)
}
