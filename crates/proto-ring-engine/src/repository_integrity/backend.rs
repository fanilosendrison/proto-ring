//! Single mutable observation/dispatch interface for both canonical cores.
use std::convert::Infallible;
use std::path::{Path, PathBuf};

use super::control::{CoherenceCheckpoint, ProcessStartKnowledge};
use super::model::RuntimeCommandObligation;
use super::observations::{Completion, Launch, ProcessObservation};
use crate::repository_state::{RepositoryState, StateCaptureError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapturePhase {
    BeforeDiscovery,
    AfterDiscovery,
    Baseline,
    PreObligation,
    PostObligation,
}

/// Original loop positions; no plan reconstruction or wire ordinal conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObligationPosition {
    pub validation: usize,
    pub instance: usize,
}

/// Implementation boundary for a trusted host, not an admission or protection API.
/// Guarded implementations must truthfully establish non-start and effect closure.
pub trait Backend {
    type Abort;
    type Process;
    fn checkpoint(&mut self, phase: CoherenceCheckpoint) -> Result<(), Self::Abort>;
    fn capture(
        &mut self,
        phase: CapturePhase,
        root: &Path,
        scope: &[PathBuf],
    ) -> Result<Result<RepositoryState, StateCaptureError>, Self::Abort>;
    fn discover(
        &mut self,
        root: &Path,
        scope: &[PathBuf],
    ) -> Result<Result<Vec<PathBuf>, StateCaptureError>, Self::Abort>;
    fn launch(
        &mut self,
        root: &Path,
        position: ObligationPosition,
        obligation: &RuntimeCommandObligation,
    ) -> Result<Launch<Self::Process>, Self::Abort>;
    fn complete(
        &mut self,
        process: Self::Process,
        position: ObligationPosition,
    ) -> Result<Completion, Self::Abort>;
    fn process_start(abort: &Self::Abort) -> ProcessStartKnowledge;
}

/// Ordinary synchronous compatibility adapter. In particular Command::status()
/// errors may arise during wait, so NotStarted here is NOT contractual proof.
/// This adapter cannot be used as a guarded production host (Abort=Infallible).
pub(super) struct Ordinary<C, D, E> {
    pub capture: C,
    pub discover: D,
    pub execute: E,
}
impl<C, D, E> Backend for Ordinary<C, D, E>
where
    C: FnMut(CapturePhase, &Path, &[PathBuf]) -> Result<RepositoryState, StateCaptureError>,
    D: FnMut(&Path, &[PathBuf]) -> Result<Vec<PathBuf>, StateCaptureError>,
    E: FnMut(&Path, &RuntimeCommandObligation) -> ProcessObservation,
{
    type Abort = Infallible;
    type Process = Completion;
    fn checkpoint(&mut self, _: CoherenceCheckpoint) -> Result<(), Infallible> {
        Ok(())
    }
    fn capture(
        &mut self,
        phase: CapturePhase,
        root: &Path,
        scope: &[PathBuf],
    ) -> Result<Result<RepositoryState, StateCaptureError>, Infallible> {
        Ok((self.capture)(phase, root, scope))
    }
    fn discover(
        &mut self,
        root: &Path,
        scope: &[PathBuf],
    ) -> Result<Result<Vec<PathBuf>, StateCaptureError>, Infallible> {
        Ok((self.discover)(root, scope))
    }
    fn launch(
        &mut self,
        root: &Path,
        _: ObligationPosition,
        obligation: &RuntimeCommandObligation,
    ) -> Result<Launch<Completion>, Infallible> {
        Ok(match (self.execute)(root, obligation) {
            ProcessObservation::NotStarted => Launch::NotStarted,
            ProcessObservation::Completed(facts) => Launch::Started(facts),
        })
    }
    fn complete(
        &mut self,
        process: Completion,
        _: ObligationPosition,
    ) -> Result<Completion, Infallible> {
        Ok(process)
    }
    fn process_start(abort: &Infallible) -> ProcessStartKnowledge {
        match *abort {}
    }
}

pub(super) fn ordinary_control() -> super::control::AttemptControl {
    // Ordinary calls have no admitted invocation identity or host qualification.
    super::control::AttemptControl::new(super::control::AttemptId::from_bytes([0; 32]))
}

pub(super) fn infallible<T>(value: Result<T, Infallible>) -> T {
    match value {
        Ok(value) => value,
        Err(impossible) => match impossible {},
    }
}
