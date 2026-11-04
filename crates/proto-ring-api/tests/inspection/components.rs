use proto_ring_api::open;
use serde_json::json;

use super::{git_governance::GitGovernance, inspect, inspection_fixture, path};

#[test]
fn all_and_each_component_are_complete_schema_conforming_views() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let all = inspect(&observed, json!({"kind":"all"}));
    let components = &all["components"];
    assert_eq!(components.as_object().unwrap().len(), 8);
    for name in [
        "bootstrap",
        "model",
        "authority",
        "governed_objects",
        "bindings",
        "repository_integrity",
        "projections",
        "evidence_requirements",
    ] {
        assert_eq!(components[name]["presence"], "present");
        let component = inspect(&observed, json!({"kind":name}));
        assert_eq!(component["result_kind"], "component");
        assert_eq!(component["component"]["name"], name);
        assert_eq!(component["component"]["data"], components[name]["data"]);
    }
}

#[test]
fn model_bootstrap_routes_and_structured_values_preserve_actual_declarations() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let model = inspect(&observed, json!({"kind":"model"}))["component"]["data"].clone();
    assert_eq!(model["model_version"], 2);
    let capability = &model["capabilities"]["repository_integrity"];
    assert_eq!(
        capability["configuration"][""],
        json!({"kind":"boolean","value":true})
    );
    assert_eq!(
        capability["configuration"]["big"],
        json!({"kind":"integer","decimal":inspection_fixture::BIG})
    );
    assert_eq!(
        capability["configuration"]["nested"],
        json!({"kind":"sequence","items":[
            {"kind":"null"},{"kind":"boolean","value":false},{"kind":"string","value":""},
            {"kind":"mapping","entries":[{"key":"","value":{"kind":"string","value":"opaque"}}]}
        ]})
    );
    assert_eq!(
        capability["routes"]["profile"],
        json!({"segments":["capabilities","repository_integrity","routes","profile"],
        "declared_path":"integrity.md","resolved_target":path(&fixture,"integrity.md")})
    );
    let bootstrap = inspect(&observed, json!({"kind":"bootstrap"}))["component"]["data"].clone();
    assert_eq!(bootstrap["carrier"], path(&fixture, "AGENTS.md"));
    assert_eq!(bootstrap["metadata"]["kind"], "mapping");
    assert_eq!(bootstrap["repository_governance"]["kind"], "mapping");
    assert!(bootstrap.to_string().contains(inspection_fixture::BIG));
}

#[test]
fn authority_objects_and_bindings_preserve_sources_roles_relations_and_scope() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let all = inspect(&observed, json!({"kind":"all"}));
    let authority = &all["components"]["authority"]["data"];
    assert_eq!(authority["carrier"], path(&fixture, "authority.md"));
    assert_eq!(
        authority["sources"]["external"]["repository_target"],
        json!(null)
    );
    assert_eq!(
        authority["sources"]["bindings"]["repository_target"],
        json!({
            "segments":["bindings","repository_target"],"declared_path":"bindings.md","resolved_target":path(&fixture,"bindings.md")
        })
    );
    assert_eq!(
        authority["responsibilities"]["quality"]["roles"]["generator"],
        "non_authoritative"
    );
    assert_eq!(
        authority["responsibilities"]["quality"]["roles"]["secondary"],
        "secondary_representation"
    );
    assert_eq!(
        authority["responsibilities"]["quality"]["precedence"],
        json!([{"higher_source":"bindings","lower_source":"integrity"}])
    );
    let objects = &all["components"]["governed_objects"]["data"];
    assert_eq!(objects["carrier"], path(&fixture, "objects.md"));
    assert_eq!(
        objects["interfaces"]["repository"]["objects"]["tree"],
        json!({"responsibilities":["quality"],"relations":[
            {"relation":"supports","responsibility":"quality","target":{"interface":"repository","object":"tree"}}
        ]})
    );
    let bindings = &all["components"]["bindings"]["data"];
    assert_eq!(bindings["source"], "bindings");
    assert_eq!(bindings["carrier"], path(&fixture, "bindings.md"));
    assert_eq!(
        bindings["bindings"]["object-contract"]["scope"],
        json!({"kind":"governed_object","target":{"interface":"repository","object":"tree"}})
    );
    assert_eq!(
        bindings["bindings"]["cap-contract"]["identity"],
        json!({"kind":"governance_contract","repository":"example/contracts",
        "commit":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","path":"contract.md"})
    );
}

#[test]
fn ri_preserves_full_profile_order_instances_commands_and_null_target() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let data =
        inspect(&observed, json!({"kind":"repository_integrity"}))["component"]["data"].clone();
    assert_eq!(data["environments"], json!(["declared-only", "z-env"]));
    assert_eq!(data["continue_after_non_satisfied"], true);
    assert_eq!(data["order"], json!(["second", "first"]));
    assert_eq!(
        data["profile_ref"],
        serde_json::to_value(observed.profile_reference().unwrap()).unwrap()
    );
    assert_eq!(data["carrier"], path(&fixture, "integrity.md"));
    let first = &data["validations"]["first"];
    assert_eq!(first["prerequisites"], json!(["second"]));
    assert_eq!(
        first["instances"],
        json!({"kind":"repository_paths","mode":"for_each","selectors":[{"kind":"glob","value":"*.md"},{"kind":"path","value":"README.md"}]})
    );
    assert_eq!(
        first["command"],
        json!({"environment":"declared-only","arguments":["","-c","printf unexpected > EFFECT-MARKER"],"undetermined_exit_codes":["-1","127","5"]})
    );
    assert_eq!(
        first["target"],
        json!({"interface":"repository","object":"tree"})
    );
    assert_eq!(data["validations"]["second"]["target"], json!(null));
}

