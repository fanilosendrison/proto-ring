mod rgs_projection_support;

use serde_json::{Value, json};

use rgs_projection_support::{complete_fixture, compose, optional_fixture, snapshot};

fn route(capability: &str, name: &str, target: &str) -> Value {
    json!({
        "route":["capabilities",capability,"routes",name],
        "declared_path":target,
        "resolved_target":target
    })
}

fn repository_target(source: &str, target: &str) -> Value {
    json!({
        "route":[source,"repository_target"],
        "declared_path":target,
        "resolved_target":target
    })
}

fn binding(kind: &str, scope: Value, identity: Value, responsibility: &str, source: &str) -> Value {
    json!({
        "present":true,
        "kind":kind,
        "scope":scope,
        "identity":identity,
        "authority":{"responsibility":responsibility,"source":source}
    })
}

#[test]
fn complete_projection_is_exact_and_read_only() {
    let fixture = complete_fixture();
    let before = snapshot(fixture.path());
    let (output, response) = compose(fixture.path(), Some("complete"));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(response["outcome"], "result");
    assert_eq!(response["value"]["state"], expected_state());
    assert_eq!(snapshot(fixture.path()), before);
    assert!(!fixture.path().join("marker-must-not-exist").exists());
}

#[test]
fn absent_optional_components_are_null_not_omitted() {
    let fixture = optional_fixture();
    let (_, response) = compose(fixture.path(), Some("complete"));
    let state = &response["value"]["state"];
    for name in [
        "governed_objects",
        "repository_integrity",
        "projection_registry",
        "evidence_requirements",
    ] {
        assert_eq!(state[name], Value::Null, "{name}");
    }
    for name in [
        "repository_governance_model",
        "governance_authority",
        "governance_bindings",
    ] {
        assert!(!state[name].is_null(), "{name}");
    }
}

