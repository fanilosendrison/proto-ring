#![forbid(unsafe_code)]

use std::path::Path;

use proto_ring_engine::governance_bootstrap::{self, GovernanceBootstrapErrorKind};
use proto_ring_engine::governance_routing;
use proto_ring_engine::repository_governance_model::{self, RepositoryGovernanceModel};
use proto_ring_engine::structured_data::{self, StructuredValue};

use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, RecordEntry, TransportValue};

use super::transport::{
    fixture_failure, record, rejection, result, string_value, structured_transport,
    transport_to_structured,
};

pub(super) fn execute(
    responsibility: &str,
    fixture: &MaterializedFixture,
) -> Result<Observation, fixture::FixtureError> {
    match responsibility {
        "structured-data.document" => structured_document(fixture),
        "structured-data.frontmatter" => structured_frontmatter(fixture),
        "governance-bootstrap.root" => bootstrap(fixture),
        "governance-routing.resolve" => routing(fixture),
        "repository-governance-model.load" => model_load(fixture),
        "repository-governance-model.binding-compatibility" => binding_compatibility(fixture),
        _ => unreachable!("foundation responsibility inventory is closed"),
    }
}

fn structured_document(
    fixture: &MaterializedFixture,
) -> Result<Observation, fixture::FixtureError> {
    require_operation(fixture, "parse_document")?;
    let bytes = input_bytes(fixture)?;
    Ok(match structured_data::parse_document_bytes(&bytes) {
        Ok(value) => result(structured_transport(&value)),
        Err(_) => rejection("structured-data.document"),
    })
}

fn structured_frontmatter(
    fixture: &MaterializedFixture,
) -> Result<Observation, fixture::FixtureError> {
    let operation = argument_string(fixture, "operation")?;
    let bytes = input_bytes(fixture)?;
    if operation == "parse_frontmatter" {
        return Ok(match structured_data::parse_frontmatter_bytes(&bytes) {
            Ok(parsed) => result(record(vec![
                ("body_hex", TransportValue::String(hex_encode(&parsed.body))),
                ("metadata", structured_transport(&parsed.metadata)),
            ])),
            Err(_) => rejection("structured-data.frontmatter"),
        });
    }
    if operation != "immediate_close_boundaries" {
        return Err(fixture_failure("unsupported frontmatter operation"));
    }
    let envelope = structured_data::extract_frontmatter_bytes(&bytes);
    let immediate_close = envelope
        .as_ref()
        .is_ok_and(|value| value.payload.is_empty());
    let payload = match &envelope {
        Ok(value) if value.payload.is_empty() => "EMPTY",
        Ok(_) => "NONEMPTY",
        Err(_) => "UNAVAILABLE",
    };
    let structured_rejected = structured_data::parse_frontmatter_bytes(&bytes).is_err();
    let bootstrap_error = fixture
        .root()
        .and_then(|root| governance_bootstrap::load(root).err());
    let bootstrap_rejected = bootstrap_error.is_some();
    let cause = match bootstrap_error.map(|error| error.kind) {
        Some(GovernanceBootstrapErrorKind::StructuredData) => "STRUCTURED_DATA",
        Some(_) => "OTHER",
        None => "NONE",
    };
    Ok(result(record(vec![
        ("bootstrap_cause_boundary", string_value(cause)),
        ("frontmatter_payload", string_value(payload)),
        (
            "governance_body_interpreted",
            TransportValue::Boolean(false),
        ),
        (
            "governance_bootstrap_boundary",
            string_value(if bootstrap_rejected {
                "CONTROLLED_REJECTION"
            } else {
                "RESULT"
            }),
        ),
        (
            "immediate_close_recognized",
            TransportValue::Boolean(immediate_close),
        ),
        (
            "structured_data_boundary",
            string_value(if structured_rejected {
                "CONTROLLED_REJECTION"
            } else {
                "RESULT"
            }),
        ),
    ])))
}

