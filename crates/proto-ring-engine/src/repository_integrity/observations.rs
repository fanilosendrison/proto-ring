//! Process facts only. A backend cannot supply RI statuses or verdicts.

/// Technical launch observation, not by itself a proof of host guarantees.
/// A guarded backend may return NotStarted only after definitively excluding start.
/// Uncertain start is an outer HostAbort/PotentiallyOccurred instead.
pub enum Launch<P> {
    NotStarted,
    Started(P),
}

/// Known principal completion. Guarded backends must also establish effect closure
/// before returning this value; AfterEachEffectLifetime checks the active boundary.
/// No output capture, receipt, or closure certificate is fabricated here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Completion {
    pub exit_success: bool,
    /// Canonical i32 decimal, or absent (e.g. known signal termination).
    pub exit_code: Option<String>,
}

pub(super) enum ProcessObservation {
    NotStarted,
    Completed(Completion),
}
impl ProcessObservation {
    #[cfg(feature = "conformance-internal")]
    pub(super) fn started(&self) -> bool {
        matches!(self, Self::Completed(_))
    }
}
