#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_engine::evidence_requirements;
use proto_ring_engine::governance_authority::{
    GovernanceAuthorityProfile, GovernedResponsibility, GovernedSource, SourceRole,
};
use proto_ring_engine::governance_bindings;
use proto_ring_engine::governance_routing::ResolvedGovernanceRoute;
use proto_ring_engine::governed_objects::{
    GovernedInterface, GovernedObject, GovernedObjectCatalog,
};
use proto_ring_engine::repository_integrity;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root exists")
        .to_path_buf()
}

#[test]
fn foreign_governed_objects_catalog_repository_is_rejected_for_governance_bindings() {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    let repository = temporary.path();
    let foreign = repository.join("foreign");
    fs::create_dir(&foreign).expect("foreign repository directory exists");
    let carrier = repository.join("bindings.md");
    fs::write(
        &carrier,
        "---\ngovernance_bindings:\n  model_version: 1\n  source: registry\n  bindings:\n    provider:\n      kind: executable_provider\n      scope:\n        kind: logical_provider\n      identity:\n        repository: provider/repository\n        commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n      authority:\n        responsibility: provider_binding\n        source: registry\n    contract:\n      kind: governance_contract\n      scope:\n        kind: governed_object\n        interface: interface\n        object: object\n      identity:\n        repository: provider/repository\n        commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n        path: docs/contracts/example.md\n      authority:\n        responsibility: contract_binding\n        source: owner\n---\n# Carrier\n",
    )
    .expect("binding carrier is written");
    let route = ResolvedGovernanceRoute {
        route: vec!["registry".to_owned()],
        declared_path: "bindings.md".to_owned(),
        target: carrier.clone(),
    };
    let authority = GovernanceAuthorityProfile {
        repository: repository.to_path_buf(),
        carrier: repository.join("authority.md"),
        model_version: 1,
        sources: BTreeMap::from([
            (
                "owner".to_owned(),
                GovernedSource {
                    id: "owner".to_owned(),
                    repository_target: None,
                },
            ),
            (
                "registry".to_owned(),
                GovernedSource {
                    id: "registry".to_owned(),
                    repository_target: Some(route.clone()),
                },
            ),
        ]),
        responsibilities: BTreeMap::from([
            (
                "contract_binding".to_owned(),
                GovernedResponsibility {
                    id: "contract_binding".to_owned(),
                    roles: BTreeMap::from([
                        ("owner".to_owned(), SourceRole::Authority),
                        ("registry".to_owned(), SourceRole::SecondaryRepresentation),
                    ]),
                    precedence: Vec::new(),
                },
            ),
            (
                "provider_binding".to_owned(),
                GovernedResponsibility {
                    id: "provider_binding".to_owned(),
                    roles: BTreeMap::from([("registry".to_owned(), SourceRole::Authority)]),
                    precedence: Vec::new(),
                },
            ),
        ]),
    };
    let object = GovernedObject {
        id: "object".to_owned(),
        responsibilities: BTreeSet::from(["contract_binding".to_owned()]),
        relations: BTreeSet::new(),
    };
    let interfaces = BTreeMap::from([(
        "interface".to_owned(),
        GovernedInterface {
            id: "interface".to_owned(),
            objects: BTreeMap::from([("object".to_owned(), object)]),
        },
    )]);
    let local_catalog = GovernedObjectCatalog {
        repository: repository.to_path_buf(),
        carrier: repository.join("objects.md"),
        model_version: 1,
        interfaces: interfaces.clone(),
    };
    governance_bindings::load(repository, &route, &authority, Some(&local_catalog))
        .expect("same-repository catalog loads");
    let foreign_catalog = GovernedObjectCatalog {
        repository: foreign,
        carrier: repository.join("foreign/objects.md"),
        model_version: 1,
        interfaces,
    };
    assert!(
        governance_bindings::load(repository, &route, &authority, Some(&foreign_catalog)).is_err(),
        "foreign catalog repository must be rejected"
    );
}

fn load_repository_integrity_target(
    repository: &Path,
    catalog_repository: &Path,
) -> Result<
    repository_integrity::ConsumerIntegrityProfile,
    repository_integrity::RepositoryIntegrityError,
