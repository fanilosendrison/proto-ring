#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct CorpusIndex {
    pub responsibility_files: Vec<String>,
    pub case_files: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationDisposition {
    RustPort,
    RetireWithoutRustPort,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResponsibilityRecordSummary {
    pub responsibility_id: String,
    pub migration_disposition: MigrationDisposition,
    pub case_file: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CaseMatrix {
    pub responsibility_id: String,
    pub vectors: Vec<Vector>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Vector {
    pub vector_id: String,
    pub fixture: Value,
    pub expected_observation: ExpectedObservation,
    pub comparison: ComparisonRule,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExpectedObservation {
    pub kind: ObservationKind,
    pub value: TransportValue,
    pub derivation: Value,
}

impl ExpectedObservation {
    pub fn observation(&self) -> Observation {
        Observation {
            kind: self.kind,
            value: self.value.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub kind: ObservationKind,
    pub value: TransportValue,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    Result,
    ControlledRejection,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ComparisonRule {
    pub kind: ComparisonKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonKind {
    Exact,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum TransportValue {
    Null(()),
    Boolean(bool),
    Integer(String),
    String(String),
    Bytes(String),
    Sequence(Vec<TransportValue>),
    Record(Vec<RecordEntry>),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecordEntry {
    pub name: String,
    pub value: TransportValue,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonObservationRecord {
    pub responsibility_id: String,
    pub vector_id: String,
    pub observation: Observation,
}
