use std::fs;
use std::path::Path;

use crate::structured_data::{self, StructuredValue};

const BRANCH_PREFIX: &str = "refs/heads/";
const MECHANISM: &str = "github-repository-ruleset";

#[derive(Debug)]
pub(super) struct Binding {
    pub owner: String,
    pub repository: String,
    pub branch: String,
    pub ruleset_id: String,
}

#[derive(Debug)]
pub(super) struct BindingError;

pub(super) fn load(path: &Path) -> Result<Binding, BindingError> {
    let bytes = fs::read(path).map_err(|_| BindingError)?;
    parse(&bytes)
}

fn parse(bytes: &[u8]) -> Result<Binding, BindingError> {
    let parsed = structured_data::parse_frontmatter_bytes(bytes).map_err(|_| BindingError)?;
    let root = mapping(&parsed.metadata)?;
    let configuration = mapping(required(root, "authoritative_ref_monotonicity")?)?;
    let repository = mapping(required(configuration, "repository")?)?;
    let protection = mapping(required(configuration, "protection")?)?;

    if nonempty_string(required(repository, "provider")?)? != "github" {
        return Err(BindingError);
    }
    let owner = nonempty_string(required(repository, "owner")?)?.to_owned();
    let repository = nonempty_string(required(repository, "name")?)?.to_owned();
    let authoritative_ref = nonempty_string(required(configuration, "authoritative_ref")?)?;
    let branch = authoritative_ref
        .strip_prefix(BRANCH_PREFIX)
        .filter(|value| !value.is_empty())
        .ok_or(BindingError)?
        .to_owned();

    if nonempty_string(required(protection, "mechanism")?)? != MECHANISM {
        return Err(BindingError);
    }
    let ruleset_id = positive_integer(required(protection, "ruleset_id")?)?.to_owned();

    Ok(Binding {
        owner,
        repository,
        branch,
        ruleset_id,
    })
}

fn required<'a>(
    values: &'a std::collections::BTreeMap<String, StructuredValue>,
    name: &str,
) -> Result<&'a StructuredValue, BindingError> {
    values.get(name).ok_or(BindingError)
}

fn mapping(
    value: &StructuredValue,
) -> Result<&std::collections::BTreeMap<String, StructuredValue>, BindingError> {
    match value {
        StructuredValue::Mapping(values) => Ok(values),
        _ => Err(BindingError),
    }
}

fn nonempty_string(value: &StructuredValue) -> Result<&str, BindingError> {
    match value {
        StructuredValue::String(value) if !value.is_empty() => Ok(value),
        _ => Err(BindingError),
    }
}

fn positive_integer(value: &StructuredValue) -> Result<&str, BindingError> {
    match value {
        StructuredValue::Integer(value)
            if value != "0"
                && !value.starts_with('-')
                && value.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            Ok(value)
        }
        _ => Err(BindingError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(ruleset_id: &str) -> Vec<u8> {
        format!(
            "---\nauthoritative_ref_monotonicity:\n  repository:\n    provider: github\n    owner: owner\n    name: repository\n  authoritative_ref: refs/heads/main\n  protection:\n    mechanism: github-repository-ruleset\n    ruleset_id: {ruleset_id}\n---\n"
        )
        .into_bytes()
    }

    #[test]
    fn ruleset_identity_uses_canonical_arbitrary_precision_integer_text() {
        assert!(parse(&binding("007")).is_err());
        let large = "1234567890123456789012345678901234567890";
        assert_eq!(parse(&binding(large)).unwrap().ruleset_id, large);
    }
}
