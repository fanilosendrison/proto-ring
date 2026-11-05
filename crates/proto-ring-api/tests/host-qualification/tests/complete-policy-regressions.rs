#[path = "support/offline-bundle.rs"]
mod offline;
use offline::Bundle;
use serde_json::json;
fn reject(b: &Bundle) {
    let error =
        api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source)
            .unwrap_err()
            .to_string();
    assert!(error.contains("H06/simple: launch policy"), "{error}");
}
#[test]
fn added_argument_matching_request_is_refused() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["Config"]["Cmd"]
            .as_array_mut()
            .unwrap()
            .push(json!("extra"))
    });
    b.edit("simple-launch.json", |v| {
        v["argv"].as_array_mut().unwrap().push(json!("extra"))
    });
    reject(&b);
}
#[test]
fn foreign_mount_matching_request_is_refused() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["Mounts"][1]["Source"] = json!("/owned/run-foreign/subject")
    });
    b.edit("simple-launch.json", |v| {
        v["mounts"][1]["source"] = json!("/owned/run-foreign/subject")
    });
    reject(&b);
}
#[test]
fn extra_mount_matching_request_is_refused() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["Mounts"].as_array_mut().unwrap().push(
            json!({"Type":"bind","Source":"/owned/run/extra","Destination":"/extra","RW":false}),
        )
    });
    b.edit("simple-launch.json", |v| {
        v["mounts"].as_array_mut().unwrap().push(
            json!({"type":"bind","source":"/owned/run/extra","destination":"/extra","rw":false}),
        )
    });
    reject(&b);
}
#[test]
fn writable_mount_matching_request_is_refused() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["Mounts"][1]["RW"] = json!(true)
    });
    b.edit("simple-launch.json", |v| v["mounts"][1]["rw"] = json!(true));
    reject(&b);
}
#[test]
fn network_control_exception_cannot_be_reused_by_simple() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["HostConfig"]["NetworkMode"] = json!("api03-123-1-network")
    });
    b.edit("simple-launch.json", |v| {
        v["network"] = json!("api03-123-1-network")
    });
    reject(&b);
}
#[test]
fn modified_argument_matching_request_is_refused() {
    let b = Bundle::new();
    b.edit("simple-created.json", |v| {
        v[0]["Config"]["Cmd"][4] = json!("output")
    });
    b.edit("simple-launch.json", |v| v["argv"][4] = json!("output"));
    reject(&b);
}
#[test]
fn ambient_credential_matching_expectations_is_refused() {
    let b = Bundle::new();
    for name in ["image.json", "simple-created.json"] {
        b.edit(name, |v| {
            v[0]["Config"]["Env"]
                .as_array_mut()
                .unwrap()
                .push(json!("GITHUB_TOKEN=nonsecret-synthetic-canary"))
        });
    }
    b.edit("simple-launch.json", |v| {
        v["env"]
            .as_array_mut()
            .unwrap()
            .push(json!("GITHUB_TOKEN=nonsecret-synthetic-canary"))
    });
    let error =
        api03_host_qualification::verification::verify(&b.root, &b.sha, "123", "1", &b.source)
            .unwrap_err()
            .to_string();
    assert!(error.contains("launch policy"), "{error}");
}
