use proto_ring_api::open;
use serde_json::{Value, json};

use super::{inspect, inspection_fixture, path};

// Independent test oracle over the fixture's declared input. It does not call
// production projectors or use illustrative API examples as engine evidence.
fn tagged(value: &Value) -> Value {
    match value {
        Value::Null => json!({"kind":"null"}),
        Value::Bool(value) => json!({"kind":"boolean","value":value}),
        Value::Number(value) => json!({"kind":"integer","decimal":value.to_string()}),
        Value::String(value) if value == "BIG-INTEGER-SENTINEL" => {
            json!({"kind":"integer","decimal":inspection_fixture::BIG})
        }
        Value::String(value) => json!({"kind":"string","value":value}),
        Value::Array(values) => {
            json!({"kind":"sequence","items":values.iter().map(tagged).collect::<Vec<_>>()})
        }
        Value::Object(values) => {
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_by_key(|(key, _)| *key);
            json!({"kind":"mapping","entries":entries.into_iter().map(|(key,value)|json!({"key":key,"value":tagged(value)})).collect::<Vec<_>>()})
        }
    }
}

#[test]
fn entire_bootstrap_and_model_match_independent_declared_input_oracle() {
    let fixture = inspection_fixture::rich();
    let text = std::fs::read_to_string(fixture.root.join("AGENTS.md")).unwrap();
    let exact = text.replace(inspection_fixture::BIG, "\"BIG-INTEGER-SENTINEL\"");
    let metadata: Value = serde_json::from_str(exact.split("---").nth(1).unwrap()).unwrap();
    let observed = open(&fixture.root).unwrap();
    let all = inspect(&observed, json!({"kind":"all"}));
    assert_eq!(
        all["components"]["bootstrap"]["data"],
        json!({"metadata":tagged(&metadata),
        "repository_governance":tagged(&metadata["repository_governance"]),"carrier":path(&fixture,"AGENTS.md")})
    );
    let declared = &metadata["repository_governance"];
    let mut capabilities = serde_json::Map::new();
    for (id, capability) in declared["capabilities"].as_object().unwrap() {
        let configuration: serde_json::Map<_, _> = capability["configuration"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), tagged(value)))
            .collect();
        let routes: serde_json::Map<_,_> = capability["routes"].as_object().unwrap().iter().map(|(key,value)|(
            key.clone(),json!({"segments":["capabilities",id,"routes",key],"declared_path":value,"resolved_target":path(&fixture,value.as_str().unwrap())})
        )).collect();
        capabilities.insert(
            id.clone(),
            json!({"configuration":configuration,"routes":routes}),
        );
    }
    assert_eq!(
        all["components"]["model"]["data"],
        json!({"model_version":2,"provider":declared["provider"],"capabilities":capabilities})
    );
}

#[test]
fn complete_authority_objects_and_bindings_match_all_declared_entries() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let all = inspect(&observed, json!({"kind":"all"}));
    let declared = fixture.read("authority.md")["governance_authority"].clone();
    let sources: serde_json::Map<_,_> = declared["sources"].as_object().unwrap().iter().map(|(id,value)| {
        let route = value.get("repository_target").map_or(Value::Null,|target|json!({
            "segments":[id,"repository_target"],"declared_path":target,"resolved_target":path(&fixture,target.as_str().unwrap())
        }));
        (id.clone(),json!({"repository_target":route}))
    }).collect();
    assert_eq!(
        all["components"]["authority"]["data"],
        json!({"model_version":1,"sources":sources,
        "responsibilities":declared["responsibilities"],"carrier":path(&fixture,"authority.md")})
    );
    let declared = fixture.read("objects.md")["governed_objects"].clone();
    assert_eq!(
        all["components"]["governed_objects"]["data"],
        json!({"model_version":1,"interfaces":declared["interfaces"],"carrier":path(&fixture,"objects.md")})
    );
    let declared = fixture.read("bindings.md")["governance_bindings"].clone();
    let bindings: serde_json::Map<_, _> = declared["bindings"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(id, value)| {
            let mut expected = value.clone();
            expected["identity"]["kind"] = value["kind"].clone();
            if value["scope"]["kind"] == "governed_object" {
                expected["scope"] = json!({"kind":"governed_object","target":{
                    "interface":value["scope"]["interface"],"object":value["scope"]["object"]
                }});
            }
            (id.clone(), expected)
        })
        .collect();
    assert_eq!(
        all["components"]["bindings"]["data"],
        json!({"model_version":1,"source":declared["source"],"bindings":bindings,"carrier":path(&fixture,"bindings.md")})
    );
}

#[test]
fn complete_ri_and_projection_registry_match_every_declared_field() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let all = inspect(&observed, json!({"kind":"all"}));
    let mut expected = fixture.read("integrity.md")["repository_integrity"].clone();
    expected["carrier"] = path(&fixture, "integrity.md");
    expected["profile_ref"] = serde_json::to_value(observed.profile_reference().unwrap()).unwrap();
    expected["environments"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    for value in expected["validations"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        value["command"].as_object_mut().unwrap().remove("kind");
        let mut codes: Vec<_> = value["command"]["undetermined_exit_codes"]
            .as_array()
            .unwrap()
            .iter()
            .map(Value::to_string)
            .collect();
        codes.sort();
        value["command"]["undetermined_exit_codes"] = json!(codes);
        value
            .as_object_mut()
            .unwrap()
            .entry("target")
            .or_insert(Value::Null);
        if let Some(selectors) = value["instances"].get_mut("selectors") {
            for selector in selectors.as_array_mut().unwrap() {
                let key = selector["kind"].as_str().unwrap().to_owned();
                selector["value"] = selector[&key].clone();
                selector.as_object_mut().unwrap().remove(&key);
            }
        }
    }
    assert_eq!(all["components"]["repository_integrity"]["data"], expected);
    let mut expected = fixture.read("projections.md")["projection_registry"].clone();
    expected["carrier"] = path(&fixture, "projections.md");
    for value in expected["projections"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        for key in ["generator_source", "boundary_source", "target", "binding"] {
            value
                .as_object_mut()
                .unwrap()
                .entry(key)
                .or_insert(Value::Null);
        }
    }
    assert_eq!(all["components"]["projections"]["data"], expected);
}
