//! Engine implementation boundary, not execution authorization or a public API service.

/// Opaque invocation identity. Creation/randomness is wired by the future invocation layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptId([u8; 32]);
impl AttemptId {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessStartKnowledge {
    DefinitivelyExcluded,
    PotentiallyOccurred,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAbortReason {
    CoherenceLost,
    AdmissionRevoked,
    EffectBindingInvalid,
    ProcessStartIndeterminate,
    ProcessCompletionIndeterminate,
    DescendantEffectsUncontained,
    HostObservationUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAbort {
    pub reason: HostAbortReason,
    pub process_start: ProcessStartKnowledge,
    pub attempt: AttemptId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoherenceCheckpoint {
    BeforeRgs,
    BeforeRealization,
    AfterRealization,
    BeforePlanAdmission,
    BeforeEachLaunch,
    AfterEachEffectLifetime,
    BeforeRequiredRiCapture,
    AfterFinalRiObservation,
}

/// Caller-retained, invocation-lifetime monotone state. No reset operation exists.
#[derive(Debug)]
pub struct AttemptControl {
    attempt: AttemptId,
    effects_possible: bool,
}
impl AttemptControl {
    pub fn new(attempt: AttemptId) -> Self {
        Self {
            attempt,
            effects_possible: false,
        }
    }
    pub fn attempt(&self) -> &AttemptId {
        &self.attempt
    }
    pub fn effects_possible(&self) -> bool {
        self.effects_possible
    }
    pub(super) fn started(&mut self) {
        self.effects_possible = true;
    }
    pub(super) fn observe<T, B: super::backend::Backend>(
        &mut self,
        value: Result<T, B::Abort>,
    ) -> Result<T, B::Abort> {
        if let Err(abort) = &value
            && B::process_start(abort) == ProcessStartKnowledge::PotentiallyOccurred
        {
            self.started();
        }
        value
    }
    pub(super) fn interrupted(&self, reason: HostAbort) -> Interruption {
        Interruption {
            reason,
            effects_possible: self.effects_possible,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Interruption {
    pub reason: HostAbort,
    pub effects_possible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuardedEvaluation {
    Completed {
        result: super::IntegrityResult,
    },
    Interrupted {
        reason: HostAbort,
        effects_possible: bool,
    },
}
