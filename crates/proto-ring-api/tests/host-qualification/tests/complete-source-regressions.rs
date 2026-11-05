#[path = "support/offline-bundle.rs"]
mod offline;
use api03_host_qualification::{evidence::Evidence, source, source_state::Guard, verification};
use offline::{Bundle, put};
use serde_json::json;
fn reject(b: &Bundle, reason: &str) {
    let e = verification::verify(&b.root, &b.sha, "123", "1", &b.source)
        .unwrap_err()
        .to_string();
    assert!(e.contains(reason), "{e}");
}
#[test]
fn different_commit_inventory_refused() {
    let b = Bundle::new();
    b.edit("source.json", |v| v["sha"] = json!("a".repeat(40)));
    reject(&b, "source inventory");
}
#[test]
fn incorrect_blob_refused() {
    let b = Bundle::new();
    b.edit("source.json", |v| {
        v["files"]["Cargo.toml"]["blob"] = json!("a".repeat(40))
    });
    reject(&b, "source inventory");
}
#[test]
fn incorrect_source_hash_refused() {
    let b = Bundle::new();
    b.edit("source.json", |v| {
        v["files"]["Cargo.toml"]["sha256"] = json!("f".repeat(64))
    });
    reject(&b, "source inventory");
}
#[test]
fn duplicate_source_path_refused() {
    let b = Bundle::new();
    let text = std::fs::read_to_string(b.root.join("source.json")).unwrap();
    let value = offline::get(&b.root, "source.json")["files"]["Cargo.toml"].clone();
    let entry = serde_json::to_string(&value).unwrap();
    let changed = text.replacen(
        "\"files\": {",
        &format!("\"files\": {{\"Cargo.toml\":{entry},"),
        1,
    );
    assert_ne!(changed, text);
    std::fs::write(b.root.join("source.json"), changed).unwrap();
    b.rehash();
    reject(&b, "duplicate source path");
}
#[test]
fn missing_source_checkpoint_refused() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("source-before_probes.json")).unwrap();
    b.rehash();
    reject(&b, "source-before_probes.json");
}
#[test]
fn missing_source_state_refused() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("source-state.json")).unwrap();
    b.rehash();
    reject(&b, "source-state.json");
}
#[test]
fn real_detected_drift_restoration_and_marker_removal_stay_invalid() {
    let b = Bundle::new();
    let source = source::trusted(&b.source, &b.sha).unwrap();
    let e = Evidence::create(&b._directory.path().join("journal")).unwrap();
    let original = std::fs::read(b.source.join("Cargo.toml")).unwrap();
    let mut guard = Guard::new(source);
    std::fs::write(b.source.join("Cargo.toml"), b"changed source").unwrap();
    assert!(guard.check(&e, &b.source, "before_probes").is_err());
    std::fs::write(b.source.join("Cargo.toml"), original).unwrap();
    assert!(guard.check(&e, &b.source, "after_probes").is_err());
    guard.finish(&e).unwrap();
    for name in [
        "source-before_probes.json",
        "source-after_probes.json",
        "source-state.json",
        "source-invalidated.json",
    ] {
        std::fs::copy(e.root.join(name), b.root.join(name)).unwrap();
    }
    b.rehash();
    reject(&b, "source invalidated");
    std::fs::remove_file(b.root.join("source-invalidated.json")).unwrap();
    b.rehash();
    reject(&b, "source invalidated");
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_api03-host-qualification"))
        .args([
            "verify",
            b.root.to_str().unwrap(),
            &b.sha,
            "123",
            "1",
            b.source.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(
        String::from_utf8(o.stderr)
            .unwrap()
            .contains("source invalidated")
    );
}
#[test]
fn source_persistence_conflict_cannot_be_success() {
    let b = Bundle::new();
    let source = source::trusted(&b.source, &b.sha).unwrap();
    let e = Evidence::create(&b._directory.path().join("conflict")).unwrap();
    e.bytes("source-before_probes.json", b"occupied").unwrap();
    let mut guard = Guard::new(source);
    assert!(guard.check(&e, &b.source, "before_probes").is_err());
}
#[test]
fn standalone_good_and_semantic_negative_apply_same_gate() {
    let b = Bundle::new();
    verification::verify(&b.root, &b.sha, "123", "1", &b.source).unwrap();
    b.edit("source-state.json", |v| v["invalidated"] = json!(true));
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_api03-host-qualification"))
        .args([
            "verify",
            b.root.to_str().unwrap(),
            &b.sha,
            "123",
            "1",
            b.source.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(
        String::from_utf8(o.stderr)
            .unwrap()
            .contains("source invalidated")
    );
}
#[test]
fn detected_false_checkpoint_cannot_be_reset_by_summary() {
    let b = Bundle::new();
    put(
        &b.root,
        "source-before_probes.json",
        &json!({"phase":"before_probes","unchanged":false}),
    );
    b.rehash();
    reject(&b, "source invalidated or checkpoint contradicted");
}
