use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};

use super::model::{ObligationResult, ObligationStatus, RuntimeCommandObligation};

pub(super) struct ProcessOutcome {
    status: ObligationStatus,
    exit_code: Option<String>,
    executed: bool,
}
impl ProcessOutcome {
    pub(super) fn started(&self) -> bool {
        self.executed
    }

    pub(super) fn into_result(self, obligation: &RuntimeCommandObligation) -> ObligationResult {
        ObligationResult {
            id: obligation.validation_id.clone(),
            status: self.status,
            exit_code: self.exit_code,
            executed: self.executed,
        }
    }
}

pub(super) fn execute(repository: &Path, obligation: &RuntimeCommandObligation) -> ProcessOutcome {
    execute_with_ambient(repository, obligation, &BTreeMap::new())
}

pub(super) fn execute_with_ambient(
    repository: &Path,
    obligation: &RuntimeCommandObligation,
    ambient_seed: &BTreeMap<String, String>,
) -> ProcessOutcome {
    let Some(program) = obligation.command_prefix.first() else {
        return undetermined_start();
    };
    let Some(environment) = &obligation.process_environment else {
        return undetermined_start();
    };
    let mut command = Command::new(program);
    // The conformance seed models inherited variables; env_clear must erase it and real ambient data.
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
        Err(_) => return undetermined_start(),
    };
    let exit_code = status.code().map(|code| code.to_string());
    let semantic_status = if status.success() {
        ObligationStatus::Satisfied
    } else if exit_code
        .as_ref()
        .is_some_and(|code| obligation.undetermined_exit_codes.contains(code))
    {
        ObligationStatus::Undetermined
    } else {
        ObligationStatus::Violated
    };
    ProcessOutcome {
        status: semantic_status,
        exit_code,
        executed: true,
    }
}

fn undetermined_start() -> ProcessOutcome {
    ProcessOutcome {
        status: ObligationStatus::Undetermined,
        exit_code: None,
        executed: false,
    }
}
