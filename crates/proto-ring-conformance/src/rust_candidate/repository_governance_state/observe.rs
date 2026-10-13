use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use proto_ring_engine::repository_governance_state::{self, RepositoryGovernanceState};

use super::super::transport::{
    fixture_failure, record, rejection, result, string_value, unique_dynamic_record,
};
use crate::fixture;
use crate::model::{Observation, TransportValue};

pub(crate) struct InvocationResult {
    pub(crate) observation: Observation,
    pub(crate) state: Option<RepositoryGovernanceState>,
}

pub(crate) fn execute(
    root: &Path,
    arguments: &TransportValue,
) -> Result<InvocationResult, fixture::FixtureError> {
    let fields = fields(arguments)?;
    require_allowed(
        &fields,
        &[
            "marker_path",
            "operation",
            "participating_paths",
            "qualified_observation",
        ],
    )?;
    if required_string(&fields, "operation")? != "compose" {
        return Err(fixture_failure(
            "unsupported RepositoryGovernanceState operation",
        ));
    }
    let qualified = optional_boolean(&fields, "qualified_observation")?.unwrap_or(false);
    let participating = optional_paths(&fields, "participating_paths")?;
    let marker = optional_string(&fields, "marker_path")?;
    let before = qualified
        .then(|| ReadOnlyProbe::capture(root, &participating))
        .transpose()?;

    let loaded = repository_governance_state::load(root);
    let state = match loaded {
        Ok(state) => state,
        Err(error) => {
            let observation = if qualified {
                result(record(vec![
                    ("stage", string_value(error.diagnostic.stage.serialized())),
                    ("status", string_value("CONTROLLED_FAILURE")),
                ]))
            } else {
                rejection("repository-governance-state.compose")
            };
            return Ok(InvocationResult {
                observation,
                state: None,
            });
        }
    };

    let observation = if qualified {
        let after = ReadOnlyProbe::capture(root, &participating)?;
        qualified_success(
            root,
            &state,
            before.expect("qualified probe exists"),
            after,
            marker,
        )?
    } else {
        normal_success(&state)?
    };
    Ok(InvocationResult {
        observation,
        state: Some(state),
    })
}

pub(crate) fn controlled_failure(stage: &str) -> InvocationResult {
    InvocationResult {
        observation: result(record(vec![
            ("stage", string_value(stage)),
            ("status", string_value("CONTROLLED_FAILURE")),
        ])),
        state: None,
    }
}

fn normal_success(state: &RepositoryGovernanceState) -> Result<Observation, fixture::FixtureError> {
    Ok(result(record(vec![
        (
            "identity_present",
            TransportValue::Boolean(!state.observed_state.identity.0.is_empty()),
        ),
        ("optional_components", optional_components(state)),
        ("scope_paths", scope_paths(state)?),
        ("stage", string_value("composition")),
    ])))
}

fn qualified_success(
    root: &Path,
    state: &RepositoryGovernanceState,
    before: ReadOnlyProbe,
    after: ReadOnlyProbe,
    marker: Option<&str>,
) -> Result<Observation, fixture::FixtureError> {
    let capabilities = state
        .repository_governance_model
        .capabilities
        .keys()
        .map(|capability| string_value(capability))
        .collect();
    let marker_absent = marker
        .map(|relative| marker_path(root, relative).map(|path| !path.exists()))
        .transpose()?
        .unwrap_or(true);
    Ok(result(record(vec![
        ("capabilities", TransportValue::Sequence(capabilities)),
        (
            "identity_present",
            TransportValue::Boolean(!state.observed_state.identity.0.is_empty()),
        ),
        ("marker_absent", TransportValue::Boolean(marker_absent)),
        ("optional_components", optional_components(state)),
        (
            "provider_interactions",
            TransportValue::Integer("0".to_owned()),
        ),
        ("purity", before.relations(&after)),
        ("scope_paths", scope_paths(state)?),
        ("stage", string_value("composition")),
    ])))
}

fn optional_components(state: &RepositoryGovernanceState) -> TransportValue {
    record(vec![
        (
            "evidence_requirements",
            TransportValue::Boolean(state.evidence_requirements.is_some()),
        ),
        (
            "governed_objects",
            TransportValue::Boolean(state.governed_objects.is_some()),
        ),
        (
            "projection_registry",
            TransportValue::Boolean(state.projection_registry.is_some()),
        ),
        (
            "repository_integrity",
            TransportValue::Boolean(state.repository_integrity.is_some()),
        ),
    ])
}

