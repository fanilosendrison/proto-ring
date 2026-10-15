use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use super::model::{RepositoryPathSelector, SelectorKind, ValidationInstances};
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

pub(super) fn select_repository_paths(
    instances: &ValidationInstances,
    repository_paths: &[PathBuf],
) -> Result<Vec<PathBuf>, RepositoryIntegrityError> {
    let ValidationInstances::RepositoryPaths { selectors, .. } = instances else {
        return Err(failure(
            "repository-path selection requires repository_paths instances",
        ));
    };
    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for selector in selectors {
        let mut matches = match selector.kind {
            SelectorKind::Path => vec![PathBuf::from(&selector.value)],
            SelectorKind::Glob => repository_paths
                .iter()
                .filter_map(|path| {
                    path_to_posix(path)
                        .filter(|candidate| glob_matches(&selector.value, candidate))
                        .map(|_| path.clone())
                })
                .collect(),
        };
        if selector.kind == SelectorKind::Glob {
            matches.sort_by_key(|path| path_to_posix(path).unwrap_or_default());
        }
        for path in matches {
            if !seen.insert(path.clone()) {
                return Err(failure("duplicate selected repository path"));
            }
            selected.push(path);
        }
    }
    Ok(selected)
}

pub(super) fn path_to_posix(path: &Path) -> Option<String> {
    path.components()
        .map(|component| match component {
            Component::Normal(value) => value.to_str().map(str::to_owned),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .map(|components| components.join("/"))
}

fn glob_matches(pattern: &str, candidate: &str) -> bool {
    let patterns: Vec<_> = pattern.split('/').collect();
    let candidates: Vec<_> = candidate.split('/').collect();
    patterns.len() == candidates.len()
        && patterns
            .iter()
            .zip(candidates)
            .all(|(pattern, candidate)| component_matches(pattern.as_bytes(), candidate.as_bytes()))
}

fn component_matches(pattern: &[u8], candidate: &[u8]) -> bool {
    let (mut pattern_index, mut candidate_index) = (0, 0);
    let (mut star, mut retry) = (None, 0);
    while candidate_index < candidate.len() {
        if pattern.get(pattern_index) == candidate.get(candidate_index) {
            pattern_index += 1;
            candidate_index += 1;
        } else if pattern.get(pattern_index) == Some(&b'*') {
            star = Some(pattern_index);
            pattern_index += 1;
            retry = candidate_index;
        } else if let Some(star_index) = star {
            retry += 1;
            candidate_index = retry;
            pattern_index = star_index + 1;
        } else {
            return false;
        }
    }
    pattern[pattern_index..].iter().all(|byte| *byte == b'*')
}

#[cfg(test)]
mod tests {
    use super::component_matches;

    #[test]
    fn star_matches_only_inside_one_path_component() {
        assert!(component_matches(b"*.rs", b"lib.rs"));
        assert!(component_matches(b"a*b*c", b"axybzc"));
        assert!(!component_matches(b"*.rs", b"lib.py"));
    }
}
