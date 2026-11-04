use super::{request, response};
use proto_ring_api::{Operation, decode_request};
use serde_json::{Value, json};

#[test]
fn complete_accepted_negative_request_corpus_is_rejected_by_real_boundary() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../../docs/api/v1/conformance/schema-cases.json"
    ))
    .unwrap();
    let mut count = 0;
    for case in cases.as_array().unwrap() {
        if case["target"] != "request" || case["expect"] != "reject" {
            continue;
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/api/v1")
            .join(case["document"].as_str().unwrap());
        let bytes = std::fs::read(path).unwrap();
        assert!(decode_request(&bytes).is_err(), "{}", case["id"]);
        count += 1;
    }
    assert_eq!(count, 4, "frozen negative request inventory changed");
}
#[test]
fn syntax_types_authorizing_fields_and_exact_u64_ranges_are_checked_across_api03_inputs() {
    let valid = json!({"receipt_id":"a".repeat(64),"validation_id":"v","instance_index":"0","stream":"stdout","offset":"0","max_bytes":1});
    assert!(decode_request(&request("read_execution_output", valid.clone())).is_ok());
    for (key, bad) in [
        ("receipt_id", json!("../secret")),
        ("validation_id", json!("")),
        ("stream", json!("merged")),
        ("offset", json!(0)),
        ("instance_index", json!(true)),
        ("authorized", json!(true)),
        ("receipt_path", json!("/forbidden")),
    ] {
        let mut input = valid.clone();
        input[key] = bad;
        let error = decode_request(&request("read_execution_output", input)).unwrap_err();
        assert_eq!(error.operation, Some(Operation::ReadExecutionOutput));
    }
    for decimal in ["00", "-1", "1.0", "1e3", "18446744073709551616"] {
        for field in ["offset", "instance_index"] {
            let mut input = valid.clone();
            input[field] = json!(decimal);
            assert!(decode_request(&request("read_execution_output", input)).is_err());
        }
    }
    let mut check = json!({"repository":{"kind":"utf8","value":"."},"expected_observation":null});
    check["host"] = json!({"authorized":true});
    assert!(decode_request(&request("check_repository_integrity", check)).is_err());
}
#[test]
fn nested_operation_spelling_is_not_attribution_and_unbounded_integer_version_is_not_coerced() {
    assert_eq!(
        response(br#"{"api_version":1,"input":{"operation":"discover"}}"#)["operation"],
        json!(null)
    );
    let raw = format!(
        r#"{{"api_version":{},"operation":"discover","input":{{}}}}"#,
        "9".repeat(400)
    );
    let error = decode_request(raw.as_bytes()).unwrap_err();
    assert_eq!(error.operation, Some(Operation::Discover));
    assert_eq!(
        response(raw.as_bytes())["error"]["code"],
        "unsupported_version"
    );
    let unicode = "é".repeat(524_288);
    let raw = request(
        "explain",
        json!({"repository":{"kind":"utf8","value":"."},"query":{"kind":"binding","id":unicode}}),
    );
    assert!(raw.len() > 1_048_576);
    assert!(decode_request(&raw).is_err());
}
