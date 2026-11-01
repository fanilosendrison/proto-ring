use super::*;
use std::io::Write;

fn channel() -> (ActionReceiptChannel, String) {
    let governed = tempfile::tempdir().unwrap();
    let mut channel = ActionReceiptChannel::new(governed.path()).unwrap();
    let command = channel
        .command(
            &serde_json::json!({"action": "exit", "code": 7}),
            "python".into(),
            Path::new("runner"),
        )
        .unwrap();
    let identity = flag_value(&command, ACTION_ID_FLAG).unwrap().to_owned();
    (channel, identity)
}

fn write_receipt(
    channel: &ActionReceiptChannel,
    identity: &str,
    request: &str,
    digest: &str,
    code: i32,
    suffix: &str,
) {
    let path = channel._storage.path().join(format!("{suffix}.json"));
    let mut file = fs::File::create(path).unwrap();
    writeln!(
        file,
        "{}",
        serde_json::json!({
            "schema_version": 1,
            "completion": "completed",
            "request_identity": request,
            "action_identity": identity,
            "action_digest": digest,
            "return_code": code,
        })
    )
    .unwrap();
}

fn profile(commands: Vec<Vec<String>>, continue_after_non_satisfied: bool) -> IntegrityProfile {
    use proto_ring_engine::repository_integrity::{
        ProfileIdentity, RuntimeCommandObligation, RuntimeValidation,
    };
    use std::collections::BTreeSet;

    IntegrityProfile {
        continue_after_non_satisfied,
        validations: commands
            .into_iter()
            .enumerate()
            .map(|(index, command_prefix)| {
                let id = format!("validation-{index}");
                RuntimeValidation {
                    id: id.clone(),
                    prerequisites: BTreeSet::new(),
                    obligations: vec![RuntimeCommandObligation {
                        validation_id: id,
                        command_prefix,
                        arguments: Vec::new(),
                        undetermined_exit_codes: BTreeSet::new(),
                        process_environment: Some(BTreeMap::new()),
                    }],
                    resolution_failed: false,
                }
            })
            .collect(),
        identity: ProfileIdentity(b"receipt-test".to_vec()),
        governed_paths: Vec::new(),
        resolution_state_identity: None,
    }
}

#[test]
fn launch_failure_is_not_confused_with_policy_skipping() {
    use proto_ring_engine::repository_integrity::{self, EvaluationContextIdentity};
    use proto_ring_engine::repository_state;

    let governed = tempfile::tempdir().unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .arg(governed.path())
            .status()
            .unwrap()
            .success()
    );
    let mut failed_launch = ActionReceiptChannel::new(governed.path()).unwrap();
    let command = failed_launch
        .command(
            &serde_json::json!({"action": "exit", "code": 0}),
            governed.path().join("missing-python").display().to_string(),
            Path::new("runner"),
        )
        .unwrap();
    let failed_profile = profile(vec![command], true);
    let failed_outcome =
        repository_integrity::conformance_internal::evaluate_with_execution_observer(
            governed.path(),
            &failed_profile,
            EvaluationContextIdentity(b"failed-launch".to_vec()),
            |_, root, scope| repository_state::capture(root, scope),
            &BTreeMap::new(),
            |obligation, started| failed_launch.observe_execution(obligation, started),
        );
    assert!(
        failed_launch
            .validate(&failed_profile, &failed_outcome)
            .is_err()
    );

    let mut policy_skipped = ActionReceiptChannel::new(governed.path()).unwrap();
    let skipped_command = policy_skipped
        .command(
            &serde_json::json!({"action": "exit", "code": 0}),
            governed.path().join("also-missing").display().to_string(),
            Path::new("runner"),
        )
        .unwrap();
    let skipped_profile = profile(
        vec![
            vec![
                governed
                    .path()
                    .join("missing-prerequisite")
                    .display()
                    .to_string(),
            ],
            skipped_command,
        ],
        false,
    );
    let skipped_outcome =
        repository_integrity::conformance_internal::evaluate_with_execution_observer(
            governed.path(),
            &skipped_profile,
            EvaluationContextIdentity(b"policy-skip".to_vec()),
            |_, root, scope| repository_state::capture(root, scope),
            &BTreeMap::new(),
            |obligation, started| policy_skipped.observe_execution(obligation, started),
        );
    policy_skipped
        .validate(&skipped_profile, &skipped_outcome)
        .unwrap();
}

#[test]
fn launch_failure_is_not_hidden_by_a_later_coherence_halt() {
    use proto_ring_engine::repository_integrity::{self, EvaluationContextIdentity};
    use proto_ring_engine::repository_state::{self, StateCaptureError};

    let governed = tempfile::tempdir().unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .arg(governed.path())
            .status()
            .unwrap()
            .success()
    );
    let mut channel = ActionReceiptChannel::new(governed.path()).unwrap();
    let command = channel
        .command(
            &serde_json::json!({"action": "exit", "code": 0}),
            governed.path().join("missing-python").display().to_string(),
            Path::new("runner"),
        )
        .unwrap();
    let profile = profile(vec![command], true);
    let outcome = repository_integrity::conformance_internal::evaluate_with_execution_observer(
        governed.path(),
        &profile,
        EvaluationContextIdentity(b"failed-launch-post-capture".to_vec()),
        |phase, root, scope| {
            if phase == repository_integrity::conformance_internal::CapturePhase::PostObligation {
                return Err(StateCaptureError::new(
                    "post-obligation capture unavailable",
                ));
            }
            repository_state::capture(root, scope)
        },
        &BTreeMap::new(),
        |obligation, started| channel.observe_execution(obligation, started),
    );
    assert!(outcome.state_coherence_halt);
    assert!(!outcome.obligations[0].executed);
    assert!(
        channel.validate(&profile, &outcome).is_err(),
        "an invoked command that failed to start must remain a harness error"
    );
}

#[test]
fn valid_receipt_matches_one_executed_action() {
    let (channel, identity) = channel();
    let digest = channel.actions[&identity].digest.clone();
    write_receipt(
        &channel,
        &identity,
        &channel.request_identity,
        &digest,
        7,
        "one",
    );
    channel
        .validate_records(BTreeMap::from([(identity, vec![7])]))
        .unwrap();
}

#[test]
fn absent_forged_and_duplicate_receipts_fail_closed() {
    let (channel, identity) = channel();
    assert!(
        channel
            .validate_records(BTreeMap::from([(identity.clone(), vec![7])]))
            .is_err()
    );

    let digest = channel.actions[&identity].digest.clone();
    write_receipt(&channel, &identity, "foreign-request", &digest, 7, "forged");
    assert!(
        channel
            .validate_records(BTreeMap::from([(identity.clone(), vec![7])]))
            .is_err()
    );

    fs::remove_file(channel._storage.path().join("forged.json")).unwrap();
    write_receipt(
        &channel,
        &identity,
        &channel.request_identity,
        "forged-digest",
        7,
        "forged-digest",
    );
    assert!(
        channel
            .validate_records(BTreeMap::from([(identity.clone(), vec![7])]))
            .is_err()
    );

    fs::remove_file(channel._storage.path().join("forged-digest.json")).unwrap();
    write_receipt(
        &channel,
        &identity,
        &channel.request_identity,
        &digest,
        7,
        "one",
    );
    write_receipt(
        &channel,
        &identity,
        &channel.request_identity,
        &digest,
        7,
        "two",
    );
    assert!(
        channel
            .validate_records(BTreeMap::from([(identity, vec![7])]))
            .is_err()
    );
}
