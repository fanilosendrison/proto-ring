//! Common API-02 request and response boundary. API-03 services are absent.
#[path = "transport/bounded-buffer.rs"]
mod bounded_buffer;
#[path = "transport/cli.rs"]
mod cli;
#[path = "transport/cli-arguments.rs"]
mod cli_arguments;
#[path = "transport/contract-schema.rs"]
mod contract_schema;
#[path = "transport/discovery-catalog.rs"]
mod discovery_catalog;
#[path = "transport/dispatch.rs"]
mod dispatch;
#[path = "transport/failure.rs"]
mod failure;
#[path = "transport/request-decoder.rs"]
mod request_decoder;
#[path = "transport/response-encoder.rs"]
mod response_encoder;
#[path = "transport/strict-json.rs"]
mod strict_json;

pub use cli::run_cli;
pub use discovery_catalog::discover;
pub use dispatch::dispatch;
pub use request_decoder::decode_request;
pub use response_encoder::encode_response;

/// Maximum UTF-8 request bytes, including any JSON whitespace.
pub const REQUEST_LIMIT: usize = 1_048_576;
/// Maximum encoded JSON response bytes; the framing LF is separate.
pub const RESPONSE_LIMIT: usize = 16_777_216;

/// Strict decode followed by the same typed dispatcher used by CLI adapters.
pub fn process_json(bytes: &[u8]) -> crate::Response {
    match decode_request(bytes) {
        Ok(request) => dispatch(request),
        Err(error) => crate::Response::Failure(error),
    }
}
