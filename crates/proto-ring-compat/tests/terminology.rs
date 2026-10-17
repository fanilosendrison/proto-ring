use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use proto_ring_conformance::fixture;
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::{CaseMatrix, ObservationKind, TransportValue};
use serde_json::{Map, Value, json};

#[rustfmt::skip]
fn binary() -> &'static str { env!("CARGO_BIN_EXE_proto-ring-compat") }
#[rustfmt::skip]
fn invoke(request: &Value) -> (Output, Value) {
    let mut child = Command::new(binary()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(request).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap(); let response = serde_json::from_slice(&output.stdout).unwrap(); (output, response)
}
#[rustfmt::skip]
fn request(operation: &str, input: Value) -> Value { json!({"transport_version":1,"operation":operation,"input":input}) }
#[rustfmt::skip]
fn transport_error(value: Value) {
    let (output, response) = invoke(&value); assert_eq!(output.status.code(), Some(2));
    assert_eq!(response, json!({"transport_version":1,"outcome":"transport_error","error":{"category":"invalid_request"}}));
}
#[rustfmt::skip]
fn matrix(responsibility: &str) -> CaseMatrix {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    load_corpus(&root).unwrap().matrices.into_iter().find(|item| item.responsibility_id == responsibility).unwrap()
}
#[rustfmt::skip]
fn plain(value: &TransportValue) -> Value { match value {
    TransportValue::Null(()) => Value::Null, TransportValue::Boolean(value) => Value::Bool(*value),
    TransportValue::Integer(value) => serde_json::from_str(value).unwrap(),
    TransportValue::String(value) | TransportValue::Bytes(value) => Value::String(value.clone()),
    TransportValue::Sequence(values) => Value::Array(values.iter().map(plain).collect()),
    TransportValue::Record(entries) => Value::Object(entries.iter().map(|entry| (entry.name.clone(), plain(&entry.value))).collect()),
} }
#[rustfmt::skip]
fn input(value: &TransportValue) -> Map<String, Value> { plain(value).as_object().unwrap().clone() }

#[rustfmt::skip]
fn translated_policy(value: &Value) -> Value {
    let mut policy = value.as_object().unwrap().clone();
    let mut location = policy["canonical_location"].as_object().unwrap().clone();
    if let Some(prefix) = location.remove("section_prefix") { location.insert("prefix".into(), prefix); }
    policy.insert("canonical_location".into(), Value::Object(location));
    let mut roles = policy["role_policy"].as_object().unwrap().clone();
    let prefixes = roles.remove("noncanonical_section_prefixes").unwrap();
    let rules = prefixes.as_object().unwrap().iter()
        .map(|(value, role)| json!({"matcher":{"kind":"section_prefix","value":value},"role":role})).collect();
    roles.insert("noncanonical_rules".into(), Value::Array(rules));
    policy.insert("role_policy".into(), Value::Object(roles)); Value::Object(policy)
}
#[rustfmt::skip]
fn format_value() -> Value { json!({"start_marker":"<!-- terms:start -->","end_marker":"<!-- terms:end -->","headers":["Key","Term","Anchor","Aliases","Deprecated","Structure"],"key_pattern":"^[a-z][a-z0-9-]*$","anchor_link_pattern":"\\[`[^`]+`\\]\\(#(term-[a-z0-9-]+)\\)","anchor_pattern":"<a id=\"(term-[a-z0-9-]+)\"></a>"}) }
#[rustfmt::skip]
fn policy_value(section_kind: &str) -> Value { json!({"allowed_roles":["canonical","other"],"section_from_heading":{"kind":section_kind},"canonical_location":{"kind":"anchor_matches_key"},"role_policy":{"canonical":"canonical","noncanonical_rules":[],"other_noncanonical":"other"}}) }

#[test]
fn terminology_transport_is_strict_and_semantic_failures_are_controlled() {
    let valid_occurrence = json!({"concepts":[],"section":"","heading":"","fingerprint":"","line":0,"canonical":false});
    for operation in [
        "normative-terminology.definition-discovery",
        "normative-terminology.occurrence-signatures",
        "normative-terminology.unknown",
    ] {
        transport_error(request(operation, json!({})));
    }
    let invalid = [
        request("normative-terminology.fingerprint", json!({"mode":"unknown","text":"x"})),
        request("normative-terminology.fingerprint", json!({"mode":"normalize_text","text":"x","extra":true})),
        request("normative-terminology.registry", json!({"markdown":"","format":{"start_marker":"x"}})),
        request("normative-terminology.registry", json!({"markdown":"","format":{"start_marker":"a","end_marker":"b","headers":"x","key_pattern":"x","anchor_link_pattern":"x","anchor_pattern":"x"}})),
        request("normative-terminology.markdown", json!({"mode":"unknown","markdown":"","format":format_value(),"policy":policy_value("identity")})),
        request("normative-terminology.markdown", json!({"mode":"parse_blocks","markdown":"","format":format_value(),"policy":{"allowed_roles":[],"section_from_heading":{"kind":"unknown"},"canonical_location":{"kind":"anchor_matches_key"},"role_policy":{"canonical":"x","noncanonical_rules":[],"other_noncanonical":"x"}}})),
        request("normative-terminology.inventory", json!({"mode":"unknown","occurrences":[],"policy":policy_value("identity")})),
        request("normative-terminology.inventory", json!({"mode":"occurrence_records","occurrences":[{"concepts":[],"section":"","heading":"","fingerprint":"","line":-1,"canonical":false}],"policy":policy_value("identity")})),
        request("normative-terminology.inventory", json!({"mode":"occurrence_records","occurrences":[{"concepts":[],"section":"","heading":"","fingerprint":"","line":"0","canonical":false}],"policy":policy_value("identity")})),
        request("normative-terminology.inventory", json!({"mode":"occurrence_records","occurrences":{},"policy":policy_value("identity")})),
        request("normative-terminology.inventory", json!({"mode":"occurrence_records","occurrences":[valid_occurrence.clone()],"policy":{"allowed_roles":["x"],"section_from_heading":{"kind":"identity"},"canonical_location":{"kind":"anchor_matches_key"},"role_policy":{"canonical":"x","noncanonical_rules":[{"matcher":{"kind":"unknown","value":"0"},"role":"x"}],"other_noncanonical":"x"}}})),
        serde_json::from_str(r#"{"transport_version":1,"operation":"normative-terminology.inventory","input":{"mode":"occurrence_records","occurrences":[{"concepts":[],"section":"","heading":"","fingerprint":"","line":18446744073709551616,"canonical":false}],"policy":{"allowed_roles":["x"],"section_from_heading":{"kind":"identity"},"canonical_location":{"kind":"anchor_matches_key"},"role_policy":{"canonical":"x","noncanonical_rules":[],"other_noncanonical":"x"}}}}"#).unwrap(),
    ];
    for value in invalid {
        transport_error(value);
    }
    let bad_role = request(
        "normative-terminology.inventory",
        json!({"mode":"occurrence_records","occurrences":[valid_occurrence],"policy":{"allowed_roles":["allowed"],"section_from_heading":{"kind":"identity"},"canonical_location":{"kind":"anchor_matches_key"},"role_policy":{"canonical":"allowed","noncanonical_rules":[],"other_noncanonical":"not-allowed"}}}),
    );
    let (output, response) = invoke(&bad_role);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(response["outcome"], "controlled_failure");
    assert_eq!(
        response["error"]["category"],
        "normative-terminology.inventory.rejected"
    );
    let mut bad_format = format_value();
    bad_format["headers"] = json!(["one"]);
    let (_, response) = invoke(&request(
        "normative-terminology.registry",
        json!({"markdown":"","format":bad_format}),
    ));
    assert_eq!(
        response["error"]["category"],
        "normative-terminology.registry.rejected"
    );
    let (_, response) = invoke(&request(
        "normative-terminology.registry",
        json!({"markdown":"malformed","format":format_value()}),
    ));
    assert_eq!(response["outcome"], "result");
    assert_eq!(
        response["value"]["diagnostics"],
        json!(["registry_marker_cardinality"])
    );
}

#[test]
fn turnlock_policy_edge_cases_and_hierarchy_are_exact() {
    let headings = [
        "2 Core",
        "2.2 Work",
        "2.2A Work",
        "0.10I Work",
        "3.1A X",
        "Heading",
        "2.2Afoo",
        "2.2A_foo",
        "2é",
    ];
    let markdown = headings
        .iter()
        .map(|heading| format!("# {heading}\n\ntext\n"))
        .collect::<Vec<_>>()
        .join("\n");
    let policy = policy_value("numbered_dotted_prefix_optional_uppercase_word_boundary");
    let (_, response) = invoke(&request(
        "normative-terminology.markdown",
        json!({"mode":"parse_blocks","markdown":markdown,"format":format_value(),"policy":policy}),
    ));
    let sections: Vec<_> = response["value"]["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|block| block["section"].as_str().unwrap())
        .collect();
    assert_eq!(
        sections,
        ["2", "2.2", "2.2A", "0.10I", "3.1A", "", "", "", ""]
    );

    let occurrences = [
        ("0", "intent", false), ("0.3A", "intent", false), ("3.1A", "obligation", false),
        ("30", "reference", false), ("5.2", "implication", false), ("8.1", "synopsis", false),
        ("20", "canonical-definition", true),
    ].into_iter().map(|(section, _, canonical)| json!({"concepts":[],"section":section,"heading":"","fingerprint":"","line":0,"canonical":canonical})).collect::<Vec<_>>();
    let policy = json!({"allowed_roles":["canonical-definition","intent","obligation","implication","reference","synopsis"],
        "section_from_heading":{"kind":"numbered_dotted_prefix_optional_uppercase_word_boundary"},
        "canonical_location":{"kind":"section_hierarchy","root":"2"},
        "role_policy":{"canonical":"canonical-definition","noncanonical_rules":[
            {"matcher":{"kind":"section_hierarchy","value":"0"},"role":"intent"},
            {"matcher":{"kind":"section_hierarchy","value":"3"},"role":"obligation"},
            {"matcher":{"kind":"section_hierarchy","value":"5"},"role":"implication"},
            {"matcher":{"kind":"section_hierarchy","value":"8"},"role":"synopsis"}],"other_noncanonical":"reference"}});
    let (_, response) = invoke(&request(
        "normative-terminology.inventory",
        json!({"mode":"occurrence_records","occurrences":occurrences,"policy":policy}),
    ));
    let roles: Vec<_> = response["value"]["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["role"].as_str().unwrap())
        .collect();
    assert_eq!(
        roles,
        [
            "intent",
            "intent",
            "obligation",
            "reference",
            "implication",
            "synopsis",
            "canonical-definition"
        ]
    );
}

#[test]
fn terminology_process_surface_matches_45_corpus_vectors() {
    let mut total = 0;
    for vector in matrix("normative-terminology.fingerprint").vectors {
        let materialized = fixture::materialize(&vector.fixture).unwrap();
        let mut args = input(materialized.arguments());
        let mode = args
            .remove("operation")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        if !matches!(mode.as_str(), "normalize_text" | "fingerprint_text") {
            continue;
        }
        args.insert("mode".into(), json!(mode));
        let (_, response) = invoke(&request(
            "normative-terminology.fingerprint",
            Value::Object(args),
        ));
        assert_eq!(
            response["value"],
            plain(&vector.expected_observation.value),
            "{}",
            vector.vector_id
        );
        total += 1;
    }
    for vector in matrix("normative-terminology.registry").vectors {
        let materialized = fixture::materialize(&vector.fixture).unwrap();
        let args = input(materialized.arguments());
        let operation = args["operation"].as_str().unwrap();
        if operation == "parse_registry" {
            let (_, response) = invoke(&request(
                "normative-terminology.registry",
                json!({"markdown":args["markdown"],"format":args["format"]}),
            ));
            assert_eq!(response["outcome"], "result", "{}", vector.vector_id);
            if vector.expected_observation.kind == ObservationKind::Result {
                let expected = plain(&vector.expected_observation.value);
                assert_eq!(
                    response["value"]["entries"], expected["entries"],
                    "{}",
                    vector.vector_id
                );
                assert_eq!(response["value"]["diagnostics"], json!([]));
            } else {
                assert!(
                    !response["value"]["diagnostics"]
                        .as_array()
                        .unwrap()
                        .is_empty(),
                    "{}",
                    vector.vector_id
                );
            }
        } else {
            let mut variants = Vec::new();
            let documents = args
                .get("documents")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_else(|| vec![args["markdown"].clone()]);
            for markdown in &documents {
                let (_, response) = invoke(&request(
                    "normative-terminology.registry",
                    json!({"markdown":markdown,"format":args["format"]}),
                ));
                let mut categories: Vec<_> = response["value"]["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_owned())
                    .collect();
                categories.sort();
                categories.dedup();
                variants.push(json!({"categories":categories,"entries":response["value"]["entries"].as_array().unwrap().len()}));
            }
            assert_eq!(
                json!({"variants":variants}),
                plain(&vector.expected_observation.value),
                "{}",
                vector.vector_id
            );
        }
        total += 1;
    }
    for vector in matrix("normative-terminology.markdown").vectors {
        let materialized = fixture::materialize(&vector.fixture).unwrap();
        let args = input(materialized.arguments());
        if args["operation"] != "parse_blocks" {
            continue;
        }
        let (_, response) = invoke(&request(
            "normative-terminology.markdown",
            json!({"mode":"parse_blocks","markdown":args["markdown"],"format":args["format"],"policy":translated_policy(&args["policy"])}),
        ));
        let anchors: Map<_, _> = response["value"]["anchors"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, values)| {
                (
                    key.clone(),
                    Value::Array(
                        values
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|item| json!([item["line"], item["section"], item["heading"]]))
                            .collect(),
                    ),
                )
            })
            .collect();
        assert_eq!(
            json!({"anchors":anchors,"blocks":response["value"]["blocks"]}),
            plain(&vector.expected_observation.value),
            "{}",
            vector.vector_id
        );
        total += 1;
    }
    for responsibility in [
        "normative-terminology.fingerprint",
        "normative-terminology.markdown",
    ] {
        for vector in matrix(responsibility).vectors {
            let materialized = fixture::materialize(&vector.fixture).unwrap();
            let args = input(materialized.arguments());
            let operation = args["operation"].as_str().unwrap();
            if !operation.starts_with("discover_") {
                continue;
            }
            let (_, response) = invoke(&request(
                "normative-terminology.markdown",
                json!({"mode":"discover_occurrences","markdown":args["markdown"],"format":args["format"],"policy":translated_policy(&args["policy"])}),
            ));
            let occurrences = response["value"]["occurrences"].as_array().unwrap();
            let diagnostics = response["value"]["diagnostics"].clone();
            let projected = match operation {
                "discover_error_categories" => {
                    json!({"category":args["category_filter"],"present":diagnostics.as_array().unwrap().contains(&args["category_filter"])})
                }
                "discover_location_summary" => {
                    let canonical: Vec<_> = occurrences
                        .iter()
                        .filter(|item| item["canonical"] == true)
                        .collect();
                    json!({"canonical_count":canonical.len(),"canonical_sections":canonical.iter().map(|item| item["section"].clone()).collect::<Vec<_>>(),"errors":diagnostics})
                }
                _ => {
                    let query = &args["query"];
                    let matched: Vec<_> = occurrences
                        .iter()
                        .filter(|item| {
                            item["canonical"] == query["canonical"]
                                && item["concepts"] == query["concepts"]
                                && item["section"] == query["section"]
                        })
                        .collect();
                    let occurrence = matched
                        .first()
                        .map(|item| {
                            let mut value = (*item).clone();
                            value.as_object_mut().unwrap().remove("line");
                            value
                        })
                        .unwrap_or(Value::Null);
                    json!({"count":matched.len(),"errors":diagnostics,"occurrence":occurrence})
                }
            };
            assert_eq!(
                projected,
                plain(&vector.expected_observation.value),
                "{}",
                vector.vector_id
            );
            total += 1;
        }
    }
    for vector in matrix("normative-terminology.inventory").vectors {
        let materialized = fixture::materialize(&vector.fixture).unwrap();
        let args = input(materialized.arguments());
        let (_, response) = invoke(&request(
            "normative-terminology.inventory",
            json!({"mode":"reconcile_inventory","entries":args["entries"],"occurrences":args["occurrences"],"inventory":args["inventory"],"policy":translated_policy(&args["policy"])}),
        ));
        let expected = plain(&vector.expected_observation.value);
        if vector.expected_observation.kind == ObservationKind::Result {
            assert_eq!(
                json!({"status":response["value"]["status"]}),
                expected,
                "{}",
                vector.vector_id
            );
        } else {
            assert_eq!(
                response["outcome"], "controlled_failure",
                "{}",
                vector.vector_id
            );
            assert_eq!(
                response["error"]["category"], expected["category"],
                "{}",
                vector.vector_id
            );
        }
        total += 1;
    }
    assert_eq!(total, 45);
}
