use super::super::backend::{Backend, CapturePhase::*};
use super::super::control::*;
use super::super::*;
use super::profiles::*;
use super::script::*;
use crate::repository_state::StateCaptureError;
use std::cell::RefCell;

fn observation_steps() -> Vec<Step> {
    vec![
        Step::Capture(BeforeDiscovery, Ok(Ok(state(1)))),
        Step::Discover(Ok(Ok(vec![]))),
        Step::Capture(AfterDiscovery, Ok(Ok(state(1)))),
    ]
}
fn ordinary(
    profile: &ConsumerIntegrityProfile,
    context: &EvaluationContext,
    steps: Vec<Step>,
) -> RealizedIntegrityProfile {
    let backend = RefCell::new(Script::new(
        steps
            .into_iter()
            .filter(|step| !matches!(step, Step::Checkpoint(..)))
            .collect(),
    ));
    let before = std::cell::Cell::new(true);
    let actual = realization::realize_with(
        &root(),
        profile,
        context,
        |root, scope| {
            let phase = if before.replace(false) {
                BeforeDiscovery
            } else {
                AfterDiscovery
            };
            backend.borrow_mut().capture(phase, root, scope).unwrap()
        },
        |root, scope| backend.borrow_mut().discover(root, scope).unwrap(),
    )
    .unwrap();
    backend.borrow().finished();
    actual
}

#[test]
fn ordinary_failure_and_host_abort_are_distinct_at_each_discovery_boundary() {
    for index in 0..3 {
        let profile = consumer(Some(paths(RepositoryPathsMode::ForEach)));
        let mut ordinary_steps = observation_steps();
        let error = StateCaptureError::new("ordinary observation unavailable");
        ordinary_steps[index] = match index {
            0 => Step::Capture(BeforeDiscovery, Ok(Err(error))),
            1 => Step::Discover(Ok(Err(error))),
            2 => Step::Capture(AfterDiscovery, Ok(Err(error))),
            _ => unreachable!(),
        };
        ordinary_steps.truncate(index + 1);
        let mut steps = vec![checkpoint(CoherenceCheckpoint::BeforeRealization)];
        steps.extend(ordinary_steps);
        steps.push(checkpoint(CoherenceCheckpoint::AfterRealization));
        let expected = RealizedIntegrityProfile {
            profile: IntegrityProfile {
                continue_after_non_satisfied: true,
                validations: vec![RuntimeValidation {
                    id: "v0".into(),
                    prerequisites: Default::default(),
                    obligations: vec![],
                    resolution_failed: true,
                }],
                identity: ProfileIdentity(vec![11]),
                governed_paths: vec![],
                resolution_state_identity: None,
            },
            evaluation_context_identity: expected_context(),
        };
        assert_eq!(ordinary(&profile, &context(), steps.clone()), expected);
        let mut backend = Script::new(steps);
        assert_eq!(
            realize_guarded(&root(), &profile, &context(), &mut backend, &mut control())
                .unwrap()
                .unwrap(),
            expected
        );
        backend.finished();

        let reason = abort(
            HostAbortReason::HostObservationUnavailable,
            ProcessStartKnowledge::DefinitivelyExcluded,
        );
        let mut steps = vec![checkpoint(CoherenceCheckpoint::BeforeRealization)];
        steps.extend(observation_steps().into_iter().take(index));
        steps.push(match index {
            0 => Step::Capture(BeforeDiscovery, Err(reason.clone())),
            1 => Step::Discover(Err(reason.clone())),
            2 => Step::Capture(AfterDiscovery, Err(reason.clone())),
            _ => unreachable!(),
        });
        let mut backend = Script::new(steps);
        let mut control = control();
        let interruption =
            realize_guarded(&root(), &profile, &context(), &mut backend, &mut control).unwrap_err();
        assert_eq!(
            interruption,
            Interruption {
                reason,
                effects_possible: false
            }
        );
        backend.finished();
    }
}

fn expected_context() -> EvaluationContextIdentity {
    // Independent framing oracle for one required opaque environment identity.
    let mut bytes = vec![];
    for value in [
        b"proto-ring evaluation-context opaque identity".as_slice(),
        b"runtime",
    ] {
        bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
        bytes.extend_from_slice(value);
    }
    bytes.push(1);
    bytes.extend_from_slice(&1u64.to_be_bytes());
    bytes.push(9);
    EvaluationContextIdentity(bytes)
}

