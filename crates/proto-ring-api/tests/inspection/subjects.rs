use proto_ring_api::{ErrorCode, InspectSelector, open};
use serde_json::json;

use super::{git_governance::GitGovernance, inspect, inspection_fixture};

#[test]
fn all_four_targeted_views_share_complete_declarations_and_full_profile_reference() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let all = inspect(&observed, json!({"kind":"all"}));
    for (kind, id, component, collection) in [
        (
            "capability",
            "repository_integrity",
            "model",
            "capabilities",
        ),
        ("validation", "first", "repository_integrity", "validations"),
        ("binding", "cap-contract", "bindings", "bindings"),
    ] {
        let result = inspect(&observed, json!({"kind":kind,"id":id}));
        assert_eq!(result["result_kind"], "subject");
        assert_eq!(result["subject"]["kind"], kind);
        assert_eq!(result["subject"]["id"], id);
        assert_eq!(
            result["subject"]["declaration"],
            all["components"][component]["data"][collection][id]
        );
        if kind == "validation" {
            assert_eq!(
                result["subject"]["profile_ref"],
                all["components"]["repository_integrity"]["data"]["profile_ref"]
            );
        }
        if kind == "binding" {
            assert_eq!(result["subject"]["registry_source"], "bindings");
        }
        if kind == "capability" {
            assert_eq!(result["subject"]["model_version"], 2);
        }
    }
    let target = json!({"interface":"repository","object":"tree"});
    let result = inspect(&observed, json!({"kind":"governed_object","target":target}));
    assert_eq!(result["subject"]["target"], target);
    assert_eq!(
        result["subject"]["declaration"],
        all["components"]["governed_objects"]["data"]["interfaces"]["repository"]["objects"]["tree"]
    );
}

#[test]
fn absent_optional_owners_are_inapplicable_not_missing_subjects() {
    let fixture = GitGovernance::minimal();
    let observed = open(&fixture.root).unwrap();
    let all = inspect(&observed, json!({"kind":"all"}));
    for component in [
        "governed_objects",
        "repository_integrity",
        "projections",
        "evidence_requirements",
    ] {
        assert_eq!(
            all["components"][component],
            json!({"presence":"absent","data":null})
        );
    }
    for selector in [
        json!({"kind":"governed_objects"}),
        json!({"kind":"repository_integrity"}),
        json!({"kind":"projections"}),
        json!({"kind":"evidence_requirements"}),
        json!({"kind":"validation","id":"unknown"}),
        json!({"kind":"governed_object","target":{"interface":"missing","object":"missing"}}),
    ] {
        let error = observed
            .inspect(serde_json::from_value(selector).unwrap())
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::ComponentNotDeclared);
        assert_eq!(error.stage, None);
        assert_eq!(error.effects_possible, Some(false));
    }
    assert!(observed.inspect(InspectSelector::Bindings {}).is_ok());
}

#[test]
fn unknown_identifiers_in_present_owners_are_not_found() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    for selector in [
        json!({"kind":"capability","id":"unknown"}),
        json!({"kind":"validation","id":"unknown"}),
        json!({"kind":"binding","id":"unknown"}),
        json!({"kind":"governed_object","target":{"interface":"repository","object":"unknown"}}),
        json!({"kind":"governed_object","target":{"interface":"unknown","object":"tree"}}),
    ] {
        let error = observed
            .inspect(serde_json::from_value(selector).unwrap())
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::SubjectNotFound);
        assert_eq!(error.stage, None);
        assert_eq!(error.attempt_id, None);
    }
}
