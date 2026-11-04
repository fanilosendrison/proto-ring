use proto_ring_api::{ErrorCode, ExplainQuery, InspectSelector, open};
use serde_json::{Value, json};

use super::{explain, explanation_fixture, git_governance::GitGovernance, path};

#[test]
fn all_six_declaration_families_match_complete_t4_views_and_exact_subject_queries() {
    let fixture = explanation_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let all = serde_json::to_value(observed.inspect(InspectSelector::All {}).unwrap()).unwrap();
    for (kind, component, collection, ids) in [
        (
            "capability",
            "model",
            "capabilities",
            vec!["repository_integrity", "shared_governance_provider"],
        ),
        (
            "validation",
            "repository_integrity",
            "validations",
            vec!["first", "second"],
        ),
        (
            "binding",
            "bindings",
            "bindings",
            vec!["provider", "cap-contract", "object-contract"],
        ),
        (
            "projection",
            "projections",
            "projections",
            vec!["reference", "generated", "historical", "maintained"],
        ),
        (
            "evidence_requirement",
            "evidence_requirements",
            "requirements",
            vec!["explicit", "sourced"],
        ),
    ] {
        for id in ids {
            let query = json!({"kind":kind,"id":id});
            let value = explain(&observed, query.clone());
            let explanation = &value["explanation"];
            let subject = &explanation["declaration"];
            assert_eq!(explanation["kind"], "declaration");
            assert_eq!(explanation["subject"], query);
            assert_eq!(subject["kind"], kind);
            assert_eq!(subject["id"], id);
            assert_eq!(
                subject["declaration"],
                all["components"][component]["data"][collection][id]
            );
            if matches!(kind, "capability" | "validation" | "binding") {
                let inspected = serde_json::to_value(
                    observed
                        .inspect(serde_json::from_value(query).unwrap())
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(subject, &inspected["subject"]);
            } else {
                let registry = &all["components"][component]["data"];
                assert_eq!(subject["registry_model_version"], registry["model_version"]);
                assert_eq!(subject["registry_authority"], registry["authority"]);
                assert_eq!(subject["carrier"], registry["carrier"]);
            }
            if kind == "validation" {
                assert_eq!(
                    subject["profile_ref"],
                    serde_json::to_value(observed.profile_reference().unwrap()).unwrap()
                );
            }
        }
    }
    let target = json!({"interface":"repository","object":"tree"});
    let query = json!({"kind":"governed_object","target":target});
    let value = explain(&observed, query.clone());
    assert_eq!(value["explanation"]["subject"], query);
    let subject = &value["explanation"]["declaration"];
    assert_eq!(subject["kind"], "governed_object");
    assert_eq!(subject["target"], target);
    assert_eq!(
        subject["declaration"],
        all["components"]["governed_objects"]["data"]["interfaces"]["repository"]["objects"]["tree"]
    );
    let inspected = serde_json::to_value(
        observed
            .inspect(serde_json::from_value(query).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(subject, &inspected["subject"]);
}

#[test]
fn registry_metadata_and_complete_declarations_match_independent_expected_facts() {
    let fixture = explanation_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    for (kind, id, source, carrier, declaration) in [
        (
            "projection",
            "generated",
            "projections",
            "projections.md",
            json!({
                "responsibility":"quality","canonical_source":"bindings","secondary_source":"secondary","mode":"generated","validation":"first",
                "generator_source":"generator","boundary_source":null,"target":{"interface":"repository","object":"tree"},"binding":"provider"
            }),
        ),
        (
            "projection",
            "reference",
            "projections",
            "projections.md",
            json!({
                "responsibility":"quality","canonical_source":"bindings","secondary_source":"secondary","mode":"reference","validation":"first",
                "generator_source":null,"boundary_source":null,"target":null,"binding":null
            }),
        ),
        (
            "evidence_requirement",
            "explicit",
            "evidence",
            "evidence.md",
            json!({
                "responsibility":"quality","instances":{"kind":"single","source":null},"evidence_classes":{"kind":"explicit","classes":["a-report","z-report"],"source":null},
                "subject_source":"bindings","context":{"required":false,"source":null},"candidate_source":"secondary","target":null
            }),
        ),
        (
            "evidence_requirement",
            "sourced",
            "evidence",
            "evidence.md",
            json!({
                "responsibility":"quality","instances":{"kind":"source","source":"bindings"},"evidence_classes":{"kind":"source","classes":[],"source":"external"},
                "subject_source":"bindings","context":{"required":true,"source":"external"},"candidate_source":"secondary","target":{"interface":"repository","object":"tree"}
            }),
        ),
    ] {
        let value = explain(&observed, json!({"kind":kind,"id":id}));
        assert_eq!(
            value["explanation"]["declaration"],
            json!({"kind":kind,"id":id,"registry_model_version":1,
            "registry_authority":{"responsibility":"registry-owner","source":source},"carrier":path(&fixture,carrier),"declaration":declaration})
        );
    }
}

fn assert_error(observed: &proto_ring_api::GovernanceObservation, query: Value, code: ErrorCode) {
    let query: ExplainQuery = serde_json::from_value(query).unwrap();
    let error = observed.explain(query).unwrap_err();
    assert_eq!(error.code, code);
    assert_eq!(error.stage, None);
    assert_eq!(error.attempt_id, None);
    assert_eq!(error.effects_possible, Some(false));
}

#[test]
fn optional_owner_absence_is_not_a_missing_subject() {
    let fixture = GitGovernance::minimal();
    let observed = open(&fixture.root).unwrap();
    for query in [
        json!({"kind":"validation","id":"first"}),
        json!({"kind":"governed_object","target":{"interface":"repository","object":"tree"}}),
        json!({"kind":"projection","id":"generated"}),
        json!({"kind":"evidence_requirement","id":"explicit"}),
    ] {
        assert_error(&observed, query, ErrorCode::ComponentNotDeclared);
    }
    explain(&observed, json!({"kind":"binding","id":"provider"}));
}

#[test]
fn missing_ids_and_each_missing_target_dimension_in_present_owners_are_not_found() {
    let fixture = explanation_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    for kind in [
        "capability",
        "validation",
        "binding",
        "projection",
        "evidence_requirement",
    ] {
        assert_error(
            &observed,
            json!({"kind":kind,"id":"missing"}),
            ErrorCode::SubjectNotFound,
        );
    }
    for (interface, object) in [("repository", "missing"), ("missing", "tree")] {
        assert_error(
            &observed,
            json!({"kind":"governed_object","target":{"interface":interface,"object":object}}),
            ErrorCode::SubjectNotFound,
        );
    }
}

#[test]
fn declared_empty_registries_report_missing_subjects_not_absent_components() {
    let fixture = GitGovernance::full();
    let observed = open(&fixture.root).unwrap();
    for kind in ["projection", "evidence_requirement"] {
        assert_error(
            &observed,
            json!({"kind":kind,"id":"missing"}),
            ErrorCode::SubjectNotFound,
        );
    }
}

#[test]
fn opaque_registry_identifier_is_echoed_exactly_without_case_trim_or_unicode_normalization() {
    let fixture = explanation_fixture::rich();
    let mut profile = fixture.read("projections.md");
    let id = " Mêta / e\u{301} ";
    let declaration = profile["projection_registry"]["projections"]
        .as_object_mut()
        .unwrap()
        .remove("generated")
        .unwrap();
    profile["projection_registry"]["projections"][id] = declaration;
    fixture.write("projections.md", profile);
    let observed = open(&fixture.root).unwrap();
    let value = explain(&observed, json!({"kind":"projection","id":id}));
    assert_eq!(value["explanation"]["declaration"]["id"], id);
    for different in [
        id.trim().to_owned(),
        id.to_uppercase(),
        " Mêta / é ".to_owned(),
    ] {
        assert_error(
            &observed,
            json!({"kind":"projection","id":different}),
            ErrorCode::SubjectNotFound,
        );
    }
}
