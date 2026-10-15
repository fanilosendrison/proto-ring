#![forbid(unsafe_code)]

mod operations;
mod protocol;

use std::io::{self, Read, Write};
use std::process::ExitCode;

use protocol::Response;

const EXIT_INVALID_REQUEST: u8 = 2;
const EXIT_INTERNAL_FAILURE: u8 = 70;

fn main() -> ExitCode {
    if std::env::args_os().len() != 1 {
        let response = Response::transport_error();
        return if write_response(&response).is_ok() {
            ExitCode::from(EXIT_INVALID_REQUEST)
        } else {
            ExitCode::from(EXIT_INTERNAL_FAILURE)
        };
    }
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() {
        return ExitCode::from(EXIT_INTERNAL_FAILURE);
    }
    let (response, exit_code) = match protocol::parse_request(&input) {
        Ok(request) => match operations::execute(request) {
            Ok(response) => (response, 0),
            Err(_) => return ExitCode::from(EXIT_INTERNAL_FAILURE),
        },
        Err(_) => (Response::transport_error(), EXIT_INVALID_REQUEST),
    };
    if write_response(&response).is_err() {
        return ExitCode::from(EXIT_INTERNAL_FAILURE);
    }
    ExitCode::from(exit_code)
}

fn write_response(response: &Response) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, response).map_err(io::Error::other)?;
    output.write_all(b"\n")?;
    output.flush()
}