#[test]
fn append_all_instances_preserve_selector_order_without_realization() {
    let fixture = inspection_fixture::rich();
    let mut profile = fixture.read("integrity.md");
    profile["repository_integrity"]["validations"]["first"]["instances"]["mode"] =
        json!("append_all");
    fixture.write("integrity.md", profile);
    let observed = open(&fixture.root).unwrap();
    let result = inspect(&observed, json!({"kind":"validation","id":"first"}));
    assert_eq!(
        result["subject"]["declaration"]["instances"],
        json!({"kind":"repository_paths","mode":"append_all","selectors":[
            {"kind":"glob","value":"*.md"},{"kind":"path","value":"README.md"}
        ]})
    );
}

#[test]
fn empty_declared_profile_remains_present_complete_and_without_a_verdict() {
    let fixture = GitGovernance::full();
    let mut profile = fixture.read("integrity.md");
    profile["repository_integrity"]["validations"] = json!({});
    profile["repository_integrity"]["order"] = json!([]);
    fixture.write("integrity.md", profile);
    let observed = open(&fixture.root).unwrap();
    let result = inspect(&observed, json!({"kind":"repository_integrity"}));
    assert_eq!(result["component"]["data"]["validations"], json!({}));
    assert_eq!(result["component"]["data"]["order"], json!([]));
    assert!(result["component"]["data"].get("verdict").is_none());
    let all = inspect(&observed, json!({"kind":"all"}));
    assert_eq!(
        all["components"]["repository_integrity"]["presence"],
        "present"
    );
}

#[test]
fn registries_preserve_all_modes_nullable_fields_nested_admission_and_authorities() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let all = inspect(&observed, json!({"kind":"all"}));
    let registry = &all["components"]["projections"]["data"];
    assert_eq!(
        registry["authority"],
        json!({"responsibility":"quality","source":"projections"})
    );
    assert_eq!(registry["carrier"], path(&fixture, "projections.md"));
    let generated = &registry["projections"]["generated"];
    assert_eq!(
        generated,
        &json!({"responsibility":"quality","canonical_source":"bindings","secondary_source":"secondary",
        "mode":"generated","validation":"first","generator_source":"generator","boundary_source":null,
        "target":{"interface":"repository","object":"tree"},"binding":"provider"})
    );
    assert_eq!(
        registry["projections"]["historical"]["boundary_source"],
        "bindings"
    );
    assert_eq!(
        registry["projections"]["maintained"]["mode"],
        "mechanically_validated_maintained"
    );
    assert_eq!(registry["projections"]["reference"]["target"], json!(null));
    let registry = &all["components"]["evidence_requirements"]["data"];
    assert_eq!(
        registry["authority"],
        json!({"responsibility":"quality","source":"evidence"})
    );
    assert_eq!(registry["carrier"], path(&fixture, "evidence.md"));
    assert_eq!(
        registry["requirements"]["sourced"],
        json!({"responsibility":"quality","instances":{"kind":"source","source":"bindings"},
        "evidence_classes":{"kind":"source","classes":[],"source":"external"},"subject_source":"bindings",
        "context":{"required":true,"source":"external"},"candidate_source":"secondary","target":{"interface":"repository","object":"tree"}})
    );
    assert_eq!(
        registry["requirements"]["explicit"],
        json!({"responsibility":"quality","instances":{"kind":"single","source":null},
        "evidence_classes":{"kind":"explicit","classes":["a-report","z-report"],"source":null},"subject_source":"bindings",
        "context":{"required":false,"source":null},"candidate_source":"secondary","target":null})
    );
}
