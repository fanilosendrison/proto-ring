#[path = "support/offline-bundle.rs"]
mod offline;
use offline::Bundle;
use serde_json::json;
fn reject(b: &Bundle, reason: &str) {
    let e = api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source)
        .unwrap_err()
        .to_string();
    assert!(e.contains(reason), "expected {reason}: {e}");
}
#[test]
fn h02_wrong_final_bytes() {
    let b = Bundle::new();
    std::fs::write(b.root.join("governance-after.bin"), b"wrong").unwrap();
    b.rehash();
    reject(&b, "H02/governance-mutation: mutation bytes");
}
#[test]
fn h02_different_target() {
    let b = Bundle::new();
    b.edit("governance-mutation.json", |v| {
        v["target"] = json!("/owned/run/subject/other")
    });
    reject(&b, "H02/governance-mutation: mutation attribution");
}
#[test]
fn h02_same_canonical_state() {
    let b = Bundle::new();
    b.edit("governance-mutation.json", |v| {
        v["after"]["observation"]["state"] = v["before"]["observation"]["state"].clone()
    });
    b.edit("rgs-after.json", |v| v["state"] = json!("a".repeat(64)));
    reject(
        &b,
        "H02/governance-mutation: canonical change not established",
    );
}
#[test]
fn h02_other_subject() {
    let b = Bundle::new();
    b.edit("governance-mutation.json", |v| {
        v["before"]["repository"]["value"] = json!("/owned/other")
    });
    reject(&b, "H02/governance-mutation: canonical mutation subject");
}
#[test]
fn h02_other_campaign() {
    let b = Bundle::new();
    b.edit("governance-mutation.json", |v| {
        v["campaign"] = json!("api03-999-1")
    });
    reject(&b, "H02/governance-mutation: mutation attribution");
}
#[test]
fn h02_missing_canonical_observation() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("rgs-before.json")).unwrap();
    b.rehash();
    reject(&b, "H02/governance-mutation:");
}
#[test]
fn h02_optional_checks_cannot_supply_required_observation() {
    let b = Bundle::new();
    b.edit("governance-mutation.json", |v| {
        v.as_object_mut().unwrap().remove("before");
    });
    b.edit("campaign.json", |v| {
        v["cases"]["H02/governance-mutation"]["checks"] =
            json!([{"kind":"equal","observed":true,"required":true}])
    });
    reject(&b, "H02/governance-mutation:");
}
#[test]
fn h02_complete_evidence_accepts_absent_optional_checks() {
    let b = Bundle::new();
    b.edit("campaign.json", |v| {
        v["cases"]["H02/governance-mutation"]["checks"] = json!([])
    });
    api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source).unwrap();
}
#[test]
fn h04_scope_contradicts_retained_provenance() {
    let b = Bundle::new();
    b.edit("canonical-scopes.json", |v| {
        v["governance"].as_array_mut().unwrap().pop();
    });
    reject(&b, "H04/positive-coverage: scope coverage");
}
#[test]
fn h04_discovered_scope_cannot_omit_an_already_governed_path() {
    let b = Bundle::new();
    b.edit("canonical-scopes.json", |v| {
        v["discovered"].as_array_mut().unwrap().remove(0);
    });
    reject(&b, "H04/positive-coverage: scope coverage");
}
#[test]
fn h06_child_role_must_match_its_instance() {
    let b = Bundle::new();
    b.edit("descendant-stream-rendezvous.json", |v| {
        v["child"]["kind"] = json!("writer")
    });
    reject(
        &b,
        "H06/descendant-stream: process attribution: rendezvous attribution mismatch",
    );
}
#[test]
fn h04_external_common_dir() {
    let b = Bundle::new();
    b.edit("positive-git-coverage.json", |v| {
        v["common_dir"] = json!("/owned/outside/.git")
    });
    reject(&b, "H04/positive-coverage: scope coverage");
}
#[test]
fn h04_external_git_dir() {
    let b = Bundle::new();
    b.edit("positive-git-coverage.json", |v| {
        v["git_dir"] = json!("/owned/outside/.git")
    });
    reject(&b, "H04/positive-coverage: scope coverage");
}
#[test]
fn h04_resolved_alias_outside() {
    let b = Bundle::new();
    b.edit("positive-git-coverage.json", |v| {
        v["paths"][0]["resolved"] = json!("/owned/run-foreign/subject/AGENTS.md")
    });
    reject(&b, "H04/positive-coverage: scope coverage");
}
#[test]
fn h04_duplicate_masks_missing_path() {
    let b = Bundle::new();
    b.edit("positive-git-coverage.json", |v| {
        v["paths"][4] = v["paths"][0].clone()
    });
    reject(&b, "H04/positive-coverage: scope coverage");
}
#[test]
fn h04_foreign_protection_domain() {
    let b = Bundle::new();
    b.edit("positive-git-coverage.json", |v| {
        v["paths"][0]["protection"] = json!("other-bind")
    });
    reject(&b, "H04/positive-coverage: scope coverage");
}
#[test]
fn h06_mixed_containers_same_campaign() {
    let b = Bundle::new();
    let other = offline::get(&b.root, "descendant-writer-rendezvous.json");
    offline::put(&b.root, "descendant-stream-rendezvous.json", &other);
    b.rehash();
    reject(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn h06_old_instance_readiness() {
    let b = Bundle::new();
    b.edit("descendant-stream-rendezvous.json", |v| {
        v["child"]["instance"] = json!("old-instance")
    });
    reject(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn h06_missing_capability_witness() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("descendant-stream-capability.json")).unwrap();
    b.rehash();
    reject(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn h06_missing_creation() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("descendant-stream-create.stdout")).unwrap();
    b.rehash();
    reject(&b, "H06/descendant-stream: launch policy");
}
#[test]
fn h06_missing_start() {
    let b = Bundle::new();
    std::fs::remove_file(b.root.join("descendant-stream-start-transport.json")).unwrap();
    b.rehash();
    reject(&b, "H06/descendant-stream: process observation");
}
#[test]
fn h06_principal_exit_without_perimeter_closure() {
    let b = Bundle::new();
    b.edit("descendant-stream-inspect.json", |v| {
        v[0]["State"]["Pid"] = json!(900)
    });
    reject(&b, "H06/descendant-stream: process observation");
}
#[test]
fn h06_other_container_terminal_inspection() {
    let b = Bundle::new();
    let other = offline::get(&b.root, "descendant-writer-inspect.json");
    offline::put(&b.root, "descendant-stream-inspect.json", &other);
    b.rehash();
    reject(&b, "H06/descendant-stream: launch policy");
}
#[test]
fn h06_namespace_pid_contradiction() {
    let b = Bundle::new();
    b.edit("descendant-stream-capability.json", |v| {
        v["namespace_child"] = json!(99)
    });
    reject(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn h06_write_capability_targets_other_object() {
    let b = Bundle::new();
    b.edit("descendant-stream-capability.json", |v| {
        v["fd_identity"]["inode"] = json!(999)
    });
    reject(&b, "H06/descendant-stream: process attribution");
}
