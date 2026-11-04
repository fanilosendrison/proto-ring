use serde_json::{Value, json};

use super::{git_governance::GitGovernance, inspection_fixture};

pub fn rich() -> GitGovernance {
    let fixture = inspection_fixture::rich();
    let mut profile = fixture.read("authority.md");
    let authority = &mut profile["governance_authority"];
    authority["sources"]["bottom"] = json!({});
    authority["responsibilities"]["quality"]["roles"]["bottom"] = json!("authority");
    authority["responsibilities"]["quality"]["precedence"]
        .as_array_mut()
        .unwrap()
        .push(json!({"higher_source":"integrity","lower_source":"bottom"}));
    authority["responsibilities"]["different"] =
        json!({"roles":{"bindings":"non_authoritative"},"precedence":[]});
    authority["responsibilities"]["unassigned"] = json!({"roles":{},"precedence":[]});
    authority["responsibilities"]["registry-owner"] =
        json!({"roles":{"projections":"authority","evidence":"authority"},"precedence":[]});
    fixture.write("authority.md", profile);
    for (carrier, registry) in [
        ("projections.md", "projection_registry"),
        ("evidence.md", "evidence_requirements"),
    ] {
        let mut profile = fixture.read(carrier);
        profile[registry]["authority"]["responsibility"] = json!("registry-owner");
        fixture.write(carrier, profile);
    }
    fixture
}

pub fn all_queries() -> Vec<Value> {
    vec![
        json!({"kind":"authority_role","responsibility":"quality","source":"external"}),
        json!({"kind":"authority_precedence","responsibility":"quality","higher_source":"bindings","lower_source":"bottom"}),
        json!({"kind":"capability","id":"repository_integrity"}),
        json!({"kind":"validation","id":"first"}),
        json!({"kind":"binding","id":"provider"}),
        json!({"kind":"governed_object","target":{"interface":"repository","object":"tree"}}),
        json!({"kind":"projection","id":"generated"}),
        json!({"kind":"evidence_requirement","id":"sourced"}),
    ]
}
