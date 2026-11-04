#[path = "support/wire-oracle.rs"]
mod wire_oracle;

use proto_ring_api::{
    Components, ExplainQuery, InspectSelector, ObservationReference, ProfileReference, Request,
    Response, StructuredValue, SubjectProjection,
};
use serde_json::json;
use wire_oracle::{example, round_trip};

#[test]
fn all_thirteen_inspect_selectors_have_exact_closed_wire_shapes() {
    for kind in [
        "all",
        "bootstrap",
        "model",
        "authority",
        "governed_objects",
        "bindings",
        "repository_integrity",
        "projections",
        "evidence_requirements",
    ] {
        round_trip::<InspectSelector>(&json!({"kind":kind}), "inspect_selector");
    }
    for kind in ["capability", "validation", "binding"] {
        round_trip::<InspectSelector>(&json!({"kind":kind,"id":"declared"}), "inspect_selector");
    }
    round_trip::<InspectSelector>(
        &json!({"kind":"governed_object","target":{"interface":"i","object":"o"}}),
        "inspect_selector",
    );
}

#[test]
fn all_eight_explanation_queries_are_typed() {
    for name in [
        "response-explain-role.json",
        "response-explain-capability.json",
        "response-explain-validation.json",
        "response-explain-binding.json",
        "response-explain-governed-object.json",
        "response-explain-projection.json",
        "response-explain-evidence-requirement.json",
    ] {
        round_trip::<ExplainQuery>(&example(name)["value"]["query"], "explain_query");
    }
    round_trip::<ExplainQuery>(
        &json!({"kind":"authority_precedence","responsibility":"r","higher_source":"a","lower_source":"b"}),
        "explain_query",
    );
}

#[test]
fn structured_values_preserve_empty_keys_and_arbitrary_precision() {
    let value = json!({"kind":"mapping","entries":[
        {"key":"","value":{"kind":"integer","decimal":"-1234567890123456789012345678901234567890"}},
        {"key":"nested","value":{"kind":"sequence","items":[
            {"kind":"null"},{"kind":"boolean","value":false},{"kind":"string","value":""}
        ]}}
    ]});
    round_trip::<StructuredValue>(&value, "structured_value");
}

#[test]
fn complete_components_preserve_all_declarations_and_optional_absence() {
    for name in [
        "response-inspect-all.json",
        "response-inspect-all-absent-ri.json",
    ] {
        round_trip::<Components>(&example(name)["value"]["components"], "components");
    }
}

#[test]
fn all_six_subject_views_share_complete_declaration_models() {
    for name in [
        "response-explain-capability.json",
        "response-explain-validation.json",
        "response-explain-binding.json",
        "response-explain-governed-object.json",
        "response-explain-projection.json",
        "response-explain-evidence-requirement.json",
    ] {
        round_trip::<SubjectProjection>(
            &example(name)["value"]["explanation"]["declaration"],
            "subject_projection",
        );
    }
}

#[test]
fn public_references_have_versioned_formats_and_no_raw_identity_field() {
    let value = example("response-inspect-validation.json");
    round_trip::<ObservationReference>(&value["value"]["observation"], "observation");
    round_trip::<ProfileReference>(&value["value"]["subject"]["profile_ref"], "profile_ref");
}

#[test]
fn all_five_request_shapes_are_representable_without_execution_services() {
    for name in [
        "request-discover.json",
        "request-inspect-absent-ri.json",
        "request-explain-role.json",
        "request-check.json",
        "request-output.json",
    ] {
        round_trip::<Request>(&example(name), "request");
    }
}

#[test]
fn nonexecuting_response_models_match_canonical_envelopes() {
    for name in [
        "response-discover.json",
        "response-inspect-all.json",
        "response-inspect-model.json",
        "response-inspect-capability.json",
        "response-inspect-validation.json",
        "response-inspect-binding.json",
        "response-inspect-governed-object.json",
        "response-explain-role.json",
        "response-explain-capability.json",
        "response-explain-validation.json",
        "response-explain-binding.json",
        "response-explain-governed-object.json",
        "response-explain-projection.json",
        "response-explain-evidence-requirement.json",
    ] {
        round_trip::<Response>(&example(name), "response");
    }
}
