use super::super::backend::CapturePhase::*;
use super::super::control::*;
use super::super::*;
use super::script::*;
use crate::repository_state::StateCaptureError;
use std::path::Path;

#[test]
fn ordinary_capture_errors_preserve_complete_results_at_every_phase() {
    for phase in [Baseline, PreObligation, PostObligation] {
        let profile = profile(&[1, 1]);
        let mut steps = vec![];
        if phase != Baseline {
            steps.extend(capture(Baseline, 1));
        }
        if phase == PostObligation {
            steps.extend(capture(PreObligation, 1));
            steps.extend(process(position(0, 0), true, Some("0")));
        }
        steps.extend([
            checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture),
            Step::Capture(
                phase,
                Ok(Err(StateCaptureError::new("ordinary unavailable"))),
            ),
            checkpoint(CoherenceCheckpoint::AfterFinalRiObservation),
        ]);
        let ordinary_steps = steps.clone();
        let mut backend = Script::new(steps);
        let mut control = control();
        let actual = evaluate_guarded(
            Path::new("."),
            &profile,
            context_identity(),
            &mut backend,
            &mut control,
        );
        let validations = vec![
            validation(
                "v0",
                ObligationStatus::Undetermined,
                vec![expected_obligation(
                    "v0",
                    ObligationStatus::Undetermined,
                    phase == PostObligation,
                    (phase == PostObligation).then_some("0"),
                )],
            ),
            validation(
                "v1",
                ObligationStatus::Undetermined,
                vec![expected_obligation(
                    "v1",
                    ObligationStatus::Undetermined,
                    false,
                    None,
                )],
            ),
        ];
        let expected = expected_result(
            validations,
            IntegrityVerdict::NonPass,
            (phase != Baseline).then_some(1),
            None,
            StateRelation::Unavailable,
            true,
        );
        assert_eq!(
            super::adapters::ordinary(&profile, ordinary_steps),
            expected
        );
        assert_eq!(actual, GuardedEvaluation::Completed { result: expected });
        assert_eq!(control.effects_possible(), phase == PostObligation);
        backend.finished();
    }
}

#[test]
fn capture_errors_after_a_prior_execution_retain_every_obligation_fact() {
    for phase in [PreObligation, PostObligation] {
        let profile = profile(&[1, 2]);
        let mut steps = capture(Baseline, 1);
        steps.extend(capture(PreObligation, 1));
        steps.extend(process(position(0, 0), true, Some("0")));
        steps.extend(capture(PostObligation, 1));
        if phase == PostObligation {
            steps.extend(capture(PreObligation, 1));
            steps.extend(process(position(1, 0), false, Some("7")));
        }
        steps.extend([
            checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture),
            Step::Capture(
                phase,
                Ok(Err(StateCaptureError::new(
                    "ordinary failure after execution",
                ))),
            ),
            checkpoint(CoherenceCheckpoint::AfterFinalRiObservation),
        ]);
        let expected = expected_result(
            vec![
                validation(
                    "v0",
                    ObligationStatus::Satisfied,
                    vec![expected_obligation(
                        "v0",
                        ObligationStatus::Satisfied,
                        true,
                        Some("0"),
                    )],
                ),
                validation(
                    "v1",
                    ObligationStatus::Undetermined,
                    vec![
                        expected_obligation(
                            "v1",
                            ObligationStatus::Undetermined,
                            phase == PostObligation,
                            (phase == PostObligation).then_some("7"),
                        ),
                        expected_obligation("v1", ObligationStatus::Undetermined, false, None),
                    ],
                ),
            ],
            IntegrityVerdict::NonPass,
            Some(1),
            None,
            StateRelation::Unavailable,
            true,
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
        assert!(control.effects_possible());
        backend.finished();
    }
}

#[test]
fn realization_a_baseline_b_final_b_is_not_equal_and_never_launches() {
    let mut profile = profile(&[1]);
    profile.resolution_state_identity = Some(state(1).identity);
    let mut steps = capture(Baseline, 2);
    steps.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
    let ordinary_steps = steps.clone();
    let mut backend = Script::new(steps);
    let mut control = control();
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
        Some(2),
        Some(2),
        StateRelation::NotEqual,
        true,
    );
    assert_eq!(
        super::adapters::ordinary(&profile, ordinary_steps),
        expected
    );
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

#[test]
fn post_drift_dominates_success_and_declared_undetermined_code() {
    for (success, code) in [(true, "0"), (false, "7")] {
        let profile = profile(&[1, 1]);
        let mut steps = capture(Baseline, 1);
        steps.extend(capture(PreObligation, 1));
        steps.extend(process(position(0, 0), success, Some(code)));
        steps.extend(capture(PostObligation, 2));
        steps.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
        let ordinary_steps = steps.clone();
        let mut backend = Script::new(steps);
        let expected = expected_result(
            vec![
                validation(
                    "v0",
                    ObligationStatus::Violated,
                    vec![expected_obligation(
                        "v0",
                        ObligationStatus::Violated,
                        true,
                        Some(code),
                    )],
                ),
                validation(
                    "v1",
                    ObligationStatus::Undetermined,
                    vec![expected_obligation(
                        "v1",
                        ObligationStatus::Undetermined,
                        false,
                        None,
                    )],
                ),
            ],
            IntegrityVerdict::NonPass,
            Some(1),
            Some(2),
            StateRelation::NotEqual,
            true,
        );
        let mut control = control();
        assert_eq!(
            super::adapters::ordinary(&profile, ordinary_steps),
            expected
        );
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
        assert!(control.effects_possible());
        backend.finished();
    }
}
