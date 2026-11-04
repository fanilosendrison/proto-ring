use super::strings;
use proto_ring_api::run_cli;
use std::io::{self, Read, Write};

struct Broken;
impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("injected input failure"))
    }
}
impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected output failure"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected flush failure"))
    }
}
#[test]
fn io_failures_are_internal_not_fabricated_successful_structured_responses() {
    let mut output = Vec::new();
    let mut diagnostics = Vec::new();
    assert_eq!(
        run_cli(
            &strings(&["json"]),
            &mut Broken,
            &mut output,
            &mut diagnostics
        ),
        70
    );
    assert!(output.is_empty());
    assert!(!diagnostics.is_empty());
    let mut diagnostics = Vec::new();
    assert_eq!(
        run_cli(
            &strings(&["discover", "--json"]),
            &mut &b""[..],
            &mut Broken,
            &mut diagnostics
        ),
        70
    );
    assert!(!diagnostics.is_empty());
    assert_eq!(
        run_cli(
            &strings(&["discover", "--json"]),
            &mut &b""[..],
            &mut Broken,
            &mut Broken
        ),
        70
    );
}
struct PartialOutput(Vec<u8>);
impl Write for PartialOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0.len() == 5 {
            return Err(io::Error::other("injected mid-frame failure"));
        }
        let count = bytes.len().min(5 - self.0.len());
        self.0.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn actual_partial_stdout_failure_is_reported_as_70_not_normal_structured_processing() {
    let mut output = PartialOutput(Vec::new());
    let mut diagnostics = Vec::new();
    assert_eq!(
        run_cli(
            &strings(&["discover", "--json"]),
            &mut &b""[..],
            &mut output,
            &mut diagnostics
        ),
        70
    );
    assert_eq!(output.0.len(), 5);
    assert!(!diagnostics.is_empty());
}

struct FlushFailure(Vec<u8>);
impl Write for FlushFailure {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected flush failure"))
    }
}
#[test]
fn flush_failure_is_not_reported_as_success_even_after_frame_writes() {
    let mut output = FlushFailure(Vec::new());
    let mut diagnostics = Vec::new();
    assert_eq!(
        run_cli(
            &strings(&["discover", "--json"]),
            &mut &b""[..],
            &mut output,
            &mut diagnostics
        ),
        70
    );
    assert_eq!(output.0.last(), Some(&b'\n'));
    assert!(!diagnostics.is_empty());
}

struct NeverRead;
impl Read for NeverRead {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("convenience command must not read stdin")
    }
}
#[test]
fn convenience_discovery_does_not_read_stdin_and_outputs_exactly_one_json_with_lf() {
    let mut output = Vec::new();
    let mut diagnostics = Vec::new();
    assert_eq!(
        run_cli(
            &strings(&["discover", "--json"]),
            &mut NeverRead,
            &mut output,
            &mut diagnostics
        ),
        0
    );
    assert!(diagnostics.is_empty());
    assert_eq!(output.last(), Some(&b'\n'));
    assert!(serde_json::from_slice::<serde_json::Value>(&output).is_ok());
}
