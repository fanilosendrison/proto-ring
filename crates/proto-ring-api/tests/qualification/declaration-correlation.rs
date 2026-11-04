use super::inspection_fixture;
use proto_ring_api::{ExplanationResult, open};
use serde_json::json;

#[test]
fn every_real_declaration_family_rejects_outer_inner_and_subject_identity_mismatches() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    for query in [
        json!({"kind":"capability","id":"repository_integrity"}),
        json!({"kind":"validation","id":"first"}),
        json!({"kind":"binding","id":"provider"}),
        json!({"kind":"governed_object","target":{"interface":"repository","object":"tree"}}),
        json!({"kind":"projection","id":"generated"}),
        json!({"kind":"evidence_requirement","id":"sourced"}),
    ] {
        let original = serde_json::to_value(
            observed
                .explain(serde_json::from_value(query.clone()).unwrap())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(original["query"], query);
        let fields = if query["kind"] == "governed_object" {
            vec!["target/interface", "target/object"]
        } else {
            vec!["id"]
        };
        for owner in ["/query", "/explanation/subject", "/explanation/declaration"] {
            for field in &fields {
                let mut mismatched = original.clone();
                *mismatched.pointer_mut(&format!("{owner}/{field}")).unwrap() = json!("different");
                assert!(
                    serde_json::from_value::<ExplanationResult>(mismatched).is_err(),
                    "{query} {owner}/{field}"
                );
            }
        }
    }
}
