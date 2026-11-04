use proto_ring_api::open;
use serde_json::json;

use super::{explain, explanation_fixture, git_governance};

#[test]
fn all_explanation_families_retain_original_results_after_edits_and_repository_removal() {
    let fixture = explanation_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let queries = explanation_fixture::all_queries();
    let original: Vec<_> = queries
        .iter()
        .map(|query| explain(&observed, query.clone()))
        .collect();
    let mut profile = fixture.read("authority.md");
    profile["governance_authority"]["responsibilities"]["quality"]["roles"]["external"] =
        json!("non_authoritative");
    profile["governance_authority"]["responsibilities"]["quality"]["precedence"] =
        json!([{"higher_source":"bindings","lower_source":"integrity"}]);
    fixture.write("authority.md", profile);
    let mut profile = fixture.read("integrity.md");
    profile["repository_integrity"]["validations"]["second"]["command"]["arguments"] =
        json!(["changed"]);
    fixture.write("integrity.md", profile);
    for (carrier, registry, collection, id) in [
        (
            "projections.md",
            "projection_registry",
            "projections",
            "generated",
        ),
        (
            "evidence.md",
            "evidence_requirements",
            "requirements",
            "sourced",
        ),
    ] {
        let mut profile = fixture.read(carrier);
        if registry == "projection_registry" {
            profile[registry][collection][id]["validation"] = json!("second");
        } else {
            profile[registry][collection][id]["context"] = json!({"required":false});
        }
        fixture.write(carrier, profile);
    }
    for (query, expected) in queries.iter().zip(&original) {
        assert_eq!(&explain(&observed, query.clone()), expected);
    }
    let reopened = open(&fixture.root).unwrap();
    let refreshed: Vec<_> = queries
        .iter()
        .map(|query| explain(&reopened, query.clone()))
        .collect();
    assert_ne!(
        original[0]["observation"]["state"],
        refreshed[0]["observation"]["state"]
    );
    assert_eq!(refreshed[0]["explanation"]["role"], "non_authoritative");
    assert_eq!(refreshed[1]["explanation"]["relation"], "DOES_NOT_OUTRANK");
    let original_validation = &original[3]["explanation"]["declaration"];
    let refreshed_validation = &refreshed[3]["explanation"]["declaration"];
    assert_eq!(
        original_validation["declaration"],
        refreshed_validation["declaration"]
    );
    assert_ne!(
        original_validation["profile_ref"],
        refreshed_validation["profile_ref"]
    );
    for i in [6, 7] {
        assert_ne!(original[i]["explanation"], refreshed[i]["explanation"]);
    }
    std::fs::remove_dir_all(&fixture.root).unwrap();
    for (query, expected) in queries.iter().zip(&original) {
        assert_eq!(&explain(&observed, query.clone()), expected);
    }
}

#[test]
fn explanations_do_not_execute_sentinels_or_mutate_carriers_git_or_scope() {
    let fixture = explanation_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let probes = [
        vec!["status", "--porcelain=v1", "-z"],
        vec!["ls-files", "--stage", "-z"],
        vec!["config", "--local", "--list", "-z"],
        vec!["symbolic-ref", "HEAD"],
    ];
    let before: Vec<_> = probes.iter().map(|args| fixture.git(args)).collect();
    let carriers = [
        "AGENTS.md",
        "authority.md",
        "bindings.md",
        "objects.md",
        "integrity.md",
        "projections.md",
        "evidence.md",
        "generator.sh",
        "secondary.md",
    ];
    let bytes: Vec<_> = carriers
        .iter()
        .map(|p| std::fs::read(fixture.root.join(p)).unwrap())
        .collect();
    let scope = observed.scope_paths().to_vec();
    let root_units = git_governance::native_units(observed.repository());
    for query in explanation_fixture::all_queries() {
        explain(&observed, query);
    }
    assert!(!fixture.root.join("EFFECT-MARKER").exists());
    assert_eq!(
        before,
        probes
            .iter()
            .map(|args| fixture.git(args))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        bytes,
        carriers
            .iter()
            .map(|p| std::fs::read(fixture.root.join(p)).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(observed.scope_paths(), scope);
    assert_eq!(
        root_units,
        git_governance::native_units(observed.repository())
    );
}