fn bootstrap(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    require_operation(fixture, "load_bootstrap")?;
    let root = fixture
        .root()
        .ok_or_else(|| fixture_failure("bootstrap requires repository"))?;
    Ok(match governance_bootstrap::load(root) {
        Ok(value) => result(record(vec![
            ("metadata", structured_transport(&value.metadata)),
            (
                "repository_governance",
                structured_transport(&StructuredValue::Mapping(value.repository_governance)),
            ),
        ])),
        Err(_) => rejection("governance-bootstrap.root"),
    })
}

fn routing(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    require_operation(fixture, "resolve")?;
    let root = fixture
        .root()
        .ok_or_else(|| fixture_failure("routing requires repository"))?;
    let routing = transport_to_structured(fixture::record_field(fixture.arguments(), "routing")?)?;
    let route = fixture::sequence(fixture::record_field(fixture.arguments(), "route")?)?
        .iter()
        .map(fixture::string)
        .map(|value| value.map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(match governance_routing::resolve(root, &routing, &route) {
        Ok(resolved) => {
            let canonical_root = root
                .canonicalize()
                .map_err(|error| fixture_failure(error.to_string()))?;
            let relative = resolved
                .target
                .strip_prefix(canonical_root)
                .map_err(|error| fixture_failure(error.to_string()))?;
            let target_relative = relative
                .components()
                .map(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .ok_or_else(|| fixture_failure("non-UTF-8 route"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .join("/");
            result(record(vec![
                ("declared_path", string_value(&resolved.declared_path)),
                (
                    "route",
                    TransportValue::Sequence(
                        route.iter().map(|value| string_value(value)).collect(),
                    ),
                ),
                ("target_relative", string_value(&target_relative)),
            ]))
        }
        Err(_) => rejection("governance-routing.resolve"),
    })
}

fn model_load(fixture: &MaterializedFixture) -> Result<Observation, fixture::FixtureError> {
    require_operation(fixture, "load_model")?;
    let observe_resolved_routes =
        optional_boolean_argument(fixture, "observe_resolved_routes")?.unwrap_or(false);
    let root = fixture
        .root()
        .ok_or_else(|| fixture_failure("model load requires repository"))?;
    Ok(match repository_governance_model::load(root) {
        Ok(model) => result(model_transport(&model, observe_resolved_routes)?),
        Err(_) => rejection("repository-governance-model.load"),
    })
}

fn binding_compatibility(
    fixture: &MaterializedFixture,
) -> Result<Observation, fixture::FixtureError> {
    require_operation(fixture, "validate_binding")?;
    let root = fixture
        .root()
        .ok_or_else(|| fixture_failure("binding check requires repository"))?;
    let model = match repository_governance_model::load(root) {
        Ok(model) => model,
        Err(_) => {
            return Ok(rejection(
                "repository-governance-model.binding-compatibility",
            ));
        }
    };
    let inputs = fixture::record_field(fixture.arguments(), "semantic_inputs")?;
    let registry = fixture::record_field(inputs, "binding_registry")?;
    let bindings = fixture::sequence(fixture::record_field(registry, "bindings")?)?;
    let mut scopes = Vec::new();
    for binding in bindings {
        if fixture::string(fixture::record_field(binding, "scope_kind")?)? == "capability" {
            scopes.push(fixture::string(fixture::record_field(binding, "capability")?)?.to_owned());
        }
    }
    Ok(
        match repository_governance_model::validate_binding_capabilities(&model, &scopes) {
            Ok(()) => result(record(vec![("compatible", TransportValue::Boolean(true))])),
            Err(_) => rejection("repository-governance-model.binding-compatibility"),
        },
    )
}

fn model_transport(
    model: &RepositoryGovernanceModel,
    observe_resolved_routes: bool,
) -> Result<TransportValue, fixture::FixtureError> {
    let capabilities = TransportValue::Sequence(
        model
            .capabilities
            .keys()
            .map(|value| string_value(value))
            .collect(),
    );
    let required_routes = TransportValue::Record(
        model
            .capabilities
            .iter()
            .map(|(capability, value)| RecordEntry {
                name: capability.clone(),
                value: TransportValue::Sequence(
                    value
                        .routes
                        .keys()
                        .map(|route| string_value(route))
                        .collect(),
                ),
            })
            .collect(),
    );
    let mut fields = vec![
        ("capabilities", capabilities),
        (
            "model_version",
            TransportValue::Integer(model.model_version.to_string()),
        ),
        (
            "provider_binding_capability",
            string_value(&model.provider.binding.capability),
        ),
        (
            "provider_binding_route",
            string_value(&model.provider.binding.route),
        ),
        ("provider_id", string_value(&model.provider.id)),
        ("required_routes", required_routes),
    ];
    if observe_resolved_routes {
        fields.push(("resolved_routes", resolved_routes_transport(model)?));
    }
    Ok(record(fields))
}

fn resolved_routes_transport(
    model: &RepositoryGovernanceModel,
) -> Result<TransportValue, fixture::FixtureError> {
    let mut capabilities = Vec::new();
    for (capability_id, capability) in &model.capabilities {
        let mut routes = Vec::new();
        for (route_id, resolved) in &capability.routes {
            routes.push(RecordEntry {
                name: route_id.clone(),
                value: record(vec![
                    ("declared_path", string_value(&resolved.declared_path)),
                    (
                        "route",
                        TransportValue::Sequence(
                            resolved
                                .route
                                .iter()
                                .map(|segment| string_value(segment))
                                .collect(),
                        ),
                    ),
                    (
                        "target_relative",
                        string_value(&portable_relative(
                            &model.repository_root,
                            &resolved.target,
                        )?),
                    ),
                ]),
            });
        }
        capabilities.push(RecordEntry {
            name: capability_id.clone(),
            value: TransportValue::Record(routes),
        });
    }
    Ok(TransportValue::Record(capabilities))
}

fn portable_relative(root: &Path, target: &Path) -> Result<String, fixture::FixtureError> {
    target
        .strip_prefix(root)
        .map_err(|error| fixture_failure(error.to_string()))?
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .ok_or_else(|| fixture_failure("non-UTF-8 route"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|components| components.join("/"))
}

fn optional_boolean_argument(
    fixture: &MaterializedFixture,
    name: &str,
) -> Result<Option<bool>, fixture::FixtureError> {
    let TransportValue::Record(entries) = fixture.arguments() else {
        return Err(fixture_failure("fixture arguments must be a record"));
    };
    match entries.iter().find(|entry| entry.name == name) {
        None => Ok(None),
        Some(RecordEntry {
            value: TransportValue::Boolean(value),
            ..
        }) => Ok(Some(*value)),
        Some(_) => Err(fixture_failure(format!("{name} must be boolean"))),
    }
}

fn require_operation(
    fixture: &MaterializedFixture,
    expected: &str,
) -> Result<(), fixture::FixtureError> {
    if argument_string(fixture, "operation")? == expected {
        Ok(())
    } else {
        Err(fixture_failure("unsupported candidate operation"))
    }
}

fn argument_string<'a>(
    fixture: &'a MaterializedFixture,
    name: &str,
) -> Result<&'a str, fixture::FixtureError> {
    fixture::string(fixture::record_field(fixture.arguments(), name)?)
}

fn input_bytes(fixture: &MaterializedFixture) -> Result<Vec<u8>, fixture::FixtureError> {
    hex_decode(argument_string(fixture, "bytes_hex")?)
}

fn hex_decode(value: &str) -> Result<Vec<u8>, fixture::FixtureError> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(fixture_failure("hex input has odd length"));
    }
    pairs
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|_| fixture_failure("invalid hex"))?;
            u8::from_str_radix(text, 16).map_err(|_| fixture_failure("invalid hex"))
        })
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}
