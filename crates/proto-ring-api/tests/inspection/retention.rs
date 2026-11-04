use proto_ring_api::open;
use serde_json::json;

use super::{git_governance, inspect, inspection_fixture};

#[test]
fn inspection_is_retained_after_repository_edits_and_removal_while_open_refreshes() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let original = inspect(&observed, json!({"kind":"all"}));
    let validation = inspect(&observed, json!({"kind":"validation","id":"first"}));
    let mut profile = fixture.read("integrity.md");
    profile["repository_integrity"]["validations"]["second"]["command"]["arguments"] =
        json!(["new"]);
    fixture.write("integrity.md", profile);
    assert_eq!(inspect(&observed, json!({"kind":"all"})), original);
    assert_eq!(
        inspect(&observed, json!({"kind":"validation","id":"first"})),
        validation
    );
    let reopened = open(&fixture.root).unwrap();
    let refreshed = inspect(&reopened, json!({"kind":"validation","id":"first"}));
    assert_ne!(
        validation["observation"]["state"],
        refreshed["observation"]["state"]
    );
    assert_ne!(
        validation["subject"]["profile_ref"],
        refreshed["subject"]["profile_ref"]
    );
    assert_eq!(
        validation["subject"]["declaration"],
        refreshed["subject"]["declaration"]
    );
    std::fs::remove_dir_all(&fixture.root).unwrap();
    assert_eq!(inspect(&observed, json!({"kind":"all"})), original);
    assert_eq!(
        inspect(&observed, json!({"kind":"validation","id":"first"})),
        validation
    );
}

#[test]
fn inspection_does_not_run_declared_commands_or_mutate_git_or_loaded_carriers() {
    let fixture = inspection_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    let probes = [
        vec!["status", "--porcelain=v1", "-z"],
        vec!["ls-files", "--stage", "-z"],
        vec!["config", "--local", "--list", "-z"],
        vec!["symbolic-ref", "HEAD"],
    ];
    let before: Vec<_> = probes.iter().map(|args| fixture.git(args)).collect();
    let before_units = git_governance::native_units(&fixture.root);
    let carriers = [
        "AGENTS.md",
        "authority.md",
        "bindings.md",
        "objects.md",
        "integrity.md",
        "projections.md",
        "evidence.md",
        "generator.sh",
    ];
    let bytes: Vec<_> = carriers
        .iter()
        .map(|p| std::fs::read(fixture.root.join(p)).unwrap())
        .collect();
    for kind in [
        "all",
        "bootstrap",
        "model",
        "authority",
        "governed_objects",
        "bindings",
        "repository_integrity",
        "projections",
        "evidence_requirements",
    ] {
        inspect(&observed, json!({"kind":kind}));
    }
    for (kind, id) in [
        ("capability", "repository_integrity"),
        ("validation", "first"),
        ("binding", "provider"),
    ] {
        inspect(&observed, json!({"kind":kind,"id":id}));
    }
    inspect(
        &observed,
        json!({"kind":"governed_object","target":{"interface":"repository","object":"tree"}}),
    );
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
    assert_eq!(
        before_units,
        git_governance::native_units(observed.repository())
    );
}
