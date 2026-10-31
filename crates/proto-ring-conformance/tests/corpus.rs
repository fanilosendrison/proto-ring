#![forbid(unsafe_code)]

#[path = "corpus/support.rs"]
mod support;

use std::collections::HashSet;

use proto_ring_conformance::harness::{CandidateExecutor, CandidateState};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::MigrationDisposition;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_conformance::standalone::{StandaloneVerdict, run_standalone};
use support::{
    IMPLEMENTED, assert_projection_regressions, assert_source_boundaries, expected_observation,
    match_count_for_responsibilities, print_responsibility_accounting, request_for,
    require_matches, root, vector_count_for_responsibilities,
};

#[test]
fn absolute_contained_target_is_rejected_by_reference_candidate() {
    let corpus = load_corpus(&root()).unwrap();
    let id = "governance-routing.resolve.absolute-path-contained-target";
    let state = ReferenceRustCandidate
        .execute(&request_for(&corpus, id))
        .unwrap();
    assert_eq!(
        state,
        CandidateState::Observation(expected_observation(&corpus, id))
    );
}

#[test]
fn published_corpus_runs_through_standalone_rust_control() {
    let corpus = load_corpus(&root()).unwrap();
    let rust_port = corpus
        .responsibilities
        .iter()
        .filter(|record| record.migration_disposition == MigrationDisposition::RustPort)
        .count();
    let retired: Vec<_> = corpus
        .responsibilities
        .iter()
        .filter(|record| {
            record.migration_disposition == MigrationDisposition::RetireWithoutRustPort
        })
        .collect();
    let vectors: usize = corpus
        .matrices
        .iter()
        .map(|matrix| matrix.vectors.len())
        .sum();
    let issue66_responsibilities: HashSet<_> = [
        "evidence-requirements.registry",
        "exact-evidence-binding.evaluate",
        "governance-bindings.registry",
        "projection-registry.registry",
        "repository-integrity.profile",
    ]
    .into_iter()
    .collect();
    let issue64_responsibilities: HashSet<_> = [
        "accepted-adr-body.immutability",
        "adr-metadata.parse",
        "adr-metadata.repository-path",
        "adr-metadata.validation",
        "canonical-adr.resolve",
        "normative-terminology.definition-discovery",
        "normative-terminology.fingerprint",
        "normative-terminology.inventory",
        "normative-terminology.markdown",
        "normative-terminology.registry",
        "portable-pattern.full-match",
    ]
    .into_iter()
    .collect();
    let issue69_responsibilities: HashSet<_> = [
        "repository-integrity.evaluate",
        "git-whitespace.validate",
        "github-authoritative-ref-monotonicity.observe",
    ]
    .into_iter()
    .collect();
    let responsibility_vector_count = |responsibility_id| {
        corpus
            .matrices
            .iter()
            .find(|matrix| matrix.responsibility_id == responsibility_id)
            .unwrap()
            .vectors
            .len()
    };
    let issue66_vectors = vector_count_for_responsibilities(&corpus, &issue66_responsibilities);
    let issue64_vectors = vector_count_for_responsibilities(&corpus, &issue64_responsibilities);
    let issue69_vectors = vector_count_for_responsibilities(&corpus, &issue69_responsibilities);
    let governance_bindings_vectors = responsibility_vector_count("governance-bindings.registry");
    let repository_integrity_vectors = responsibility_vector_count("repository-integrity.profile");
    let evidence_requirements_vectors =
        responsibility_vector_count("evidence-requirements.registry");
    let exact_evidence_binding_vectors =
        responsibility_vector_count("exact-evidence-binding.evaluate");
    let issue68_vectors = responsibility_vector_count("repository-governance-state.compose");

    assert_eq!(issue66_vectors, 555);
    assert_eq!(governance_bindings_vectors, 109);
    assert_eq!(repository_integrity_vectors, 173);
    assert_eq!(evidence_requirements_vectors, 120);
    assert_eq!(exact_evidence_binding_vectors, 44);
    assert_eq!(issue64_vectors, 209);
    assert_eq!(issue68_vectors, 25);
    assert_eq!(issue69_vectors, 69);
    assert_eq!(
        (
            corpus.responsibilities.len(),
            rust_port,
            retired.len(),
            corpus.matrices.len(),
            vectors,
        ),
        (30, 29, 1, 29, 1153)
    );

    let vector_ids: HashSet<_> = corpus
        .matrices
        .iter()
        .flat_map(|matrix| matrix.vectors.iter())
        .map(|vector| vector.vector_id.as_str())
        .collect();
    for mandatory_id in [
        "accepted-adr-body.immutability.non-git-root",
        "adr-metadata.parse.bare-cr-rejected",
        "adr-metadata.parse.crlf-rejected",
        "adr-metadata.parse.utf8-bom-rejected",
        "adr-metadata.validation.both-valid-schemas-both-fail",
        "adr-metadata.validation.duplicate-string-list",
        "adr-metadata.validation.empty-string-rejected",
        "adr-metadata.validation.malformed-base-overlay-still-evaluated",
        "adr-metadata.validation.malformed-base-overlay-success",
        "adr-metadata.validation.malformed-overlay-base-still-evaluated",
        "adr-metadata.validation.malformed-overlay-base-success",
        "normative-terminology.definition-discovery.equation-colon-equals",
        "normative-terminology.definition-discovery.equation-double-equals-excluded",
        "normative-terminology.definition-discovery.equation-single-equals",
        "normative-terminology.registry.cross-entry-alias-reuse",
        "normative-terminology.registry.intra-entry-casefold-duplicate",
        "portable-pattern.full-match.bound-over-decimal-runtime-limit",
        "portable-pattern.full-match.bound-over-u64",
        "portable-pattern.full-match.end-anchor-trailing-lf",
    ] {
        assert!(vector_ids.contains(mandatory_id), "missing {mandatory_id}");
    }
    assert_eq!(
        retired[0].responsibility_id,
        "shared-governance-provider.check"
    );
    assert!(retired[0].case_file.is_none());
    assert_eq!(corpus.index.case_files.len(), 29);

    assert_projection_regressions(&corpus);
    assert_source_boundaries(&corpus);
    let records = run_standalone(&corpus, &ReferenceRustCandidate)
        .expect("standalone Rust control must conform");
    let implemented_ids: HashSet<_> = records
        .iter()
        .map(|record| record.responsibility_id.as_str())
        .collect();
    assert_eq!(implemented_ids, IMPLEMENTED.iter().copied().collect());
    assert_eq!(records.len(), 1153);
    assert!(
        records
            .iter()
            .all(|record| record.verdict == StandaloneVerdict::Match)
    );

    let issue66_matches = match_count_for_responsibilities(&records, &issue66_responsibilities);
    let issue64_matches = match_count_for_responsibilities(&records, &issue64_responsibilities);
    let issue69_matches = match_count_for_responsibilities(&records, &issue69_responsibilities);
    assert_eq!(issue66_matches, 555);
    assert_eq!(issue64_matches, 209);
    assert_eq!(issue69_matches, 69);
    assert_eq!(
        records
            .iter()
            .filter(|record| record.responsibility_id == "repository-governance-state.compose")
            .count(),
        25
    );

    require_matches(
        &records,
        &[
            "structured-data.document.beyond-i64",
            "structured-data.document.int-limit-641",
            "structured-data.document.int-limit-4301",
            "governance-routing.resolve.missing-parent",
            "governance-routing.resolve.file-parent",
            "governance-routing.resolve.file-trailing-slash",
            "governance-routing.resolve.file-dot",
            "governance-routing.resolve.broken-symlink",
            "governance-routing.resolve.symlink-cycle",
            "governance-routing.resolve.symlink-parent",
            "governance-routing.resolve.outside-final",
            "governance-routing.resolve.absolute-path-contained-target",
            "repository-governance-model.load.valid-model-v1",
            "repository-governance-model.load.valid-model-v2",
            "repository-governance-model.load.unsupported-version",
            "repository-governance-model.load.unknown-root-key",
            "repository-governance-model.load.malformed-capability",
            "repository-governance-model.load.projection-integrity-requires-repository-integrity",
            "repository-governance-model.load.projection-integrity-with-repository-integrity",
            "governed-objects.catalog.projection-target-nonparticipation-support",
            "exact-evidence-binding.evaluate.requirement-duplicate-classes",
            "exact-evidence-binding.evaluate.empty-context-ignored-when-not-required",
            "exact-evidence-binding.evaluate.unknown-current-subject-precedes-malformed-candidate",
            "exact-evidence-binding.evaluate.unknown-current-context-precedes-malformed-candidate",
            "evidence-requirements.registry.foreign-catalog-repository",
            "governance-bindings.registry.foreign-catalog-repository",
            "repository-integrity.profile.foreign-catalog-repository",
            "repository-governance-model.binding-compatibility.compatible-binding",
            "repository-governance-model.binding-compatibility.missing-required-route",
            "repository-governance-model.binding-compatibility.undeclared-capability",
        ],
    );

    eprintln!(
        "CORPUS_RESPONSIBILITIES=30\nRUST_PORT_RESPONSIBILITIES=29\nRETIRE_WITHOUT_RUST_PORT_RESPONSIBILITIES=1\nCORPUS_MATRICES=29\nCORPUS_VECTORS=1153"
    );
    eprintln!(
        "RUST_IMPLEMENTED_RESPONSIBILITIES=29\nRUST_IMPLEMENTED_VECTORS=1153\nRUST_UNIMPLEMENTED_RESPONSIBILITIES=0\nRUST_UNIMPLEMENTED_VECTORS=0\nRUST_VS_EXPECTED_MATCH=1153\nRUST_VS_EXPECTED_MISMATCH=0\nRUST_VS_EXPECTED_NOT_AVAILABLE=0"
    );
    print_responsibility_accounting("ISSUE64", issue64_vectors, issue64_matches);
    print_responsibility_accounting("ISSUE69", issue69_vectors, issue69_matches);
    eprintln!(
        "ISSUE66_RESPONSIBILITIES=5\nISSUE66_VECTORS={issue66_vectors}\nISSUE66_RUST_MATCH={issue66_matches}"
    );
    eprintln!("ACTIVE_PYTHON_BRIDGE_EXECUTIONS=0");
    eprintln!("STANDALONE_RUST_CONTROL=PASS");
}
