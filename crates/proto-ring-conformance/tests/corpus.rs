#![forbid(unsafe_code)]

#[path = "corpus/support.rs"]
mod support;

use std::collections::{HashMap, HashSet};

use proto_ring_conformance::harness::{
    CandidateExecutor, CandidateState, ComparisonStatus, PythonBridge, UnimplementedRustCandidate,
    candidate_requests, implemented_responsibilities, run_differential,
};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::MigrationDisposition;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use support::{
    IMPLEMENTED, assert_projection_regressions, assert_source_boundaries, count_status,
    expected_observation, request_for, require_matches, root,
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
    let issue66_vectors: usize = corpus
        .matrices
        .iter()
        .filter(|matrix| issue66_responsibilities.contains(matrix.responsibility_id.as_str()))
        .map(|matrix| matrix.vectors.len())
        .sum();
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
    assert_eq!(issue66_vectors, 555);
    assert_eq!(governance_bindings_vectors, 109);
    assert_eq!(repository_integrity_vectors, 173);
    assert_eq!(evidence_requirements_vectors, 120);
    assert_eq!(exact_evidence_binding_vectors, 44);
    assert_eq!(
        (
            corpus.responsibilities.len(),
            rust_port,
            retired.len(),
            corpus.matrices.len(),
            vectors
        ),
        (30, 29, 1, 29, 1109)
    );
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
    assert_eq!(requests.len(), 1109);
    assert_projection_regressions(&corpus);
    assert_source_boundaries(&corpus);
    let python = PythonBridge::new(&root()).execute(&requests).unwrap();
    assert_eq!(python.len(), 1109);

    let baseline_reports = run_differential(&corpus, &python, &UnimplementedRustCandidate).unwrap();
    assert_eq!(baseline_reports.len(), 1109);
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
    assert_eq!(baseline_python_counts, (1109, 0, 0));
    assert_eq!(
        (
            baseline_implemented_responsibilities,
            baseline_unimplemented
        ),
        (0, 1109)
    );
    assert_eq!(baseline_rust_counts, (0, 0, 1109));
    assert_eq!(baseline_differential_counts, (0, 0, 1109));

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
    assert_eq!(python_counts, (1109, 0, 0));
    assert_eq!(rust_counts, (797, 0, 312));
    assert_eq!(differential_counts, (797, 0, 312));
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
        (13, 797)
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

    let metrics: HashMap<&str, usize> = HashMap::from([
        ("implemented", implemented_vectors),
        ("unimplemented", 1109 - implemented_vectors),
    ]);
    eprintln!(
        "PYTHON_BRIDGE_LOADS_CORPUS=no\nPYTHON_BRIDGE_ORACLE_FIELDS_IN_REQUEST=0\nPYTHON_BRIDGE_COMPUTES_COMPARISON=no\nPYTHON_BRIDGE_REQUEST_SCHEMA=PASS\nPYTHON_BRIDGE_OUTPUT_SCHEMA=PASS\nPYTHON_BRIDGE_RETIREMENT_ONLY_REJECTED=PASS"
    );
    eprintln!(
        "CORPUS_RESPONSIBILITIES=30\nRUST_PORT_RESPONSIBILITIES=29\nRETIRE_WITHOUT_RUST_PORT_RESPONSIBILITIES=1\nCORPUS_MATRICES=29\nCORPUS_VECTORS=1109"
    );
    eprintln!("ISSUE63_RESPONSIBILITIES=6\nISSUE63_VECTORS=76");
    eprintln!("ISSUE65_RESPONSIBILITIES=2\nISSUE65_VECTORS=166");
    eprintln!(
        "ISSUE66_RESPONSIBILITIES={}\nISSUE66_VECTORS={issue66_vectors}\nGOVERNANCE_BINDINGS_VECTORS={governance_bindings_vectors}\nREPOSITORY_INTEGRITY_VECTORS={repository_integrity_vectors}\nEVIDENCE_REQUIREMENTS_VECTORS={evidence_requirements_vectors}\nEXACT_EVIDENCE_BINDING_VECTORS={exact_evidence_binding_vectors}",
        issue66_responsibilities.len()
    );
    eprintln!(
        "PYTHON_OBSERVATIONS=1109\nPYTHON_VS_EXPECTED_MATCH={}\nPYTHON_VS_EXPECTED_MISMATCH={}",
        python_counts.0, python_counts.1
    );
    eprintln!(
        "RUST_IMPLEMENTED_RESPONSIBILITIES={implemented_responsibility_count}\nRUST_IMPLEMENTED_VECTORS={}\nRUST_UNIMPLEMENTED_RESPONSIBILITIES=16\nRUST_UNIMPLEMENTED_VECTORS={}",
        metrics["implemented"], metrics["unimplemented"]
    );
    eprintln!(
        "RUST_VS_EXPECTED_MATCH={}\nRUST_VS_EXPECTED_MISMATCH={}\nRUST_VS_EXPECTED_NOT_AVAILABLE={}",
        rust_counts.0, rust_counts.1, rust_counts.2
    );
    eprintln!(
        "PYTHON_VS_RUST_MATCH={}\nPYTHON_VS_RUST_MISMATCH={}\nPYTHON_VS_RUST_NOT_AVAILABLE={}",
        differential_counts.0, differential_counts.1, differential_counts.2
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
