use serde_json::json;

use super::git_governance::GitGovernance;

pub const BIG: &str = "-12345678901234567890123456789012345678901234567890";

pub fn rich() -> GitGovernance {
    let fixture = GitGovernance::full();
    let mut model = fixture.read("AGENTS.md");
    model["repository_governance"]["capabilities"]["repository_integrity"]["configuration"] = json!({
        "":true,"big":"BIG-INTEGER-SENTINEL","nested":[null,false,"",{"":"opaque"}]
    });
    fixture.write("AGENTS.md", model);
    let path = fixture.root.join("AGENTS.md");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path, text.replace("\"BIG-INTEGER-SENTINEL\"", BIG)).unwrap();
    let mut authority = fixture.read("authority.md");
    for (source, carrier, role) in [
        ("secondary", "secondary.md", "secondary_representation"),
        ("generator", "generator.sh", "non_authoritative"),
    ] {
        authority["governance_authority"]["sources"][source] = json!({"repository_target":carrier});
        authority["governance_authority"]["responsibilities"]["quality"]["roles"][source] =
            json!(role);
    }
    authority["governance_authority"]["sources"]["external"] = json!({});
    authority["governance_authority"]["responsibilities"]["quality"]["roles"]["external"] =
        json!("authority");
    authority["governance_authority"]["responsibilities"]["quality"]["precedence"] = json!([
        {"higher_source":"bindings","lower_source":"integrity"}
    ]);
    fixture.write("authority.md", authority);
    std::fs::write(fixture.root.join("secondary.md"), "# Declared projection\n").unwrap();
    std::fs::write(
        fixture.root.join("generator.sh"),
        "#!/bin/sh\nprintf unexpected > EFFECT-MARKER\n",
    )
    .unwrap();
    let mut objects = fixture.read("objects.md");
    objects["governed_objects"]["interfaces"]["repository"]["objects"]["tree"]["relations"] = json!([
        {"relation":"supports","responsibility":"quality","target":{"interface":"repository","object":"tree"}}
    ]);
    fixture.write("objects.md", objects);
    let mut bindings = fixture.read("bindings.md");
    for (id, scope) in [
        (
            "cap-contract",
            json!({"kind":"capability","capability":"repository_integrity"}),
        ),
        (
            "object-contract",
            json!({"kind":"governed_object","interface":"repository","object":"tree"}),
        ),
    ] {
        bindings["governance_bindings"]["bindings"][id] = json!({
            "kind":"governance_contract","scope":scope,
            "identity":{"repository":"example/contracts","commit":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","path":"contract.md"},
            "authority":{"responsibility":"quality","source":"bindings"}
        });
    }
    fixture.write("bindings.md", bindings);
    let mut integrity = fixture.read("integrity.md");
    integrity["repository_integrity"]["environments"] = json!(["z-env", "declared-only"]);
    integrity["repository_integrity"]["continue_after_non_satisfied"] = json!(true);
    let first = &mut integrity["repository_integrity"]["validations"]["first"];
    first["prerequisites"] = json!(["second"]);
    first["instances"] = json!({"kind":"repository_paths","mode":"for_each","selectors":[
        {"kind":"glob","glob":"*.md"},{"kind":"path","path":"README.md"}
    ]});
    first["command"]["arguments"] = json!(["", "-c", "printf unexpected > EFFECT-MARKER"]);
    first["command"]["undetermined_exit_codes"] = json!([127, -1, 5]);
    first["target"] = json!({"interface":"repository","object":"tree"});
    fixture.write("integrity.md", integrity);
    let mut registry = fixture.read("projections.md");
    for (id, mode, extra) in [
        ("reference", "reference", json!({})),
        (
            "generated",
            "generated",
            json!({"generator_source":"generator","binding":"provider","target":{"interface":"repository","object":"tree"}}),
        ),
        (
            "maintained",
            "mechanically_validated_maintained",
            json!({"target":{"interface":"repository","object":"tree"}}),
        ),
        (
            "historical",
            "bounded_historical_snapshot",
            json!({"boundary_source":"bindings"}),
        ),
    ] {
        let mut declaration = json!({"responsibility":"quality","canonical_source":"bindings",
            "secondary_source":"secondary","mode":mode,"validation":"first"});
        declaration
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        registry["projection_registry"]["projections"][id] = declaration;
    }
    fixture.write("projections.md", registry);
    let mut evidence = fixture.read("evidence.md");
    evidence["evidence_requirements"]["requirements"] = json!({
        "explicit":{"responsibility":"quality","instances":{"kind":"single"},
            "evidence_classes":{"kind":"explicit","classes":["z-report","a-report"]},
            "subject":{"source":"bindings"},"context":{"required":false},"candidates":{"source":"secondary"}},
        "sourced":{"responsibility":"quality","instances":{"kind":"source","source":"bindings"},
            "evidence_classes":{"kind":"source","source":"external"},
            "subject":{"source":"bindings"},"context":{"required":true,"source":"external"},
            "candidates":{"source":"secondary"},"target":{"interface":"repository","object":"tree"}}
    });
    fixture.write("evidence.md", evidence);
    fixture
}
