//! Run independent expectations through the ordinary adapter as well.
use super::super::backend::{Backend, CapturePhase, ObligationPosition};
use super::super::model::EvaluationCapturePhase;
use super::super::observations::{Launch, ProcessObservation};
use super::super::*;
use super::script::*;
use std::cell::RefCell;
use std::path::Path;

pub fn ordinary(profile: &IntegrityProfile, steps: Vec<Step>) -> IntegrityResult {
    // Ordinary checkpoints are intentionally no-ops. All actual observations and
    // positions must still consume precisely the same non-checkpoint script.
    let backend = RefCell::new(Script::new(
        steps
            .into_iter()
            .filter(|step| !matches!(step, Step::Checkpoint(..)))
            .collect(),
    ));
    let validation = std::cell::Cell::new((0, 0));
    let result = evaluate::evaluate_with(
        Path::new("."),
        profile,
        context_identity(),
        |phase, root, scope| {
            backend
                .borrow_mut()
                .capture(
                    match phase {
                        EvaluationCapturePhase::Baseline => CapturePhase::Baseline,
                        EvaluationCapturePhase::PreObligation => CapturePhase::PreObligation,
                        EvaluationCapturePhase::PostObligation => CapturePhase::PostObligation,
                    },
                    root,
                    scope,
                )
                .expect("ordinary has no abort")
        },
        |root, obligation| {
            let (mut v, mut i) = validation.get();
            // Position lookup is test-adapter bookkeeping only, not production.
            while profile.validations[v].id != obligation.validation_id {
                v += 1;
                i = 0;
            }
            let position = ObligationPosition {
                validation: v,
                instance: i,
            };
            validation.set((v, i + 1));
            let mut backend = backend.borrow_mut();
            match backend
                .launch(root, position, obligation)
                .expect("ordinary launch")
            {
                Launch::NotStarted => ProcessObservation::NotStarted,
                Launch::Started(()) => ProcessObservation::Completed(
                    backend.complete((), position).expect("ordinary completion"),
                ),
            }
        },
    );
    backend.borrow().finished();
    result
}
