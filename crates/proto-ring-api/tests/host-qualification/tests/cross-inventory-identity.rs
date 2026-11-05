#[path = "support/offline-bundle.rs"]
mod offline;
use offline::{Bundle, get};
use serde_json::json;
fn reject(b: &Bundle) {
    let e = api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source)
        .unwrap_err()
        .to_string();
    assert!(e.contains("H04/positive-coverage: scope coverage"), "{e}");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_api03-host-qualification"))
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
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("H04/positive-coverage: scope coverage")
    );
}
#[test]
fn canonical_file_cannot_alias_git_head_identity() {
    let b = Bundle::new();
    let head =
        get(&b.root, "positive-git-identities.json")["administration"][1]["identity"].clone();
    b.edit("positive-git-coverage.json", |v| {
        v["paths"][0]["identity"] = head
    });
    reject(&b);
}
#[test]
fn zero_inode_is_not_a_scope_witness() {
    let b = Bundle::new();
    b.edit("positive-git-coverage.json", |v| {
        v["paths"][0]["identity"]["inode"] = json!(0)
    });
    reject(&b);
}
#[test]
fn scope_identity_cannot_omit_complete_fields() {
    let b = Bundle::new();
    b.edit("positive-git-coverage.json", |v| {
        v["paths"][0]["identity"]
            .as_object_mut()
            .unwrap()
            .remove("device");
    });
    reject(&b);
}
