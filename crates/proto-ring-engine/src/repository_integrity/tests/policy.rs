use super::super::backend::CapturePhase::*;
use super::super::control::*;
use super::super::*;
use super::script::*;
use std::path::Path;

fn assert_both(
    profile: &IntegrityProfile,
    steps: Vec<Step>,
    expected: IntegrityResult,
    effects: bool,
) {
    assert_eq!(super::adapters::ordinary(profile, steps.clone()), expected);
    let mut backend = Script::new(steps);
    let mut control = control();
    assert_eq!(
        evaluate_guarded(
            Path::new("."),
            profile,
            context_identity(),
            &mut backend,
            &mut control
        ),
        GuardedEvaluation::Completed { result: expected }
    );
    assert_eq!(control.effects_possible(), effects);
    backend.finished();
}

#[test]
fn continuation_applies_after_all_instances_and_violated_dominates_undetermined() {
    for continuing in [false, true] {
        let mut profile = profile(&[2, 1]);
        profile.continue_after_non_satisfied = continuing;
        let mut steps = capture(Baseline, 1);
        for (instance, code) in [(0, "1"), (1, "7")] {
            steps.extend(capture(PreObligation, 1));
            steps.extend(process(position(0, instance), false, Some(code)));
            steps.extend(capture(PostObligation, 1));
        }
        if continuing {
            steps.extend(capture(PreObligation, 1));
            steps.extend(process(position(1, 0), true, Some("0")));
            steps.extend(capture(PostObligation, 1));
        }
        steps.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
        let expected = expected_result(
            vec![
                validation(
                    "v0",
                    ObligationStatus::Violated,
                    vec![
                        expected_obligation("v0", ObligationStatus::Violated, true, Some("1")),
                        expected_obligation("v0", ObligationStatus::Undetermined, true, Some("7")),
                    ],
                ),
                validation(
                    "v1",
                    if continuing {
                        ObligationStatus::Satisfied
                    } else {
                        ObligationStatus::Undetermined
                    },
                    vec![expected_obligation(
                        "v1",
                        if continuing {
                            ObligationStatus::Satisfied
                        } else {
                            ObligationStatus::Undetermined
                        },
                        continuing,
                        continuing.then_some("0"),
                    )],
                ),
            ],
            IntegrityVerdict::NonPass,
            Some(1),
            Some(1),
            StateRelation::Equal,
            false,
        );
        assert_both(&profile, steps, expected, true);
    }
}

#[test]
fn satisfied_zero_instances_enable_dependents_but_resolution_failure_does_not() {
    for failed in [false, true] {
        let mut profile = profile(&[0, 1]);
        profile.validations[0].resolution_failed = failed;
        profile.validations[1].prerequisites.insert("v0".into());
        let mut steps = capture(Baseline, 1);
        if !failed {
            steps.extend(capture(PreObligation, 1));
            steps.extend(process(position(1, 0), true, Some("0")));
            steps.extend(capture(PostObligation, 1));
        }
        steps.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
        let status = if failed {
            ObligationStatus::Undetermined
        } else {
            ObligationStatus::Satisfied
        };
        let expected = expected_result(
            vec![
                validation("v0", status, vec![]),
                validation(
                    "v1",
                    status,
                    vec![expected_obligation(
                        "v1",
                        status,
                        !failed,
                        (!failed).then_some("0"),
                    )],
                ),
            ],
            if failed {
                IntegrityVerdict::NonPass
            } else {
                IntegrityVerdict::Pass
            },
            Some(1),
            Some(1),
            StateRelation::Equal,
            false,
        );
        assert_both(&profile, steps, expected, !failed);
    }
}

#[test]
fn empty_profile_and_resolved_zero_instances_can_pass_without_execution() {
    for counts in [vec![], vec![0]] {
        let profile = profile(&counts);
        let mut steps = capture(Baseline, 1);
        steps.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
        let validations = if counts.is_empty() {
            vec![]
        } else {
            vec![validation("v0", ObligationStatus::Satisfied, vec![])]
        };
        assert_both(
            &profile,
            steps,
            expected_result(
                validations,
                IntegrityVerdict::Pass,
                Some(1),
                Some(1),
                StateRelation::Equal,
                false,
            ),
            false,
        );
    }
}

#[test]
fn pre_drift_halts_even_with_continuation_and_marks_all_remaining_instances() {
    let profile = profile(&[2, 1]);
    let mut steps = capture(Baseline, 1);
    steps.extend(capture(PreObligation, 2));
    steps.push(checkpoint(CoherenceCheckpoint::AfterFinalRiObservation));
    let expected = expected_result(
        vec![
            validation(
                "v0",
                ObligationStatus::Undetermined,
                vec![expected_obligation("v0", ObligationStatus::Undetermined, false, None); 2],
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
    assert_both(&profile, steps, expected, false);
}
