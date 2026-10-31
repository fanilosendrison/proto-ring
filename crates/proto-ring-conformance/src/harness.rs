#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::Serialize;
use serde_json::Value;

use crate::loader::LoadedCorpus;
use crate::model::Observation;

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
