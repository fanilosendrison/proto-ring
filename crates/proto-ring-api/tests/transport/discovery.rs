use super::{API, request, response};
use proto_ring_api::discover;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[test]
fn entire_discovery_equals_independent_accepted_example_and_fixed_catalog() {
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../docs/api/v1/examples/response-discover.json"
    ))
    .unwrap();
    let value = response(&request("discover", json!({})));
    assert_eq!(value, expected);
    assert_eq!(
        serde_json::to_value(discover().unwrap()).unwrap(),
        expected["value"]
    );
    assert_eq!(
        value["value"]["operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["operation"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "discover",
            "inspect",
            "explain",
            "check_repository_integrity",
            "read_execution_output"
        ]
    );
}
fn refs(value: &Value, names: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if key == "$ref" {
                    let name = child.as_str().unwrap().strip_prefix("#/$defs/").unwrap();
                    assert!(!name.contains('/'));
                    names.insert(name.into());
                } else {
                    refs(child, names);
                }
            }
        }
        Value::Array(array) => {
            for child in array {
                refs(child, names);
            }
        }
        _ => {}
    }
}
#[test]
fn all_ten_schemas_are_exact_local_closures_and_accept_canonical_requests_and_responses() {
    let api: Value = serde_json::from_str(API).unwrap();
    for entry in discover().unwrap().operations {
        let operation = serde_json::to_value(entry.operation).unwrap();
        for (kind, schema) in [
            ("request", entry.request_schema),
            ("response", entry.response_schema),
        ] {
            let mut fragment = schema.clone();
            fragment.as_object_mut().unwrap().remove("$defs");
            let mut needed = BTreeSet::new();
            refs(&fragment, &mut needed);
            loop {
                let old = needed.clone();
                for name in &old {
                    refs(&api["$defs"][name], &mut needed);
                }
                if old == needed {
                    break;
                }
            }
            assert_eq!(
                schema["$defs"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                needed
            );
            for name in needed {
                assert_eq!(schema["$defs"][&name], api["$defs"][&name]);
            }
            let validator = jsonschema::validator_for(&schema).unwrap();
            let cases: Value = serde_json::from_str(include_str!(
                "../../../../docs/api/v1/conformance/schema-cases.json"
            ))
            .unwrap();
            let mut checked = 0;
            for case in cases.as_array().unwrap() {
                if case["target"] != kind || case["expect"] != "accept" {
                    continue;
                }
                let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../docs/api/v1")
                    .join(case["document"].as_str().unwrap());
                let fixture: Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
                if fixture["operation"] == operation {
                    assert!(validator.is_valid(&fixture), "{operation} {kind}");
                    checked += 1;
                }
            }
            assert!(checked > 0);
        }
    }
}
