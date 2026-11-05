use super::super::backend::CapturePhase::*;
use super::super::control::*;
use super::super::*;
use super::script::*;
use std::path::Path;

#[test]
fn facts_have_one_classification_in_both_adaptations() {
    for (started, success, code, expected_status) in [
        (true, true, Some("0"), ObligationStatus::Satisfied),
        (true, false, Some("1"), ObligationStatus::Violated),
        (true, false, Some("7"), ObligationStatus::Undetermined),
        (true, false, None, ObligationStatus::Violated),
        (false, false, None, ObligationStatus::Undetermined),
    ] {
        let profile = profile(&[1]);
        let mut steps = capture(Baseline, 1);
        steps.extend(capture(PreObligation, 1));
        if started {
            steps.extend(process(position(0, 0), success, code));
        } else {
            steps.extend([
                checkpoint(CoherenceCheckpoint::BeforeEachLaunch),
                Step::Launch(position(0, 0), Ok(false)),
            ]);
        }
        steps.extend(capture(PostObligation, 1));
        steps.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
        let expected = expected_result(
            vec![validation(
                "v0",
                expected_status,
                vec![expected_obligation("v0", expected_status, started, code)],
            )],
            if expected_status == ObligationStatus::Satisfied {
                IntegrityVerdict::Pass
            } else {
                IntegrityVerdict::NonPass
            },
            Some(1),
            Some(1),
            StateRelation::Equal,
            false,
        );
        assert_eq!(super::adapters::ordinary(&profile, steps.clone()), expected);
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
            GuardedEvaluation::Completed { result: expected }
        );
        assert_eq!(control.effects_possible(), started);
        backend.finished();
    }
}

#[test]
fn unknown_start_wait_and_uncontained_effects_never_become_results() {
    for reason_kind in [
        HostAbortReason::ProcessStartIndeterminate,
        HostAbortReason::ProcessCompletionIndeterminate,
        HostAbortReason::DescendantEffectsUncontained,
    ] {
        let mut steps = capture(Baseline, 1);
        steps.extend(capture(PreObligation, 1));
        steps.push(checkpoint(CoherenceCheckpoint::BeforeEachLaunch));
        let reason = abort(reason_kind, ProcessStartKnowledge::PotentiallyOccurred);
        if reason_kind == HostAbortReason::ProcessStartIndeterminate {
            steps.push(Step::Launch(position(0, 0), Err(reason.clone())));
        } else {
            steps.push(Step::Launch(position(0, 0), Ok(true)));
            steps.push(Step::Complete(position(0, 0), Err(reason.clone())));
        }
        let mut backend = Script::new(steps);
        let mut control = control();
        assert_eq!(
            evaluate_guarded(
                Path::new("."),
                &profile(&[1, 1]),
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
}

#[test]
fn missing_or_known_unlaunchable_realizations_only_observe_pre_and_post() {
    for missing in [false, true] {
        let mut profile = profile(&[1]);
        if missing {
            profile.validations[0].obligations[0].process_environment = None;
        } else {
            profile.validations[0].obligations[0].command_prefix.clear();
        }
        let mut steps = capture(Baseline, 1);
        steps.extend(capture(PreObligation, 1));
        steps.extend(capture(PostObligation, 1));
        steps.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
        let expected = expected_result(
            vec![validation(
                "v0",
                ObligationStatus::Undetermined,
                vec![expected_obligation(
                    "v0",
                    ObligationStatus::Undetermined,
                    false,
                    None,
                )],
            )],
            IntegrityVerdict::NonPass,
            Some(1),
            Some(1),
            StateRelation::Equal,
            false,
        );
        assert_eq!(super::adapters::ordinary(&profile, steps.clone()), expected);
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
            GuardedEvaluation::Completed { result: expected }
        );
        assert!(!control.effects_possible());
        backend.finished();
    }
}
