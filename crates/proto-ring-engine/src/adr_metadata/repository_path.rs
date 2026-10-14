use std::path::{Path, PathBuf};

use super::{AdrMetadataError, failure};

pub fn resolve_repository_path(
    repository_root: &Path,
    relative: &str,
) -> Result<PathBuf, AdrMetadataError> {
    if relative.is_empty() {
        return Err(failure("repository-relative path must be nonempty"));
    }
    let declared = Path::new(relative);
    if declared.is_absolute() {
        return Err(failure("repository-relative path must not be absolute"));
    }
    let resolved_root = repository_root
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve repository root: {error}")))?;
    let resolved = resolved_root
        .join(declared)
        .canonicalize()
        .map_err(|error| failure(format!("cannot resolve repository-relative path: {error}")))?;
    if !resolved.starts_with(&resolved_root) {
        return Err(failure("resolved path is outside repository root"));
    }
    Ok(resolved)
}
