use super::super::backend::CapturePhase::*;
use super::super::control::*;
use super::super::*;
use super::script::*;
use crate::repository_state::StateCaptureError;
use std::path::Path;

fn completed_first() -> Vec<Step> {
    let mut steps = capture(Baseline, 1);
    steps.extend(capture(PreObligation, 1));
    steps.extend(process(position(0, 0), true, Some("0")));
    steps.extend(capture(PostObligation, 1));
    steps
}

#[test]
fn every_checkpoint_capture_launch_and_wait_abort_is_exact_and_terminal() {
    let profile = profile(&[1, 1]);
    let mut full = completed_first();
    full.extend(capture(PreObligation, 1));
    full.extend(process(position(1, 0), true, Some("0")));
    full.extend(capture(PostObligation, 1));
    full.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
    for cut in 0..full.len() {
        for knowledge in [
            ProcessStartKnowledge::DefinitivelyExcluded,
            ProcessStartKnowledge::PotentiallyOccurred,
        ] {
            let reason = abort(
                match &full[cut] {
                    Step::Launch(..) => HostAbortReason::ProcessStartIndeterminate,
                    Step::Complete(..) => HostAbortReason::ProcessCompletionIndeterminate,
                    _ => HostAbortReason::CoherenceLost,
                },
                knowledge,
            );
            let mut steps = full[..cut].to_vec();
            let prior_started = steps.iter().any(|s| matches!(s, Step::Launch(_, Ok(true))));
            steps.push(match &full[cut] {
                Step::Checkpoint(phase, _) => Step::Checkpoint(*phase, Err(reason.clone())),
                Step::Capture(phase, _) => Step::Capture(*phase, Err(reason.clone())),
                Step::Launch(position, _) => Step::Launch(*position, Err(reason.clone())),
                Step::Complete(position, _) => Step::Complete(*position, Err(reason.clone())),
                Step::Discover(_) => unreachable!(),
            });
            let expected_effects =
                prior_started || knowledge == ProcessStartKnowledge::PotentiallyOccurred;
            let mut backend = Script::new(steps);
            let mut control = control();
            assert_eq!(
                evaluate_guarded(
                    Path::new("."),
                    &profile,
                    context_identity(),
                    &mut backend,
                    &mut control
                ),
                GuardedEvaluation::Interrupted {
                    reason,
                    effects_possible: expected_effects
                }
            );
            assert_eq!(control.effects_possible(), expected_effects);
            assert_eq!(control.attempt().as_bytes(), &[42; 32]);
            backend.finished();
        }
    }
}

#[test]
fn second_not_started_then_excluded_abort_preserves_first_process_effects() {
    let profile = profile(&[1, 1]);
    let mut steps = completed_first();
    steps.extend(capture(PreObligation, 1));
    steps.extend([
        checkpoint(CoherenceCheckpoint::BeforeEachLaunch),
        Step::Launch(position(1, 0), Ok(false)),
        checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture),
    ]);
    let reason = abort(
        HostAbortReason::HostObservationUnavailable,
        ProcessStartKnowledge::DefinitivelyExcluded,
    );
    steps.push(Step::Capture(PostObligation, Err(reason.clone())));
    let mut backend = Script::new(steps);
    let mut control = control();
    assert_eq!(
        evaluate_guarded(
            Path::new("."),
            &profile,
            context_identity(),
            &mut backend,
            &mut control
        ),
        GuardedEvaluation::Interrupted {
            reason,
            effects_possible: true
        }
    );
    assert!(control.effects_possible());
    backend.finished();
}

