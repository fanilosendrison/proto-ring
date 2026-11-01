use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use proto_ring_engine::adr_metadata::sha256;
use proto_ring_engine::repository_integrity::{
    IntegrityProfile, IntegrityResult, RuntimeCommandObligation,
};
use serde::Deserialize;
use serde_json::Value;
use tempfile::TempDir;

use crate::fixture::{FixtureError, FixtureError as Error};

type R<T> = Result<T, FixtureError>;
static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const ACTION_ID_FLAG: &str = "--action-identity";

#[derive(Clone)]
struct ExpectedAction {
    digest: String,
    execution_attempts: Vec<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompletionReceipt {
    schema_version: u8,
    completion: String,
    request_identity: String,
    action_identity: String,
    action_digest: String,
    return_code: i32,
}

pub(crate) struct ActionReceiptChannel {
    _storage: TempDir,
    request_identity: String,
    actions: BTreeMap<String, ExpectedAction>,
}

impl ActionReceiptChannel {
    pub(crate) fn new(governed_root: &Path) -> R<Self> {
        let parent = governed_root
            .parent()
            .ok_or_else(|| failure("governed fixture root has no external parent"))?;
        let storage = tempfile::Builder::new()
            .prefix("proto-ring-action-receipts-")
            .tempdir_in(parent)
            .map_err(failure)?;
        if storage.path().starts_with(governed_root) {
            return Err(failure(
                "action receipt storage is inside the governed fixture",
            ));
        }
        let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        Ok(Self {
            _storage: storage,
            request_identity: format!("request-{}-{sequence}", process::id()),
            actions: BTreeMap::new(),
        })
    }

    pub(crate) fn command(
        &mut self,
        action: &Value,
        python: String,
        runner: &Path,
    ) -> R<Vec<String>> {
        let payload = serde_json::to_vec(action).map_err(failure)?;
        let digest = sha256(&payload);
        let action_identity = format!("{}-action-{}", self.request_identity, self.actions.len());
        self.actions.insert(
            action_identity.clone(),
            ExpectedAction {
                digest: digest.clone(),
                execution_attempts: Vec::new(),
            },
        );
        Ok(vec![
            python,
            "-B".to_owned(),
            utf8(runner)?,
            "--command-action".to_owned(),
            hex(&payload),
            "--receipt-directory".to_owned(),
            utf8(self._storage.path())?,
            "--request-identity".to_owned(),
            self.request_identity.clone(),
            ACTION_ID_FLAG.to_owned(),
            action_identity,
            "--action-digest".to_owned(),
            digest,
            "--".to_owned(),
        ])
    }

    pub(crate) fn observe_execution(
        &mut self,
        obligation: &RuntimeCommandObligation,
        process_started: bool,
    ) {
        let Some(identity) = flag_value(&obligation.command_prefix, ACTION_ID_FLAG) else {
            return;
        };
        if let Some(action) = self.actions.get_mut(identity) {
            action.execution_attempts.push(process_started);
        }
    }

    pub(crate) fn validate(&self, profile: &IntegrityProfile, outcome: &IntegrityResult) -> R<()> {
        let mut expected: BTreeMap<String, Vec<i32>> = self
            .actions
            .keys()
            .map(|identity| (identity.clone(), Vec::new()))
            .collect();
        if profile.validations.len() != outcome.validations.len() {
            return Err(failure("receipt validation shape mismatch"));
        }
        for (runtime, result) in profile.validations.iter().zip(&outcome.validations) {
            if runtime.obligations.len() != result.obligations.len() {
                return Err(failure("receipt obligation shape mismatch"));
            }
            for (obligation, observed) in runtime.obligations.iter().zip(&result.obligations) {
                let Some(identity) = flag_value(&obligation.command_prefix, ACTION_ID_FLAG) else {
                    continue;
                };
                let codes = expected
                    .get_mut(identity)
                    .ok_or_else(|| failure("foreign action identity in command prefix"))?;
                if observed.executed {
                    let code = observed
                        .exit_code
                        .as_deref()
                        .ok_or_else(|| failure("receipt validation found no integer exit code"))?
                        .parse::<i32>()
                        .map_err(failure)?;
                    codes.push(code);
                }
            }
        }
        for (identity, action) in &self.actions {
            if action.execution_attempts.iter().any(|started| !started) {
                return Err(failure("receipt-bound action failed to launch"));
            }
            if action.execution_attempts.len() != expected[identity].len() {
                return Err(failure(
                    "execution observations contradict receipt-bound obligations",
                ));
            }
        }
        self.validate_records(expected)
    }

    fn validate_records(&self, mut expected: BTreeMap<String, Vec<i32>>) -> R<()> {
        let mut observed: BTreeMap<String, Vec<i32>> = self
            .actions
            .keys()
            .map(|identity| (identity.clone(), Vec::new()))
            .collect();
        let entries = fs::read_dir(self._storage.path())
            .map_err(|error| failure(format!("receipt storage is unavailable: {error}")))?;
        for entry in entries {
            let entry = entry.map_err(failure)?;
            if !entry.file_type().map_err(failure)?.is_file() {
                return Err(failure("receipt storage contains a non-file entry"));
            }
            let bytes = fs::read(entry.path()).map_err(failure)?;
            let receipt: CompletionReceipt = serde_json::from_slice(&bytes).map_err(failure)?;
            if receipt.schema_version != 1
                || receipt.completion != "completed"
                || receipt.request_identity != self.request_identity
            {
                return Err(failure(
                    "receipt request identity or completion marker is invalid",
                ));
            }
            let action = self
                .actions
                .get(&receipt.action_identity)
                .ok_or_else(|| failure("receipt contains a foreign action identity"))?;
            if receipt.action_digest != action.digest {
                return Err(failure("receipt action digest is invalid"));
            }
            observed
                .get_mut(&receipt.action_identity)
                .expect("registered action has an observation bucket")
                .push(receipt.return_code);
        }
        for values in expected.values_mut() {
            values.sort_unstable();
        }
        for values in observed.values_mut() {
            values.sort_unstable();
        }
        if observed != expected {
            return Err(failure(
                "action completion receipts do not match executed obligations",
            ));
        }
        Ok(())
    }
}

fn flag_value<'a>(arguments: &'a [String], flag: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

fn utf8(path: &Path) -> R<String> {
    path.as_os_str()
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| failure("action receipt path is not UTF-8"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn failure(error: impl ToString) -> Error {
    FixtureError::new(error.to_string())
}

#[cfg(test)]
#[path = "action-receipts-tests.rs"]
mod tests;
