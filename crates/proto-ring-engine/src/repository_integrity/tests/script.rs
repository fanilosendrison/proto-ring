//! Deterministic boundary double; it establishes no real host protection.
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use super::super::backend::{Backend, CapturePhase, ObligationPosition};
use super::super::control::*;
use super::super::observations::{Completion, Launch};
use super::super::*;
use crate::repository_state::{RepositoryState, RepositoryStateIdentity, StateCaptureError};

type Observation<T> = Result<Result<T, StateCaptureError>, HostAbort>;

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Checkpoint(CoherenceCheckpoint),
    Capture(CapturePhase),
    Discover,
    Launch(ObligationPosition),
    Complete(ObligationPosition),
}

#[derive(Clone)]
pub enum Step {
    Checkpoint(CoherenceCheckpoint, Result<(), HostAbort>),
    Capture(CapturePhase, Observation<RepositoryState>),
    Discover(Observation<Vec<PathBuf>>),
    Launch(ObligationPosition, Result<bool, HostAbort>),
    Complete(ObligationPosition, Result<Completion, HostAbort>),
}
impl Step {
    fn event(&self) -> Event {
        match self {
            Self::Checkpoint(phase, _) => Event::Checkpoint(*phase),
            Self::Capture(phase, _) => Event::Capture(*phase),
            Self::Discover(_) => Event::Discover,
            Self::Launch(position, _) => Event::Launch(*position),
            Self::Complete(position, _) => Event::Complete(*position),
        }
    }
}

pub struct Script {
    steps: VecDeque<Step>,
    expected: Vec<Event>,
    trace: Vec<Event>,
}
impl Script {
    pub fn new(steps: Vec<Step>) -> Self {
        Self {
            expected: steps.iter().map(Step::event).collect(),
            steps: steps.into(),
            trace: Vec::new(),
        }
    }
    fn next(&mut self, event: Event) -> Step {
        let step = self
            .steps
            .pop_front()
            .expect("unexpected backend operation");
        assert_eq!(step.event(), event);
        self.trace.push(event);
        step
    }
    pub fn finished(&self) {
        assert!(self.steps.is_empty(), "unconsumed script steps");
        assert_eq!(self.trace, self.expected);
    }
}
impl Backend for Script {
    type Abort = HostAbort;
    type Process = ();

    fn checkpoint(&mut self, phase: CoherenceCheckpoint) -> Result<(), HostAbort> {
        let Step::Checkpoint(_, value) = self.next(Event::Checkpoint(phase)) else {
            unreachable!()
        };
        value
    }
    fn capture(
        &mut self,
        phase: CapturePhase,
        _: &Path,
        _: &[PathBuf],
    ) -> Observation<RepositoryState> {
        let Step::Capture(_, value) = self.next(Event::Capture(phase)) else {
            unreachable!()
        };
        value
    }
    fn discover(&mut self, _: &Path, scope: &[PathBuf]) -> Observation<Vec<PathBuf>> {
        assert!(scope.is_empty());
        let Step::Discover(value) = self.next(Event::Discover) else {
            unreachable!()
        };
        value
    }
    fn launch(
        &mut self,
        _: &Path,
        position: ObligationPosition,
        _: &RuntimeCommandObligation,
    ) -> Result<Launch<()>, HostAbort> {
        let Step::Launch(_, value) = self.next(Event::Launch(position)) else {
            unreachable!()
        };
        value.map(|started| {
            if started {
                Launch::Started(())
            } else {
                Launch::NotStarted
            }
        })
    }
    fn complete(&mut self, (): (), position: ObligationPosition) -> Result<Completion, HostAbort> {
        let Step::Complete(_, value) = self.next(Event::Complete(position)) else {
            unreachable!()
        };
        value
    }
    fn process_start(abort: &HostAbort) -> ProcessStartKnowledge {
        abort.process_start
    }
}

