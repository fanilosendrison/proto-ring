#[path = "support/offline-bundle.rs"]
mod offline;
use offline::Bundle;
use serde_json::json;
fn reject(b: &Bundle, case: &str) {
    let error =
        api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source)
            .unwrap_err()
            .to_string();
    assert!(error.contains(case), "{error}");
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
    assert!(String::from_utf8(out.stderr).unwrap().contains(case));
}
#[test]
fn empty_binding_replacement_is_not_observed_invalidation() {
    let b = Bundle::new();
    b.edit("replaced-binding.json", |v| v["after"] = json!({}));
    reject(&b, "H04/replaced-binding");
}
#[test]
fn empty_guard_replacement_is_not_observed_invalidation() {
    let b = Bundle::new();
    b.edit("guard-loss.json", |v| v["after"] = json!({}));
    reject(&b, "H08/guard-loss");
}
#[test]
fn every_after_identity_field_is_required() {
    for (file, case) in [
        ("replaced-binding.json", "H04/replaced-binding"),
        ("guard-loss.json", "H08/guard-loss"),
    ] {
        for field in ["device", "inode", "mode", "links"] {
            let b = Bundle::new();
            b.edit(file, |v| {
                v["after"].as_object_mut().unwrap().remove(field);
            });
            reject(&b, case);
        }
    }
}
#[test]
fn malformed_after_inode_cannot_compare_unequal() {
    for (file, case) in [
        ("replaced-binding.json", "H04/replaced-binding"),
        ("guard-loss.json", "H08/guard-loss"),
    ] {
        for bad in [json!(null), json!("not-an-inode"), json!(0)] {
            let b = Bundle::new();
            b.edit(file, |v| v["after"]["inode"] = bad);
            reject(&b, case);
        }
    }
}
#[test]
fn unchanged_object_is_not_replacement() {
    for (file, case) in [
        ("replaced-binding.json", "H04/replaced-binding"),
        ("guard-loss.json", "H08/guard-loss"),
    ] {
        let b = Bundle::new();
        b.edit(file, |v| v["after"] = v["before"].clone());
        reject(&b, case);
    }
}
#[test]
fn negative_alias_requires_an_observed_owned_target() {
    let b = Bundle::new();
    b.edit("escaping-path.json", |v| v["resolved"] = json!(""));
    reject(&b, "escaping alias not established");
}
#[test]
fn negative_git_fixture_root_cannot_be_arbitrary() {
    let b = Bundle::new();
    b.edit("external-git-dir.json", |v| v["root"] = json!("unrelated"));
    reject(&b, "negative Git coverage not established");
}
#[test]
fn invalidation_must_belong_to_campaign_and_resource() {
    for (file, case) in [
        ("replaced-binding.json", "H04/replaced-binding"),
        ("guard-loss.json", "H08/guard-loss"),
    ] {
        for key in ["path", "campaign"] {
            let b = Bundle::new();
            b.edit(file, |v| v[key] = json!("foreign"));
            reject(&b, case);
        }
    }
}
