use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use proto_ring_conformance::fixture::{self, MaterializedFixture};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::TransportValue;
use serde_json::{Value, json};

const EXPLICIT_MARKER: &str = "PROTO_RING_COMPAT_EXPLICIT_MARKER";
const AMBIENT_MARKER: &str = "PROTO_RING_COMPAT_AMBIENT_ONLY";
const PROBE_MARKER: &str = "PROTO_RING_COMPAT_ENVIRONMENT_PROBE";

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_proto-ring-compat")
}

fn invoke(bytes: &[u8], ambient_marker: bool) -> Output {
    let mut command = Command::new(binary());
    command.stdin(Stdio::piped()).stdout(Stdio::piped());
    if ambient_marker {
        command.env(AMBIENT_MARKER, "must-not-propagate");
    }
    let mut child = command.spawn().expect("compatibility binary must start");
    child
        .stdin
        .take()
        .expect("stdin must be piped")
        .write_all(bytes)
        .expect("request must be written");
    child.wait_with_output().expect("runner must complete")
}

fn invoke_json(request: &Value) -> (Output, Value) {
    let output = invoke(
        &serde_json::to_vec(request).expect("request serializes"),
        false,
    );
    assert_eq!(
        output.stdout.iter().filter(|byte| **byte == b'\n').count(),
        1
    );
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    let response = serde_json::from_slice(&output.stdout).expect("response must be JSON");
    (output, response)
}

fn assert_transport_error_output(output: Output) {
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("transport response must be JSON"),
        json!({
            "transport_version": 1,
            "outcome": "transport_error",
            "error": {"category": "invalid_request"}
        })
    );
}

fn assert_transport_error(request: &[u8]) {
    assert_transport_error_output(invoke(request, false));
}

#[test]
fn transport_v1_is_strict() {
    let valid = json!({"transport_version":1,"operation":"engine.identity","input":{}});
    assert_eq!(invoke_json(&valid).0.status.code(), Some(0));
    for invalid in [
        br#"{"transport_version":1,"operation":"engine.identity","input":{},"extra":true}"#
            .as_slice(),
        br#"{"transport_version":1,"operation":"engine.identity","input":{"extra":true}}"#,
        br#"{"transport_version":2,"operation":"engine.identity","input":{}}"#,
        br#"{"transport_version":1,"operation":"unknown","input":{}}"#,
        br#"{"transport_version":1,"#,
        b"\xff".as_slice(),
    ] {
        assert_transport_error(invalid);
    }
    assert_transport_error_output(
        Command::new(binary())
            .arg("--repository")
            .output()
            .expect("compatibility binary must reject CLI arguments"),
    );
}

#[test]
fn engine_identity_reports_source_identity() {
    let (_, response) =
        invoke_json(&json!({"transport_version":1,"operation":"engine.identity","input":{}}));
    assert_eq!(response["outcome"], "result");
    assert_eq!(
        response["value"]["source_repository"],
        "fanilosendrison/proto-ring"
    );
    assert_eq!(response["value"]["transport_version"], 1);
    let commit = response["value"]["source_commit"]
        .as_str()
        .expect("commit string");
    assert_eq!(commit.len(), 40);
    assert!(
        commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    );
}

fn minimal_rgs_fixture() -> (MaterializedFixture, TransportValue) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus = load_corpus(&root).expect("corpus must load");
    let vector = corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == "repository-governance-state.compose")
        .and_then(|matrix| {
            matrix
                .vectors
                .iter()
                .find(|vector| vector.vector_id == "repository-governance-state.compose.minimal-v2")
        })
        .expect("minimal RGS vector must exist");
    (
        fixture::materialize(&vector.fixture).expect("RGS fixture must materialize"),
        vector.expected_observation.value.clone(),
    )
}

#[derive(Debug, Eq, PartialEq)]
struct RepositorySnapshot {
    files: Vec<(String, Vec<u8>)>,
    head: Vec<u8>,
    index: Vec<u8>,
    config: Vec<u8>,
}

