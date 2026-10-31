#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::compare::{ComparisonOutcome, compare_observation};
use crate::harness::{
    CandidateExecutor, CandidateRequest, CandidateState, HarnessError, candidate_requests,
};
use crate::loader::LoadedCorpus;
use crate::model::{ComparisonKind, Observation};

#[derive(Clone, Debug)]
pub struct CandidateExecution {
    pub responsibility_id: String,
    pub vector_id: String,
    pub state: CandidateState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StandaloneVerdict {
    Match,
}

#[derive(Clone, Debug, Serialize)]
pub struct StandaloneRecord {
    pub responsibility_id: String,
    pub vector_id: String,
    pub observation: Observation,
    pub comparison: ComparisonKind,
    pub verdict: StandaloneVerdict,
}

pub fn run_standalone(
    corpus: &LoadedCorpus,
    candidate: &dyn CandidateExecutor,
) -> Result<Vec<StandaloneRecord>, HarnessError> {
    let executions = candidate_requests(corpus)
        .into_iter()
        .map(|request| execute(candidate, request))
        .collect::<Result<Vec<_>, _>>()?;
    evaluate_standalone_executions(corpus, executions)
}

fn execute(
    candidate: &dyn CandidateExecutor,
    request: CandidateRequest,
) -> Result<CandidateExecution, HarnessError> {
    let state = candidate.execute(&request)?;
    Ok(CandidateExecution {
        responsibility_id: request.responsibility_id,
        vector_id: request.vector_id,
        state,
    })
}

pub fn evaluate_standalone_executions(
    corpus: &LoadedCorpus,
    executions: Vec<CandidateExecution>,
) -> Result<Vec<StandaloneRecord>, HarnessError> {
    let mut expected = HashMap::new();
    for matrix in &corpus.matrices {
        for vector in &matrix.vectors {
            if expected
                .insert(
                    vector.vector_id.as_str(),
                    (matrix.responsibility_id.as_str(), vector),
                )
                .is_some()
            {
                return Err(failure(format!(
                    "duplicate corpus vector ID: {}",
                    vector.vector_id
                )));
            }
        }
    }

    let mut observed = HashMap::new();
    for execution in executions {
        let Some((responsibility_id, _)) = expected.get(execution.vector_id.as_str()) else {
            return Err(failure(format!(
                "unexpected candidate result: {}",
                execution.vector_id
            )));
        };
        if *responsibility_id != execution.responsibility_id {
            return Err(failure(format!(
                "candidate responsibility mismatch: {}",
                execution.vector_id
            )));
        }
        let vector_id = execution.vector_id.clone();
        if observed.insert(vector_id.clone(), execution).is_some() {
            return Err(failure(format!("duplicate candidate result: {vector_id}")));
        }
    }

    let missing: Vec<_> = expected
        .keys()
        .filter(|vector_id| !observed.contains_key(**vector_id))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(failure(format!("missing candidate result: {missing:?}")));
    }

    let mut records = Vec::with_capacity(expected.len());
    let mut responsibilities = HashSet::new();
    for matrix in &corpus.matrices {
        for vector in &matrix.vectors {
            let execution = observed
                .remove(vector.vector_id.as_str())
                .expect("complete inventory was checked above");
            let observation = match execution.state {
                CandidateState::Observation(observation) => observation,
                CandidateState::Unimplemented => {
                    return Err(failure(format!(
                        "unimplemented candidate: {}",
                        vector.vector_id
                    )));
                }
            };
            let expected_observation = vector.expected_observation.observation();
            if compare_observation(&observation, &expected_observation, &vector.comparison)
                == ComparisonOutcome::Mismatch
            {
                return Err(failure(format!(
                    "observation mismatch: {} actual={observation:?} expected={expected_observation:?}",
                    vector.vector_id
                )));
            }
            responsibilities.insert(matrix.responsibility_id.as_str());
            records.push(StandaloneRecord {
                responsibility_id: matrix.responsibility_id.clone(),
                vector_id: vector.vector_id.clone(),
                observation,
                comparison: vector.comparison.kind,
                verdict: StandaloneVerdict::Match,
            });
        }
    }
    if responsibilities.len() != corpus.matrices.len() {
        return Err(failure("incomplete responsibility inventory".to_owned()));
    }
    Ok(records)
}

fn failure(message: String) -> HarnessError {
    HarnessError::new(message)
}
