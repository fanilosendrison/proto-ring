#[path = "support/offline-bundle.rs"]
mod offline;
use offline::Bundle;
use serde_json::json;
fn reject(b: &Bundle, reason: &str) {
    let e = api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source)
        .unwrap_err()
        .to_string();
    assert!(e.contains(reason), "{e}");
}
#[test]
fn git_identity_observation_is_mandatory() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("positive-git-identities.json")).unwrap();
    b.rehash();
    reject(&b, "H04/positive-coverage");
}
#[test]
fn root_witness_cannot_contradict_linkage() {
    let b = Bundle::new();
    b.edit("subject-root-identity.json", |v| v["inode"] = json!(999));
    reject(&b, "subject root identity contradicted");
}
#[test]
fn git_linkage_cannot_resolve_outside() {
    let b = Bundle::new();
    b.edit("positive-git-identities.json", |v| {
        v["administration"][0]["resolved"] = json!("/outside/.git")
    });
    reject(&b, "filesystem linkage/alias unresolved");
}
#[test]
fn git_file_identity_cannot_be_empty() {
    let b = Bundle::new();
    b.edit("positive-git-identities.json", |v| {
        v["administration"][1]["identity"] = json!({})
    });
    reject(&b, "H04/positive-coverage");
}
#[test]
fn git_dependency_absence_cannot_be_asserted_without_record() {
    let b = Bundle::new();
    b.edit("positive-git-identities.json", |v| {
        v["administration"].as_array_mut().unwrap().pop();
    });
    reject(&b, "required Git identity coverage missing");
}
#[test]
fn namespace_link_must_be_real_format_not_empty() {
    let b = Bundle::new();
    b.edit("descendant-stream-rendezvous.json", |v| {
        v["parent"]["pid_namespace"] = json!("");
        v["child"]["pid_namespace"] = json!("");
    });
    b.edit("descendant-stream-capability.json", |v| {
        v["pid_namespace"] = json!("")
    });
    reject(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn zero_inode_descriptor_cannot_compare_equal() {
    let b = Bundle::new();
    b.edit("descendant-stream-capability.json", |v| {
        v["fd_identity"]["inode"] = json!(0);
        v["target_identity"]["inode"] = json!(0);
    });
    reject(&b, "filesystem identity/permissions incomplete");
}
#[test]
fn every_emitted_process_inventory_field_is_required() {
    for field in [
        "interpreter",
        "interpreter_sha256",
        "environment_names",
        "namespaces",
        "mountinfo",
        "cgroup",
        "cwd",
        "python",
    ] {
        let b = Bundle::new();
        b.edit("policy-logs.stdout", |v| {
            v.as_object_mut().unwrap().remove(field);
        });
        reject(&b, "H01/process-policy: process inventory");
    }
}
#[test]
fn environment_names_cannot_omit_an_image_variable() {
    let b = Bundle::new();
    b.edit("policy-logs.stdout", |v| {
        v["environment_names"].as_array_mut().unwrap().pop();
    });
    reject(
        &b,
        "effective environment inventory contradicts image/defaults",
    );
}
#[test]
fn mount_inventory_cannot_omit_the_program() {
    let b = Bundle::new();
    b.edit("policy-logs.stdout",|v|v["mountinfo"]=json!("10 1 0:1 / / ro - overlay overlay ro\n12 10 0:3 / /owned/run/subject ro - ext4 /subject ro\n"));
    reject(&b, "protected process mount not observed read-only");
}
#[test]
fn interpreter_hash_must_match_independent_image_bytes() {
    let b = Bundle::new();
    b.edit("policy-logs.stdout", |v| {
        v["interpreter_sha256"] = json!("a".repeat(64))
    });
    reject(&b, "image/process interpreter identity contradicted");
}
#[test]
fn interpreter_image_copy_is_required() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("image-interpreter.bin")).unwrap();
    b.rehash();
    reject(&b, "image-interpreter.bin");
}
#[test]
fn interpreter_image_copy_must_belong_to_same_container() {
    let b = Bundle::new();
    b.edit("image-interpreter.json", |v| {
        v["container_id"] = json!("a".repeat(64))
    });
    reject(&b, "image/process interpreter identity contradicted");
}
#[test]
fn docker_kernel_must_match_host_kernel_observation() {
    let b = Bundle::new();
    b.edit("runtime-info.json", |v| v["kernel"] = json!("6.9.0-other"));
    std::fs::copy(b.root.join("runtime-info.json"), b.root.join("runtime.txt")).unwrap();
    b.rehash();
    reject(&b, "runtime/kernel/cgroup correspondence invalid");
}
#[test]
fn structured_runtime_version_cannot_be_missing() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("runtime-version.json")).unwrap();
    b.rehash();
    reject(&b, "runtime-version.json");
}
