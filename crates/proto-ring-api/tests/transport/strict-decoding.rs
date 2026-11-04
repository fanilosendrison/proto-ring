use super::{request, response};
use proto_ring_api::{Operation, decode_request};
use serde_json::json;

fn rejected(raw: &[u8], operation: Option<Operation>, code: &str) {
    let error =
        decode_request(raw).expect_err(std::str::from_utf8(raw).unwrap_or("invalid UTF-8 input"));
    assert_eq!(error.operation, operation);
    let value = response(raw);
    assert_eq!(value["outcome"], "protocol_error");
    assert_eq!(value["error"]["code"], code);
}
#[test]
fn duplicates_at_every_depth_are_rejected_without_losing_later_root_operation() {
    for raw in [
        r#"{"api_version":1,"operation":"inspect","input":{"repository":{"kind":"utf8","value":".","value":"else"},"selector":{"kind":"all"}}}"#,
        r#"{"api_version":1,"input":{"repository":{"kind":"utf8","value":"."},"selector":{"kind":"all","kind":"model"}},"operation":"inspect"}"#,
        r#"{"operation":"inspect","input":{"repository":{"kind":"utf8","value":"."},"selector":{"kind":"governed_object","target":{"interface":"r","object":"a","object":"b"}}},"api_version":1}"#,
        r#"{"api_version":1,"api_version":1,"operation":"inspect","input":{}}"#,
        r#"{"operation":"inspect","operation":"inspect","api_version":1,"input":{}}"#,
        r#"{"operation":"inspect","api_version":1,"input":{},"extra":[{"x":1,"\u0078":2}]}"#,
    ] {
        rejected(raw.as_bytes(), Some(Operation::Inspect), "invalid_request");
    }
    rejected(
        br#"{"operation":"inspect","operation":"explain","api_version":1,"input":{}}"#,
        None,
        "invalid_request",
    );
}
#[test]
fn malformed_types_enums_unknown_properties_authorizing_fields_and_versions_fail_closed() {
    for raw in [
        r#"{"api_version":1.0,"operation":"discover","input":{}}"#,
        r#"{"api_version":1e0,"operation":"discover","input":{}}"#,
        r#"{"api_version":"1","operation":"discover","input":{}}"#,
        r#"{"api_version":true,"operation":"discover","input":{}}"#,
        r#"{"operation":"discover","input":{}}"#,
        r#"{"api_version":1,"operation":"discover","input":{"authorized":true}}"#,
        r#"{"api_version":1,"operation":"discover","input":{},"host":{}}"#,
        r#"{"api_version":1,"operation":"discover","input":[]}"#,
    ] {
        rejected(raw.as_bytes(), Some(Operation::Discover), "invalid_request");
    }
    rejected(
        br#"{"api_version":2,"operation":"discover","input":{}}"#,
        Some(Operation::Discover),
        "unsupported_version",
    );
    rejected(
        br#"{"api_version":184467440737095516160,"operation":"discover","input":{}}"#,
        Some(Operation::Discover),
        "unsupported_version",
    );
    for input in [
        json!({"repository":{"kind":"utf8","value":"."},"selector":{"kind":"all","extra":true}}),
        json!({"repository":{"kind":"utf8","value":"."},"selector":{"kind":"projection","id":"x"}}),
    ] {
        rejected(
            &request("inspect", input),
            Some(Operation::Inspect),
            "invalid_request",
        );
    }
}
#[test]
fn syntax_root_utf8_and_concatenation_fail_with_only_reliably_known_attribution() {
    for raw in [
        b"".as_slice(),
        b"[]",
        b"null",
        b"NaN",
        b"Infinity",
        b"{}",
        br#"{"operation":"unknown"}"#,
    ] {
        rejected(raw, None, "invalid_request");
    }
    for raw in [
        br#"{"operation":"inspect","api_version":1,"input":NaN}"#.as_slice(),
        br#"{"operation":"inspect","input":"unterminated}"#,
        br#"{"operation":"inspect"}{}"#,
        br#"{"operation":"inspect","input":Infinity}"#,
    ] {
        rejected(raw, Some(Operation::Inspect), "invalid_request");
    }
    rejected(
        b"{\"operation\":\"inspect\",\"input\":\"\xff\"}",
        Some(Operation::Inspect),
        "invalid_request",
    );
    rejected(b"\xff", None, "invalid_request");
}
#[test]
fn exact_request_byte_limit_accepts_boundary_and_rejects_overflow_explicitly() {
    let mut bytes = request("discover", json!({}));
    bytes.resize(1_048_575, b' ');
    assert!(decode_request(&bytes).is_ok());
    bytes.push(b' ');
    assert!(decode_request(&bytes).is_ok());
    bytes.push(b' ');
    rejected(&bytes, Some(Operation::Discover), "invalid_request");
    let json = request("discover", json!({}));
    assert_eq!(response(&json)["outcome"], "result");
}
#[test]
fn common_boundary_validates_native_units_and_all_api03_syntax_before_dispatch() {
    let foreign = if cfg!(unix) {
        json!({"kind":"windows_wide_be_base64","value":"AC4="})
    } else {
        json!({"kind":"unix_bytes_base64","value":"Lg=="})
    };
    for path in [
        json!({"kind":"utf8","value":""}),
        json!({"kind":"utf8","value":"a\u{0}b"}),
        foreign,
        json!({"kind":"unix_bytes_base64","value":"Zh=="}),
        json!({"kind":"windows_wide_be_base64","value":"QQ=="}),
    ] {
        rejected(
            &request(
                "check_repository_integrity",
                json!({"repository":path,"expected_observation":null}),
            ),
            Some(Operation::CheckRepositoryIntegrity),
            "invalid_request",
        );
    }
    for max in [json!(0), json!(65537), json!(1.0), json!("1")] {
        rejected(
            &request(
                "read_execution_output",
                json!({"receipt_id":"a".repeat(64),"validation_id":"v","instance_index":"0","stream":"stdout","offset":"0","max_bytes":max}),
            ),
            Some(Operation::ReadExecutionOutput),
            "invalid_request",
        );
    }
}
