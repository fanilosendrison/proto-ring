use super::git_governance::{GitGovernance, native_units};
use proto_ring_api::{
    ErrorCode, GovernanceStage as Stage, InspectSelector, NativePath, open, process_json,
};
use serde_json::{Value, json};

fn wire(repository: &std::path::Path) -> Value {
    let request=serde_json::to_vec(&json!({"api_version":1,"operation":"inspect","input":{"repository":NativePath::from_path(repository).unwrap(),"selector":{"kind":"all"}}})).unwrap();
    serde_json::to_value(process_json(&request)).unwrap()
}
#[test]
fn ten_real_construction_failures_preserve_exact_stages_and_never_return_partial_values() {
    for (kind, expected) in [
        ("not-git", Stage::RepositoryState),
        ("bootstrap", Stage::Bootstrap),
        ("model-one", Stage::RepositoryGovernanceModel),
        ("authority", Stage::GovernanceAuthority),
        ("objects", Stage::GovernedObjects),
        ("bindings", Stage::GovernanceBindings),
        ("integrity", Stage::RepositoryIntegrity),
        ("projections", Stage::ProjectionIntegrity),
        ("evidence", Stage::EvidenceRequirements),
        ("composition", Stage::Composition),
    ] {
        let fixture = GitGovernance::full();
        match kind {
            "not-git" => std::fs::remove_dir_all(fixture.root.join(".git")).unwrap(),
            "bootstrap" => {
                std::fs::write(fixture.root.join("AGENTS.md"), b"invalid bootstrap\n").unwrap()
            }
            "model-one" => {
                let mut model = fixture.read("AGENTS.md");
                model["repository_governance"]["model_version"] = json!(1);
                fixture.write("AGENTS.md", model);
            }
            "composition" => {
                let mut bindings = fixture.read("bindings.md");
                bindings["governance_bindings"]["bindings"]["uncomposable"] = json!({"kind":"governance_contract","scope":{"kind":"capability","capability":"not-declared"},
                    "identity":{"repository":"example/contract","commit":"b".repeat(40),"path":"contract.md"},"authority":{"responsibility":"quality","source":"bindings"}});
                fixture.write("bindings.md", bindings);
            }
            other => fixture.write(&format!("{other}.md"), json!({})),
        }
        let error = open(&fixture.root).unwrap_err();
        assert_eq!(error.code, ErrorCode::GovernanceLoadFailed);
        assert_eq!(error.stage, Some(expected), "{kind}");
        let response = wire(&fixture.root);
        assert_eq!(response["outcome"], "controlled_failure");
        assert_eq!(response["error"]["code"], "governance_load_failed");
        assert_eq!(
            response["error"]["stage"],
            serde_json::to_value(expected).unwrap()
        );
        assert!(response.get("value").is_none());
        assert_eq!(response["error"]["attempt_id"], json!(null));
        assert_eq!(response["error"]["effects_possible"], false);
    }
}
#[test]
fn missing_ri_route_target_fails_at_model_resolution_while_invalid_existing_profile_fails_at_ri() {
    let fixture = GitGovernance::full();
    std::fs::remove_file(fixture.root.join("integrity.md")).unwrap();
    assert_eq!(
        open(&fixture.root).unwrap_err().stage,
        Some(Stage::RepositoryGovernanceModel)
    );
    let response = wire(&fixture.root);
    assert_eq!(response["error"]["stage"], "repository_governance_model");
    assert!(response.get("value").is_none());
    fixture.write("integrity.md", json!({"repository_integrity":{}}));
    assert_eq!(
        open(&fixture.root).unwrap_err().stage,
        Some(Stage::RepositoryIntegrity)
    );
    let minimal = GitGovernance::minimal();
    let observed = open(&minimal.root).unwrap();
    assert_eq!(
        observed
            .inspect(InspectSelector::RepositoryIntegrity {})
            .unwrap_err()
            .code,
        ErrorCode::ComponentNotDeclared
    );
}
#[test]
fn references_distinguish_repository_and_scope_domains_without_check_admission_semantics() {
    let first = GitGovernance::minimal();
    let second = GitGovernance::minimal();
    let a = open(&first.root).unwrap();
    let b = open(&second.root).unwrap();
    assert_ne!(a.reference().repository, b.reference().repository);
    assert_ne!(a.reference().scope, b.reference().scope);
    assert_ne!(native_units(a.repository()), native_units(b.repository()));
    std::fs::write(first.root.join("extra.md"), b"extra scope\n").unwrap();
    let mut authority = first.read("authority.md");
    authority["governance_authority"]["sources"]["extra"] = json!({"repository_target":"extra.md"});
    authority["governance_authority"]["responsibilities"]["quality"]["roles"]["extra"] =
        json!("authority");
    first.write("authority.md", authority);
    let changed = open(&first.root).unwrap();
    assert_eq!(a.reference().repository, changed.reference().repository);
    assert_ne!(a.reference().scope, changed.reference().scope);
    let request=serde_json::to_vec(&json!({"api_version":1,"operation":"inspect","input":{"repository":NativePath::from_path(&first.root).unwrap(),"selector":{"kind":"all"},"expected_observation":a.reference()}})).unwrap();
    assert_eq!(
        serde_json::to_value(process_json(&request)).unwrap()["error"]["code"],
        "invalid_request"
    );
}
#[test]
fn unprojected_carrier_body_cannot_escape_as_raw_state_identity() {
    let fixture = GitGovernance::full();
    let path = fixture.root.join("AGENTS.md");
    let marker = "UNPROJECTED-RAW-STATE-BODY-9d931";
    let mut body = std::fs::read(&path).unwrap();
    body.extend_from_slice(marker.as_bytes());
    std::fs::write(path, body).unwrap();
    let observed = open(&fixture.root).unwrap();
    assert!(!format!("{observed:?}").contains(marker));
    let result =
        serde_json::to_string(&observed.inspect(InspectSelector::All {}).unwrap()).unwrap();
    assert!(!result.contains(marker));
    assert_eq!(observed.reference().state.as_str().len(), 64);
    assert_eq!(
        wire(&fixture.root)["value"]["observation"]["coherence_basis"],
        "ordinary_capture"
    );
}
