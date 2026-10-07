#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use proto_ring_engine::governance_authority::{
    GovernanceAuthorityProfile, GovernedResponsibility, GovernedSource, PrecedenceEdge,
    ResponsibilityLookup, RoleLookup, SourceRole, authority_sources, outranks, role_of,
    sources_with_role,
};

fn profile() -> GovernanceAuthorityProfile {
    let sources = ["a", "b", "c", "secondary"]
        .into_iter()
        .map(|id| {
            (
                id.to_owned(),
                GovernedSource {
                    id: id.to_owned(),
                    repository_target: None,
                },
            )
        })
        .collect();
    let responsibilities = BTreeMap::from([
        (
            "r".to_owned(),
            GovernedResponsibility {
                id: "r".to_owned(),
                roles: BTreeMap::from([
                    ("a".to_owned(), SourceRole::Authority),
                    ("b".to_owned(), SourceRole::Authority),
                    ("c".to_owned(), SourceRole::Authority),
                    ("secondary".to_owned(), SourceRole::SecondaryRepresentation),
                ]),
                precedence: vec![
                    PrecedenceEdge {
                        higher_source: "a".to_owned(),
                        lower_source: "b".to_owned(),
                    },
                    PrecedenceEdge {
                        higher_source: "b".to_owned(),
                        lower_source: "c".to_owned(),
                    },
                ],
            },
        ),
        (
            "empty".to_owned(),
            GovernedResponsibility {
                id: "empty".to_owned(),
                roles: BTreeMap::new(),
                precedence: Vec::new(),
            },
        ),
    ]);
    GovernanceAuthorityProfile {
        repository: PathBuf::from("repository"),
        carrier: PathBuf::from("authority.md"),
        model_version: 1,
        sources,
        responsibilities,
    }
}

#[test]
fn role_lookup_preserves_all_three_states() {
    let profile = profile();
    assert_eq!(
        role_of(&profile, "r", "a"),
        RoleLookup::Role(SourceRole::Authority)
    );
    assert_eq!(role_of(&profile, "r", "missing"), RoleLookup::Undeclared);
    assert_eq!(
        role_of(&profile, "missing", "a"),
        RoleLookup::UnknownResponsibility
    );
}

#[test]
fn role_sets_preserve_empty_known_and_unknown_responsibilities() {
    let profile = profile();
    assert_eq!(
        sources_with_role(&profile, "r", SourceRole::SecondaryRepresentation),
        ResponsibilityLookup::Value(BTreeSet::from(["secondary".to_owned()]))
    );
    assert_eq!(
        authority_sources(&profile, "empty"),
        ResponsibilityLookup::Value(BTreeSet::new())
    );
    assert_eq!(
        authority_sources(&profile, "missing"),
        ResponsibilityLookup::UnknownResponsibility
    );
}

#[test]
fn outranking_is_strict_directional_transitive_and_scoped() {
    let profile = profile();
    assert_eq!(
        outranks(&profile, "r", "a", "c"),
        ResponsibilityLookup::Value(true)
    );
    assert_eq!(
        outranks(&profile, "r", "b", "a"),
        ResponsibilityLookup::Value(false)
    );
    assert_eq!(
        outranks(&profile, "r", "a", "a"),
        ResponsibilityLookup::Value(false)
    );
    assert_eq!(
        outranks(&profile, "missing", "a", "b"),
        ResponsibilityLookup::UnknownResponsibility
    );
}
