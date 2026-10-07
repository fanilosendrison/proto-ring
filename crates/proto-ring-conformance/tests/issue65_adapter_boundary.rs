#![forbid(unsafe_code)]

use proto_ring_conformance::harness::{CandidateExecutor, CandidateState};
use proto_ring_conformance::model::ObservationKind;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;

#[path = "issue65_adapter_boundary/cases.rs"]
mod cases;

#[test]
fn malformed_issue65_adapter_shapes_are_harness_errors() {
    let requests = cases::malformed_requests();
    assert_eq!(requests.len(), 23);
    let mut harness_errors = 0;
    for request in requests {
        match ReferenceRustCandidate.execute(&request) {
            Err(_) => harness_errors += 1,
            Ok(CandidateState::Observation(observation)) => {
                assert_ne!(observation.kind, ObservationKind::ControlledRejection);
                panic!("{} unexpectedly produced an observation", request.vector_id);
            }
            Ok(CandidateState::Unimplemented) => panic!("{} was unimplemented", request.vector_id),
        }
    }
    assert_eq!(harness_errors, 23);
    eprintln!("ISSUE65_ADAPTER_MALFORMED_SHAPE_TESTS=23/23");
    eprintln!("ISSUE65_ADAPTER_MALFORMED_SHAPE_HARNESS_ERRORS=23/23");
    eprintln!("ISSUE65_ADAPTER_MALFORMED_SHAPE_CONTROLLED_REJECTIONS=0");
}
