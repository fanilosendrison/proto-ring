use std::path::Path;

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub fn example(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/api/v1/examples")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).expect("accepted fixture")).expect("fixture JSON")
}

pub fn round_trip<T: DeserializeOwned + Serialize>(value: &Value, definition: &str) {
    let model: T = serde_json::from_value(value.clone()).expect("typed model must decode fixture");
    let encoded = serde_json::to_value(model).expect("typed model must encode");
    assert_eq!(&encoded, value, "model must preserve every field and null");
    let canonical: Value = serde_json::from_str(include_str!(
        "../../../../docs/api/v1/schemas/api.schema.json"
    ))
    .expect("accepted schema");
    let schema = serde_json::json!({
        "$schema": canonical["$schema"],
        "$defs": canonical["$defs"],
        "$ref": format!("#/$defs/{definition}"),
    });
    let validator = jsonschema::validator_for(&schema).expect("canonical definition compiles");
    if let Some(error) = validator.iter_errors(&encoded).next() {
        panic!("model violates canonical {definition}: {error}");
    }
}
