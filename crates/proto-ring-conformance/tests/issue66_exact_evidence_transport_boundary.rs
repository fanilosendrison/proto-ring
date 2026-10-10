#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::{ObservationKind, TransportValue};
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use serde_json::{Value, json};

const RESPONSIBILITY: &str = "exact-evidence-binding.evaluate";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root exists")
        .to_path_buf()
}

fn published_request(vector_id: &str) -> CandidateRequest {
    let corpus = load_corpus(&root()).expect("published corpus loads");
    let matrix = corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == RESPONSIBILITY)
        .expect("Exact Evidence Binding matrix exists");
    let vector = matrix
        .vectors
        .iter()
        .find(|vector| vector.vector_id == vector_id)
        .expect("published vector exists");
    CandidateRequest {
        responsibility_id: matrix.responsibility_id.clone(),
        vector_id: vector.vector_id.clone(),
        fixture: vector.fixture.clone(),
    }
}

fn record_field_mut<'a>(record: &'a mut Value, name: &str) -> &'a mut Value {
    let object = record
        .as_object_mut()
        .expect("transport value is an object");
    assert_eq!(object.len(), 2, "transport value has exact shape");
    assert_eq!(object.get("type"), Some(&Value::String("record".into())));
    let entries = object
        .get_mut("value")
        .and_then(Value::as_array_mut)
        .expect("record value is an array");
    for entry in entries {
        let entry = entry.as_object_mut().expect("record entry is an object");
        assert_eq!(entry.len(), 2, "record entry has exact shape");
        assert!(entry.get("name").is_some_and(Value::is_string));
        assert!(entry.contains_key("value"));
        if entry.get("name").and_then(Value::as_str) == Some(name) {
            return entry.get_mut("value").expect("record entry value exists");
        }
    }
    panic!("record field exists: {name}");
}

fn sequence_values_mut(sequence: &mut Value) -> &mut Vec<Value> {
    let object = sequence
        .as_object_mut()
        .expect("transport value is an object");
    assert_eq!(object.len(), 2, "transport value has exact shape");
    assert_eq!(object.get("type"), Some(&Value::String("sequence".into())));
    object
        .get_mut("value")
        .and_then(Value::as_array_mut)
        .expect("sequence value is an array")
}

fn fixture_input_mut(request: &mut CandidateRequest) -> &mut Value {
    let fixture = request
        .fixture
        .as_object_mut()
        .expect("published fixture is an object");
    assert_eq!(fixture.len(), 2, "published inline fixture has exact shape");
    assert_eq!(fixture.get("kind").and_then(Value::as_str), Some("inline"));
    fixture.get_mut("input").expect("inline input exists")
}

#[test]
fn combined_transport_failures_precede_semantic_rejection() {
    let published_id = "exact-evidence-binding.evaluate.requirement-duplicate-classes";
    let control = published_request(published_id);
    let control_state = ReferenceRustCandidate
        .execute(&control)
        .expect("well-formed duplicate classes execute");
    let CandidateState::Observation(control_observation) = control_state else {
        panic!("published duplicate-class vector must be implemented");
    };
    assert_eq!(
        control_observation.kind,
        ObservationKind::ControlledRejection
    );
    let TransportValue::Record(category_record) = control_observation.value else {
        panic!("controlled rejection value must be a record");
    };
    assert!(category_record.iter().any(|entry| {
        entry.name == "category"
            && entry.value
                == TransportValue::String("exact-evidence-binding.evaluate.rejected".into())
    }));

    let cases = [
        (
            "binding-evidence-class-boolean",
            "binding",
            "evidence_class",
            json!({"type":"boolean","value":true}),
        ),
        (
            "binding-subject-integer",
            "binding",
            "subject_hex",
            json!({"type":"integer","value":"1"}),
        ),
        (
            "binding-subject-invalid-hex",
            "binding",
            "subject_hex",
            json!({"type":"string","value":"zz"}),
        ),
        (
            "binding-context-invalid-hex",
            "binding",
            "context_hex",
            json!({"type":"string","value":"zz"}),
        ),
        (
            "requirement-subject-integer",
            "requirement",
            "subject_hex",
            json!({"type":"integer","value":"1"}),
        ),
        (
            "requirement-subject-invalid-hex",
            "requirement",
            "subject_hex",
            json!({"type":"string","value":"zz"}),
        ),
        (
            "requirement-context-required-string",
            "requirement",
            "context_required",
            json!({"type":"string","value":"false"}),
        ),
        (
            "requirement-context-invalid-hex",
            "requirement",
            "context_hex",
            json!({"type":"string","value":"zz"}),
        ),
    ];
    let case_count = cases.len();
    let mut harness_errors = 0;
    let controlled_rejections = 0;
    for (label, owner, field, replacement) in cases {
        let mut request = published_request(published_id);
        request.vector_id = format!("synthetic.issue66.{label}");
        let owner = record_field_mut(fixture_input_mut(&mut request), owner);
        *record_field_mut(owner, field) = replacement;
        match ReferenceRustCandidate.execute(&request) {
            Err(_) => harness_errors += 1,
            Ok(CandidateState::Observation(observation)) => panic!(
                "{label} returned observation {:?} instead of HarnessError",
                observation.kind
            ),
            Ok(CandidateState::Unimplemented) => {
                panic!("{label} returned Unimplemented instead of HarnessError")
            }
        }
    }
    eprintln!("ISSUE66_EEB_RUST_COMBINED_TRANSPORT_CASES={case_count}");
    eprintln!("ISSUE66_EEB_RUST_COMBINED_TRANSPORT_HARNESS_ERRORS={harness_errors}");
    eprintln!("ISSUE66_EEB_RUST_COMBINED_TRANSPORT_CONTROLLED_REJECTIONS={controlled_rejections}");
    assert_eq!(harness_errors, case_count);
}

#[test]
fn history_parses_full_invocation_before_semantic_rejection() {
    let mut request = published_request("exact-evidence-binding.evaluate.historical-change");
    request.vector_id = "synthetic.issue66.history-full-invocation-parse".into();
    let comparisons = sequence_values_mut(record_field_mut(
        fixture_input_mut(&mut request),
        "comparisons",
    ));
    assert_eq!(
        comparisons.len(),
        2,
        "published history has two comparisons"
    );
    let (first, second) = comparisons.split_at_mut(1);
    let historical = record_field_mut(&mut first[0], "historical_requirement");
    let classes = sequence_values_mut(record_field_mut(historical, "admitted_classes"));
    assert_eq!(classes.len(), 1, "historical requirement has one class");
    classes.push(classes[0].clone());
    let binding = record_field_mut(&mut second[0], "binding");
    *record_field_mut(binding, "subject_hex") = json!({"type":"string","value":"zz"});

    assert!(
        ReferenceRustCandidate.execute(&request).is_err(),
        "later malformed transport must not be masked by earlier semantic rejection"
    );
    eprintln!("ISSUE66_EEB_HISTORY_FULL_INVOCATION_PARSE=PASS");
}