pub fn checkpoint(phase: CoherenceCheckpoint) -> Step {
    Step::Checkpoint(phase, Ok(()))
}
pub fn state(identity: u8) -> RepositoryState {
    RepositoryState {
        repository: PathBuf::from("."),
        identity: RepositoryStateIdentity(vec![identity]),
        scope_paths: vec![],
    }
}
pub fn capture(phase: CapturePhase, identity: u8) -> Vec<Step> {
    vec![
        checkpoint(CoherenceCheckpoint::BeforeRequiredRiCapture),
        Step::Capture(phase, Ok(Ok(state(identity)))),
    ]
}
pub fn position(validation: usize, instance: usize) -> ObligationPosition {
    ObligationPosition {
        validation,
        instance,
    }
}
pub fn process(position: ObligationPosition, success: bool, code: Option<&str>) -> Vec<Step> {
    vec![
        checkpoint(CoherenceCheckpoint::BeforeEachLaunch),
        Step::Launch(position, Ok(true)),
        Step::Complete(
            position,
            Ok(Completion {
                exit_success: success,
                exit_code: code.map(str::to_owned),
            }),
        ),
        checkpoint(CoherenceCheckpoint::AfterEachEffectLifetime),
    ]
}
pub fn abort(reason: HostAbortReason, process_start: ProcessStartKnowledge) -> HostAbort {
    HostAbort {
        reason,
        process_start,
        attempt: AttemptId::from_bytes([42; 32]),
    }
}
pub fn control() -> AttemptControl {
    AttemptControl::new(AttemptId::from_bytes([42; 32]))
}

pub fn profile(counts: &[usize]) -> IntegrityProfile {
    IntegrityProfile {
        continue_after_non_satisfied: true,
        validations: counts
            .iter()
            .enumerate()
            .map(|(index, count)| {
                let id = format!("v{index}");
                RuntimeValidation {
                    id: id.clone(),
                    prerequisites: Default::default(),
                    resolution_failed: false,
                    obligations: (0..*count)
                        .map(|_| RuntimeCommandObligation {
                            validation_id: id.clone(),
                            command_prefix: vec!["scripted-not-a-real-program".into()],
                            arguments: vec!["base".into()],
                            undetermined_exit_codes: ["7".into()].into(),
                            process_environment: Some(Default::default()),
                        })
                        .collect(),
                }
            })
            .collect(),
        identity: ProfileIdentity(vec![11]),
        governed_paths: vec![],
        resolution_state_identity: None,
    }
}
pub fn context_identity() -> EvaluationContextIdentity {
    EvaluationContextIdentity(vec![12])
}

// Independently authored expected fields, never computed by an engine aggregator.
pub fn expected_obligation(
    id: &str,
    status: ObligationStatus,
    executed: bool,
    code: Option<&str>,
) -> ObligationResult {
    ObligationResult {
        id: id.into(),
        status,
        executed,
        exit_code: code.map(str::to_owned),
    }
}
pub fn expected_result(
    validations: Vec<ValidationResult>,
    verdict: IntegrityVerdict,
    baseline: Option<u8>,
    final_state: Option<u8>,
    relation: StateRelation,
    halt: bool,
) -> IntegrityResult {
    IntegrityResult {
        profile_identity: ProfileIdentity(vec![11]),
        evaluation_context_identity: context_identity(),
        verdict,
        baseline_state_identity: baseline.map(|id| state(id).identity),
        final_state_identity: final_state.map(|id| state(id).identity),
        state_relation: relation,
        obligations: validations
            .iter()
            .flat_map(|v| v.obligations.clone())
            .collect(),
        validations,
        state_coherence_halt: halt,
    }
}
pub fn validation(
    id: &str,
    status: ObligationStatus,
    obligations: Vec<ObligationResult>,
) -> ValidationResult {
    ValidationResult {
        id: id.into(),
        status,
        obligations,
    }
}
