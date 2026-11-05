#![forbid(unsafe_code)]
//! Portable engine implementation-boundary integration; no real host qualification.
use proto_ring_engine::repository_integrity::{self as ri, *};
use proto_ring_engine::repository_state::{
    RepositoryState, RepositoryStateIdentity, StateCaptureError,
};
use ri::backend::{Backend, CapturePhase, ObligationPosition};
use ri::control::*;
use ri::observations::{Completion, Launch};
use std::path::{Path, PathBuf};

struct Boundary {
    trace: Vec<String>,
    interrupt: Option<CoherenceCheckpoint>,
    attempt: AttemptId,
}
impl Backend for Boundary {
    type Abort = HostAbort;
    type Process = Completion;
    fn checkpoint(&mut self, phase: CoherenceCheckpoint) -> Result<(), HostAbort> {
        self.trace.push(format!("checkpoint:{phase:?}"));
        if self.interrupt == Some(phase) {
            Err(HostAbort {
                reason: HostAbortReason::AdmissionRevoked,
                process_start: ProcessStartKnowledge::DefinitivelyExcluded,
                attempt: self.attempt.clone(),
            })
        } else {
            Ok(())
        }
    }
    fn capture(
        &mut self,
        phase: CapturePhase,
        root: &Path,
        scope: &[PathBuf],
    ) -> Result<Result<RepositoryState, StateCaptureError>, HostAbort> {
        self.trace.push(format!("capture:{phase:?}"));
        Ok(Ok(RepositoryState {
            repository: root.into(),
            scope_paths: scope.into(),
            identity: RepositoryStateIdentity(vec![5]),
        }))
    }
    fn discover(
        &mut self,
        _: &Path,
        _: &[PathBuf],
    ) -> Result<Result<Vec<PathBuf>, StateCaptureError>, HostAbort> {
        panic!("single instances must not discover")
    }
    fn launch(
        &mut self,
        _: &Path,
        position: ObligationPosition,
        obligation: &RuntimeCommandObligation,
    ) -> Result<Launch<Completion>, HostAbort> {
        self.trace.push(format!("launch:{position:?}"));
        assert_eq!(
            position,
            ObligationPosition {
                validation: 0,
                instance: 0
            }
        );
        assert_eq!(obligation.arguments, ["base"]);
        Ok(Launch::Started(Completion {
            exit_success: true,
            exit_code: Some("0".into()),
        }))
    }
    fn complete(
        &mut self,
        facts: Completion,
        position: ObligationPosition,
    ) -> Result<Completion, HostAbort> {
        self.trace.push(format!("complete:{position:?}"));
        Ok(facts)
    }
    fn process_start(abort: &HostAbort) -> ProcessStartKnowledge {
        abort.process_start
    }
}
fn consumer(root: PathBuf) -> ConsumerIntegrityProfile {
    ConsumerIntegrityProfile {
        repository: root,
        carrier: "profile.md".into(),
        model_version: 1,
        authority: ProfileAuthority {
            responsibility: "quality".into(),
            source: "consumer".into(),
        },
        environments: ["runtime".into()].into(),
        continue_after_non_satisfied: false,
        validations: [(
            "validation".into(),
            ValidationDefinition {
                id: "validation".into(),
                responsibility: "quality".into(),
                prerequisites: Default::default(),
                instances: ValidationInstances::Single,
                command: CommandBinding {
                    environment: "runtime".into(),
                    arguments: vec!["base".into()],
                    undetermined_exit_codes: Default::default(),
                },
                target: None,
            },
        )]
        .into(),
        order: vec!["validation".into()],
        identity: ProfileIdentity(vec![3]),
    }
}

#[test]
fn retained_control_and_normal_configuration_hooks_cross_both_cores() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let context = EvaluationContext {
        realizations: [(
            "runtime".into(),
            ValidationEnvironmentRealization {
                environment_id: "runtime".into(),
                identity: vec![4],
                command_prefix: vec!["host-bound-program".into()],
                process_environment: Default::default(),
            },
        )]
        .into(),
    };
    for interrupt in [None, Some(CoherenceCheckpoint::AfterFinalRiObservation)] {
        let attempt = AttemptId::from_bytes([19; 32]);
        let mut control = AttemptControl::new(attempt.clone());
        let mut backend = Boundary {
            trace: vec![],
            interrupt,
            attempt: attempt.clone(),
        };
        let realized = ri::realize_guarded(
            &root,
            &consumer(root.clone()),
            &context,
            &mut backend,
            &mut control,
        )
        .unwrap()
        .unwrap();
        assert!(!control.effects_possible());
        let result = ri::evaluate_guarded(
            &root,
            &realized.profile,
            realized.evaluation_context_identity.clone(),
            &mut backend,
            &mut control,
        );
        let expected = IntegrityResult {
            verdict: IntegrityVerdict::Pass,
            profile_identity: ProfileIdentity(vec![3]),
            evaluation_context_identity: realized.evaluation_context_identity,
            baseline_state_identity: Some(RepositoryStateIdentity(vec![5])),
            final_state_identity: Some(RepositoryStateIdentity(vec![5])),
            state_relation: StateRelation::Equal,
            state_coherence_halt: false,
            validations: vec![ValidationResult {
                id: "validation".into(),
                status: ObligationStatus::Satisfied,
                obligations: vec![ObligationResult {
                    id: "validation".into(),
                    status: ObligationStatus::Satisfied,
                    executed: true,
                    exit_code: Some("0".into()),
                }],
            }],
            obligations: vec![ObligationResult {
                id: "validation".into(),
                status: ObligationStatus::Satisfied,
                executed: true,
                exit_code: Some("0".into()),
            }],
        };
        assert_eq!(
            result,
            if interrupt.is_none() {
                GuardedEvaluation::Completed { result: expected }
            } else {
                GuardedEvaluation::Interrupted {
                    reason: HostAbort {
                        reason: HostAbortReason::AdmissionRevoked,
                        process_start: ProcessStartKnowledge::DefinitivelyExcluded,
                        attempt,
                    },
                    effects_possible: true,
                }
            }
        );
        assert!(control.effects_possible());
        assert_eq!(
            backend.trace,
            [
                "checkpoint:BeforeRealization",
                "checkpoint:AfterRealization",
                "checkpoint:BeforeRequiredRiCapture",
                "capture:Baseline",
                "checkpoint:BeforeRequiredRiCapture",
                "capture:PreObligation",
                "checkpoint:BeforeEachLaunch",
                "launch:ObligationPosition { validation: 0, instance: 0 }",
                "complete:ObligationPosition { validation: 0, instance: 0 }",
                "checkpoint:AfterEachEffectLifetime",
                "checkpoint:BeforeRequiredRiCapture",
                "capture:PostObligation",
                "checkpoint:AfterFinalRiObservation",
            ]
        );
    }
}
