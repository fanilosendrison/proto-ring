#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use tempfile::NamedTempFile;

use crate::compare::{ComparisonOutcome, compare_observation};
use crate::loader::LoadedCorpus;
use crate::model::{ComparisonRule, Observation, PythonObservationRecord, Vector};

#[derive(Clone, Debug, Serialize)]
pub struct CandidateRequest {
    pub responsibility_id: String,
    pub vector_id: String,
    pub fixture: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateState {
    Observation(Observation),
    Unimplemented,
}

pub trait CandidateExecutor {
    fn execute(&self, request: &CandidateRequest) -> Result<CandidateState, HarnessError>;
}

#[derive(Debug)]
pub struct UnimplementedRustCandidate;

impl CandidateExecutor for UnimplementedRustCandidate {
    fn execute(&self, _request: &CandidateRequest) -> Result<CandidateState, HarnessError> {
        Ok(CandidateState::Unimplemented)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonStatus {
    Match,
    Mismatch,
    NotAvailable,
}

#[derive(Clone, Debug)]
pub struct DifferentialReport {
    pub responsibility_id: String,
    pub vector_id: String,
    pub expected_observation: Observation,
    pub python_observation: Observation,
    pub rust_candidate: CandidateState,
    pub python_vs_expected: ComparisonStatus,
    pub rust_vs_expected: ComparisonStatus,
    pub python_vs_rust: ComparisonStatus,
}

#[derive(Debug)]
pub struct HarnessError(String);

impl HarnessError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for HarnessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for HarnessError {}

pub struct PythonBridge {
    repository_root: PathBuf,
}

pub fn serialize_candidate_requests(
    requests: &[CandidateRequest],
) -> Result<Vec<u8>, HarnessError> {
    let mut bytes = serde_json::to_vec_pretty(requests)
        .map_err(|error| failure(format!("cannot serialize bridge request: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

impl PythonBridge {
    pub fn new(repository_root: &Path) -> Self {
        Self {
            repository_root: repository_root.to_path_buf(),
        }
    }

    pub fn execute(
        &self,
        requests: &[CandidateRequest],
    ) -> Result<HashMap<String, Observation>, HarnessError> {
        let expected = request_inventory(requests)?;
        let request_file = NamedTempFile::new()
            .map_err(|error| failure(format!("cannot create bridge request: {error}")))?;
        fs::write(request_file.path(), serialize_candidate_requests(requests)?)
            .map_err(|error| failure(format!("cannot write bridge request: {error}")))?;
        let output_file = NamedTempFile::new()
            .map_err(|error| failure(format!("cannot create bridge output: {error}")))?;
        let python = std::env::var_os("PROTO_RING_PYTHON").unwrap_or_else(|| "python3".into());
        let completed = Command::new(python)
            .current_dir(&self.repository_root)
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .arg("-B")
            .arg("tests/conformance_python_bridge.py")
            .arg("--request")
            .arg(request_file.path())
            .arg("--output")
            .arg(output_file.path())
            .output()
            .map_err(|error| failure(format!("cannot execute Python bridge: {error}")))?;
        if !completed.status.success() {
            return Err(failure(format!(
                "Python bridge failed: {}",
                String::from_utf8_lossy(&completed.stderr).trim()
            )));
        }
        let bytes = fs::read(output_file.path())
            .map_err(|error| failure(format!("cannot read bridge output: {error}")))?;
        let records: Vec<PythonObservationRecord> = serde_json::from_slice(&bytes)
            .map_err(|error| failure(format!("invalid bridge output: {error}")))?;
        validate_bridge_records(records, &expected)
    }
}

fn request_inventory(
    requests: &[CandidateRequest],
) -> Result<HashMap<String, String>, HarnessError> {
    let mut expected = HashMap::new();
    for request in requests {
        if expected
            .insert(request.vector_id.clone(), request.responsibility_id.clone())
            .is_some()
        {
            return Err(failure(format!(
                "duplicate requested vector ID: {}",
                request.vector_id
            )));
        }
    }
    Ok(expected)
}

fn validate_bridge_records(
    records: Vec<PythonObservationRecord>,
    expected: &HashMap<String, String>,
) -> Result<HashMap<String, Observation>, HarnessError> {
    let mut observations = HashMap::new();
    for record in records {
        let responsibility_id = expected
            .get(&record.vector_id)
            .ok_or_else(|| failure(format!("unknown bridge vector: {}", record.vector_id)))?;
        if responsibility_id != &record.responsibility_id {
            return Err(failure(format!(
                "bridge responsibility mismatch: {}",
                record.vector_id
            )));
        }
        if observations
            .insert(record.vector_id.clone(), record.observation)
            .is_some()
        {
            return Err(failure(format!(
                "duplicate bridge vector: {}",
                record.vector_id
            )));
        }
    }
    let missing: Vec<_> = expected
        .keys()
        .filter(|vector_id| !observations.contains_key(*vector_id))
        .collect();
    if !missing.is_empty() {
        return Err(failure(format!("missing bridge vectors: {missing:?}")));
    }
    Ok(observations)
}

pub fn candidate_requests(corpus: &LoadedCorpus) -> Vec<CandidateRequest> {
    corpus
        .matrices
        .iter()
        .flat_map(|matrix| {
            matrix.vectors.iter().map(|vector| CandidateRequest {
                responsibility_id: matrix.responsibility_id.clone(),
                vector_id: vector.vector_id.clone(),
                fixture: vector.fixture.clone(),
            })
        })
        .collect()
}

pub fn run_differential(
    corpus: &LoadedCorpus,
    python_observations: &HashMap<String, Observation>,
    rust_candidate: &dyn CandidateExecutor,
) -> Result<Vec<DifferentialReport>, HarnessError> {
    let mut reports = Vec::new();
    for matrix in &corpus.matrices {
        for vector in &matrix.vectors {
            let python = python_observations
                .get(&vector.vector_id)
                .ok_or_else(|| failure(format!("missing Python vector: {}", vector.vector_id)))?;
            let request = CandidateRequest {
                responsibility_id: matrix.responsibility_id.clone(),
                vector_id: vector.vector_id.clone(),
                fixture: vector.fixture.clone(),
            };
            let rust_state = rust_candidate.execute(&request)?;
            reports.push(build_report(request, vector, python.clone(), rust_state));
        }
    }
    Ok(reports)
}

fn build_report(
    request: CandidateRequest,
    vector: &Vector,
    python_observation: Observation,
    rust_candidate: CandidateState,
) -> DifferentialReport {
    let expected_observation = vector.expected_observation.observation();
    let python_vs_expected = compared(
        &python_observation,
        &expected_observation,
        &vector.comparison,
    );
    let (rust_vs_expected, python_vs_rust) = match &rust_candidate {
        CandidateState::Observation(rust) => (
            compared(rust, &expected_observation, &vector.comparison),
            compared(&python_observation, rust, &vector.comparison),
        ),
        CandidateState::Unimplemented => (
            ComparisonStatus::NotAvailable,
            ComparisonStatus::NotAvailable,
        ),
    };
    DifferentialReport {
        responsibility_id: request.responsibility_id,
        vector_id: request.vector_id,
        expected_observation,
        python_observation,
        rust_candidate,
        python_vs_expected,
        rust_vs_expected,
        python_vs_rust,
    }
}

fn compared(
    actual: &Observation,
    expected: &Observation,
    rule: &ComparisonRule,
) -> ComparisonStatus {
    match compare_observation(actual, expected, rule) {
        ComparisonOutcome::Match => ComparisonStatus::Match,
        ComparisonOutcome::Mismatch => ComparisonStatus::Mismatch,
    }
}

fn failure(message: String) -> HarnessError {
    HarnessError::new(message)
}

pub fn implemented_responsibilities(reports: &[DifferentialReport]) -> usize {
    reports
        .iter()
        .filter_map(|report| match report.rust_candidate {
            CandidateState::Observation(_) => Some(&report.responsibility_id),
            CandidateState::Unimplemented => None,
        })
        .collect::<HashSet<_>>()
        .len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ComparisonKind, ExpectedObservation, ObservationKind, TransportValue};

    struct MatchingCandidate(HashMap<String, Observation>);
    struct MismatchingCandidate(HashMap<String, Observation>);

    fn execute_from(
        observations: &HashMap<String, Observation>,
        request: &CandidateRequest,
    ) -> Result<CandidateState, HarnessError> {
        observations
            .get(&request.vector_id)
            .cloned()
            .map(CandidateState::Observation)
            .ok_or_else(|| failure(format!("missing test observation: {}", request.vector_id)))
    }

    impl CandidateExecutor for MatchingCandidate {
        fn execute(&self, request: &CandidateRequest) -> Result<CandidateState, HarnessError> {
            execute_from(&self.0, request)
        }
    }

    impl CandidateExecutor for MismatchingCandidate {
        fn execute(&self, request: &CandidateRequest) -> Result<CandidateState, HarnessError> {
            execute_from(&self.0, request)
        }
    }

    fn integer(value: &str) -> Observation {
        Observation {
            kind: ObservationKind::Result,
            value: TransportValue::Integer(value.into()),
        }
    }

    fn vector(expected: Observation) -> Vector {
        Vector {
            vector_id: "synthetic.vector".into(),
            fixture: Value::Null,
            expected_observation: ExpectedObservation {
                kind: expected.kind,
                value: expected.value,
                derivation: Value::Null,
            },
            comparison: ComparisonRule {
                kind: ComparisonKind::Exact,
            },
        }
    }

    fn request() -> CandidateRequest {
        CandidateRequest {
            responsibility_id: "synthetic.responsibility".into(),
            vector_id: "synthetic.vector".into(),
            fixture: Value::Null,
        }
    }

    #[test]
    fn all_three_channels_match_independently() {
        let expected = integer("1");
        let candidate =
            MatchingCandidate(HashMap::from([("synthetic.vector".into(), integer("1"))]));
        let report = build_report(
            request(),
            &vector(expected),
            integer("1"),
            candidate.execute(&request()).unwrap(),
        );
        assert_eq!(report.python_vs_expected, ComparisonStatus::Match);
        assert_eq!(report.rust_vs_expected, ComparisonStatus::Match);
        assert_eq!(report.python_vs_rust, ComparisonStatus::Match);
    }

    #[test]
    fn rust_mismatch_does_not_change_python_conformance() {
        let candidate =
            MismatchingCandidate(HashMap::from([("synthetic.vector".into(), integer("2"))]));
        let report = build_report(
            request(),
            &vector(integer("1")),
            integer("1"),
            candidate.execute(&request()).unwrap(),
        );
        assert_eq!(report.python_vs_expected, ComparisonStatus::Match);
        assert_eq!(report.rust_vs_expected, ComparisonStatus::Mismatch);
        assert_eq!(report.python_vs_rust, ComparisonStatus::Mismatch);
    }

    #[test]
    fn python_and_rust_can_match_while_both_are_nonconformant() {
        let candidate =
            MatchingCandidate(HashMap::from([("synthetic.vector".into(), integer("2"))]));
        let report = build_report(
            request(),
            &vector(integer("1")),
            integer("2"),
            candidate.execute(&request()).unwrap(),
        );
        assert_eq!(report.python_vs_expected, ComparisonStatus::Mismatch);
        assert_eq!(report.rust_vs_expected, ComparisonStatus::Mismatch);
        assert_eq!(report.python_vs_rust, ComparisonStatus::Match);
    }
}
