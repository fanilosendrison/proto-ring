#![forbid(unsafe_code)]
// Compiled only by the isolated qualification probe, never by the normal API
// dependency graph. Reuses existing engine controls; introduces no facade seam.
#[path = "git-governance.rs"]
mod git_governance;

use git_governance::GitGovernance;
use proto_ring_engine::repository_governance_state::{
    RepositoryGovernanceStage as Stage, conformance_internal as control,
};

#[test]
fn valid_governance_body_mutation_fails_at_repository_state() {
    let fixture = GitGovernance::minimal();
    let error = control::load_with_after_initial_capture_hook(&fixture.root, |root| {
        let path = root.join("AGENTS.md");
        let mut bytes = std::fs::read(&path).unwrap();
        bytes.extend_from_slice(b"\nchanged body\n");
        std::fs::write(path, bytes).unwrap();
        Ok(())
    })
    .unwrap_err();
    assert_eq!(error.diagnostic.stage, Stage::RepositoryState);
    eprintln!("VALID_STATE_DRIFT_STAGE={:?}", error.diagnostic.stage);
}
#[test]
fn corrupted_binding_during_load_preserves_component_failure_not_fabricated_drift_stage() {
    let fixture = GitGovernance::minimal();
    let error = control::load_with_after_initial_capture_hook(&fixture.root, |root| {
        std::fs::write(root.join("bindings.md"), b"changed\n").unwrap();
        Ok(())
    })
    .unwrap_err();
    assert_eq!(error.diagnostic.stage, Stage::GovernanceBindings);
    eprintln!("CORRUPTED_BINDING_STAGE={:?}", error.diagnostic.stage);
}
#[test]
fn deterministic_scope_override_fails_at_composition() {
    let fixture = GitGovernance::minimal();
    let before = ["AGENTS.md", "authority.md", "bindings.md"].map(std::path::PathBuf::from);
    let mut after = before.to_vec();
    after.push("additional-scope".into());
    let error = control::load_with_scope_overrides(&fixture.root, &before, &after).unwrap_err();
    assert_eq!(error.diagnostic.stage, Stage::Composition);
    eprintln!("SCOPE_DRIFT_STAGE={:?}", error.diagnostic.stage);
}
#[test]
fn equal_scopes_positive_control_returns_actual_original_state() {
    let fixture = GitGovernance::minimal();
    let scope = ["AGENTS.md", "authority.md", "bindings.md"].map(std::path::PathBuf::from);
    let loaded = control::load_with_scope_overrides(&fixture.root, &scope, &scope).unwrap();
    assert_eq!(loaded.observed_state.scope_paths, scope);
    assert_eq!(
        git_governance::native_units(&loaded.repository),
        git_governance::native_units(&fixture.root)
    );
}
#[test]
fn actual_public_nonexecuting_paths_have_zero_measured_provider_entries() {
    use proto_ring_api::{ExplainQuery, InspectSelector, NativePath};
    use proto_ring_engine::github_authoritative_ref_monotonicity::conformance_internal::with_provider_observation_forbidden;
    let fixture = GitGovernance::full();
    let (_,receipt)=with_provider_observation_forbidden(|| {
        proto_ring_api::discover().unwrap();
        let observed=proto_ring_api::open(&fixture.root).unwrap();
        observed.inspect(InspectSelector::All{}).unwrap();
        observed.explain(ExplainQuery::Capability{id:"repository_integrity".try_into().unwrap()}).unwrap();
        for (operation,input) in [
            ("inspect",serde_json::json!({"repository":NativePath::from_path(&fixture.root).unwrap(),"selector":{"kind":"all"}})),
            ("explain",serde_json::json!({"repository":NativePath::from_path(&fixture.root).unwrap(),"query":{"kind":"validation","id":"first"}})),
        ] {
            let request=serde_json::to_vec(&serde_json::json!({"api_version":1,"operation":operation,"input":input})).unwrap();
            let response=proto_ring_api::process_json(&request);
            assert!(matches!(response,proto_ring_api::Response::Success(_)));
        }
    }).unwrap();
    assert_eq!(receipt.invocations(), 0);
    eprintln!("FACADE_PROVIDER_ENTRIES={}", receipt.invocations());
}
