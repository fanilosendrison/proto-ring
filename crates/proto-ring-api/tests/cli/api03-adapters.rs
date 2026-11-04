use super::{invoke, parsed, strings};
use serde_json::json;

#[test]
fn check_and_output_adapters_only_report_the_missing_api03_service() {
    let temp = tempfile::tempdir().unwrap();
    let hostile = temp.path().join("not-a-context-file");
    std::fs::create_dir(&hostile).unwrap();
    let mut check = strings(&[
        "check",
        "--repository",
        "must-not-open",
        "--expected-observation-json",
        "null",
        "--local-context-file",
    ]);
    check.push(hostile.into_os_string());
    check.push("--json".into());
    let output = strings(&[
        "output",
        "--receipt",
        &"a".repeat(64),
        "--validation",
        "v",
        "--instance",
        "0",
        "--stream",
        "stderr",
        "--offset",
        "18446744073709551615",
        "--max-bytes",
        "65536",
        "--json",
    ]);
    for (args, operation) in [
        (check, "check_repository_integrity"),
        (output, "read_execution_output"),
    ] {
        let value = parsed(&invoke(&args, b"", temp.path()), 0);
        assert_eq!(value["operation"], operation);
        assert_eq!(value["outcome"], "controlled_failure");
        assert_eq!(value["error"]["code"], "internal_failure");
        assert_eq!(value["error"]["stage"], json!(null));
        assert_eq!(value["error"]["attempt_id"], json!(null));
        assert_eq!(value["error"]["effects_possible"], false);
        assert!(
            value["error"]["detail"]
                .as_str()
                .unwrap()
                .contains("API-03")
        );
    }
    assert!(!temp.path().join("must-not-open").exists());
}
#[test]
fn convenience_fragments_cannot_inject_root_fields_and_invalid_json_flags_keep_wire_attribution() {
    let temp = tempfile::tempdir().unwrap();
    let injection = r#"{}},"operation":"discover","input":{}"#;
    let args = strings(&[
        "inspect",
        "--repository",
        ".",
        "--selector-json",
        injection,
        "--json",
    ]);
    assert_eq!(
        parsed(&invoke(&args, b"", temp.path()), 2)["operation"],
        "inspect"
    );
    let raw = br#"{"api_version":1,"operation":"discover","input":{}}"#;
    assert_eq!(
        parsed(
            &invoke(&strings(&["json", "--unknown"]), raw, temp.path()),
            2
        )["operation"],
        "discover"
    );
}
#[cfg(unix)]
#[test]
fn nonunicode_flags_and_queries_fail_without_lossy_interpretation() {
    use std::os::unix::ffi::OsStringExt;
    let temp = tempfile::tempdir().unwrap();
    let bad = std::ffi::OsString::from_vec(vec![0xff]);
    let mut args = strings(&["explain", "--repository", ".", "--query-json"]);
    args.push(bad);
    args.push("--json".into());
    assert_eq!(
        parsed(&invoke(&args, b"", temp.path()), 2)["operation"],
        "explain"
    );
}