> {
    let carrier = repository.join("integrity.md");
    fs::write(
        &carrier,
        "---\nrepository_integrity:\n  model_version: 1\n  authority:\n    responsibility: repository-integrity-profile\n    source: profile-source\n  environments:\n  - python\n  continue_after_non_satisfied: true\n  validations:\n    validation:\n      responsibility: repository-validation\n      prerequisites: []\n      instances:\n        kind: single\n      command:\n        kind: command\n        environment: python\n        arguments: []\n        undetermined_exit_codes: []\n      target:\n        interface: interface\n        object: object\n  order:\n  - validation\n---\n# Carrier\n",
    )
    .expect("integrity carrier is written");
    let route = ResolvedGovernanceRoute {
        route: vec!["profile".to_owned()],
        declared_path: "integrity.md".to_owned(),
        target: carrier,
    };
    let authority = GovernanceAuthorityProfile {
        repository: repository.to_path_buf(),
        carrier: repository.join("authority.md"),
        model_version: 1,
        sources: BTreeMap::from([(
            "profile-source".to_owned(),
            GovernedSource {
                id: "profile-source".to_owned(),
                repository_target: Some(route.clone()),
            },
        )]),
        responsibilities: BTreeMap::from([
            (
                "repository-integrity-profile".to_owned(),
                GovernedResponsibility {
                    id: "repository-integrity-profile".to_owned(),
                    roles: BTreeMap::from([("profile-source".to_owned(), SourceRole::Authority)]),
                    precedence: Vec::new(),
                },
            ),
            (
                "repository-validation".to_owned(),
                GovernedResponsibility {
                    id: "repository-validation".to_owned(),
                    roles: BTreeMap::new(),
                    precedence: Vec::new(),
                },
            ),
        ]),
    };
    let catalog = GovernedObjectCatalog {
        repository: catalog_repository.to_path_buf(),
        carrier: catalog_repository.join("objects.md"),
        model_version: 1,
        interfaces: BTreeMap::from([(
            "interface".to_owned(),
            GovernedInterface {
                id: "interface".to_owned(),
                objects: BTreeMap::from([(
                    "object".to_owned(),
                    GovernedObject {
                        id: "object".to_owned(),
                        responsibilities: BTreeSet::from(["repository-validation".to_owned()]),
                        relations: BTreeSet::new(),
                    },
                )]),
            },
        )]),
    };
    repository_integrity::load(repository, &route, &authority, Some(&catalog))
}

#[test]
fn foreign_governed_objects_catalog_repository_is_rejected_for_repository_integrity() {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    let repository = temporary.path();
    let foreign = repository.join("foreign");
    fs::create_dir(&foreign).expect("foreign repository directory exists");
    assert!(
        load_repository_integrity_target(repository, &foreign).is_err(),
        "foreign catalog repository must be rejected"
    );
}

#[test]
fn same_repository_governed_objects_catalog_target_is_admitted_for_repository_integrity() {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    let repository = temporary.path();
    load_repository_integrity_target(repository, repository)
        .expect("same-repository catalog target must load");
}

fn load_evidence_requirements_target(
    repository: &Path,
    catalog_repository: &Path,
) -> Result<
    evidence_requirements::EvidenceRequirementRegistry,
    evidence_requirements::EvidenceRequirementsError,
