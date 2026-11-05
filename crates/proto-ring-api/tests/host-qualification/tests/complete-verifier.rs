#[path = "support/offline-bundle.rs"]
mod offline;
use api03_host_qualification::verification;
use offline::{Bundle, get, put};
use serde_json::json;
fn verify(b: &Bundle) -> api03_host_qualification::Result<()> {
    verification::verify(&b.root, &b.sha, "123", "1", &b.source)
}
fn rejects(b: &Bundle, reason: &str) {
    let e = verify(b)
        .expect_err("faulty complete bundle accepted")
        .to_string();
    assert!(e.contains(reason), "expected {reason}, observed {e}");
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
    assert!(String::from_utf8(o.stderr).unwrap().contains(reason));
}
#[test]
fn complete_independent_positive_bundle_is_accepted() {
    let b = Bundle::new();
    verify(&b).unwrap();
}
#[test]
fn standalone_complete_positive_is_accepted() {
    let b = Bundle::new();
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
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}
#[test]
fn matching_root_user_is_forbidden() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["Config"]["User"] = json!("0:0")
    });
    b.edit("simple-launch.json", |v| v["user"] = json!("0:0"));
    rejects(&b, "H06/simple: launch policy");
}
#[test]
fn matching_host_network_is_forbidden() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["HostConfig"]["NetworkMode"] = json!("host")
    });
    b.edit("simple-launch.json", |v| v["network"] = json!("host"));
    rejects(&b, "H06/simple: launch policy");
}
#[test]
fn expected_and_observed_program_substitution_is_forbidden() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["Config"]["Cmd"][0] = json!("other")
    });
    b.edit("simple-launch.json", |v| v["argv"][0] = json!("other"));
    rejects(&b, "H06/simple: launch policy");
}
#[test]
fn h02_proof_cannot_be_omitted_by_removing_producer_checks() {
    let b = Bundle::new();
    b.edit("campaign.json", |v| {
        v["cases"]["H02/governance-mutation"]["checks"] = json!([])
    });
    std::fs::remove_file(b.root.join("governance-effect.bin")).unwrap();
    b.rehash();
    rejects(&b, "H02/governance-mutation");
}
#[test]
fn h02_unchanged_final_bytes_refused() {
    let b = Bundle::new();
    std::fs::write(
        b.root.join("governance-after.bin"),
        include_bytes!("../fixtures/authority.md"),
    )
    .unwrap();
    b.rehash();
    rejects(&b, "H02/governance-mutation: mutation bytes");
}
#[test]
fn source_change_marker_cannot_be_ignored() {
    let b = Bundle::new();
    std::fs::write(b.root.join("source-change.txt"), b"detected, then restored").unwrap();
    b.rehash();
    rejects(&b, "source invalidated");
}
#[test]
fn required_source_omission_refused() {
    let b = Bundle::new();
    b.edit("source.json", |v| {
        v["files"]
            .as_object_mut()
            .unwrap()
            .remove("crates/proto-ring-engine/src/lib.rs");
    });
    rejects(&b, "source inventory");
}
#[test]
fn scope_omission_refused() {
    let b = Bundle::new();
    b.edit("positive-git-coverage.json", |v| {
        v["paths"].as_array_mut().unwrap().pop();
    });
    rejects(&b, "H04/positive-coverage: scope coverage");
}
#[test]
fn h06_other_container_rendezvous_refused() {
    let b = Bundle::new();
    b.edit("descendant-stream-rendezvous.json", |v| {
        v["container_id"] = json!("f".repeat(64))
    });
    rejects(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn h06_other_campaign_consistent_child_and_parent_refused() {
    let b = Bundle::new();
    b.edit("descendant-stream-rendezvous.json", |v| {
        v["campaign"] = json!("api03-999-1");
        v["child"]["campaign"] = json!("api03-999-1");
    });
    rejects(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn h06_order_refused() {
    let b = Bundle::new();
    b.edit("descendant-stream-rendezvous.json", |v| {
        v["events"].as_array_mut().unwrap().swap(6, 7)
    });
    rejects(&b, "H06/descendant-stream: process attribution");
}
#[test]
fn wrong_inventory_digest_detected() {
    let b = Bundle::new();
    std::fs::write(b.root.join("runtime.txt"), b"changed without rehash").unwrap();
    rejects(&b, "inventory mismatch");
}
#[test]
fn globally_incomplete_bundle_refused() {
    let b = Bundle::new();
    let mut v = get(&b.root, "campaign.json");
    v["cases"].as_object_mut().unwrap().remove("H07/streams");
    put(&b.root, "campaign.json", &v);
    b.rehash();
    rejects(&b, "mandatory subcase");
}
