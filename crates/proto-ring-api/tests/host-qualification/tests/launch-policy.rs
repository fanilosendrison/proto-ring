use api03_host_qualification::{evidence::Campaign, verification};
use serde_json::{Value, json};
use std::fs;
fn fixture() -> (Value, Value) {
    let actual:Value=serde_json::from_str(r#"[{"Image":"sha256:141fe9b385f3bb6d618c07ace1836c0b3c72bf525566ac3eb13cd1f405069877","Config":{"User":"1001:1001","WorkingDir":"/subject","Cmd":["python3","probe.py"],"Env":["LANG=C"]},"HostConfig":{"Privileged":false,"ReadonlyRootfs":true,"LogConfig":{"Type":"local","Config":{"max-size":"1m","max-file":"1","compress":"false"}},"PidsLimit":64,"Memory":536870912,"NanoCpus":1000000000,"PidMode":"","IpcMode":"private","CapDrop":["ALL"],"CapAdd":null,"SecurityOpt":["no-new-privileges"],"Devices":null,"DeviceRequests":null,"PortBindings":{},"NetworkMode":"none"},"Mounts":[{"Source":"/owned/subject","Destination":"/subject","RW":true,"Type":"bind"}]}]"#).unwrap();
    let expected:Value=serde_json::from_str(r#"{"network":"none","user":"1001:1001","cwd":"/subject","argv":["python3","probe.py"],"env":["LANG=C"],"mounts":[{"source":"/owned/subject","destination":"/subject","rw":true,"type":"bind"}]}"#).unwrap();
    (actual, expected)
}
#[test]
fn exact_inspection_policy_passes() {
    let (a, e) = fixture();
    verification::container_policy(&a, &e).unwrap();
}
#[test]
fn substituted_image_fails() {
    let (mut a, e) = fixture();
    a[0]["Image"] = json!("mutable-tag");
    assert!(verification::container_policy(&a, &e).is_err());
}
#[test]
fn unexpected_mount_fails() {
    let (mut a, e) = fixture();
    a[0]["Mounts"][0]["Source"] = json!("/foreign");
    assert!(verification::container_policy(&a, &e).is_err());
}
#[test]
fn host_pid_namespace_fails() {
    let (mut a, e) = fixture();
    a[0]["HostConfig"]["PidMode"] = json!("host");
    assert!(verification::container_policy(&a, &e).is_err());
}
#[test]
fn credential_environment_substitution_fails() {
    let (mut a, e) = fixture();
    a[0]["Config"]["Env"] = json!(["GITHUB_TOKEN=nonsecret-canary"]);
    assert!(verification::container_policy(&a, &e).is_err());
}
#[test]
fn source_run_attempt_mismatch_rejected_before_other_evidence() {
    let d = tempfile::tempdir().unwrap();
    let c = Campaign::new("a".repeat(40), "b".repeat(40), "123".into(), "1".into());
    fs::write(
        d.path().join("campaign.json"),
        serde_json::to_vec(&c).unwrap(),
    )
    .unwrap();
    for (sha, run, attempt) in [
        ("c".repeat(40), "123", "1"),
        ("a".repeat(40), "999", "1"),
        ("a".repeat(40), "123", "2"),
    ] {
        let error = verification::verify(d.path(), &sha, run, attempt, d.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("source/run/attempt mismatch"));
    }
}
