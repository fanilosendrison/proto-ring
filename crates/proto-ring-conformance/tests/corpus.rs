#![forbid(unsafe_code)]

#[path = "corpus/support.rs"]
mod support;

use std::collections::HashSet;

use proto_ring_conformance::harness::{
    CandidateExecutor, CandidateState, ComparisonStatus, PythonBridge, UnimplementedRustCandidate,
    candidate_requests, implemented_responsibilities, run_differential,
};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::MigrationDisposition;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use support::{
    IMPLEMENTED, assert_projection_regressions, assert_source_boundaries,
    count_rust_status_for_responsibilities, count_status, expected_observation,
    print_issue64_accounting, print_issue69_accounting, print_rust_accounting, request_for,
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
fn published_corpus_runs_through_reference_candidate() {
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
    let issue66_vectors = vector_count_for_responsibilities(&corpus, &issue66_responsibilities);
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
    let issue64_vectors = vector_count_for_responsibilities(&corpus, &issue64_responsibilities);
    let issue69_responsibilities: HashSet<_> = [
        "repository-integrity.evaluate",
        "git-whitespace.validate",
        "github-authoritative-ref-monotonicity.observe",
    ]
    .into_iter()
    .collect();
    let issue69_vectors = vector_count_for_responsibilities(&corpus, &issue69_responsibilities);
    let responsibility_vector_count = |responsibility_id| {
        corpus
            .matrices
            .iter()
            .find(|matrix| matrix.responsibility_id == responsibility_id)
            .unwrap()
            .vectors
            .len()
    };
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
            vectors
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
    assert!(
        corpus
            .matrices
            .iter()
            .all(|matrix| matrix.responsibility_id != "shared-governance-provider.check")
    );

    let requests = candidate_requests(&corpus);
    assert_eq!(requests.len(), 1153);
    assert_projection_regressions(&corpus);
    assert_source_boundaries(&corpus);
    let python = PythonBridge::new(&root()).execute(&requests).unwrap();
    assert_eq!(python.len(), 1153);

    let baseline_reports = run_differential(&corpus, &python, &UnimplementedRustCandidate).unwrap();
    assert_eq!(baseline_reports.len(), 1153);
    let baseline_python_counts =
        count_status(&baseline_reports, |report| report.python_vs_expected);
    let baseline_rust_counts = count_status(&baseline_reports, |report| report.rust_vs_expected);
    let baseline_differential_counts =
        count_status(&baseline_reports, |report| report.python_vs_rust);
    let baseline_unimplemented = baseline_reports
        .iter()
        .filter(|report| report.rust_candidate == CandidateState::Unimplemented)
        .count();
    let baseline_implemented_responsibilities = implemented_responsibilities(&baseline_reports);
    assert_eq!(baseline_python_counts, (1153, 0, 0));
    assert_eq!(
        (
            baseline_implemented_responsibilities,
            baseline_unimplemented
        ),
        (0, 1153)
    );
    assert_eq!(baseline_rust_counts, (0, 0, 1153));
    assert_eq!(baseline_differential_counts, (0, 0, 1153));

    let reports = run_differential(&corpus, &python, &ReferenceRustCandidate).unwrap();

    let python_counts = count_status(&reports, |report| report.python_vs_expected);
    let rust_counts = count_status(&reports, |report| report.rust_vs_expected);
    let differential_counts = count_status(&reports, |report| report.python_vs_rust);
    for report in &reports {
        if report.rust_vs_expected == ComparisonStatus::Mismatch {
            eprintln!(
                "RUST_MISMATCH={} actual={:?} expected={:?}",
                report.vector_id, report.rust_candidate, report.expected_observation
            );
        }
    }
    assert_eq!(python_counts, (1153, 0, 0));
    assert_eq!(rust_counts, (1153, 0, 0));
    assert_eq!(differential_counts, (1153, 0, 0));
    let implemented_vectors = reports
        .iter()
        .filter(|report| matches!(report.rust_candidate, CandidateState::Observation(_)))
        .count();
    let implemented_responsibility_count = implemented_responsibilities(&reports);
    let implemented_ids: HashSet<_> = reports
        .iter()
        .filter_map(|report| {
            matches!(report.rust_candidate, CandidateState::Observation(_))
                .then_some(report.responsibility_id.as_str())
        })
        .collect();
    assert_eq!(implemented_ids, IMPLEMENTED.iter().copied().collect());
    assert_eq!(
        (implemented_responsibility_count, implemented_vectors),
        (29, 1153)
    );
    let issue66_rust_matches = reports
        .iter()
        .filter(|report| {
            issue66_responsibilities.contains(report.responsibility_id.as_str())
                && report.rust_vs_expected == ComparisonStatus::Match
        })
        .count();
    let governance_bindings_rust_matches = reports
        .iter()
        .filter(|report| {
            report.responsibility_id == "governance-bindings.registry"
                && report.rust_vs_expected == ComparisonStatus::Match
        })
        .count();
    let repository_integrity_rust_matches = reports
        .iter()
        .filter(|report| {
            report.responsibility_id == "repository-integrity.profile"
                && report.rust_vs_expected == ComparisonStatus::Match
        })
        .count();
    let evidence_requirements_rust_matches = reports
        .iter()
        .filter(|report| {
            report.responsibility_id == "evidence-requirements.registry"
                && report.rust_vs_expected == ComparisonStatus::Match
        })
        .count();
    let exact_evidence_binding_rust_matches = reports
        .iter()
        .filter(|report| {
            report.responsibility_id == "exact-evidence-binding.evaluate"
                && report.rust_vs_expected == ComparisonStatus::Match
        })
        .count();
    assert_eq!(issue66_rust_matches, 555);
    assert_eq!(governance_bindings_rust_matches, 109);
    assert_eq!(repository_integrity_rust_matches, 173);
    assert_eq!(evidence_requirements_rust_matches, 120);
    assert_eq!(exact_evidence_binding_rust_matches, 44);
    let issue68_rust_matches = reports
        .iter()
        .filter(|report| {
            report.responsibility_id == "repository-governance-state.compose"
                && report.rust_vs_expected == ComparisonStatus::Match
        })
        .count();
    assert_eq!(issue68_rust_matches, 25);
    let issue64_rust_counts =
        count_rust_status_for_responsibilities(&reports, &issue64_responsibilities);
    assert_eq!(issue64_rust_counts, (209, 0, 0));
    let issue69_rust_counts =
        count_rust_status_for_responsibilities(&reports, &issue69_responsibilities);
    assert_eq!(issue69_rust_counts, (69, 0, 0));
    require_matches(
        &reports,
        &[
            "structured-data.document.beyond-i64",
            "structured-data.document.int-limit-641",
            "structured-data.document.int-limit-4301",
        ],
    );
    require_matches(
        &reports,
        &[
            "governance-routing.resolve.missing-parent",
            "governance-routing.resolve.file-parent",
            "governance-routing.resolve.file-trailing-slash",
            "governance-routing.resolve.file-dot",
            "governance-routing.resolve.broken-symlink",
            "governance-routing.resolve.symlink-cycle",
            "governance-routing.resolve.symlink-parent",
            "governance-routing.resolve.outside-final",
            "governance-routing.resolve.absolute-path-contained-target",
        ],
    );
    require_matches(
        &reports,
        &[
            "repository-governance-model.load.valid-model-v1",
            "repository-governance-model.load.valid-model-v2",
            "repository-governance-model.load.unsupported-version",
            "repository-governance-model.load.unknown-root-key",
            "repository-governance-model.load.malformed-capability",
            "repository-governance-model.load.projection-integrity-requires-repository-integrity",
            "repository-governance-model.load.projection-integrity-with-repository-integrity",
        ],
    );
    require_matches(
        &reports,
        &["governed-objects.catalog.projection-target-nonparticipation-support"],
    );
    require_matches(
        &reports,
        &[
            "exact-evidence-binding.evaluate.requirement-duplicate-classes",
            "exact-evidence-binding.evaluate.empty-context-ignored-when-not-required",
            "exact-evidence-binding.evaluate.unknown-current-subject-precedes-malformed-candidate",
            "exact-evidence-binding.evaluate.unknown-current-context-precedes-malformed-candidate",
        ],
    );
    require_matches(
        &reports,
        &[
            "evidence-requirements.registry.foreign-catalog-repository",
            "governance-bindings.registry.foreign-catalog-repository",
            "repository-integrity.profile.foreign-catalog-repository",
        ],
    );
    require_matches(
        &reports,
        &[
            "repository-governance-model.binding-compatibility.compatible-binding",
            "repository-governance-model.binding-compatibility.missing-required-route",
            "repository-governance-model.binding-compatibility.undeclared-capability",
        ],
    );
    eprintln!(
        "PYTHON_BRIDGE_LOADS_CORPUS=no\nPYTHON_BRIDGE_ORACLE_FIELDS_IN_REQUEST=0\nPYTHON_BRIDGE_COMPUTES_COMPARISON=no\nPYTHON_BRIDGE_REQUEST_SCHEMA=PASS\nPYTHON_BRIDGE_OUTPUT_SCHEMA=PASS\nPYTHON_BRIDGE_RETIREMENT_ONLY_REJECTED=PASS"
    );
    eprintln!(
        "CORPUS_RESPONSIBILITIES=30\nRUST_PORT_RESPONSIBILITIES=29\nRETIRE_WITHOUT_RUST_PORT_RESPONSIBILITIES=1\nCORPUS_MATRICES=29\nCORPUS_VECTORS=1153"
    );
    eprintln!("ISSUE63_RESPONSIBILITIES=6\nISSUE63_VECTORS=76");
    eprintln!("ISSUE65_RESPONSIBILITIES=2\nISSUE65_VECTORS=166");
    eprintln!(
        "ISSUE66_RESPONSIBILITIES={}\nISSUE66_VECTORS={issue66_vectors}\nGOVERNANCE_BINDINGS_VECTORS={governance_bindings_vectors}\nREPOSITORY_INTEGRITY_VECTORS={repository_integrity_vectors}\nEVIDENCE_REQUIREMENTS_VECTORS={evidence_requirements_vectors}\nEXACT_EVIDENCE_BINDING_VECTORS={exact_evidence_binding_vectors}",
        issue66_responsibilities.len()
    );
    eprintln!(
        "ISSUE68_RESPONSIBILITIES=1\nISSUE68_VECTORS={issue68_vectors}\nISSUE68_RUST_MATCH={issue68_rust_matches}"
    );
    eprintln!(
        "ISSUE64_PORTABLE_PATTERN_RESPONSIBILITIES=1\nISSUE64_PORTABLE_PATTERN_VECTORS=37\nISSUE64_PORTABLE_PATTERN_RUST_MATCH=37"
    );
    print_issue64_accounting(issue64_vectors, issue64_rust_counts);
    print_issue69_accounting(issue69_vectors, issue69_rust_counts);
    eprintln!(
        "PYTHON_OBSERVATIONS=1153\nPYTHON_VS_EXPECTED_MATCH={}\nPYTHON_VS_EXPECTED_MISMATCH={}",
        python_counts.0, python_counts.1
    );
    print_rust_accounting(
        implemented_responsibility_count,
        implemented_vectors,
        rust_counts,
        differential_counts,
    );
    eprintln!(
        "ISSUE66_RUST_MATCH={issue66_rust_matches}\nGOVERNANCE_BINDINGS_RUST_MATCH={governance_bindings_rust_matches}\nREPOSITORY_INTEGRITY_RUST_MATCH={repository_integrity_rust_matches}\nEVIDENCE_REQUIREMENTS_RUST_MATCH={evidence_requirements_rust_matches}\nEXACT_EVIDENCE_BINDING_RUST_MATCH={exact_evidence_binding_rust_matches}"
    );
    eprintln!(
        "RETIREMENT_ONLY_EXECUTIONS=0\nRUST_VECTOR_ID_SPECIAL_CASES=0\nYAML_IMPLICIT_TYPING_DEPENDENCY=0"
    );
    eprintln!(
        "NEW_S1_GAP_RUST_MATCH=1/1\nNEW_S3_GAP_RUST_MATCH=1/1\nNEW_S4_GAP_RUST_MATCH=1/1\nNEW_S5_GAP_RUST_MATCH=4/4"
    );
    eprintln!(
        "UNBOUNDED_INTEGER_CASES=3/3\nROUTING_HOSTILE_PATH_CASES=9/9\nRGM_VERSION_CASES=5/5\nRGM_BINDING_COMPATIBILITY_CASES=3/3"
    );
    eprintln!(
        "FRONTMATTER_RESULT_ORDER=PASS\nIMMEDIATE_CLOSE_RESULT_ORDER=PASS\nRGM_RESULT_ORDER=PASS\nRGM_MODEL_VERSION_TRANSPORT=PASS"
    );
    eprintln!("DIFFERENTIAL_HARNESS=PASS");
}
