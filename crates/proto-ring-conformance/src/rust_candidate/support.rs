#![forbid(unsafe_code)]
use crate::fixture;
use crate::model::TransportValue;
use crate::rust_candidate::transport::{exact_record, fixture_failure, unique_dynamic_record};
use proto_ring_engine::governance_authority::{
    GovernanceAuthorityProfile, GovernedResponsibility, GovernedSource, PrecedenceEdge, SourceRole,
};
use proto_ring_engine::governance_bindings::{
    BindingAuthority, BindingKind, ExecutableProviderIdentity, GovernanceBinding,
    GovernanceBindingIdentity, GovernanceBindingRegistry, GovernanceBindingScope,
    GovernanceContractIdentity, ScopeKind,
};
use proto_ring_engine::governance_routing::ResolvedGovernanceRoute;
use proto_ring_engine::governed_objects::{
    GovernedInterface, GovernedObject, GovernedObjectCatalog,
};
use proto_ring_engine::repository_integrity::{
    CommandBinding, ConsumerIntegrityProfile, ProfileAuthority, ProfileIdentity,
    ValidationDefinition, ValidationInstances,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;
pub(super) fn support_authority(
    root: &Path,
    value: &TransportValue,
) -> Result<GovernanceAuthorityProfile, fixture::FixtureError> {
    let authority = exact_record(
        value,
        "support authority",
        &["carrier", "responsibilities", "sources"],
    )?;
    let carrier = string_field(&authority, "carrier")?;
    let mut sources = BTreeMap::new();
    for (source_id, value) in
        unique_dynamic_record(field(&authority, "sources"), "support sources")?
    {
        require_nonempty(source_id, "support source ID")?;
        let declaration =
            exact_record(value, "support source declaration", &["repository_target"])?;
        // semantic_inputs.authority represents an already-loaded Governance Authority
        // profile. Rehydrate its resolved source bindings directly; do not rerun Canonical
        // Governance Routing or filesystem availability checks in the conformance adapter.
        let repository_target = match field(&declaration, "repository_target") {
            TransportValue::Null(()) => None,
            TransportValue::String(path) => Some(profile_route(root, path, source_id)),
            _ => {
                return Err(fixture_failure(
                    "support repository_target must be string or null",
                ));
            }
        };
        sources.insert(
            source_id.to_owned(),
            GovernedSource {
                id: source_id.to_owned(),
                repository_target,
            },
        );
    }
    let mut responsibilities = BTreeMap::new();
    for (responsibility_id, value) in unique_dynamic_record(
        field(&authority, "responsibilities"),
        "support responsibilities",
    )? {
        require_nonempty(responsibility_id, "support responsibility ID")?;
        let declaration = exact_record(value, "support responsibility", &["precedence", "roles"])?;
        responsibilities.insert(
            responsibility_id.to_owned(),
            GovernedResponsibility {
                id: responsibility_id.to_owned(),
                roles: support_roles(field(&declaration, "roles"))?,
                precedence: support_precedence(field(&declaration, "precedence"))?,
            },
        );
    }
    Ok(GovernanceAuthorityProfile {
        repository: root.to_path_buf(),
        carrier: root.join(carrier),
        model_version: 1,
        sources,
        responsibilities,
    })
}
fn support_roles(
    value: &TransportValue,
) -> Result<BTreeMap<String, SourceRole>, fixture::FixtureError> {
    unique_dynamic_record(value, "support roles")?
        .into_iter()
        .map(|(source_id, value)| {
            require_nonempty(source_id, "support role source ID")?;
            let role = match fixture::string(value)? {
                "authority" => SourceRole::Authority,
                "secondary_representation" => SourceRole::SecondaryRepresentation,
                "non_authoritative" => SourceRole::NonAuthoritative,
                _ => return Err(fixture_failure("unknown support source role")),
            };
            Ok((source_id.to_owned(), role))
        })
        .collect()
}
pub(super) fn support_catalog(
    root: &Path,
    value: &TransportValue,
) -> Result<GovernedObjectCatalog, fixture::FixtureError> {
    let catalog = exact_record(value, "support catalog", &["carrier", "interfaces"])?;
    let mut interfaces = BTreeMap::new();
    for (interface_id, value) in
        unique_dynamic_record(field(&catalog, "interfaces"), "support interfaces")?
    {
        require_nonempty(interface_id, "support interface ID")?;
        let mut objects = BTreeMap::new();
        for (object_id, value) in unique_dynamic_record(value, "support objects")? {
            require_nonempty(object_id, "support object ID")?;
            let declaration = exact_record(value, "support object", &["responsibilities"])?;
            let responsibilities = string_set(
                field(&declaration, "responsibilities"),
                "support responsibilities",
            )?;
            objects.insert(
                object_id.to_owned(),
                GovernedObject {
                    id: object_id.to_owned(),
                    responsibilities,
                    relations: Default::default(),
                },
            );
        }
        interfaces.insert(
            interface_id.to_owned(),
            GovernedInterface {
                id: interface_id.to_owned(),
                objects,
            },
        );
    }
    Ok(GovernedObjectCatalog {
        repository: root.to_path_buf(),
        carrier: root.join(string_field(&catalog, "carrier")?),
        model_version: 1,
        interfaces,
    })
}
fn support_precedence(
    value: &TransportValue,
) -> Result<Vec<PrecedenceEdge>, fixture::FixtureError> {
    fixture::sequence(value)?
        .iter()
        .map(|edge| {
            let fields = exact_record(
                edge,
                "support precedence entry",
                &["higher_source", "lower_source"],
            )?;
            Ok(PrecedenceEdge {
                higher_source: string_field(&fields, "higher_source")?.to_owned(),
                lower_source: string_field(&fields, "lower_source")?.to_owned(),
            })
        })
        .collect()
}
pub(super) fn support_integrity_profile(
    root: &Path,
    value: &TransportValue,
) -> Result<ConsumerIntegrityProfile, fixture::FixtureError> {
    let profile = exact_record(
        value,
        "support integrity profile",
        &[
            "authority",
            "carrier",
            "continue_after_non_satisfied",
            "environments",
            "identity",
            "order",
            "validations",
        ],
    )?;
    let authority = exact_record(
        field(&profile, "authority"),
        "support profile authority",
        &["responsibility", "source"],
    )?;
    let environments = string_set(field(&profile, "environments"), "support environments")?;
    let mut validations = BTreeMap::new();
    for value in fixture::sequence(field(&profile, "validations"))? {
        let validation = exact_record(
            value,
            "support validation",
            &[
                "arguments",
                "environment",
                "id",
                "prerequisites",
                "responsibility",
                "undetermined_exit_codes",
            ],
        )?;
        let id = string_field(&validation, "id")?.to_owned();
        require_nonempty(&id, "support validation ID")?;
        if validations.contains_key(&id) {
            return Err(fixture_failure("duplicate support validation ID"));
        }
        validations.insert(
            id.clone(),
            ValidationDefinition {
                id,
                responsibility: string_field(&validation, "responsibility")?.to_owned(),
                prerequisites: string_set(
                    field(&validation, "prerequisites"),
                    "support prerequisites",
                )?,
                instances: ValidationInstances::Single,
                command: CommandBinding {
                    environment: string_field(&validation, "environment")?.to_owned(),
                    arguments: string_vec(field(&validation, "arguments"), "support arguments")?,
                    undetermined_exit_codes: fixture::sequence(field(
                        &validation,
                        "undetermined_exit_codes",
                    ))?
                    .iter()
                    .map(|value| match value {
                        TransportValue::Integer(value) => Ok(value.clone()),
                        _ => Err(fixture_failure("support exit code must be integer")),
                    })
                    .collect::<Result<_, _>>()?,
                },
                target: None,
            },
        );
    }
    Ok(ConsumerIntegrityProfile {
        repository: root.to_path_buf(),
        carrier: root.join(string_field(&profile, "carrier")?),
        model_version: 1,
        authority: ProfileAuthority {
            responsibility: string_field(&authority, "responsibility")?.to_owned(),
            source: string_field(&authority, "source")?.to_owned(),
        },
        environments,
        continue_after_non_satisfied: match field(&profile, "continue_after_non_satisfied") {
            TransportValue::Boolean(value) => *value,
            _ => return Err(fixture_failure("support continuation must be boolean")),
        },
        validations,
        order: string_vec(field(&profile, "order"), "support order")?,
        identity: ProfileIdentity({
            let identity = string_field(&profile, "identity")?;
            require_nonempty(identity, "support profile identity")?;
            identity.as_bytes().to_vec()
        }),
    })
}
pub(super) fn support_bindings(
    root: &Path,
    value: &TransportValue,
) -> Result<GovernanceBindingRegistry, fixture::FixtureError> {
    let registry = exact_record(
        value,
        "support bindings",
        &["bindings", "carrier", "source"],
    )?;
    let mut bindings = BTreeMap::new();
    for value in fixture::sequence(field(&registry, "bindings"))? {
        let entries = unique_dynamic_record(value, "support binding")?;
        let fields: Fields<'_> = entries.into_iter().collect();
        let kind = match required_transport_string(&fields, "kind")? {
            "executable_provider" => BindingKind::ExecutableProvider,
            "governance_contract" => BindingKind::GovernanceContract,
            _ => return Err(fixture_failure("unknown support binding kind")),
        };
        let scope_kind = match required_transport_string(&fields, "scope_kind")? {
            "logical_provider" => ScopeKind::LogicalProvider,
            "capability" => ScopeKind::Capability,
            _ => return Err(fixture_failure("unknown support scope kind")),
        };
        let required: BTreeSet<_> = [
            "commit",
            "id",
            "kind",
            "repository",
            "responsibility",
            "scope_kind",
            "source",
        ]
        .into_iter()
        .chain((kind == BindingKind::GovernanceContract).then_some("path"))
        .chain((scope_kind == ScopeKind::Capability).then_some("capability"))
        .collect();
        if fields.len() != required.len() || fields.keys().any(|name| !required.contains(name)) {
            return Err(fixture_failure(
                "support binding has missing or unknown field",
            ));
        }
        let id = string_field(&fields, "id")?.to_owned();
        let repository = string_field(&fields, "repository")?.to_owned();
        let commit = string_field(&fields, "commit")?.to_owned();
        let identity = match kind {
            BindingKind::ExecutableProvider => {
                GovernanceBindingIdentity::ExecutableProvider(ExecutableProviderIdentity {
                    repository,
                    commit,
                })
            }
            BindingKind::GovernanceContract => {
                GovernanceBindingIdentity::GovernanceContract(GovernanceContractIdentity {
                    repository,
                    commit,
                    path: string_field(&fields, "path")?.to_owned(),
                })
            }
        };
        if bindings.contains_key(&id) {
            return Err(fixture_failure("duplicate support binding ID"));
        }
        bindings.insert(
            id.clone(),
            GovernanceBinding {
                id,
                kind,
                scope: GovernanceBindingScope {
                    kind: scope_kind,
                    capability: fields
                        .get("capability")
                        .map(|value| fixture::string(value).map(str::to_owned))
                        .transpose()?,
                    interface: None,
                    object: None,
                },
                identity,
                authority: BindingAuthority {
                    responsibility: string_field(&fields, "responsibility")?.to_owned(),
                    source: string_field(&fields, "source")?.to_owned(),
                },
            },
        );
    }
    Ok(GovernanceBindingRegistry {
        repository: root.to_path_buf(),
        carrier: root.join(string_field(&registry, "carrier")?),
        model_version: 1,
        source: string_field(&registry, "source")?.to_owned(),
        bindings,
    })
}
fn string_vec(value: &TransportValue, label: &str) -> Result<Vec<String>, fixture::FixtureError> {
    fixture::sequence(value)?
        .iter()
        .map(|value| fixture::string(value).map(str::to_owned))
        .collect::<Result<_, _>>()
        .map_err(|error| fixture_failure(format!("{label}: {error}")))
}
fn string_set(
    value: &TransportValue,
    label: &str,
) -> Result<BTreeSet<String>, fixture::FixtureError> {
    let values = string_vec(value, label)?;
    let set: BTreeSet<_> = values.iter().cloned().collect();
    if values.iter().any(String::is_empty) || set.len() != values.len() {
        Err(fixture_failure(format!(
            "{label} must contain unique nonempty strings"
        )))
    } else {
        Ok(set)
    }
}
fn profile_route(root: &Path, path: &str, responsibility: &str) -> ResolvedGovernanceRoute {
    ResolvedGovernanceRoute {
        route: vec![
            "capabilities".to_owned(),
            responsibility.to_owned(),
            "routes".to_owned(),
            "profile".to_owned(),
        ],
        declared_path: path.to_owned(),
        target: root.join(path),
    }
}
fn field<'a>(fields: &Fields<'a>, name: &str) -> &'a TransportValue {
    fields.get(name).copied().expect("exact field exists")
}
fn string_field<'a>(fields: &Fields<'a>, name: &str) -> Result<&'a str, fixture::FixtureError> {
    fixture::string(field(fields, name))
}
fn required_transport_string<'a>(
    fields: &Fields<'a>,
    name: &str,
) -> Result<&'a str, fixture::FixtureError> {
    fields
        .get(name)
        .ok_or_else(|| fixture_failure(format!("support value is missing {name}")))
        .and_then(|value| fixture::string(value))
}
fn require_nonempty(value: &str, label: &str) -> Result<(), fixture::FixtureError> {
    if value.is_empty() {
        Err(fixture_failure(format!("{label} must be nonempty")))
    } else {
        Ok(())
    }
}
