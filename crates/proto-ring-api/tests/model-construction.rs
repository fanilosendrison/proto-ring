#[path = "support/wire-oracle.rs"]
mod wire_oracle;

use proto_ring_api::{
    ApiError, ComponentProjection, Components, ErrorCode, Explanation, ExplanationResult,
    FailureOutcome, FailureResponse, GovernanceStage, Inspection, InspectionResult, ModelVersion,
    ObservationProvenance, Operation, Presence, Response, SubjectProjection, VersionOne,
};
use serde_json::{Value, json};
use wire_oracle::{example, round_trip};

fn provenance() -> ObservationProvenance {
    let value = example("response-inspect-all.json")["value"].clone();
    serde_json::from_value(json!({
        "observation":value["observation"], "repository":value["repository"],
        "scope_paths":value["scope_paths"],
    }))
    .unwrap()
}

#[test]
fn each_complete_component_can_be_constructed_without_engine_imports() {
    let components: Components =
        serde_json::from_value(example("response-inspect-all.json")["value"]["components"].clone())
            .unwrap();
    let Presence::Present(objects) = components.governed_objects else {
        panic!("fixture")
    };
    let Presence::Present(integrity) = components.repository_integrity else {
        panic!("fixture")
    };
    let Presence::Present(projections) = components.projections else {
        panic!("fixture")
    };
    let Presence::Present(evidence) = components.evidence_requirements else {
        panic!("fixture")
    };
    for component in [
        ComponentProjection::Bootstrap(components.bootstrap.data),
        ComponentProjection::Model(components.model.data),
        ComponentProjection::Authority(components.authority.data),
        ComponentProjection::GovernedObjects(objects),
        ComponentProjection::Bindings(components.bindings.data),
        ComponentProjection::RepositoryIntegrity(integrity),
        ComponentProjection::Projections(projections),
        ComponentProjection::EvidenceRequirements(evidence),
    ] {
        let result = Inspection::new(
            provenance(),
            InspectionResult::Component(Box::new(component)),
        )
        .unwrap();
        round_trip::<Inspection>(&serde_json::to_value(result).unwrap(), "inspect_value");
    }
}

#[test]
fn optional_absence_preserves_every_key_and_explicit_null_data() {
    let mut components = example("response-inspect-all.json")["value"]["components"].clone();
    for name in [
        "governed_objects",
        "repository_integrity",
        "projections",
        "evidence_requirements",
    ] {
        components[name] = json!({"presence":"absent","data":null});
    }
    round_trip::<Components>(&components, "components");
}

#[test]
fn capability_declaration_type_is_shared_across_all_three_views() {
    let components: Components =
        serde_json::from_value(example("response-inspect-all.json")["value"]["components"].clone())
            .unwrap();
    let id = "repository_integrity".try_into().unwrap();
    let declaration = components.model.data.capabilities.get(&id).unwrap().clone();
    let subject = SubjectProjection::Capability {
        id,
        model_version: ModelVersion::V2,
        declaration,
    };
    let inspection = Inspection::new(
        provenance(),
        InspectionResult::Subject(Box::new(subject.clone())),
    )
    .unwrap();
    let explanation =
        ExplanationResult::new(provenance(), Explanation::Declaration(Box::new(subject)));
    round_trip::<Inspection>(&serde_json::to_value(inspection).unwrap(), "inspect_value");
    round_trip::<ExplanationResult>(&serde_json::to_value(explanation).unwrap(), "explain_value");
}

#[test]
fn declaration_only_explanation_subjects_are_not_inspection_selectors() {
    for name in [
        "response-explain-projection.json",
        "response-explain-evidence-requirement.json",
    ] {
        let subject =
            serde_json::from_value(example(name)["value"]["explanation"]["declaration"].clone())
                .unwrap();
        assert!(
            Inspection::new(provenance(), InspectionResult::Subject(Box::new(subject))).is_err()
        );
    }
}

#[test]
fn role_and_precedence_results_preserve_each_distinct_consultation() {
    for role in [
        "authority",
        "secondary_representation",
        "non_authoritative",
        "UNKNOWN_RESPONSIBILITY",
        "UNDECLARED",
    ] {
        let mut value = example("response-explain-role.json")["value"].clone();
        value["explanation"]["role"] = json!(role);
        round_trip::<ExplanationResult>(&value, "explain_value");
    }
    for relation in ["OUTRANKS", "DOES_NOT_OUTRANK", "UNKNOWN_RESPONSIBILITY"] {
        let mut value = example("response-explain-role.json")["value"].clone();
        value["query"] = json!({"kind":"authority_precedence","responsibility":"r","higher_source":"a","lower_source":"b"});
        value["explanation"] = value["query"].clone();
        value["explanation"]["relation"] = json!(relation);
        round_trip::<ExplanationResult>(&value, "explain_value");
    }
}

#[test]
fn every_governance_stage_serializes_in_a_load_failure_without_text_classification() {
    for stage in [
        GovernanceStage::RepositoryState,
        GovernanceStage::Bootstrap,
        GovernanceStage::RepositoryGovernanceModel,
        GovernanceStage::GovernanceAuthority,
        GovernanceStage::GovernedObjects,
        GovernanceStage::GovernanceBindings,
        GovernanceStage::RepositoryIntegrity,
        GovernanceStage::ProjectionIntegrity,
        GovernanceStage::EvidenceRequirements,
        GovernanceStage::Composition,
    ] {
        let failure = FailureResponse {
            api_version: VersionOne::V1,
            operation: Some(Operation::Inspect),
            outcome: FailureOutcome::ControlledFailure,
            error: ApiError {
                code: ErrorCode::GovernanceLoadFailed,
                stage: Some(stage),
                detail: None,
                attempt_id: None,
                effects_possible: Some(false),
            },
        };
        round_trip::<Response>(&serde_json::to_value(failure).unwrap(), "response");
    }
}

#[test]
fn all_canonical_failure_fixtures_are_lossless_typed_diagnostics() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/api/v1/examples");
    let mut count = 0;
    for entry in std::fs::read_dir(directory).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        if name.starts_with("response-error-") {
            round_trip::<Response>(&example(&name), "response");
            count += 1;
        }
    }
    // receipt_persistence_failed has distinct pre-effect and post-effect fixtures.
    assert_eq!(count, 25, "canonical error fixture inventory");
}

#[test]
fn api03_integration_failure_shape_has_no_fake_receipt_or_effects() {
    for operation in [
        Operation::CheckRepositoryIntegrity,
        Operation::ReadExecutionOutput,
    ] {
        let failure = FailureResponse {
            api_version: VersionOne::V1,
            operation: Some(operation),
            outcome: FailureOutcome::ControlledFailure,
            error: ApiError {
                code: ErrorCode::InternalFailure,
                stage: None,
                detail: Some("API-03 service is absent in this development build".to_owned()),
                attempt_id: None,
                effects_possible: Some(false),
            },
        };
        let value = serde_json::to_value(failure).unwrap();
        assert_eq!(value.get("value"), None);
        assert_eq!(value["error"]["stage"], Value::Null);
        round_trip::<Response>(&value, "response");
    }
}