fn scope_paths(state: &RepositoryGovernanceState) -> Result<TransportValue, fixture::FixtureError> {
    state
        .observed_state
        .scope_paths
        .iter()
        .map(|path| {
            path.to_str().map(string_value).ok_or_else(|| {
                fixture_failure("scope path is not representable by UTF-8 transport")
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(TransportValue::Sequence)
}

type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;

fn fields(value: &TransportValue) -> Result<Fields<'_>, fixture::FixtureError> {
    Ok(
        unique_dynamic_record(value, "RepositoryGovernanceState invocation")?
            .into_iter()
            .collect(),
    )
}

fn require_allowed(fields: &Fields<'_>, allowed: &[&str]) -> Result<(), fixture::FixtureError> {
    if let Some(name) = fields.keys().find(|name| !allowed.contains(name)) {
        Err(fixture_failure(format!(
            "RepositoryGovernanceState invocation has unknown field: {name}"
        )))
    } else {
        Ok(())
    }
}

fn required_string<'a>(fields: &Fields<'a>, name: &str) -> Result<&'a str, fixture::FixtureError> {
    fixture::string(
        fields
            .get(name)
            .copied()
            .ok_or_else(|| fixture_failure(format!("missing invocation field: {name}")))?,
    )
}

fn optional_string<'a>(
    fields: &Fields<'a>,
    name: &str,
) -> Result<Option<&'a str>, fixture::FixtureError> {
    fields
        .get(name)
        .map(|value| fixture::string(value))
        .transpose()
}

fn optional_boolean(
    fields: &Fields<'_>,
    name: &str,
) -> Result<Option<bool>, fixture::FixtureError> {
    fields
        .get(name)
        .map(|value| match value {
            TransportValue::Boolean(value) => Ok(*value),
            _ => Err(fixture_failure(format!(
                "invocation field {name} must be boolean"
            ))),
        })
        .transpose()
}

fn optional_paths(fields: &Fields<'_>, name: &str) -> Result<Vec<PathBuf>, fixture::FixtureError> {
    fields
        .get(name)
        .map(|value| {
            fixture::sequence(value)?
                .iter()
                .map(|value| fixture::string(value).map(PathBuf::from))
                .collect()
        })
        .transpose()
        .map(Option::unwrap_or_default)
}

fn marker_path(root: &Path, relative: &str) -> Result<PathBuf, fixture::FixtureError> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(fixture_failure("marker path escapes fixture root"));
    }
    Ok(root.join(relative))
}

#[derive(Eq, PartialEq)]
struct ReadOnlyProbe {
    bytes: Vec<(PathBuf, Vec<u8>)>,
    config: Vec<u8>,
    head: Vec<u8>,
    index: Vec<u8>,
    status: Vec<u8>,
}

impl ReadOnlyProbe {
    fn capture(root: &Path, paths: &[PathBuf]) -> Result<Self, fixture::FixtureError> {
        let bytes = paths
            .iter()
            .map(|relative| {
                fs::read(root.join(relative))
                    .map(|bytes| (relative.clone(), bytes))
                    .map_err(|error| fixture_failure(format!("purity byte probe failed: {error}")))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            bytes,
            config: test_git(root, &["config", "--local", "--list", "-z"])?,
            head: test_git(root, &["rev-parse", "HEAD"])?,
            index: test_git(root, &["ls-files", "--stage", "-z"])?,
            status: test_git(root, &["status", "--porcelain=v1", "-z"])?,
        })
    }

    fn relations(&self, after: &Self) -> TransportValue {
        record(vec![
            ("bytes", TransportValue::Boolean(self.bytes == after.bytes)),
            (
                "config",
                TransportValue::Boolean(self.config == after.config),
            ),
            ("head", TransportValue::Boolean(self.head == after.head)),
            ("index", TransportValue::Boolean(self.index == after.index)),
            (
                "status",
                TransportValue::Boolean(self.status == after.status),
            ),
        ])
    }
}

fn test_git(root: &Path, arguments: &[&str]) -> Result<Vec<u8>, fixture::FixtureError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|error| fixture_failure(format!("read-only Git probe failed: {error}")))?;
    if !output.status.success() {
        return Err(fixture_failure(format!(
            "read-only Git probe exited {:?}",
            output.status.code()
        )));
    }
    Ok(output.stdout)
}