fn expected_state() -> Value {
    json!({
        "repository_governance_model":{
            "model_version":2,
            "provider":{"id":"proto-ring","binding":{"capability":"shared_governance_provider","route":"registry"}},
            "capabilities":{
                "architecture_decisions":{
                    "configuration":{
                        "enabled":{"type":"boolean","value":true},
                        "policy":{"type":"record","value":[{"name":"strict","value":{"type":"boolean","value":true}}]}
                    },
                    "routes":{
                        "history":route("architecture_decisions","history","adr-history.md"),
                        "profile":route("architecture_decisions","profile","adr-profile.yaml")
                    }
                },
                "evidence_requirements":{"configuration":{},"routes":{"registry":route("evidence_requirements","registry","evidence.md")}},
                "governance_authority":{"configuration":{},"routes":{"profile":route("governance_authority","profile","authority.md")}},
                "governed_objects":{"configuration":{},"routes":{"profile":route("governed_objects","profile","objects.md")}},
                "projection_integrity":{"configuration":{},"routes":{"registry":route("projection_integrity","registry","projections.md")}},
                "repository_integrity":{"configuration":{},"routes":{"profile":route("repository_integrity","profile","integrity.md")}},
                "shared_governance_provider":{
                    "configuration":{"required":{"type":"boolean","value":true}},
                    "routes":{"registry":route("shared_governance_provider","registry","bindings.md")}
                }
            }
        },
        "governance_authority":{
            "model_version":1,
            "sources":{
                "authority":{"repository_target":repository_target("authority","canonical.md")},
                "bindings":{"repository_target":repository_target("bindings","bindings.md")},
                "boundary":{"repository_target":repository_target("boundary","boundary.md")},
                "candidates":{"repository_target":repository_target("candidates","candidates.md")},
                "evidence":{"repository_target":repository_target("evidence","evidence.md")},
                "generator":{"repository_target":repository_target("generator","generator.md")},
                "integrity":{"repository_target":repository_target("integrity","integrity.md")},
                "projections":{"repository_target":repository_target("projections","projections.md")},
                "secondary":{"repository_target":repository_target("secondary","secondary.md")},
                "subject":{"repository_target":repository_target("subject","subject.md")}
            },
            "responsibilities":{
                "evidence-registry":{"roles":{"evidence":"authority"},"precedence":[]},
                "evidence-required":{"roles":{"authority":"authority","candidates":"non_authoritative","subject":"non_authoritative"},"precedence":[]},
                "integrity-profile":{"roles":{"integrity":"authority"},"precedence":[]},
                "object-governance":{
                    "roles":{"authority":"authority","bindings":"secondary_representation","boundary":"authority","generator":"non_authoritative","secondary":"secondary_representation"},
                    "precedence":[{"higher_source":"authority","lower_source":"boundary"}]
                },
                "projection-registry":{"roles":{"projections":"authority"},"precedence":[]},
                "provider-binding":{"roles":{"bindings":"authority"},"precedence":[]},
                "repository-validation":{"roles":{"integrity":"authority"},"precedence":[]}
            }
        },
        "governed_objects":{
            "model_version":1,
            "interfaces":{"governance":{"objects":{
                "object-a":{
                    "responsibilities":["evidence-required","object-governance"],
                    "relations":[{"relation":"depends_on","responsibility":"object-governance","target":{"interface":"governance","object":"object-b"}}]
                },
                "object-b":{"responsibilities":["object-governance"],"relations":[]}
            }}}
        },
        "governance_bindings":{
            "model_version":1,
            "source":"bindings",
            "bindings":{
                "contract":binding("governance_contract",json!({"kind":"capability","capability":"governed_objects","interface":null,"object":null}),json!({"kind":"governance_contract","repository":"example/contracts","commit":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","path":"docs/contracts/governed-objects.md"}),"object-governance","authority"),
                "provider":binding("executable_provider",json!({"kind":"logical_provider","capability":null,"interface":null,"object":null}),json!({"kind":"executable_provider","repository":"example/provider","commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}),"provider-binding","bindings")
            }
        },
        "repository_integrity":{
            "model_version":1,
            "authority":{"responsibility":"integrity-profile","source":"integrity"},
            "environments":["native","python"],
            "continue_after_non_satisfied":true,
            "identity_present":true,
            "validations":{
                "validate-paths":{
                    "id":"validate-paths","responsibility":"object-governance","prerequisites":["validate-profile"],
                    "instances":{"kind":"repository_paths","mode":"append_all","selectors":[{"kind":"path","value":"tracked.txt"},{"kind":"glob","value":"*.md"}]},
                    "command":{"environment":"python","arguments":["marker-must-not-exist"],"undetermined_exit_codes":["7","9"]},
                    "target":{"interface":"governance","object":"object-a"}
                },
                "validate-profile":{
                    "id":"validate-profile","responsibility":"repository-validation","prerequisites":[],
                    "instances":{"kind":"single","mode":null,"selectors":[]},
                    "command":{"environment":"native","arguments":["check","profile"],"undetermined_exit_codes":[]},
                    "target":null
                }
            },
            "order":["validate-profile","validate-paths"]
        },
        "projection_registry":{
            "model_version":1,
            "authority":{"responsibility":"projection-registry","source":"projections"},
            "projections":{
                "object-projection":{
                    "id":"object-projection","responsibility":"object-governance","canonical_source":"authority","secondary_source":"secondary","mode":"generated","validation":"validate-paths","generator_source":"generator","boundary_source":null,
                    "target":{"interface":"governance","object":"object-a"},"binding":"contract"
                }
            }
        },
        "evidence_requirements":{
            "authority_source":"evidence","model_version":1,
            "registry_authority":{"responsibility":"evidence-registry","source":"evidence"},
            "requirement_ids":["explicit","sourced"],
            "requirements":{
                "explicit":{"id":"explicit","responsibility":"evidence-required","instances":{"kind":"single","source":null},"evidence_classes":{"kind":"explicit","classes":["report","trace"],"source":null},"subject_source":"subject","context":{"required":false,"source":null},"candidate_source":"candidates","target":{"interface":"governance","object":"object-a"}},
                "sourced":{"id":"sourced","responsibility":"evidence-required","instances":{"kind":"source","source":"subject"},"evidence_classes":{"kind":"source","classes":null,"source":"candidates"},"subject_source":"subject","context":{"required":true,"source":"subject"},"candidate_source":"candidates","target":null}
            }
        }
    })
}
