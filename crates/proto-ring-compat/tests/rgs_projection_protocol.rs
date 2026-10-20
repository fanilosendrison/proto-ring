mod rgs_projection_support;

use serde_json::{Value, json};

use rgs_projection_support::{complete_fixture, compose, invoke};

fn value_keys(value: &Value) -> Vec<&str> {
    value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn omitted_projection_preserves_the_exact_summary_value_shape() {
    let fixture = complete_fixture();
    let (output, response) = compose(fixture.path(), None);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        value_keys(&response["value"]),
        [
            "bindings",
            "components",
            "identity_present",
            "scope_paths",
            "stage"
        ]
    );
    assert!(response["value"].get("state").is_none());
}

#[test]
fn explicit_summary_is_identical_to_the_omitted_default() {
    let fixture = complete_fixture();
    let (_, omitted) = compose(fixture.path(), None);
    let (_, explicit) = compose(fixture.path(), Some("summary"));
    assert_eq!(explicit["value"], omitted["value"]);
    assert!(explicit["value"].get("state").is_none());
}

#[test]
fn complete_adds_only_state_to_the_existing_summary() {
    let fixture = complete_fixture();
    let (_, summary) = compose(fixture.path(), None);
    let (_, complete) = compose(fixture.path(), Some("complete"));
    let mut complete_value = complete["value"].as_object().unwrap().clone();
    let state = complete_value.remove("state").expect("complete state");
    assert_eq!(Value::Object(complete_value), summary["value"]);
    assert_eq!(
        value_keys(&state),
        [
            "evidence_requirements",
            "governance_authority",
            "governance_bindings",
            "governed_objects",
            "projection_registry",
            "repository_governance_model",
            "repository_integrity"
        ]
    );
}

#[test]
fn compose_projection_input_is_closed() {
    let fixture = complete_fixture();
    for input in [
        json!({"repository":fixture.path(),"projection":"other"}),
        json!({"repository":fixture.path(),"projection":1}),
        json!({"repository":fixture.path(),"projection":null}),
        json!({"repository":fixture.path(),"projection":"summary","extra":true}),
    ] {
        let (output, response) = invoke(&json!({
            "transport_version":1,
            "operation":"repository-governance-state.compose",
            "input":input
        }));
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(
            response,
            json!({"transport_version":1,"outcome":"transport_error","error":{"category":"invalid_request"}})
        );
    }
}
