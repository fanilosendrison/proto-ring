use super::{inspection_fixture, invoke, parsed, strings};
use proto_ring_api::{NativePath, open};
use serde_json::json;

#[test]
fn real_rust_json_and_convenience_cli_inspection_and_explanation_are_identical() {
    let fixture = inspection_fixture::rich();
    let cwd = tempfile::tempdir().unwrap();
    let observed = open(&fixture.root).unwrap();
    assert!(!super::git_governance::native_units(observed.repository()).is_empty());
    for selector in [
        json!({"kind":"all"}),
        json!({"kind":"validation","id":"first"}),
        json!({"kind":"binding","id":"provider"}),
    ] {
        let mut args = strings(&["inspect", "--repository"]);
        args.push(fixture.root.clone().into_os_string());
        args.extend(strings(&[
            "--selector-json",
            &selector.to_string(),
            "--json",
        ]));
        let cli = parsed(&invoke(&args, b"", cwd.path()), 0);
        assert_eq!(
            cli["value"],
            serde_json::to_value(
                observed
                    .inspect(serde_json::from_value(selector.clone()).unwrap())
                    .unwrap()
            )
            .unwrap()
        );
        let request=serde_json::to_vec(&json!({"api_version":1,"operation":"inspect","input":{"repository":NativePath::from_path(&fixture.root).unwrap(),"selector":selector}})).unwrap();
        assert_eq!(
            cli,
            parsed(&invoke(&strings(&["json"]), &request, cwd.path()), 0)
        );
    }
    for query in [
        json!({"kind":"authority_role","responsibility":"quality","source":"generator"}),
        json!({"kind":"projection","id":"generated"}),
        json!({"kind":"evidence_requirement","id":"sourced"}),
    ] {
        let mut args = strings(&["explain", "--repository"]);
        args.push(fixture.root.clone().into_os_string());
        args.extend(strings(&["--query-json", &query.to_string(), "--json"]));
        let cli = parsed(&invoke(&args, b"", cwd.path()), 0);
        assert_eq!(
            cli["value"],
            serde_json::to_value(
                observed
                    .explain(serde_json::from_value(query).unwrap())
                    .unwrap()
            )
            .unwrap()
        );
    }
    assert!(!fixture.root.join("EFFECT-MARKER").exists());
}
#[test]
fn native_repository_arguments_are_not_lossily_converted_or_normalized() {
    #[cfg(unix)]
    let name = {
        use std::os::unix::ffi::OsStringExt;
        std::ffi::OsString::from_vec(b"root-\xff- \n".to_vec())
    };
    #[cfg(windows)]
    let name = std::ffi::OsString::from("root-ユニコード");
    let fixture = super::git_governance::GitGovernance::named(&name, false);
    let minimal = super::git_governance::GitGovernance::minimal();
    assert!(open(&minimal.root).is_ok());
    let observed = open(&fixture.root).unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let mut args = strings(&["inspect", "--repository"]);
    let original_argument = fixture.root.parent().unwrap().join(&name);
    #[cfg(unix)]
    assert!(original_argument.to_str().is_none());
    args.push(original_argument.into_os_string());
    args.extend(strings(&["--selector", "all", "--json"]));
    let value = parsed(&invoke(&args, b"", cwd.path()), 0);
    assert_eq!(
        value["value"]["repository"],
        serde_json::to_value(NativePath::from_path(observed.repository()).unwrap()).unwrap()
    );
    assert_eq!(
        value["value"]["observation"],
        serde_json::to_value(observed.reference()).unwrap()
    );
}
