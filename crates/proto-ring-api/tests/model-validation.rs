#[path = "support/wire-oracle.rs"]
mod wire_oracle;

use proto_ring_api::{
    ApiError, AuthoritySource, Base64Text, BindingScope, Components,
    EvidenceRequirementDeclaration, ExplainQuery, ExplanationResult, FailureResponse, HexDigest,
    Identifier, InspectSelector, Inspection, NativePath, Presence, ProjectionDeclaration, Request,
    Response, SignedDecimal, StructuredValue, U64Decimal, ValidationDeclaration,
    ValidationInstances,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use wire_oracle::example;

fn rejects<T: DeserializeOwned>(value: Value) {
    assert!(
        serde_json::from_value::<T>(value.clone()).is_err(),
        "unexpected acceptance: {value}"
    );
}

#[test]
fn closed_variants_reject_extra_fields_and_unknown_discriminants() {
    wire_oracle::round_trip::<InspectSelector>(&json!({"kind":"all"}), "inspect_selector");
    rejects::<InspectSelector>(json!({"kind":"all","id":"extra"}));
    rejects::<InspectSelector>(json!({"kind":"projection","id":"p"}));
    rejects::<InspectSelector>(json!({"kind":"validation","id":""}));
    rejects::<ExplainQuery>(
        json!({"kind":"authority_role","responsibility":"r","source":"s","extra":0}),
    );
    rejects::<NativePath>(json!({"kind":"display","value":"path"}));
    rejects::<NativePath>(json!({"kind":"utf8","value":""}));
    rejects::<StructuredValue>(json!({"kind":"null","value":null}));
    rejects::<StructuredValue>(json!({"kind":"integer","decimal":123}));
    rejects::<StructuredValue>(json!({"kind":"boolean","value":"true"}));
    rejects::<BindingScope>(json!({"kind":"logical_provider","extra":null}));
    rejects::<ValidationInstances>(json!({"kind":"single","extra":null}));
}

#[test]
fn scalar_types_preserve_exact_text_without_numeric_coercions() {
    for value in ["", "-0", "+1", "01", "-01", "1.0", "1e2", " 1", "١"] {
        assert!(SignedDecimal::try_from(value).is_err(), "{value}");
    }
    for value in ["", "-1", "01", "+1", "18446744073709551616"] {
        assert!(U64Decimal::try_from(value).is_err(), "{value}");
    }
    assert_eq!(
        U64Decimal::try_from("18446744073709551615")
            .unwrap()
            .as_str(),
        "18446744073709551615"
    );
    assert!(Identifier::try_from("").is_err());
    assert_eq!(Identifier::try_from(" ").unwrap().as_str(), " ");
    for value in ["A".repeat(64), "a".repeat(63), "g".repeat(64)] {
        assert!(HexDigest::try_from(value).is_err());
    }
    for value in ["YQ", "YR==", "YQ===", "YQ==\n", "____"] {
        assert!(Base64Text::try_from(value).is_err(), "{value}");
    }
    assert_eq!(Base64Text::try_from("YQ==").unwrap().as_str(), "YQ==");
}

#[test]
fn mapping_entries_reject_duplicate_or_unsorted_keys_without_coalescing() {
    for keys in [["a", "a"], ["b", "a"]] {
        rejects::<StructuredValue>(json!({"kind":"mapping","entries":[
            {"key":keys[0],"value":{"kind":"null"}},
            {"key":keys[1],"value":{"kind":"null"}}
        ]}));
    }
}

#[test]
fn required_nullable_declaration_fields_cannot_disappear() {
    rejects::<AuthoritySource>(json!({}));
    let fixtures = [
        (
            "response-explain-projection.json",
            vec!["generator_source", "boundary_source", "target", "binding"],
        ),
        ("response-explain-validation.json", vec!["target"]),
        ("response-explain-evidence-requirement.json", vec!["target"]),
    ];
    for (name, fields) in fixtures {
        let declaration =
            example(name)["value"]["explanation"]["declaration"]["declaration"].clone();
        for field in fields {
            let mut missing = declaration.clone();
            missing.as_object_mut().unwrap().remove(field);
            match name {
                "response-explain-projection.json" => rejects::<ProjectionDeclaration>(missing),
                "response-explain-validation.json" => rejects::<ValidationDeclaration>(missing),
                _ => rejects::<EvidenceRequirementDeclaration>(missing),
            }
        }
    }
    for nested in ["instances", "evidence_classes", "context"] {
        let mut declaration = example("response-explain-evidence-requirement.json")
            ["value"]["explanation"]["declaration"]["declaration"].clone();
        declaration[nested]
            .as_object_mut()
            .unwrap()
            .remove("source");
        rejects::<EvidenceRequirementDeclaration>(declaration);
    }
}

#[test]
fn mandatory_components_and_explicit_optional_absence_are_distinct() {
    let base = example("response-inspect-all.json")["value"]["components"].clone();
    for name in ["bootstrap", "model", "authority", "bindings"] {
        let mut changed = base.clone();
        changed[name] = json!({"presence":"absent","data":null});
        rejects::<Components>(changed);
    }
    rejects::<Presence<Value>>(json!({"presence":"absent"}));
    rejects::<Presence<Value>>(json!({"presence":"absent","data":{}}));
    rejects::<Presence<Value>>(json!({"presence":"absent","data":[]}));
    rejects::<Presence<Value>>(json!({"presence":"absent","data":null,"extra":true}));
    let mut missing = base;
    missing
        .as_object_mut()
        .unwrap()
        .remove("repository_integrity");
    rejects::<Components>(missing);
}

#[test]
fn inspection_models_reject_mismatched_or_surplus_payloads() {
    let mut value = example("response-inspect-model.json")["value"].clone();
    value["selector"]["kind"] = json!("authority");
    rejects::<Inspection>(value);
    let mut value = example("response-inspect-capability.json")["value"].clone();
    value["selector"]["id"] = json!("other");
    rejects::<Inspection>(value);
    let mut value = example("response-inspect-model.json")["value"].clone();
    value["components"] = Value::Null;
    rejects::<Inspection>(value);
}

#[test]
fn explanation_models_reject_kind_and_identifier_disagreement() {
    for name in [
        "response-explain-capability.json",
        "response-explain-projection.json",
    ] {
        let base = example(name)["value"].clone();
        let mut changed = base.clone();
        changed["query"]["id"] = json!("other");
        rejects::<ExplanationResult>(changed);
        let mut changed = base;
        changed["explanation"]["subject"]["id"] = json!("other");
        rejects::<ExplanationResult>(changed);
    }
    let mut changed = example("response-explain-role.json")["value"].clone();
    changed["query"]["source"] = json!("other");
    rejects::<ExplanationResult>(changed);
}

#[test]
fn request_syntax_is_versioned_closed_and_does_not_grant_effects() {
    let base = example("request-check.json");
    let mut missing = base.clone();
    missing["input"]
        .as_object_mut()
        .unwrap()
        .remove("expected_observation");
    rejects::<Request>(missing);
    let mut extra = base;
    extra["input"]["authorized"] = json!(true);
    rejects::<Request>(extra);
    for version in [json!(0), json!(2), json!(1.0), json!("1"), json!(true)] {
        let mut changed = example("request-discover.json");
        changed["api_version"] = version;
        rejects::<Request>(changed);
    }
    for limit in [json!(0), json!(65537), json!(1.5), json!("1")] {
        let mut changed = example("request-output.json");
        changed["input"]["max_bytes"] = limit;
        rejects::<Request>(changed);
    }
}

#[test]
fn response_union_never_accepts_value_and_error_together() {
    let mut response = example("response-load-failure.json");
    response["value"] = Value::Null;
    rejects::<Response>(response);
    let mut response = example("response-inspect-model.json");
    response["error"] = Value::Null;
    rejects::<Response>(response);
}

#[test]
fn error_models_require_each_nullable_property_and_operation_field() {
    let base = example("response-load-failure.json");
    for field in ["stage", "detail", "attempt_id", "effects_possible"] {
        let mut missing = base["error"].clone();
        missing.as_object_mut().unwrap().remove(field);
        rejects::<ApiError>(missing);
    }
    let mut missing = base;
    missing.as_object_mut().unwrap().remove("operation");
    rejects::<FailureResponse>(missing);
}
