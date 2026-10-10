use super::model::{ConsumerIntegrityProfile, ProfileIdentity, ValidationInstances};

pub(super) fn profile_identity(profile: &ConsumerIntegrityProfile) -> ProfileIdentity {
    let mut output = Vec::new();
    field(
        &mut output,
        "model_version",
        &profile.model_version.to_string(),
    );
    field(
        &mut output,
        "authority.responsibility",
        &profile.authority.responsibility,
    );
    field(&mut output, "authority.source", &profile.authority.source);
    for environment in &profile.environments {
        field(&mut output, "environment", environment);
    }
    field(
        &mut output,
        "continue",
        if profile.continue_after_non_satisfied {
            "true"
        } else {
            "false"
        },
    );
    for (id, validation) in &profile.validations {
        field(&mut output, "validation.id", id);
        field(
            &mut output,
            "validation.responsibility",
            &validation.responsibility,
        );
        for prerequisite in &validation.prerequisites {
            field(&mut output, "prerequisite", prerequisite);
        }
        match &validation.target {
            Some(target) => {
                field(&mut output, "target.present", "true");
                field(&mut output, "target.interface", &target.interface_id);
                field(&mut output, "target.object", &target.object_id);
            }
            None => field(&mut output, "target.present", "false"),
        }
        field(&mut output, "instances.kind", validation.instances.kind());
        if let ValidationInstances::RepositoryPaths { mode, selectors } = &validation.instances {
            field(&mut output, "instances.mode", mode.serialized());
            for selector in selectors {
                field(&mut output, "selector.kind", selector.kind.serialized());
                field(&mut output, "selector.value", &selector.value);
            }
        }
        field(
            &mut output,
            "command.environment",
            &validation.command.environment,
        );
        for argument in &validation.command.arguments {
            field(&mut output, "command.argument", argument);
        }
        for code in &validation.command.undetermined_exit_codes {
            field(&mut output, "command.undetermined_exit_code", code);
        }
    }
    for id in &profile.order {
        field(&mut output, "order", id);
    }
    ProfileIdentity(output)
}

fn field(output: &mut Vec<u8>, tag: &str, value: &str) {
    length(output, tag.len());
    output.extend_from_slice(tag.as_bytes());
    length(output, value.len());
    output.extend_from_slice(value.as_bytes());
}

fn length(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    use super::*;
    use crate::repository_integrity::{
        CommandBinding, ProfileAuthority, RepositoryPathSelector, RepositoryPathsMode,
        SelectorKind, ValidationDefinition,
    };

    fn profile(arguments: &[&str], selectors: &[&str], codes: &[&str]) -> ConsumerIntegrityProfile {
        let validation = ValidationDefinition {
            id: "validation".to_owned(),
            responsibility: "responsibility".to_owned(),
            prerequisites: ["prerequisite".to_owned()].into_iter().collect(),
            instances: ValidationInstances::RepositoryPaths {
                mode: RepositoryPathsMode::ForEach,
                selectors: selectors
                    .iter()
                    .map(|value| RepositoryPathSelector {
                        kind: SelectorKind::Path,
                        value: (*value).to_owned(),
                    })
                    .collect(),
            },
            command: CommandBinding {
                environment: "environment".to_owned(),
                arguments: arguments.iter().map(|value| (*value).to_owned()).collect(),
                undetermined_exit_codes: codes.iter().map(|value| (*value).to_owned()).collect(),
            },
            target: None,
        };
        ConsumerIntegrityProfile {
            repository: PathBuf::new(),
            carrier: PathBuf::new(),
            model_version: 1,
            authority: ProfileAuthority {
                responsibility: "profile".to_owned(),
                source: "source".to_owned(),
            },
            environments: BTreeSet::from(["environment".to_owned()]),
            continue_after_non_satisfied: true,
            validations: BTreeMap::from([("validation".to_owned(), validation)]),
            order: vec!["validation".to_owned()],
            identity: ProfileIdentity(Vec::new()),
        }
    }

    #[test]
    fn unordered_sets_normalize() {
        let left = profile(&["arg"], &["a"], &["2", "3"]);
        let right = profile(&["arg"], &["a"], &["3", "2"]);
        assert_eq!(profile_identity(&left), profile_identity(&right));
    }

    #[test]
    fn ordered_sequences_do_not_normalize() {
        let left = profile(&["first", "second"], &["a", "b"], &["2"]);
        let arguments_changed = profile(&["second", "first"], &["a", "b"], &["2"]);
        let selectors_changed = profile(&["first", "second"], &["b", "a"], &["2"]);
        assert_ne!(
            profile_identity(&left),
            profile_identity(&arguments_changed)
        );
        assert_ne!(
            profile_identity(&left),
            profile_identity(&selectors_changed)
        );
    }

    #[test]
    fn arbitrary_precision_exit_code_is_retained_exactly() {
        let value = format!("1{}", "0".repeat(500));
        let profile = profile(&[], &[], &[&value]);
        assert!(
            profile.validations["validation"]
                .command
                .undetermined_exit_codes
                .contains(&value)
        );
        assert!(!profile_identity(&profile).0.is_empty());
    }
}
