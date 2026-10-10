use super::model::{RepositoryPathSelector, SelectorKind};
use super::{RepositoryIntegrityError, failure};

pub(super) fn validate_selector(
    kind: SelectorKind,
    value: String,
) -> Result<RepositoryPathSelector, RepositoryIntegrityError> {
    if value.is_empty() || value.starts_with('/') || value.contains('\\') || value.contains('\0') {
        return Err(failure("selector must be nonempty relative POSIX form"));
    }
    if value
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(failure("selector contains forbidden path component"));
    }
    match kind {
        SelectorKind::Path if value.contains('*') => {
            return Err(failure("literal path contains wildcard"));
        }
        SelectorKind::Glob if value.contains("**") || value.contains(['?', '[', ']', '{', '}']) => {
            return Err(failure("glob contains forbidden syntax"));
        }
        _ => {}
    }
    Ok(RepositoryPathSelector { kind, value })
}
