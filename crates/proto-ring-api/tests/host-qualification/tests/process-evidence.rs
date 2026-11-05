//! Independent literal observations; no Docker and no producer-generated PASS fixture.
use api03_host_qualification::{evidence::Check, verification};
use serde_json::json;
use std::fs;
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("p-inspect.json"),br#"[{"Id":"test-container","State":{"Running":false,"Pid":0,"ExitCode":23,"Error":"","StartedAt":"2026-01-01T00:00:00Z"}}]"#).unwrap();
    for part in ["start", "wait", "logs"] {
        fs::write(
            root.join(format!("p-{part}-transport.json")),
            br#"{"code":0,"guard_triggered":false}"#,
        )
        .unwrap();
    }
    fs::write(root.join("p-wait.stdout"), b"23\n").unwrap();
    fs::write(root.join("p-logs.stdout"), b"\x00\xff").unwrap();
    fs::write(root.join("p-logs.stderr"), b"\xfe").unwrap();
    dir
}
fn check() -> Check {
    Check::Process {
        prefix: "p".into(),
        code: 23,
        stdout_hex: "00ff".into(),
        stderr_hex: "fe".into(),
    }
}
#[test]
fn non_utf8_streams_and_process_code_are_exact() {
    let d = fixture();
    verification::checks(d.path(), &[check()]).unwrap();
}
#[test]
fn contradictory_output_is_not_pass() {
    let d = fixture();
    fs::write(d.path().join("p-logs.stdout"), b"All tests passed").unwrap();
    assert!(verification::checks(d.path(), &[check()]).is_err());
}
#[test]
fn missing_output_is_not_empty_success() {
    let d = fixture();
    fs::remove_file(d.path().join("p-logs.stderr")).unwrap();
    assert!(verification::checks(d.path(), &[check()]).is_err());
}
#[test]
fn transport_failure_is_not_process_denial() {
    let d = fixture();
    fs::write(
        d.path().join("p-start-transport.json"),
        br#"{"code":125,"guard_triggered":false}"#,
    )
    .unwrap();
    assert!(verification::checks(d.path(), &[check()]).is_err());
}
#[test]
fn transport_deadline_is_incomplete() {
    let d = fixture();
    fs::write(
        d.path().join("p-wait-transport.json"),
        br#"{"code":0,"guard_triggered":true}"#,
    )
    .unwrap();
    assert!(verification::checks(d.path(), &[check()]).is_err());
}
#[test]
fn program_never_started_is_not_denial() {
    let d = fixture();
    let mut v = verification::json(d.path(), "p-inspect.json").unwrap();
    v[0]["State"]["StartedAt"] = json!("0001-01-01T00:00:00Z");
    fs::write(
        d.path().join("p-inspect.json"),
        serde_json::to_vec(&v).unwrap(),
    )
    .unwrap();
    assert!(verification::checks(d.path(), &[check()]).is_err());
}
#[test]
fn principal_exit_does_not_override_open_perimeter() {
    let d = fixture();
    let mut v = verification::json(d.path(), "p-inspect.json").unwrap();
    v[0]["State"]["Pid"] = json!(123);
    fs::write(
        d.path().join("p-inspect.json"),
        serde_json::to_vec(&v).unwrap(),
    )
    .unwrap();
    assert!(verification::checks(d.path(), &[check()]).is_err());
}
#[test]
fn oversized_stream_refused() {
    let d = fixture();
    fs::write(d.path().join("p-logs.stdout"), vec![0; 65537]).unwrap();
    assert!(verification::checks(d.path(), &[check()]).is_err());
}
#[test]
fn namespace_identity_not_recyclable_pid_alone() {
    let d = fixture();
    let c = Check::Closed {
        inspect: "p-inspect.json".into(),
        container_id: "another-container".into(),
    };
    assert!(verification::checks(d.path(), &[c]).is_err());
}
#[test]
fn empty_checks_cannot_qualify() {
    let d = fixture();
    assert!(verification::checks(d.path(), &[]).is_err());
}
