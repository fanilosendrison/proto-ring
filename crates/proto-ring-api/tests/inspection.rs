#[path = "support/git-governance.rs"]
mod git_governance;
#[path = "support/inspection-fixture.rs"]
mod inspection_fixture;

#[path = "inspection/completeness.rs"]
mod completeness;
#[path = "inspection/components.rs"]
mod components;
#[path = "inspection/retention.rs"]
mod retention;
#[path = "inspection/subjects.rs"]
mod subjects;

use proto_ring_api::{GovernanceObservation, InspectSelector};
use serde_json::{Value, json};

fn inspect(observation: &GovernanceObservation, selector: Value) -> Value {
    let selector: InspectSelector = serde_json::from_value(selector).unwrap();
    let result = observation.inspect(selector.clone()).unwrap();
    assert_eq!(result.selector(), &selector);
    assert_eq!(result.provenance(), observation.provenance());
    let value = serde_json::to_value(result).unwrap();
    static VALIDATOR: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
    let validator = VALIDATOR.get_or_init(|| {
        let canonical: Value = serde_json::from_str(include_str!("../../../docs/api/v1/schemas/api.schema.json")).unwrap();
        jsonschema::validator_for(&json!({"$schema":canonical["$schema"],"$defs":canonical["$defs"],"$ref":"#/$defs/inspect_value"})).unwrap()
    });
    if let Some(error) = validator.iter_errors(&value).next() {
        panic!("{error}");
    }
    value
}

fn path(fixture: &git_governance::GitGovernance, relative: &str) -> Value {
    json!({"kind":"utf8","value":fixture.root.join(relative).to_str().unwrap()})
}