#[test]
fn completed_control_survives_realization_and_a_later_baseline_abort() {
    let mut backend = Script::new({
        let mut s = completed_first();
        s.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
        s
    });
    let mut control = control();
    assert!(matches!(
        evaluate_guarded(
            Path::new("."),
            &profile(&[1]),
            context_identity(),
            &mut backend,
            &mut control
        ),
        GuardedEvaluation::Completed { .. }
    ));
    assert!(control.effects_possible());
    backend.finished();
    let mut backend = Script::new(vec![
        checkpoint(CoherenceCheckpoint::BeforeRealization),
        checkpoint(CoherenceCheckpoint::AfterRealization),
    ]);
    let realized = realize_guarded(
        &super::profiles::root(),
        &super::profiles::consumer(None),
        &super::profiles::context(),
        &mut backend,
        &mut control,
    )
    .unwrap()
    .unwrap();
    assert!(realized.profile.validations.is_empty());
    assert!(control.effects_possible());
    backend.finished();
    let reason = abort(
        HostAbortReason::AdmissionRevoked,
        ProcessStartKnowledge::DefinitivelyExcluded,
    );
    let mut backend = Script::new(vec![
        checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture),
        Step::Capture(Baseline, Err(reason.clone())),
    ]);
    assert_eq!(
        evaluate_guarded(
            Path::new("."),
            &profile(&[]),
            context_identity(),
            &mut backend,
            &mut control
        ),
        GuardedEvaluation::Interrupted {
            reason,
            effects_possible: true
        }
    );
    backend.finished();
}

#[test]
fn final_checkpoint_is_required_on_every_ordinary_early_exit() {
    let mut shifted = profile(&[1]);
    shifted.resolution_state_identity = Some(state(2).identity);
    let mut policy = profile(&[1, 1]);
    policy.continue_after_non_satisfied = false;
    let unavailable = || Ok(Err(StateCaptureError::new("ordinary capture unavailable")));
    let mut pre_error = capture(Baseline, 1);
    pre_error.extend([
        checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture),
        Step::Capture(PreObligation, unavailable()),
    ]);
    let mut pre_drift = capture(Baseline, 1);
    pre_drift.extend(capture(PreObligation, 2));
    let mut post_error = capture(Baseline, 1);
    post_error.extend(capture(PreObligation, 1));
    post_error.extend(process(position(0, 0), true, Some("0")));
    post_error.extend([
        checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture),
        Step::Capture(PostObligation, unavailable()),
    ]);
    let mut post_drift = capture(Baseline, 1);
    post_drift.extend(capture(PreObligation, 1));
    post_drift.extend(process(position(0, 0), true, Some("0")));
    post_drift.extend(capture(PostObligation, 2));
    let mut policy_steps = capture(Baseline, 1);
    policy_steps.extend(capture(PreObligation, 1));
    policy_steps.extend(process(position(0, 0), false, Some("1")));
    policy_steps.extend(capture(PostObligation, 1));
    for (profile, mut steps, effects) in [
        (profile(&[]), capture(Baseline, 1), false),
        (profile(&[0]), capture(Baseline, 1), false),
        (
            profile(&[1]),
            vec![
                checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture),
                Step::Capture(Baseline, unavailable()),
            ],
            false,
        ),
        (shifted, capture(Baseline, 1), false),
        (profile(&[1]), pre_error, false),
        (profile(&[1]), pre_drift, false),
        (profile(&[1]), post_error, true),
        (profile(&[1]), post_drift, true),
        (policy, policy_steps, true),
    ] {
        let reason = abort(
            HostAbortReason::CoherenceLost,
            ProcessStartKnowledge::DefinitivelyExcluded,
        );
        steps.push(Step::Checkpoint(
            CoherenceCheckpoint::AfterFinalRiObservation,
            Err(reason.clone()),
        ));
        let mut backend = Script::new(steps);
        let mut control = control();
        assert_eq!(
            evaluate_guarded(
                Path::new("."),
                &profile,
                context_identity(),
                &mut backend,
                &mut control
            ),
            GuardedEvaluation::Interrupted {
                reason,
                effects_possible: effects
            }
        );
        assert_eq!(control.effects_possible(), effects);
        backend.finished();
    }
}
