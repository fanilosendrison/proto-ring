#![forbid(unsafe_code)]

use jsonschema::draft202012;
use serde_json::{Value, json};

fn qualified_validator(schema: &Value) -> jsonschema::Validator {
    draft202012::options()
        .should_validate_formats(true)
        .offline()
        .build(schema)
        .expect("qualified Draft 2020-12 schema builds offline")
}

#[test]
fn draft_2020_12_meta_schema_and_required_keywords_are_supported() {
    let malformed = json!({"type": "not-a-json-schema-type"});
    assert!(draft202012::meta::validate(&malformed).is_err());

    let admitted = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {"kind": {"const": "ADR"}},
        "required": ["kind"]
    });
    assert!(draft202012::meta::validate(&admitted).is_ok());
    let validator = qualified_validator(&admitted);
    assert!(validator.is_valid(&json!({"kind": "ADR"})));
    assert!(!validator.is_valid(&json!({"kind": "other"})));

    eprintln!("JSONSCHEMA_DRAFT_2020_12=QUALIFIED");
}

#[test]
fn date_format_assertions_are_calendar_aware() {
    let schema = json!({"type": "string", "format": "date"});
    let validator = qualified_validator(&schema);
    assert!(!validator.is_valid(&json!("2025-02-29")));
    assert!(validator.is_valid(&json!("2024-02-29")));

    eprintln!("JSONSCHEMA_FORMAT_ASSERTIONS=QUALIFIED");
}

#[test]
fn offline_mode_rejects_network_and_filesystem_retrieval() {
    for reference in [
        "https://example.invalid/external-schema.json",
        "file:///definitely-not-readable/issue64-schema.json",
    ] {
        let schema = json!({"$ref": reference});
        let error = draft202012::options()
            .should_validate_formats(true)
            .offline()
            .build(&schema)
            .expect_err("offline validator rejects unresolved external reference");
        assert!(error.to_string().contains("Retrieval is disabled"));
    }

    eprintln!("JSONSCHEMA_OFFLINE=QUALIFIED");
}

#[test]
fn arbitrary_precision_numeric_assertion_preserves_exact_value() {
    let schema: Value = serde_json::from_str("{\"const\":18446744073709551616}")
        .expect("arbitrary-precision schema number parses from text");
    let exact: Value = serde_json::from_str("18446744073709551616")
        .expect("arbitrary-precision instance parses from text");
    let adjacent: Value = serde_json::from_str("18446744073709551617")
        .expect("adjacent arbitrary-precision instance parses from text");
    assert_eq!(exact.to_string(), "18446744073709551616");
    let validator = qualified_validator(&schema);
    assert!(validator.is_valid(&exact));
    assert!(!validator.is_valid(&adjacent));

    eprintln!("JSONSCHEMA_ARBITRARY_PRECISION=QUALIFIED");
    eprintln!("JSONSCHEMA_DEPENDENCY_QUALIFICATION=GO");
}
