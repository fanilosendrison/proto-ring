#[path = "support/offline-bundle.rs"]
mod offline;
use offline::{Bundle, get, put};
use serde_json::json;
fn reject(b: &Bundle, reason: &str) {
    let error =
        api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source)
            .unwrap_err()
            .to_string();
    assert!(error.contains(reason), "{error}");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_api03-host-qualification"))
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
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains(reason));
}
fn omit(b: &Bundle, name: &str) {
    if b.root.join(name).exists() {
        std::fs::remove_file(b.root.join(name)).unwrap();
    }
    b.rehash();
}
#[test]
fn subject_root_identity_cannot_be_omitted() {
    let b = Bundle::new();
    omit(&b, "subject-root-identity.json");
    reject(&b, "H04/positive-coverage");
}
#[test]
fn jointly_missing_namespaces_cannot_compare_equal() {
    let b = Bundle::new();
    b.edit("descendant-stream-rendezvous.json", |v| {
        v["parent"].as_object_mut().unwrap().remove("pid_namespace");
        v["child"].as_object_mut().unwrap().remove("pid_namespace");
    });
    b.edit("descendant-stream-capability.json", |v| {
        v.as_object_mut().unwrap().remove("pid_namespace");
    });
    reject(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn two_empty_descriptor_identities_are_not_a_witness() {
    let b = Bundle::new();
    b.edit("descendant-stream-capability.json", |v| {
        v["fd_identity"] = json!({});
        v["target_identity"] = json!({});
    });
    reject(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn arbitrary_runtime_text_is_not_an_inventory() {
    let b = Bundle::new();
    std::fs::write(
        b.root.join("runtime.txt"),
        b"arbitrary non-observation longer than ten bytes",
    )
    .unwrap();
    b.rehash();
    reject(&b, "H01/runtime");
}
#[test]
fn kernel_observation_is_mandatory() {
    let b = Bundle::new();
    omit(&b, "kernel.json");
    reject(&b, "H01/runner");
}
#[test]
fn interpreter_fingerprint_cannot_be_disabled_by_producer_checks() {
    let b = Bundle::new();
    b.edit("policy-logs.stdout", |v| {
        v.as_object_mut().unwrap().remove("interpreter_sha256");
    });
    b.edit("campaign.json", |v| {
        v["cases"]["H05/environment"]["checks"] = json!([])
    });
    reject(&b, "H01/process-policy");
}
#[test]
fn jointly_omitted_process_inventory_is_not_optional() {
    let b = Bundle::new();
    let mut v = get(&b.root, "policy-logs.stdout");
    for key in [
        "interpreter",
        "environment_names",
        "namespaces",
        "mountinfo",
        "cgroup",
        "cwd",
        "python",
    ] {
        v.as_object_mut().unwrap().remove(key);
    }
    put(&b.root, "policy-logs.stdout", &v);
    b.rehash();
    reject(&b, "H01/process-policy");
}
