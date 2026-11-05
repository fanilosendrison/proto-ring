//! Monotone source-integrity observations, separate from expected subject mutation.
use crate::{
    Result, ensure,
    evidence::Evidence,
    source::{self, Source},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub phase: String,
    pub unchanged: bool,
    #[serde(default)]
    pub detail: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceState {
    pub invalidated: bool,
    pub checks: Vec<Observation>,
}
pub struct Guard {
    pub state: SourceState,
    source: Source,
}
impl Guard {
    pub fn new(source: Source) -> Self {
        Self {
            state: SourceState::default(),
            source,
        }
    }
    pub fn check(&mut self, e: &Evidence, root: &Path, phase: &str) -> Result<()> {
        ensure(
            matches!(phase, "before_probes" | "after_probes"),
            "unknown source checkpoint",
        )?;
        ensure(
            !self.state.checks.iter().any(|v| v.phase == phase),
            "source checkpoint repeated",
        )?;
        let current = source::source_unchanged(root, &self.source);
        let observation = Observation {
            phase: phase.into(),
            unchanged: current.is_ok(),
            detail: current.as_ref().err().map(ToString::to_string),
        };
        self.state.invalidated |= current.is_err();
        self.state.checks.push(observation.clone());
        // Failure to persist any mandatory observation aborts, never creates a valid result.
        if let Err(error) = e.json(&format!("source-{phase}.json"), &observation) {
            self.state.invalidated = true;
            return Err(error);
        }
        if current.is_err() && !e.root.join("source-invalidated.json").exists() {
            e.json("source-invalidated.json", &self.state)?;
        }
        ensure(
            !self.state.invalidated,
            "source invalidated; restoration cannot clear detection",
        )
    }
    pub fn finish(&self, e: &Evidence) -> Result<()> {
        e.json("source-state.json", &self.state)
    }
}
pub fn verify(e: &Path, authorized: &Source) -> Result<()> {
    let claimed: Source =
        serde_json::from_slice(&crate::verification::read_bytes(e, "source.json")?)?;
    ensure(
        claimed.sha == authorized.sha && claimed.tree == authorized.tree,
        "source inventory commit/tree differs from authority",
    )?;
    ensure(
        claimed.files.keys().eq(authorized.files.keys()),
        "source inventory required file membership mismatch",
    )?;
    for (path, expected) in &authorized.files {
        ensure(
            claimed.files.get(path) == Some(expected),
            &format!("source inventory entry differs: {path}"),
        )?;
    }
    let state: SourceState =
        serde_json::from_slice(&crate::verification::read_bytes(e, "source-state.json")?)?;
    ensure(
        !state.invalidated
            && !e.join("source-change.txt").exists()
            && !e.join("source-invalidated.json").exists(),
        "source invalidated",
    )?;
    ensure(
        state.checks.len() == 2
            && state.checks[0].phase == "before_probes"
            && state.checks[1].phase == "after_probes",
        "mandatory source checkpoint missing or reordered",
    )?;
    for observation in &state.checks {
        let retained: Observation = serde_json::from_slice(&crate::verification::read_bytes(
            e,
            &format!("source-{}.json", observation.phase),
        )?)?;
        ensure(
            retained.phase == observation.phase
                && retained.unchanged == observation.unchanged
                && retained.unchanged
                && retained.detail.is_none()
                && observation.detail.is_none(),
            "source invalidated or checkpoint contradicted",
        )?;
    }
    Ok(())
}
