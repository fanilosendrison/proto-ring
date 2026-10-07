#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::model::{CaseMatrix, CorpusIndex, MigrationDisposition, ResponsibilityRecordSummary};

#[derive(Debug)]
pub struct CorpusLoadError(String);

impl Display for CorpusLoadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CorpusLoadError {}

#[derive(Debug)]
pub struct LoadedCorpus {
    pub index: CorpusIndex,
    pub responsibilities: Vec<ResponsibilityRecordSummary>,
    pub matrices: Vec<CaseMatrix>,
}

pub fn load_corpus(repository_root: &Path) -> Result<LoadedCorpus, CorpusLoadError> {
    let corpus_root = repository_root.join("conformance/v1");
    let canonical_root = corpus_root
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve corpus root: {error}")))?;
    let index: CorpusIndex = read_json(&canonical_root, "index.json")?;

    require_unique_paths(&index.responsibility_files, "responsibility")?;
    require_unique_paths(&index.case_files, "case")?;

    let mut responsibilities = Vec::with_capacity(index.responsibility_files.len());
    let mut responsibility_positions = HashMap::new();
    for relative in &index.responsibility_files {
        let record: ResponsibilityRecordSummary = read_json(&canonical_root, relative)?;
        if responsibility_positions
            .insert(record.responsibility_id.clone(), responsibilities.len())
            .is_some()
        {
            return Err(failure(format!(
                "duplicate responsibility ID: {}",
                record.responsibility_id
            )));
        }
        responsibilities.push(record);
    }

    let mut matrices = Vec::with_capacity(index.case_files.len());
    let mut matrix_paths = HashMap::new();
    let mut vector_ids = HashSet::new();
    for relative in &index.case_files {
        let matrix: CaseMatrix = read_json(&canonical_root, relative)?;
        if matrix_paths
            .insert(matrix.responsibility_id.clone(), relative.clone())
            .is_some()
        {
            return Err(failure(format!(
                "duplicate matrix responsibility ID: {}",
                matrix.responsibility_id
            )));
        }
        for vector in &matrix.vectors {
            if !vector_ids.insert(vector.vector_id.clone()) {
                return Err(failure(format!(
                    "duplicate vector ID: {}",
                    vector.vector_id
                )));
            }
        }
        matrices.push(matrix);
    }

    for responsibility in &responsibilities {
        let matrix_path = matrix_paths.get(&responsibility.responsibility_id);
        match responsibility.migration_disposition {
            MigrationDisposition::RustPort => {
                let declared = responsibility.case_file.as_ref().ok_or_else(|| {
                    failure(format!(
                        "rust_port responsibility has no case file: {}",
                        responsibility.responsibility_id
                    ))
                })?;
                if matrix_path != Some(declared) {
                    return Err(failure(format!(
                        "rust_port matrix mismatch: {}",
                        responsibility.responsibility_id
                    )));
                }
            }
            MigrationDisposition::RetireWithoutRustPort => {
                if responsibility.case_file.is_some() || matrix_path.is_some() {
                    return Err(failure(format!(
                        "retirement-only responsibility has a matrix: {}",
                        responsibility.responsibility_id
                    )));
                }
            }
        }
    }

    for responsibility_id in matrix_paths.keys() {
        let position = responsibility_positions
            .get(responsibility_id)
            .ok_or_else(|| failure(format!("matrix has no responsibility: {responsibility_id}")))?;
        if responsibilities[*position].migration_disposition != MigrationDisposition::RustPort {
            return Err(failure(format!(
                "matrix belongs to non-rust responsibility: {responsibility_id}"
            )));
        }
    }

    Ok(LoadedCorpus {
        index,
        responsibilities,
        matrices,
    })
}

fn require_unique_paths(paths: &[String], kind: &str) -> Result<(), CorpusLoadError> {
    let mut unique = HashSet::new();
    for path in paths {
        if !unique.insert(path) {
            return Err(failure(format!("duplicate {kind} path: {path}")));
        }
    }
    Ok(())
}

fn read_json<T: DeserializeOwned>(
    canonical_root: &Path,
    relative: &str,
) -> Result<T, CorpusLoadError> {
    let path = checked_path(canonical_root, relative)?;
    let bytes = fs::read(&path)
        .map_err(|error| failure(format!("cannot read {}: {error}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| failure(format!("invalid JSON in {}: {error}", path.display())))
}

fn checked_path(canonical_root: &Path, relative: &str) -> Result<PathBuf, CorpusLoadError> {
    let relative_path = Path::new(relative);
    if relative.is_empty()
        || relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(failure(format!("unsafe corpus path: {relative}")));
    }
    let joined = canonical_root.join(relative_path);
    let canonical = joined
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve {relative}: {error}")))?;
    if !canonical.starts_with(canonical_root) || !canonical.is_file() {
        return Err(failure(format!("corpus path escapes root: {relative}")));
    }
    Ok(canonical)
}

fn failure(message: String) -> CorpusLoadError {
    CorpusLoadError(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_validation_rejects_absolute_and_parent_paths() {
        let root = Path::new("/tmp");
        assert!(checked_path(root, "/absolute").is_err());
        assert!(checked_path(root, "../outside").is_err());
        assert!(checked_path(root, "./index.json").is_err());
    }
}
