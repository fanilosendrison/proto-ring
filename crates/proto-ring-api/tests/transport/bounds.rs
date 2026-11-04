use super::{git_governance::GitGovernance, request, response};
use proto_ring_api::{InspectSelector, NativePath, open};
use serde_json::json;

#[test]
fn real_mapping_projection_hits_exact_response_boundary_and_never_silently_truncates() {
    let fixture = GitGovernance::minimal();
    let mut model = fixture.read("AGENTS.md");
    model["repository_governance"]["capabilities"]["governance_authority"]["configuration"][""] =
        json!("");
    fixture.write("AGENTS.md", model.clone());
    let path = serde_json::to_value(NativePath::from_path(&fixture.root).unwrap()).unwrap();
    let wire = request(
        "inspect",
        json!({"repository":path,"selector":{"kind":"model"}}),
    );
    let base = proto_ring_api::encode_response(&proto_ring_api::process_json(&wire))
        .unwrap()
        .len();
    assert_eq!(response(&wire)["outcome"], "result");
    for length in [16_777_215, 16_777_216, 16_777_217] {
        let count = length - base;
        model["repository_governance"]["capabilities"]["governance_authority"]["configuration"][""] =
            json!("x".repeat(count));
        fixture.write("AGENTS.md", model.clone());
        let result = proto_ring_api::process_json(&wire);
        let encoded = proto_ring_api::encode_response(&result).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        if length <= 16_777_216 {
            assert_eq!(encoded.len(), length);
            assert_eq!(value["outcome"], "result");
            assert_eq!(value["value"]["component"]["data"]["capabilities"]["governance_authority"]["configuration"][""]["value"].as_str().unwrap().len(),count);
        } else {
            assert_eq!(value["outcome"], "controlled_failure");
            assert_eq!(value["error"]["code"], "response_too_large");
            assert!(value.get("value").is_none());
        }
    }
    // Both standalone observation operations must emit only the small explicit
    // failure frame when the complete real projection cannot fit.
    model["repository_governance"]["capabilities"]["governance_authority"]["configuration"][""] =
        json!("x".repeat(16_779_264));
    fixture.write("AGENTS.md", model);
    for tail in [
        vec!["--selector", "model"],
        vec![
            "--query-json",
            r#"{"kind":"capability","id":"governance_authority"}"#,
        ],
    ] {
        let verb = if tail[0] == "--selector" {
            "inspect"
        } else {
            "explain"
        };
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_proto-ring"))
            .arg(verb)
            .arg("--repository")
            .arg(&fixture.root)
            .args(tail)
            .arg("--json")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        assert!(output.stdout.len() < 1024);
        assert_eq!(output.stdout.last(), Some(&b'\n'));
        assert_eq!(
            output.stdout.iter().filter(|byte| **byte == b'\n').count(),
            1
        );
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["operation"], verb);
        assert_eq!(value["error"]["code"], "response_too_large");
        assert!(value.get("value").is_none());
    }
}
#[test]
fn large_real_ri_profile_keeps_all_members_commands_and_original_order() {
    let fixture = GitGovernance::full();
    let mut profile = fixture.read("integrity.md");
    let template = profile["repository_integrity"]["validations"]["first"].clone();
    let mut validations = serde_json::Map::new();
    let mut order = Vec::new();
    for index in 0..256 {
        let id = format!("validation-{index:04}");
        let mut validation = template.clone();
        validation["command"]["arguments"] = json!(["sentinel", id]);
        validations.insert(id.clone(), validation);
        order.push(id);
    }
    order.reverse();
    profile["repository_integrity"]["validations"] = json!(validations);
    profile["repository_integrity"]["order"] = json!(order);
    fixture.write("integrity.md", profile);
    let observed = open(&fixture.root).unwrap();
    let value = response(&request(
        "inspect",
        json!({"repository":NativePath::from_path(&fixture.root).unwrap(),"selector":{"kind":"repository_integrity"}}),
    ));
    assert_eq!(
        value["value"],
        serde_json::to_value(
            observed
                .inspect(InspectSelector::RepositoryIntegrity {})
                .unwrap()
        )
        .unwrap()
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_proto-ring"))
        .args(["inspect", "--repository"])
        .arg(&fixture.root)
        .args(["--selector", "repository_integrity", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        value
    );
    let profile = &value["value"]["component"]["data"];
    assert_eq!(profile["order"], json!(order));
    assert_eq!(profile["validations"].as_object().unwrap().len(), 256);
    for id in order {
        assert_eq!(
            profile["validations"][&id]["command"]["arguments"],
            json!(["sentinel", id])
        );
    }
}
