#[path = "support/explanation-fixture.rs"]
mod explanation_fixture;
#[path = "support/git-governance.rs"]
mod git_governance;
#[path = "support/inspection-fixture.rs"]
mod inspection_fixture;

#[path = "explanation/authority.rs"]
mod authority;
#[path = "explanation/declarations.rs"]
mod declarations;
#[path = "explanation/retention.rs"]
mod retention;

use proto_ring_api::{ExplainQuery, ExplanationResult, GovernanceObservation};
use serde_json::{Value, json};

fn explain(observation: &GovernanceObservation, query: Value) -> Value {
    let typed: ExplainQuery = serde_json::from_value(query.clone()).unwrap();
    let result = observation.explain(typed.clone()).unwrap();
    assert_eq!(result.query(), typed);
    assert_eq!(result.provenance(), observation.provenance());
    let value = serde_json::to_value(&result).unwrap();
    assert_eq!(value["query"], query);
    static VALIDATOR: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
    let validator = VALIDATOR.get_or_init(|| {
        let canonical: Value = serde_json::from_str(include_str!("../../../docs/api/v1/schemas/api.schema.json")).unwrap();
        jsonschema::validator_for(&json!({"$schema":canonical["$schema"],"$defs":canonical["$defs"],"$ref":"#/$defs/explain_value"})).unwrap()
    });
    if let Some(error) = validator.iter_errors(&value).next() {
        panic!("{error}");
    }
    let round_trip: ExplanationResult = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(round_trip, result);
    value
}

fn path(fixture: &git_governance::GitGovernance, relative: &str) -> Value {
    json!({"kind":"utf8","value":fixture.root.join(relative).to_str().unwrap()})
}
