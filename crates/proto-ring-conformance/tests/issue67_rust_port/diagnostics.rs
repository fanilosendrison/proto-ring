use proto_ring_conformance::harness::{CandidateRequest, CandidateState};
use proto_ring_conformance::model::Observation;

pub(crate) fn assert_all_match(
    requests: &[CandidateRequest],
    actual: &[CandidateState],
    expected: &[Observation],
) {
    let mismatches: Vec<_> = requests
        .iter()
        .zip(actual)
        .zip(expected)
        .filter_map(|((request, state), expected)| {
            (state != &CandidateState::Observation(expected.clone())).then_some((
                request.vector_id.as_str(),
                state,
                expected,
            ))
        })
        .collect();

    assert!(
        mismatches.is_empty(),
        "RepositoryState vector mismatches: {mismatches:#?}"
    );
}
