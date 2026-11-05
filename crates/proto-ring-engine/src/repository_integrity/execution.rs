use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};

use super::model::{ObligationResult, ObligationStatus, RuntimeCommandObligation};
use super::observations::{Completion, ProcessObservation};

/// Single engine classification of facts, shared by ordinary and guarded execution.
pub(super) fn classify(
    observation: ProcessObservation,
    obligation: &RuntimeCommandObligation,
) -> ObligationResult {
    match observation {
        ProcessObservation::NotStarted => ObligationResult::undetermined(obligation),
        ProcessObservation::Completed(facts) => {
            let status = if facts.exit_success {
                ObligationStatus::Satisfied
            } else if facts
                .exit_code
                .as_ref()
                .is_some_and(|code| obligation.undetermined_exit_codes.contains(code))
            {
                ObligationStatus::Undetermined
            } else {
                ObligationStatus::Violated
            };
            ObligationResult {
                id: obligation.validation_id.clone(),
                status,
                exit_code: facts.exit_code,
                executed: true,
            }
        }
    }
}

pub(super) fn execute(
    repository: &Path,
    obligation: &RuntimeCommandObligation,
) -> ProcessObservation {
    execute_with_ambient(repository, obligation, &BTreeMap::new())
}

/// Historical ordinary synchronous runner, not a qualified host implementation.
/// A status() error can originate in waiting, not only spawning. Its compatibility
/// fallback is retained but must never be promoted to contractual ProvenNotStarted.
pub(super) fn execute_with_ambient(
    repository: &Path,
    obligation: &RuntimeCommandObligation,
    ambient_seed: &BTreeMap<String, String>,
) -> ProcessObservation {
    let Some(program) = obligation.command_prefix.first() else {
        return ProcessObservation::NotStarted;
    };
    let Some(environment) = &obligation.process_environment else {
        return ProcessObservation::NotStarted;
    };
    let mut command = Command::new(program);
    // The conformance seed models inherited variables; env_clear erases it and real ambient data.
    command
        .envs(ambient_seed)
        .env_clear()
        .envs(environment)
        .args(obligation.command_prefix.iter().skip(1))
        .args(&obligation.arguments)
        .current_dir(repository)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let status = match command.status() {
        Ok(status) => status,
        Err(_) => return ProcessObservation::NotStarted,
    };
    ProcessObservation::Completed(Completion {
        exit_success: status.success(),
        exit_code: status.code().map(|code| code.to_string()),
    })
}