fn repository_snapshot(root: &Path) -> RepositorySnapshot {
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let git = root.join(".git");
    let mut head = fs::read(git.join("HEAD")).expect("HEAD must be readable");
    if let Some(reference) = std::str::from_utf8(&head)
        .ok()
        .and_then(|value| value.trim().strip_prefix("ref: "))
    {
        head.extend(fs::read(git.join(reference)).expect("HEAD ref must be readable"));
    }
    RepositorySnapshot {
        files,
        head,
        index: fs::read(git.join("index")).expect("index must be readable"),
        config: fs::read(git.join("config")).expect("config must be readable"),
    }
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<(String, Vec<u8>)>) {
    for entry in fs::read_dir(directory).expect("fixture directory must be readable") {
        let entry = entry.expect("fixture entry must be readable");
        if directory == root && entry.file_name() == ".git" {
            continue;
        }
        let path = entry.path();
        if entry
            .file_type()
            .expect("fixture type must be readable")
            .is_dir()
        {
            collect_files(root, &path, files);
        } else {
            files.push((
                path.strip_prefix(root)
                    .expect("fixture path must be relative")
                    .to_string_lossy()
                    .replace('\\', "/"),
                fs::read(path).expect("fixture bytes must be readable"),
            ));
        }
    }
}

fn record_boolean(value: &TransportValue, name: &str) -> bool {
    match fixture::record_field(value, name).expect("expected field must exist") {
        TransportValue::Boolean(value) => *value,
        _ => panic!("expected field must be boolean"),
    }
}

#[test]
fn rgs_compose_matches_corpus_and_is_read_only() {
    let (materialized, expected) = minimal_rgs_fixture();
    let root = materialized.root().expect("RGS fixture has repository");
    let before = repository_snapshot(root);
    let (_, response) = invoke_json(&json!({
        "transport_version":1,
        "operation":"repository-governance-state.compose",
        "input":{"repository":root,"binding_ids":["provider"]}
    }));
    assert_eq!(response["outcome"], "result");
    assert_eq!(response["value"]["stage"], "composition");
    assert_eq!(response["value"]["identity_present"], true);
    let expected_scope = match fixture::record_field(&expected, "scope_paths").unwrap() {
        TransportValue::Sequence(values) => values
            .iter()
            .map(|value| fixture::string(value).unwrap())
            .collect::<Vec<_>>(),
        _ => panic!("scope_paths must be a sequence"),
    };
    assert_eq!(response["value"]["scope_paths"], json!(expected_scope));
    let optional = fixture::record_field(&expected, "optional_components").unwrap();
    for name in [
        "governed_objects",
        "repository_integrity",
        "projection_registry",
        "evidence_requirements",
    ] {
        assert_eq!(
            response["value"]["components"][name],
            record_boolean(optional, name)
        );
    }
    assert_eq!(
        response["value"]["components"]["repository_governance_model"],
        true
    );
    assert_eq!(
        response["value"]["components"]["governance_authority"],
        true
    );
    assert_eq!(response["value"]["components"]["governance_bindings"], true);
    let provider = &response["value"]["bindings"]["provider"];
    assert_eq!(provider["present"], true);
    assert_eq!(provider["kind"], "executable_provider");
    assert_eq!(
        provider["scope"],
        json!({"kind":"logical_provider","capability":null,"interface":null,"object":null})
    );
    assert_eq!(
        provider["identity"],
        json!({"kind":"executable_provider","repository":"provider/repository","commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"})
    );
    assert_eq!(
        provider["authority"],
        json!({"responsibility":"provider-binding","source":"bindings"})
    );
    assert_eq!(repository_snapshot(root), before);
}

const AGENTS: &str = "---\nrepository_governance:\n  model_version: 2\n  provider:\n    id: proto-ring\n    binding:\n      capability: shared_governance_provider\n      route: registry\n  capabilities:\n    shared_governance_provider:\n      configuration: {}\n      routes:\n        registry: governance-bindings.md\n    governance_authority:\n      configuration: {}\n      routes:\n        profile: governance-authority.md\n    repository_integrity:\n      configuration: {}\n      routes:\n        profile: repository-integrity.md\n---\n# Carrier\n";
const AUTHORITY: &str = "---\ngovernance_authority:\n  model_version: 1\n  sources:\n    bindings:\n      repository_target: governance-bindings.md\n    integrity:\n      repository_target: repository-integrity.md\n  responsibilities:\n    provider-binding:\n      roles:\n        bindings: authority\n      precedence: []\n    integrity-profile:\n      roles:\n        integrity: authority\n      precedence: []\n    repository-validation:\n      roles:\n        integrity: authority\n      precedence: []\n---\n# Carrier\n";
const BINDINGS: &str = "---\ngovernance_bindings:\n  model_version: 1\n  source: bindings\n  bindings:\n    provider:\n      kind: executable_provider\n      scope:\n        kind: logical_provider\n      identity:\n        repository: provider/repository\n        commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n      authority:\n        responsibility: provider-binding\n        source: bindings\n---\n# Carrier\n";

