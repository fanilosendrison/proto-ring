use super::{invoke, parsed, strings};
use serde_json::{Value, json};

#[test]
fn discover_runs_outside_git_from_other_cwd_and_matches_full_accepted_catalog() {
    let temp = tempfile::tempdir().unwrap();
    let output = invoke(&strings(&["discover", "--json"]), b"", temp.path());
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../docs/api/v1/examples/response-discover.json"
    ))
    .unwrap();
    assert_eq!(parsed(&output, 0), expected);
    assert_eq!(
        parsed(
            &invoke(
                &strings(&["json"]),
                br#"{"api_version":1,"operation":"discover","input":{}}"#,
                temp.path()
            ),
            0
        ),
        expected
    );
}
#[test]
fn strict_cli_syntax_rejects_unknown_duplicate_missing_and_exclusive_arguments_with_attribution() {
    let temp = tempfile::tempdir().unwrap();
    for (args, operation) in [
        (vec![], json!(null)),
        (vec!["unknown"], json!(null)),
        (vec!["discover"], json!("discover")),
        (vec!["discover", "--json", "--json"], json!("discover")),
        (
            vec!["discover", "--json", "--repository", "."],
            json!("discover"),
        ),
        (vec!["inspect", "--json"], json!("inspect")),
        (
            vec![
                "inspect",
                "--repository",
                ".",
                "--selector",
                "all",
                "--selector-json",
                "{}",
                "--json",
            ],
            json!("inspect"),
        ),
        (
            vec![
                "inspect",
                "--repository",
                ".",
                "--selector",
                "projection",
                "--json",
            ],
            json!("inspect"),
        ),
        (
            vec![
                "inspect",
                "--repository",
                ".",
                "--repository",
                ".",
                "--selector",
                "all",
                "--json",
            ],
            json!("inspect"),
        ),
        (
            vec![
                "inspect",
                "--repository",
                ".",
                "--selector-json",
                r#"{"kind":"all","kind":"model"}"#,
                "--json",
            ],
            json!("inspect"),
        ),
        (
            vec![
                "explain",
                "--repository",
                ".",
                "--query-json",
                r#"{"kind":"capability","id":"x","id":"y"}"#,
                "--json",
            ],
            json!("explain"),
        ),
        (
            vec![
                "explain",
                "--repository",
                ".",
                "--query-json",
                "NaN",
                "--json",
            ],
            json!("explain"),
        ),
        (vec!["json", "--json"], json!(null)),
        (vec!["json", "--local-context-file"], json!(null)),
    ] {
        let value = parsed(&invoke(&strings(&args), b"", temp.path()), 2);
        assert_eq!(value["operation"], operation, "{args:?}");
        assert_eq!(value["outcome"], "protocol_error");
        assert_eq!(value["error"]["code"], "invalid_request");
    }
}
#[test]
fn json_cli_preserves_attribution_on_payload_version_syntax_and_byte_failures() {
    let temp = tempfile::tempdir().unwrap();
    for raw in [br#"{"operation":"explain","api_version":9,"input":{}}"#.as_slice(),br#"{"operation":"explain","input":NaN}"#,br#"{"operation":"explain","api_version":1,"input":{"query":{"kind":"capability","id":"a","id":"b"}}}"#] {
        let value=parsed(&invoke(&strings(&["json"]),raw,temp.path()),2);assert_eq!(value["operation"],"explain");
    }
    let mut request = br#"{"api_version":1,"operation":"discover","input":{}}"#.to_vec();
    request.resize(1_048_576, b' ');
    assert_eq!(
        parsed(&invoke(&strings(&["json"]), &request, temp.path()), 0)["outcome"],
        "result"
    );
    request.push(b' ');
    assert_eq!(
        parsed(&invoke(&strings(&["json"]), &request, temp.path()), 2)["operation"],
        "discover"
    );
}
#[test]
fn hostile_local_context_path_is_not_read_for_nonexecuting_or_missing_api03_services() {
    let temp = tempfile::tempdir().unwrap();
    let context = temp.path().join("unread-context-directory");
    std::fs::create_dir(&context).unwrap();
    let args = vec![
        "json".into(),
        "--local-context-file".into(),
        context.into_os_string(),
    ];
    for (operation, input) in [
        ("discover", json!({})),
        (
            "inspect",
            json!({"repository":{"kind":"utf8","value":"missing"},"selector":{"kind":"all"}}),
        ),
        (
            "explain",
            json!({"repository":{"kind":"utf8","value":"missing"},"query":{"kind":"binding","id":"x"}}),
        ),
        (
            "check_repository_integrity",
            json!({"repository":{"kind":"utf8","value":"missing"},"expected_observation":null}),
        ),
        (
            "read_execution_output",
            json!({"receipt_id":"a".repeat(64),"validation_id":"v","instance_index":"0","stream":"stdout","offset":"0","max_bytes":1}),
        ),
    ] {
        let request =
            serde_json::to_vec(&json!({"api_version":1,"operation":operation,"input":input}))
                .unwrap();
        let value = parsed(&invoke(&args, &request, temp.path()), 0);
        assert_eq!(value["operation"], operation);
        if operation != "discover" {
            let code = value["error"]["code"].as_str().unwrap();
            assert!(matches!(
                code,
                "governance_load_failed" | "internal_failure"
            ));
        }
        if operation == "check_repository_integrity" || operation == "read_execution_output" {
            assert!(
                value["error"]["detail"]
                    .as_str()
                    .unwrap()
                    .contains("API-03")
            );
        }
    }
}