> {
    let carrier = repository.join("evidence.md");
    fs::write(
        &carrier,
        "---\nevidence_requirements:\n  model_version: 1\n  authority:\n    responsibility: evidence-registry\n    source: registry\n  requirements:\n    gate-a:\n      responsibility: evidence-required\n      instances:\n        kind: single\n      evidence_classes:\n        kind: explicit\n        classes:\n        - check\n      subject:\n        source: owner\n      context:\n        required: false\n      candidates:\n        source: candidates\n      target:\n        interface: interface\n        object: object\n---\n# Carrier\n",
    )
    .expect("evidence carrier is written");
    let route = ResolvedGovernanceRoute {
        route: vec!["registry".to_owned()],
        declared_path: "evidence.md".to_owned(),
        target: carrier,
    };
    let authority = GovernanceAuthorityProfile {
        repository: repository.to_path_buf(),
        carrier: repository.join("authority.md"),
        model_version: 1,
        sources: BTreeMap::from([
            (
                "candidates".to_owned(),
                GovernedSource {
                    id: "candidates".to_owned(),
                    repository_target: None,
                },
            ),
            (
                "owner".to_owned(),
                GovernedSource {
                    id: "owner".to_owned(),
                    repository_target: None,
                },
            ),
            (
                "registry".to_owned(),
                GovernedSource {
                    id: "registry".to_owned(),
                    repository_target: Some(route.clone()),
                },
            ),
        ]),
        responsibilities: BTreeMap::from([
            (
                "evidence-registry".to_owned(),
                GovernedResponsibility {
                    id: "evidence-registry".to_owned(),
                    roles: BTreeMap::from([("registry".to_owned(), SourceRole::Authority)]),
                    precedence: Vec::new(),
                },
            ),
            (
                "evidence-required".to_owned(),
                GovernedResponsibility {
                    id: "evidence-required".to_owned(),
                    roles: BTreeMap::from([("owner".to_owned(), SourceRole::Authority)]),
                    precedence: Vec::new(),
                },
            ),
        ]),
    };
    let catalog = GovernedObjectCatalog {
        repository: catalog_repository.to_path_buf(),
        carrier: catalog_repository.join("objects.md"),
        model_version: 1,
        interfaces: BTreeMap::from([(
            "interface".to_owned(),
            GovernedInterface {
                id: "interface".to_owned(),
                objects: BTreeMap::from([(
                    "object".to_owned(),
                    GovernedObject {
                        id: "object".to_owned(),
                        responsibilities: BTreeSet::from(["evidence-required".to_owned()]),
                        relations: BTreeSet::new(),
                    },
                )]),
            },
        )]),
    };
    evidence_requirements::load(repository, &route, &authority, Some(&catalog))
}

#[test]
fn foreign_governed_objects_catalog_repository_is_rejected() {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    let repository = temporary.path();
    let foreign = repository.join("foreign");
    fs::create_dir(&foreign).expect("foreign repository directory exists");
    assert!(
        load_evidence_requirements_target(repository, &foreign).is_err(),
        "foreign catalog repository must be rejected"
    );
}

#[test]
fn same_repository_governed_objects_catalog_target_is_admitted() {
    let temporary = tempfile::tempdir().expect("temporary repository exists");
    let repository = temporary.path();
    load_evidence_requirements_target(repository, repository)
        .expect("same-repository catalog target must load");
}

#[test]
fn invalid_foreign_catalog_transport_is_a_harness_error() {
    let corpus = load_corpus(&root()).expect("published corpus loads");
    for (responsibility_id, vector_id) in [
        (
            "governance-bindings.registry",
            "governance-bindings.registry.foreign-catalog-repository",
        ),
        (
            "repository-integrity.profile",
            "repository-integrity.profile.foreign-catalog-repository",
        ),
        (
            "evidence-requirements.registry",
            "evidence-requirements.registry.foreign-catalog-repository",
        ),
    ] {
        let vector = corpus
            .matrices
            .iter()
            .flat_map(|matrix| &matrix.vectors)
            .find(|vector| vector.vector_id == vector_id)
            .expect("foreign catalog vector exists");
        for replacement in [
            serde_json::json!({"type": "string", "value": "bindings"}),
            serde_json::json!({"type": "integer", "value": "1"}),
        ] {
            let mut fixture = vector.fixture.clone();
            let entries = fixture["invocation"]["value"]
                .as_array_mut()
                .expect("invocation is a record");
            let selector = entries
                .iter_mut()
                .find(|entry| entry["name"] == "foreign_support_repository")
                .expect("foreign support selector exists");
            selector["value"] = replacement;
            let request = CandidateRequest {
                responsibility_id: responsibility_id.to_owned(),
                vector_id: "issue66.invalid-foreign-catalog-transport".to_owned(),
                fixture,
            };
            assert!(
                ReferenceRustCandidate.execute(&request).is_err(),
                "invalid transport must remain a harness failure"
            );
        }
    }
}
