use proto_ring_api::open;
use serde_json::json;

use super::{explain, explanation_fixture};

#[test]
fn role_consultations_preserve_all_five_states_and_responsibility_scope() {
    let fixture = explanation_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    for (responsibility, source, expected) in [
        ("quality", "bindings", "authority"),
        ("quality", "secondary", "secondary_representation"),
        ("quality", "generator", "non_authoritative"),
        ("missing", "bindings", "UNKNOWN_RESPONSIBILITY"),
        ("quality", "missing", "UNDECLARED"),
        ("registry-owner", "bindings", "UNDECLARED"),
        ("different", "bindings", "non_authoritative"),
        ("Quality", "bindings", "UNKNOWN_RESPONSIBILITY"),
    ] {
        let value = explain(
            &observed,
            json!({"kind":"authority_role","responsibility":responsibility,"source":source}),
        );
        assert_eq!(
            value["explanation"],
            json!({"kind":"authority_role","responsibility":responsibility,"source":source,"role":expected})
        );
    }
}

#[test]
fn precedence_preserves_direct_transitive_and_incomparable_relations_without_inversion() {
    let fixture = explanation_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    for (responsibility, higher, lower, expected) in [
        ("quality", "bindings", "integrity", "OUTRANKS"),
        ("quality", "integrity", "bottom", "OUTRANKS"),
        ("quality", "bindings", "bottom", "OUTRANKS"),
        ("quality", "bottom", "bindings", "DOES_NOT_OUTRANK"),
        ("quality", "bindings", "external", "DOES_NOT_OUTRANK"),
        ("quality", "external", "bindings", "DOES_NOT_OUTRANK"),
        ("quality", "bindings", "bindings", "DOES_NOT_OUTRANK"),
        ("quality", "missing", "bindings", "DOES_NOT_OUTRANK"),
        ("quality", "bindings", "missing", "DOES_NOT_OUTRANK"),
        ("different", "bindings", "bottom", "DOES_NOT_OUTRANK"),
        ("missing", "bindings", "bottom", "UNKNOWN_RESPONSIBILITY"),
        (
            "missing",
            "missing",
            "also-missing",
            "UNKNOWN_RESPONSIBILITY",
        ),
    ] {
        let value = explain(
            &observed,
            json!({"kind":"authority_precedence","responsibility":responsibility,"higher_source":higher,"lower_source":lower}),
        );
        assert_eq!(
            value["explanation"],
            json!({"kind":"authority_precedence","responsibility":responsibility,"higher_source":higher,"lower_source":lower,"relation":expected})
        );
    }
}

#[test]
fn empty_responsibility_remains_known_and_unknown_responsibility_is_a_successful_consultation() {
    let fixture = explanation_fixture::rich();
    let observed = open(&fixture.root).unwrap();
    for (responsibility, role, relation) in [
        ("unassigned", "UNDECLARED", "DOES_NOT_OUTRANK"),
        ("absent", "UNKNOWN_RESPONSIBILITY", "UNKNOWN_RESPONSIBILITY"),
    ] {
        assert_eq!(
            explain(
                &observed,
                json!({"kind":"authority_role","responsibility":responsibility,"source":"missing"})
            )["explanation"]["role"],
            role
        );
        assert_eq!(
            explain(
                &observed,
                json!({"kind":"authority_precedence","responsibility":responsibility,"higher_source":"bindings","lower_source":"bottom"})
            )["explanation"]["relation"],
            relation
        );
    }
}
