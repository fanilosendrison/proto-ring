#![cfg(unix)]
use api03_host_qualification::rendezvous::{self, Binding};
use serde_json::{Value, json};
use std::{fs, os::unix::fs::MetadataExt, path::Path, process::Command};
const ID: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
fn record() -> Value {
    json!({"campaign":"api03-123-1","instance":"api03-123-1-descendant-writer","container_id":ID,"hostname":"cccccccccccc","pid_namespace":"pid:[123]","pid":2,"ppid":1,"write_fd":3,"kind":"writer"})
}
fn binding(root: &Path) -> Binding<'static> {
    Binding {
        campaign: "api03-123-1",
        instance: "api03-123-1-descendant-writer",
        container_id: ID,
        kind: "writer",
        uid: fs::metadata(root).unwrap().uid(),
    }
}
fn publish(root: &Path, value: &Value) -> std::process::Output {
    Command::new(std::env::var_os("PROTO_RING_PYTHON").unwrap_or_else(||"python3".into())).args(["-I","-B","-c","import runpy,sys,json; runpy.run_path(sys.argv[1])['publish'](sys.argv[2], 'ready.json', json.loads(sys.argv[3]))",concat!(env!("CARGO_MANIFEST_DIR"),"/fixtures/rendezvous.py"),root.to_str().unwrap(),&serde_json::to_string(value).unwrap()]).output().unwrap()
}
#[test]
fn absent_final_ignores_present_temporary() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join(".ready-owned"), b"{}").unwrap();
    assert!(
        rendezvous::read(&d.path().join("ready.json"), &binding(d.path()))
            .unwrap()
            .is_none()
    );
}
#[test]
fn partial_temporary_is_never_ready() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join(".ready-partial"), b"{\"campaign\":").unwrap();
    assert!(
        rendezvous::read(&d.path().join("ready.json"), &binding(d.path()))
            .unwrap()
            .is_none()
    );
}
#[test]
fn malformed_final_is_not_retried_or_accepted() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("ready.json"), b"{\"campaign\":").unwrap();
    assert!(rendezvous::read(&d.path().join("ready.json"), &binding(d.path())).is_err());
}
#[test]
fn full_publication_is_accepted() {
    let d = tempfile::tempdir().unwrap();
    let out = publish(d.path(), &record());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let ready = rendezvous::read(&d.path().join("ready.json"), &binding(d.path()))
        .unwrap()
        .unwrap();
    assert_eq!(ready.container_id, ID);
    assert_eq!(ready.write_fd, Some(3));
    assert!(fs::read_dir(d.path()).unwrap().all(|v| {
        !v.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".ready-")
    }));
}
#[test]
fn duplicate_publication_never_replaces_final() {
    let d = tempfile::tempdir().unwrap();
    assert!(publish(d.path(), &record()).status.success());
    let original = fs::read(d.path().join("ready.json")).unwrap();
    let mut second = record();
    second["pid"] = json!(3);
    assert!(!publish(d.path(), &second).status.success());
    assert_eq!(fs::read(d.path().join("ready.json")).unwrap(), original);
}
#[test]
fn foreign_campaign_or_instance_rejected() {
    for key in ["campaign", "instance", "container_id"] {
        let d = tempfile::tempdir().unwrap();
        let mut value = record();
        value[key] = json!(if key == "container_id" {
            "d".repeat(64)
        } else {
            "foreign".into()
        });
        fs::write(
            d.path().join("ready.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        assert!(rendezvous::read(&d.path().join("ready.json"), &binding(d.path())).is_err());
    }
}
#[test]
fn symlink_final_cannot_be_followed() {
    let d = tempfile::tempdir().unwrap();
    fs::write(
        d.path().join("target"),
        serde_json::to_vec(&record()).unwrap(),
    )
    .unwrap();
    std::os::unix::fs::symlink("target", d.path().join("ready.json")).unwrap();
    assert!(rendezvous::read(&d.path().join("ready.json"), &binding(d.path())).is_err());
}
