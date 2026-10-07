#![forbid(unsafe_code)]

use proto_ring_conformance::harness::CandidateRequest;
use proto_ring_conformance::model::{RecordEntry, TransportValue};
use serde_json::json;

const AUTHORITY_ARTIFACT: &str = "---\ngovernance_authority:\n  model_version: 1\n  sources:\n    owner: {}\n  responsibilities:\n    r:\n      roles:\n        owner: authority\n      precedence: []\n---\n# Carrier\n";
const OBJECTS_ARTIFACT: &str = "---\ngoverned_objects:\n  model_version: 1\n  interfaces:\n    a:\n      objects:\n        one:\n          responsibilities:\n          - r\n          relations: []\n---\n# Carrier\n";
fn record(entries: Vec<(&str, TransportValue)>) -> TransportValue {
    TransportValue::Record(
        entries
            .into_iter()
            .map(|(name, value)| RecordEntry {
                name: name.to_owned(),
                value,
            })
            .collect(),
    )
}

fn string(value: &str) -> TransportValue {
    TransportValue::String(value.to_owned())
}

fn source_declaration(target: TransportValue) -> TransportValue {
    record(vec![("repository_target", target)])
}

fn responsibility(roles: TransportValue, precedence: TransportValue) -> TransportValue {
    record(vec![("precedence", precedence), ("roles", roles)])
}

fn valid_sources() -> TransportValue {
    record(vec![(
        "owner",
        source_declaration(TransportValue::Null(())),
    )])
}

fn valid_roles() -> TransportValue {
    record(vec![("owner", string("authority"))])
}

fn valid_responsibilities() -> TransportValue {
    record(vec![(
        "r",
        responsibility(valid_roles(), TransportValue::Sequence(Vec::new())),
    )])
}

fn support_authority(sources: TransportValue, responsibilities: TransportValue) -> TransportValue {
    record(vec![
        ("carrier", string("authority.md")),
        ("responsibilities", responsibilities),
        ("sources", sources),
    ])
}

fn semantic_inputs(authority: TransportValue) -> TransportValue {
    record(vec![
        ("authority", authority),
        ("bindings", TransportValue::Null(())),
        ("catalog", TransportValue::Null(())),
        ("integrity_profile", TransportValue::Null(())),
    ])
}

fn valid_objects_query() -> TransportValue {
    record(vec![
        ("interface", string("a")),
        ("object", string("one")),
        ("operation", string("object_responsibilities")),
    ])
}

fn authority_invocation(query: TransportValue) -> TransportValue {
    record(vec![
        ("operation", string("load_authority")),
        ("path", string("authority.md")),
        ("query", query),
    ])
}

fn objects_invocation(authority: TransportValue, query: TransportValue) -> TransportValue {
    record(vec![
        ("foreign_authority", TransportValue::Boolean(false)),
        ("operation", string("load_objects")),
        ("path", string("objects.md")),
        ("query", query),
        ("semantic_inputs", semantic_inputs(authority)),
    ])
}

fn request(name: &str, responsibility: &str, invocation: TransportValue) -> CandidateRequest {
    CandidateRequest {
        responsibility_id: responsibility.to_owned(),
        vector_id: format!("synthetic.adapter-boundary.{name}"),
        fixture: json!({
            "kind": "repository_plan",
            "steps": [
                {"op": "write_utf8", "path": "authority.md", "text": AUTHORITY_ARTIFACT},
                {"op": "write_utf8", "path": "objects.md", "text": OBJECTS_ARTIFACT}
            ],
            "invocation": serde_json::to_value(invocation).unwrap()
        }),
    }
}

fn valid_support() -> TransportValue {
    support_authority(valid_sources(), valid_responsibilities())
}

