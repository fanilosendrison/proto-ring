//! Consumer-owned declarations for realization probes.
use super::super::*;
use std::path::PathBuf;

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
pub fn consumer(instances: Option<ValidationInstances>) -> ConsumerIntegrityProfile {
    let definitions = instances.map(|instances| ValidationDefinition {
        id: "v0".into(),
        responsibility: "consumer-quality".into(),
        prerequisites: Default::default(),
        instances,
        command: CommandBinding {
            environment: "runtime".into(),
            arguments: vec!["base".into()],
            undetermined_exit_codes: ["7".into()].into(),
        },
        target: None,
    });
    ConsumerIntegrityProfile {
        repository: root(),
        carrier: PathBuf::from("profile.md"),
        model_version: 1,
        authority: ProfileAuthority {
            responsibility: "consumer-quality".into(),
            source: "consumer".into(),
        },
        environments: ["runtime".into()].into(),
        continue_after_non_satisfied: true,
        order: definitions
            .as_ref()
            .map(|_| vec!["v0".into()])
            .unwrap_or_default(),
        validations: definitions
            .map(|definition| [("v0".into(), definition)].into())
            .unwrap_or_default(),
        identity: ProfileIdentity(vec![11]),
    }
}
pub fn paths(mode: RepositoryPathsMode) -> ValidationInstances {
    ValidationInstances::RepositoryPaths {
        mode,
        selectors: vec![RepositoryPathSelector {
            kind: SelectorKind::Glob,
            value: "src/*.rs".into(),
        }],
    }
}
pub fn context() -> EvaluationContext {
    EvaluationContext {
        realizations: [(
            "runtime".into(),
            ValidationEnvironmentRealization {
                environment_id: "runtime".into(),
                identity: vec![9],
                command_prefix: vec!["program".into(), "prefix".into()],
                process_environment: [("ONLY".into(), "explicit".into())].into(),
            },
        )]
        .into(),
    }
}
