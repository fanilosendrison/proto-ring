#[path = "transport/bounds.rs"]
mod bounds;
#[path = "transport/discovery.rs"]
mod discovery;
#[path = "transport/dispatch.rs"]
mod dispatch;
#[path = "support/git-governance.rs"]
mod git_governance;
#[path = "support/inspection-fixture.rs"]
mod inspection_fixture;
#[path = "transport/request-adversaries.rs"]
mod request_adversaries;
#[path = "transport/strict-decoding.rs"]
mod strict_decoding;

use serde_json::{Value, json};
const API: &str = include_str!("../../../docs/api/v1/schemas/api.schema.json");
fn schema_valid(value: &Value) {
    static VALIDATOR: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
    let validator = VALIDATOR.get_or_init(|| {
        let api: Value = serde_json::from_str(API).unwrap();
        jsonschema::validator_for(
            &json!({"$schema":api["$schema"],"$defs":api["$defs"],"$ref":"#/$defs/response"}),
        )
        .unwrap()
    });
    if let Some(error) = validator.iter_errors(value).next() {
        panic!("{error}");
    }
}
fn response(bytes: &[u8]) -> Value {
    let response = proto_ring_api::process_json(bytes);
    let bytes = proto_ring_api::encode_response(&response).unwrap();
    let value = serde_json::from_slice(&bytes).unwrap();
    schema_valid(&value);
    value
}
fn request(operation: &str, input: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({"api_version":1,"operation":operation,"input":input})).unwrap()
}