pub(super) fn malformed_requests() -> Vec<CandidateRequest> {
    let ga = "governance-authority.profile";
    let objects = "governed-objects.catalog";
    let mut cases = vec![
        request(
            "01-duplicate-ga-operation",
            ga,
            authority_invocation(record(vec![
                ("operation", string("role_of")),
                ("operation", string("role_of")),
                ("responsibility", string("r")),
                ("source", string("owner")),
            ])),
        ),
        request(
            "02-unknown-ga-query-field",
            ga,
            authority_invocation(record(vec![
                ("operation", string("role_of")),
                ("responsibility", string("r")),
                ("source", string("owner")),
                ("extra", TransportValue::Null(())),
            ])),
        ),
        request(
            "03-missing-ga-source",
            ga,
            authority_invocation(record(vec![
                ("operation", string("role_of")),
                ("responsibility", string("r")),
            ])),
        ),
        request(
            "04-unknown-ga-operation",
            ga,
            authority_invocation(record(vec![("operation", string("unknown"))])),
        ),
        request(
            "05-wrong-type-ga-source",
            ga,
            authority_invocation(record(vec![
                ("operation", string("role_of")),
                ("responsibility", string("r")),
                ("source", TransportValue::Boolean(false)),
            ])),
        ),
    ];
    let duplicate_sources = record(vec![
        ("owner", source_declaration(TransportValue::Null(()))),
        ("owner", source_declaration(TransportValue::Null(()))),
    ]);
    cases.push(request(
        "06-duplicate-support-source",
        objects,
        objects_invocation(
            support_authority(duplicate_sources, valid_responsibilities()),
            valid_objects_query(),
        ),
    ));
    let duplicate_responsibilities = record(vec![
        (
            "r",
            responsibility(valid_roles(), TransportValue::Sequence(Vec::new())),
        ),
        (
            "r",
            responsibility(valid_roles(), TransportValue::Sequence(Vec::new())),
        ),
    ]);
    cases.push(request(
        "07-duplicate-support-responsibility",
        objects,
        objects_invocation(
            support_authority(valid_sources(), duplicate_responsibilities),
            valid_objects_query(),
        ),
    ));
    let duplicate_roles = record(vec![
        ("owner", string("authority")),
        ("owner", string("authority")),
    ]);
    cases.push(request(
        "08-duplicate-support-role",
        objects,
        objects_invocation(
            support_authority(
                valid_sources(),
                record(vec![(
                    "r",
                    responsibility(duplicate_roles, TransportValue::Sequence(Vec::new())),
                )]),
            ),
            valid_objects_query(),
        ),
    ));
    let unknown_inputs = record(vec![
        ("authority", valid_support()),
        ("bindings", TransportValue::Null(())),
        ("catalog", TransportValue::Null(())),
        ("integrity_profile", TransportValue::Null(())),
        ("extra", TransportValue::Null(())),
    ]);
    cases.push(request(
        "09-unknown-semantic-input",
        objects,
        record(vec![
            ("foreign_authority", TransportValue::Boolean(false)),
            ("operation", string("load_objects")),
            ("path", string("objects.md")),
            ("query", valid_objects_query()),
            ("semantic_inputs", unknown_inputs),
        ]),
    ));
    let missing_authority = record(vec![
        ("bindings", TransportValue::Null(())),
        ("catalog", TransportValue::Null(())),
        ("integrity_profile", TransportValue::Null(())),
    ]);
    cases.push(request(
        "10-missing-semantic-authority",
        objects,
        record(vec![
            ("foreign_authority", TransportValue::Boolean(false)),
            ("operation", string("load_objects")),
            ("path", string("objects.md")),
            ("query", valid_objects_query()),
            ("semantic_inputs", missing_authority),
        ]),
    ));
    let duplicate_carrier = record(vec![
        ("carrier", string("authority.md")),
        ("carrier", string("authority.md")),
        ("responsibilities", valid_responsibilities()),
        ("sources", valid_sources()),
    ]);
    cases.push(request(
        "11-duplicate-authority-carrier",
        objects,
        objects_invocation(duplicate_carrier, valid_objects_query()),
    ));
    let unknown_authority = record(vec![
        ("carrier", string("authority.md")),
        ("responsibilities", valid_responsibilities()),
        ("sources", valid_sources()),
        ("extra", TransportValue::Null(())),
    ]);
    cases.push(request(
        "12-unknown-authority-field",
        objects,
        objects_invocation(unknown_authority, valid_objects_query()),
    ));
    let source_unknown = record(vec![(
        "owner",
        record(vec![
            ("repository_target", TransportValue::Null(())),
            ("extra", TransportValue::Null(())),
        ]),
    )]);
    cases.push(request(
        "13-source-unknown-field",
        objects,
        objects_invocation(
            support_authority(source_unknown, valid_responsibilities()),
            valid_objects_query(),
        ),
    ));
    let source_wrong_type = record(vec![(
        "owner",
        source_declaration(TransportValue::Boolean(false)),
    )]);
    cases.push(request(
        "14-source-target-wrong-type",
        objects,
        objects_invocation(
            support_authority(source_wrong_type, valid_responsibilities()),
            valid_objects_query(),
        ),
    ));
    let responsibility_unknown = record(vec![(
        "r",
        record(vec![
            ("precedence", TransportValue::Sequence(Vec::new())),
            ("roles", valid_roles()),
            ("extra", TransportValue::Null(())),
        ]),
    )]);
    cases.push(request(
        "15-responsibility-unknown-field",
        objects,
        objects_invocation(
            support_authority(valid_sources(), responsibility_unknown),
            valid_objects_query(),
        ),
    ));
    let responsibility_missing_roles = record(vec![(
        "r",
        record(vec![("precedence", TransportValue::Sequence(Vec::new()))]),
    )]);
    cases.push(request(
        "16-responsibility-missing-roles",
        objects,
        objects_invocation(
            support_authority(valid_sources(), responsibility_missing_roles),
            valid_objects_query(),
        ),
    ));
    let missing_higher =
        TransportValue::Sequence(vec![record(vec![("lower_source", string("owner"))])]);
    cases.push(request(
        "17-precedence-missing-higher",
        objects,
        objects_invocation(
            support_authority(
                valid_sources(),
                record(vec![("r", responsibility(valid_roles(), missing_higher))]),
            ),
            valid_objects_query(),
        ),
    ));
    let unknown_precedence = TransportValue::Sequence(vec![record(vec![
        ("higher_source", string("owner")),
        ("lower_source", string("owner")),
        ("extra", TransportValue::Null(())),
    ])]);
    cases.push(request(
        "18-precedence-unknown-field",
        objects,
        objects_invocation(
            support_authority(
                valid_sources(),
                record(vec![(
                    "r",
                    responsibility(valid_roles(), unknown_precedence),
                )]),
            ),
            valid_objects_query(),
        ),
    ));
    let object_queries = [
        (
            "19-duplicate-object-operation",
            record(vec![
                ("interface", string("a")),
                ("object", string("one")),
                ("operation", string("object_relations")),
                ("operation", string("object_relations")),
            ]),
        ),
        (
            "20-unknown-object-query-field",
            record(vec![
                ("interface", string("a")),
                ("object", string("one")),
                ("operation", string("object_relations")),
                ("extra", TransportValue::Null(())),
            ]),
        ),
        (
            "21-missing-object-query-object",
            record(vec![
                ("interface", string("a")),
                ("operation", string("object_relations")),
            ]),
        ),
        (
            "22-unknown-object-operation",
            record(vec![
                ("interface", string("a")),
                ("object", string("one")),
                ("operation", string("unknown")),
            ]),
        ),
        (
            "23-wrong-type-object-interface",
            record(vec![
                ("interface", TransportValue::Boolean(false)),
                ("object", string("one")),
                ("operation", string("object_relations")),
            ]),
        ),
    ];
    cases.extend(
        object_queries.into_iter().map(|(name, query)| {
            request(name, objects, objects_invocation(valid_support(), query))
        }),
    );
    cases
}
