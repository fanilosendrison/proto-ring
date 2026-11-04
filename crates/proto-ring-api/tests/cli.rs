#[path = "cli/api03-adapters.rs"]
mod api03_adapters;
#[path = "cli/equivalence.rs"]
mod equivalence;
#[path = "support/git-governance.rs"]
mod git_governance;
#[path = "cli/grammar.rs"]
mod grammar;
#[path = "support/inspection-fixture.rs"]
mod inspection_fixture;
#[path = "cli/io-boundaries.rs"]
mod io_boundaries;

use serde_json::Value;
use std::{
    ffi::OsString,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};
fn invoke(args: &[OsString], input: &[u8], cwd: &Path) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_proto-ring"));
    for (key, _) in std::env::vars_os() {
        if key
            .to_str()
            .is_some_and(|key| key.to_ascii_uppercase().starts_with("GIT_"))
        {
            command.env_remove(key);
        }
    }
    let mut child = command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let bytes = input.to_vec();
    let writer = std::thread::spawn(move || {
        if let Err(error) = stdin.write_all(&bytes) {
            assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        }
    });
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    output
}
fn strings(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
fn parsed(output: &Output, exit: i32) -> Value {
    assert_eq!(output.status.code(), Some(exit), "{:?}", output.stderr);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    assert_eq!(
        output.stdout.iter().filter(|byte| **byte == b'\n').count(),
        1
    );
    serde_json::from_slice(&output.stdout[..output.stdout.len() - 1]).unwrap()
}
