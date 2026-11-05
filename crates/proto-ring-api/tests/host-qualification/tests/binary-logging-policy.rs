#[path = "support/offline-bundle.rs"]
mod offline;
use offline::Bundle;
use serde_json::json;
fn reject(b: &Bundle, reason: &str) {
    let error =
        api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source)
            .unwrap_err()
            .to_string();
    assert!(error.contains(reason), "{error}");
}
#[test]
fn json_logging_is_not_an_admitted_binary_capture_backend() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["HostConfig"]["LogConfig"] = json!({"Type":"json-file","Config":{}})
    });
    reject(
        &b,
        "H06/simple: launch policy: lossless bounded logging policy not applied",
    );
}
#[test]
fn unbounded_logging_configuration_is_not_accepted() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["HostConfig"]["LogConfig"]["Config"] = json!({})
    });
    reject(
        &b,
        "H06/simple: launch policy: lossless bounded logging policy not applied",
    );
}
#[test]
fn missing_local_facility_is_not_skipped() {
    let b = Bundle::new();
    b.edit("runtime-info.json", |v| {
        v["logging_drivers"] = json!(["json-file"])
    });
    std::fs::copy(b.root.join("runtime-info.json"), b.root.join("runtime.txt")).unwrap();
    b.rehash();
    reject(&b, "required Docker local logging facility unavailable");
}
#[test]
fn utf8_replacement_does_not_change_expected_fixture_bytes() {
    let b = Bundle::new();
    std::fs::write(
        b.root.join("output-logs.stdout"),
        b"api03-stdout\0\xef\xbf\xbd\n",
    )
    .unwrap();
    std::fs::write(
        b.root.join("output-logs.stderr"),
        b"api03-stderr\xef\xbf\xbd\n",
    )
    .unwrap();
    b.rehash();
    reject(
        &b,
        "H07/streams: process observation: missing, oversized or contradictory bytes",
    );
}