fn git_step(arguments: &[&str]) -> Value {
    json!({"op":"git","argv":arguments,"environment":{},"stdin_hex":null,"capture_stdout_as":null})
}

fn ri_fixture(validation_id: &str, arguments: &[&str]) -> MaterializedFixture {
    let yaml_arguments = arguments
        .iter()
        .map(|argument| format!("        - {argument}\n"))
        .collect::<String>();
    let profile = format!(
        "---\nrepository_integrity:\n  model_version: 1\n  authority:\n    responsibility: integrity-profile\n    source: integrity\n  environments:\n  - native_test\n  continue_after_non_satisfied: true\n  validations:\n    {validation_id}:\n      responsibility: repository-validation\n      prerequisites: []\n      instances:\n        kind: single\n      command:\n        kind: command\n        environment: native_test\n        arguments:\n{yaml_arguments}        undetermined_exit_codes: []\n  order:\n  - {validation_id}\n---\n# Carrier\n"
    );
    let fixture_value = json!({"kind":"repository_plan","environment":{},"faults":[],"steps":[
        git_step(&["init","-q","-b","main"]),
        git_step(&["config","user.name","Compatibility Test"]),
        git_step(&["config","user.email","compat@example.invalid"]),
        {"op":"write_utf8","path":"AGENTS.md","text":AGENTS},
        {"op":"write_utf8","path":"governance-authority.md","text":AUTHORITY},
        {"op":"write_utf8","path":"governance-bindings.md","text":BINDINGS},
        {"op":"write_utf8","path":"repository-integrity.md","text":profile},
        git_step(&["add","-A"]), git_step(&["commit","-q","-m","governed fixture"])
    ],"invocation":{"type":"record","value":[]}});
    fixture::materialize(&fixture_value).expect("RI fixture must materialize")
}

fn evaluate_request(
    root: &Path,
    prefix: Vec<String>,
    environment: BTreeMap<String, String>,
) -> Value {
    json!({"transport_version":1,"operation":"repository-integrity.evaluate","input":{
        "repository":root,"evaluation_context":{"realizations":[{
            "environment_id":"native_test","identity_utf8":"native-test-environment-v1",
            "command_prefix":prefix,"process_environment":environment
        }]}
    }})
}

fn assert_pass(response: &Value, validation_id: &str) {
    assert_eq!(response["outcome"], "result");
    assert_eq!(response["value"]["verdict"], "PASS");
    assert_eq!(response["value"]["state_relation"], "EQUAL");
    assert_eq!(response["value"]["state_coherence_halt"], false);
    let validation = &response["value"]["validations"][0];
    assert_eq!(validation["id"], validation_id);
    assert_eq!(validation["status"], "SATISFIED");
    assert_eq!(validation["obligations"][0]["executed"], true);
    assert_eq!(validation["obligations"][0]["status"], "SATISFIED");
    assert_eq!(validation["obligations"][0]["exit_code"], "0");
    assert_eq!(
        response["value"]["obligations"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        response["value"]["obligations"][0],
        validation["obligations"][0]
    );
}

#[test]
fn repository_integrity_evaluates_native_command() {
    let materialized = ri_fixture("native-smoke", &["--version"]);
    let root = materialized.root().unwrap();
    let environment = BTreeMap::from([("PATH".to_owned(), std::env::var("PATH").unwrap())]);
    let (_, response) = invoke_json(&evaluate_request(root, vec!["git".to_owned()], environment));
    assert_pass(&response, "native-smoke");
}

#[test]
fn repository_integrity_uses_only_explicit_environment() {
    let materialized = ri_fixture(
        "environment-smoke",
        &["--exact", "environment_probe", "--nocapture"],
    );
    let root = materialized.root().unwrap();
    let mut environment = BTreeMap::from([
        (PROBE_MARKER.to_owned(), "1".to_owned()),
        (EXPLICIT_MARKER.to_owned(), "present".to_owned()),
    ]);
    if let Ok(system_root) = std::env::var("SystemRoot") {
        environment.insert("SystemRoot".to_owned(), system_root);
    }
    let request = evaluate_request(
        root,
        vec![
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ],
        environment,
    );
    let mut command = Command::new(binary());
    command
        .env(AMBIENT_MARKER, "must-not-propagate")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_pass(&response, "environment-smoke");
}

#[test]
fn environment_probe() {
    if std::env::var(PROBE_MARKER).as_deref() != Ok("1") {
        return;
    }
    assert_eq!(std::env::var(EXPLICIT_MARKER).as_deref(), Ok("present"));
    assert!(std::env::var_os(AMBIENT_MARKER).is_none());
}
