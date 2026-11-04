use super::{
    REQUEST_LIMIT,
    cli_arguments::{self, Invocation},
    decode_request, encode_response, process_json,
};
use crate::{FailureOutcome, Response};
use std::{
    ffi::{OsStr, OsString},
    io::{Read, Write},
};

/// Run one invocation without the executable name. All protocol bytes are
/// prepared before touching stdout. I/O failures return 70; stderr is best-effort
/// on an already failed processing path, never substituted for a JSON success.
pub fn run_cli(
    args: &[OsString],
    input: &mut dyn Read,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> i32 {
    let response = match cli_arguments::adapt(args) {
        Ok(Invocation::Wire(bytes)) => process_json(&bytes),
        Ok(Invocation::Stdin) => match read_request(input) {
            Ok(bytes) => process_json(&bytes),
            Err(()) => return io_failure(diagnostics, "cannot read request stdin"),
        },
        Err(mut error) => {
            if args.first().is_some_and(|arg| arg == OsStr::new("json")) {
                let bytes = match read_request(input) {
                    Ok(bytes) => bytes,
                    Err(()) => return io_failure(diagnostics, "cannot read request stdin"),
                };
                error.operation = match decode_request(&bytes) {
                    Ok(request) => Some(request.operation()),
                    Err(failure) => failure.operation,
                };
            }
            Response::Failure(error)
        }
    };
    let status = if matches!(response,Response::Failure(ref error) if error.outcome==FailureOutcome::ProtocolError)
    {
        2
    } else {
        0
    };
    let bytes = match encode_response(&response) {
        Ok(bytes) => bytes,
        Err(_) => return io_failure(diagnostics, "cannot encode a canonical bounded response"),
    };
    if output
        .write_all(&bytes)
        .and_then(|()| output.write_all(b"\n"))
        .and_then(|()| output.flush())
        .is_err()
    {
        return io_failure(diagnostics, "cannot write response stdout");
    }
    status
}
fn read_request(input: &mut dyn Read) -> Result<Vec<u8>, ()> {
    let mut bytes = Vec::new();
    // Read one byte beyond the budget to distinguish exact-boundary EOF from
    // overflow. Oversize input is rejected, never dispatched as a prefix.
    input
        .take((REQUEST_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    Ok(bytes)
}
fn io_failure(diagnostics: &mut dyn Write, message: &str) -> i32 {
    let _ = writeln!(diagnostics, "proto-ring: {message}");
    70
}