#[test]
fn zero_paths_single_empty_and_ordinary_drift_preserve_resolution_semantics() {
    for (instances, drift, count, failed, arguments) in [
        (
            Some(paths(RepositoryPathsMode::ForEach)),
            false,
            0,
            false,
            vec![],
        ),
        (
            Some(paths(RepositoryPathsMode::AppendAll)),
            false,
            1,
            false,
            vec![vec!["base"]],
        ),
        (
            Some(paths(RepositoryPathsMode::ForEach)),
            true,
            0,
            true,
            vec![],
        ),
        (
            Some(ValidationInstances::Single),
            false,
            1,
            false,
            vec![vec!["base"]],
        ),
        (None, false, 0, false, vec![]),
    ] {
        let has_paths = instances
            .as_ref()
            .is_some_and(|i| matches!(i, ValidationInstances::RepositoryPaths { .. }));
        let profile = consumer(instances);
        let mut steps = vec![checkpoint(CoherenceCheckpoint::BeforeRealization)];
        if has_paths {
            steps.extend(observation_steps());
        }
        if drift {
            *steps.last_mut().unwrap() = Step::Capture(AfterDiscovery, Ok(Ok(state(2))));
        }
        steps.push(checkpoint(CoherenceCheckpoint::AfterRealization));
        let ordinary_result = ordinary(&profile, &context(), steps.clone());
        let mut backend = Script::new(steps);
        let actual = realize_guarded(&root(), &profile, &context(), &mut backend, &mut control())
            .unwrap()
            .unwrap();
        for result in [&actual, &ordinary_result] {
            assert_eq!(result.profile.validations.len(), profile.order.len());
            if let Some(validation) = result.profile.validations.first() {
                assert_eq!(validation.resolution_failed, failed);
                assert_eq!(validation.obligations.len(), count);
                assert_eq!(
                    validation
                        .obligations
                        .iter()
                        .map(|o| o.arguments.clone())
                        .collect::<Vec<_>>(),
                    arguments
                );
                for obligation in &validation.obligations {
                    assert_eq!(obligation.command_prefix, ["program", "prefix"]);
                    assert_eq!(
                        obligation.process_environment,
                        Some([("ONLY".into(), "explicit".into())].into())
                    );
                }
                assert_eq!(result.evaluation_context_identity, expected_context());
            }
            assert_eq!(
                result.profile.resolution_state_identity,
                (has_paths && !drift).then(|| state(1).identity)
            );
            assert_eq!(result.profile.identity, ProfileIdentity(vec![11]));
        }
        backend.finished();
    }
}

#[test]
fn realization_checkpoints_cover_clean_errors_but_never_continue_after_abort() {
    for phase in [
        CoherenceCheckpoint::BeforeRealization,
        CoherenceCheckpoint::AfterRealization,
    ] {
        let reason = abort(
            HostAbortReason::CoherenceLost,
            ProcessStartKnowledge::DefinitivelyExcluded,
        );
        let mut steps = vec![];
        if phase == CoherenceCheckpoint::AfterRealization {
            steps.push(checkpoint(CoherenceCheckpoint::BeforeRealization));
        }
        steps.push(Step::Checkpoint(phase, Err(reason.clone())));
        let mut backend = Script::new(steps);
        assert_eq!(
            realize_guarded(
                &root(),
                &consumer(None),
                &context(),
                &mut backend,
                &mut control()
            )
            .unwrap_err(),
            Interruption {
                reason,
                effects_possible: false
            }
        );
        backend.finished();
    }
    let mut invalid_context = context();
    invalid_context
        .realizations
        .get_mut("runtime")
        .unwrap()
        .environment_id = "other".into();
    let mut wrong_root = consumer(None);
    wrong_root.repository = root().join("not-the-root");
    for (repository, profile, context) in [
        (root().join("crates"), consumer(None), context()),
        (root(), wrong_root, context()),
        (
            root(),
            consumer(Some(ValidationInstances::Single)),
            invalid_context,
        ),
    ] {
        let mut backend = Script::new(vec![
            checkpoint(CoherenceCheckpoint::BeforeRealization),
            checkpoint(CoherenceCheckpoint::AfterRealization),
        ]);
        assert!(
            realize_guarded(
                &repository,
                &profile,
                &context,
                &mut backend,
                &mut control()
            )
            .unwrap()
            .is_err()
        );
        backend.finished();
    }
}

#[test]
fn selectors_preserve_order_literals_scope_and_expanded_arguments() {
    let instances = ValidationInstances::RepositoryPaths {
        mode: RepositoryPathsMode::ForEach,
        selectors: vec![
            RepositoryPathSelector {
                kind: SelectorKind::Path,
                value: "absent.rs".into(),
            },
            RepositoryPathSelector {
                kind: SelectorKind::Glob,
                value: "src/*.rs".into(),
            },
        ],
    };
    let profile = consumer(Some(instances));
    let steps = vec![
        checkpoint(CoherenceCheckpoint::BeforeRealization),
        Step::Capture(BeforeDiscovery, Ok(Ok(state(1)))),
        Step::Discover(Ok(Ok(vec!["src/z.rs".into(), "src/a.rs".into()]))),
        Step::Capture(AfterDiscovery, Ok(Ok(state(1)))),
        checkpoint(CoherenceCheckpoint::AfterRealization),
    ];
    let mut backend = Script::new(steps.clone());
    let guarded = realize_guarded(&root(), &profile, &context(), &mut backend, &mut control())
        .unwrap()
        .unwrap();
    for result in [guarded, ordinary(&profile, &context(), steps)] {
        assert_eq!(
            result.profile.governed_paths,
            ["absent.rs", "src/a.rs", "src/z.rs"].map(std::path::PathBuf::from)
        );
        assert_eq!(
            result.profile.validations[0]
                .obligations
                .iter()
                .map(|o| o.arguments.clone())
                .collect::<Vec<_>>(),
            vec![
                vec!["base", "absent.rs"],
                vec!["base", "src/a.rs"],
                vec!["base", "src/z.rs"]
            ]
        );
    }
    backend.finished();
}
