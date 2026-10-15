use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use proto_ring_conformance::fixture::{self, MaterializedFixture};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::{CaseMatrix, TransportValue};
use serde_json::{Value, json};

const EXPLICIT_MARKER: &str = "PROTO_RING_COMPAT_EXPLICIT_MARKER";
const AMBIENT_MARKER: &str = "PROTO_RING_COMPAT_AMBIENT_ONLY";
const PROBE_MARKER: &str = "PROTO_RING_COMPAT_ENVIRONMENT_PROBE";

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_proto-ring-compat")
}

fn invoke(bytes: &[u8], environment: &[(&str, &str)], path: Option<&Path>) -> Output {
    let mut command = Command::new(binary());
    command.stdin(Stdio::piped()).stdout(Stdio::piped());
    command.envs(environment.iter().copied());
    if let Some(path) = path {
        command.env("PATH", path);
    }
    let mut child = command.spawn().expect("compatibility binary must start");
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    child.wait_with_output().expect("runner must complete")
}

fn invoke_json_with(
    request: &Value,
    environment: &[(&str, &str)],
    path: Option<&Path>,
) -> (Output, Value) {
    let output = invoke(&serde_json::to_vec(request).unwrap(), environment, path);
    assert_eq!(
        output.stdout.iter().filter(|byte| **byte == b'\n').count(),
        1
    );
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    let response = serde_json::from_slice(&output.stdout).expect("response must be JSON");
    (output, response)
}

fn invoke_json(request: &Value) -> (Output, Value) {
    invoke_json_with(request, &[], None)
}

fn assert_transport_error_output(output: Output) {
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!({"transport_version":1,"outcome":"transport_error","error":{"category":"invalid_request"}})
    );
}

fn assert_transport_error(bytes: &[u8]) {
    assert_transport_error_output(invoke(bytes, &[], None));
}

#[test]
fn transport_v1_is_strict() {
    assert_eq!(
        invoke_json(&json!({"transport_version":1,"operation":"engine.identity","input":{}}))
            .0
            .status
            .code(),
        Some(0)
    );
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
    let invalid_b2 = [
        json!({"transport_version":1,"operation":"accepted-adr-body.immutability","input":{}}),
        json!({"transport_version":1,"operation":"accepted-adr-body.immutability","input":{"repository":""}}),
        json!({"transport_version":1,"operation":"accepted-adr-body.immutability","input":{"repository":1}}),
        json!({"transport_version":1,"operation":"accepted-adr-body.immutability","input":{"repository":".","extra":true}}),
        json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"environment":{}}}),
        json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":"","environment":{}}}),
        json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":1,"environment":{}}}),
        json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":"."}}),
        json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":".","environment":[]}}),
        json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":".","environment":{"NAME":1}}}),
        json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":".","environment":{},"extra":true}}),
        json!({"transport_version":1,"operation":"github-authoritative-ref-monotonicity.observe","input":{}}),
        json!({"transport_version":1,"operation":"github-authoritative-ref-monotonicity.observe","input":{"binding_path":""}}),
        json!({"transport_version":1,"operation":"github-authoritative-ref-monotonicity.observe","input":{"binding_path":1}}),
        json!({"transport_version":1,"operation":"github-authoritative-ref-monotonicity.observe","input":{"binding_path":"binding.md","extra":true}}),
    ];
    for request in invalid_b2 {
        assert_transport_error(&serde_json::to_vec(&request).unwrap());
    }
    assert_transport_error_output(Command::new(binary()).arg("--repository").output().unwrap());
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
    let commit = response["value"]["source_commit"].as_str().unwrap();
    assert_eq!(commit.len(), 40);
    assert!(
        commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    );
}

fn matrix(responsibility: &str) -> CaseMatrix {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    load_corpus(&root)
        .unwrap()
        .matrices
        .into_iter()
        .find(|matrix| matrix.responsibility_id == responsibility)
        .expect("responsibility matrix must exist")
}

#[rustfmt::skip]
#[derive(Debug, Eq, PartialEq)]
struct RepositorySnapshot { files: Vec<(String, Vec<u8>)>, head: Vec<u8>, index: Option<Vec<u8>>, config: Option<Vec<u8>> }

#[rustfmt::skip]
fn repository_snapshot(root: &Path) -> RepositorySnapshot {
    fn collect(root: &Path, directory: &Path, files: &mut Vec<(String, Vec<u8>)>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if directory == root && entry.file_name() == ".git" { continue; }
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() { collect(root, &path, files); }
            else { files.push((path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"), fs::read(path).unwrap())); }
        }
    }
    let mut files = Vec::new();
    collect(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let git = root.join(".git");
    let head = Command::new("git").args(["-C", root.to_str().unwrap(), "rev-parse", "HEAD"]).output().unwrap().stdout;
    RepositorySnapshot { files, head, index: fs::read(git.join("index")).ok(), config: fs::read(git.join("config")).ok() }
}

#[rustfmt::skip]
fn transport_json(value: &TransportValue) -> Value {
    match value {
        TransportValue::Null(()) => Value::Null,
        TransportValue::Boolean(value) => Value::Bool(*value),
        TransportValue::Integer(value) => serde_json::from_str(value).unwrap(),
        TransportValue::String(value) | TransportValue::Bytes(value) => Value::String(value.clone()),
        TransportValue::Sequence(values) => Value::Array(values.iter().map(transport_json).collect()),
        TransportValue::Record(entries) => Value::Object(entries.iter().map(|entry| (entry.name.clone(), transport_json(&entry.value))).collect()),
    }
}

#[rustfmt::skip]
#[test]
fn accepted_adr_body_matches_all_vectors_and_is_read_only() {
    let matrix = matrix("accepted-adr-body.immutability");
    assert_eq!(matrix.vectors.len(), 22);
    for vector in matrix.vectors {
        let materialized = fixture::materialize(&vector.fixture).unwrap();
        let root = materialized.root().unwrap();
        let before = repository_snapshot(root);
        let (output, response) = invoke_json(&json!({"transport_version":1,"operation":"accepted-adr-body.immutability","input":{"repository":root}}));
        assert_eq!(output.status.code(), Some(0), "{}", vector.vector_id);
        assert_eq!(response["outcome"], "result", "{}", vector.vector_id);
        assert_eq!(response["value"], transport_json(&vector.expected_observation.value), "{}", vector.vector_id);
        assert_eq!(repository_snapshot(root), before, "{}", vector.vector_id);
    }
}

#[rustfmt::skip]
fn whitespace_environment(materialized: &MaterializedFixture) -> BTreeMap<String, String> {
    assert_eq!(fixture::string(fixture::record_field(materialized.arguments(), "operation").unwrap()).unwrap(), "check_whitespace");
    let mut environment = fixture::record(fixture::record_field(materialized.arguments(), "environment").unwrap())
        .unwrap().iter().map(|entry| (entry.name.clone(), fixture::string(&entry.value).unwrap().to_owned())).collect::<BTreeMap<_, _>>();
    if let Some(path) = environment.get_mut("GITHUB_EVENT_PATH")
        && Path::new(path).is_relative() { *path = materialized.root().unwrap().join(&*path).to_string_lossy().into_owned(); }
    environment
}

fn empty_directory() -> MaterializedFixture {
    fixture::materialize(
        &json!({"kind":"repository_plan","steps":[],"invocation":{"type":"record","value":[]}}),
    )
    .unwrap()
}

#[rustfmt::skip]
#[test]
fn git_whitespace_matches_all_vectors_and_is_read_only() {
    let matrix = matrix("git-whitespace.validate");
    assert_eq!(matrix.vectors.len(), 15);
    for vector in &matrix.vectors {
        let unavailable = vector.vector_id == "git-whitespace.validate.git-unavailable";
        let fixture_value = if unavailable { &vector.fixture["base"] } else { &vector.fixture };
        let materialized = fixture::materialize(fixture_value).unwrap();
        let root = materialized.root().unwrap();
        let environment = whitespace_environment(&materialized);
        let empty = unavailable.then(empty_directory);
        let before = repository_snapshot(root);
        let request = json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":root,"environment":environment}});
        let (output, response) = invoke_json_with(&request, &[], empty.as_ref().and_then(MaterializedFixture::root));
        assert_eq!(output.status.code(), Some(0), "{}", vector.vector_id);
        assert_eq!(response["outcome"], "result", "{}", vector.vector_id);
        assert_eq!(response["value"], transport_json(&vector.expected_observation.value), "{}", vector.vector_id);
        assert_eq!(repository_snapshot(root), before, "{}", vector.vector_id);
    }
    let clean = fixture::materialize(&matrix.vectors[0].fixture).unwrap();
    let root = clean.root().unwrap();
    let request = json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":root,"environment":{}}});
    let ambient = [("GITHUB_EVENT_NAME", "push"), ("GITHUB_EVENT_PATH", "ambient-must-not-be-read")];
    let (_, response) = invoke_json_with(&request, &ambient, None);
    assert_eq!(response["value"], json!({"status":"PASS","domains":[]}));
}

#[test]
fn github_arm_malformed_binding_is_a_violated_result() {
    let vector = matrix("github-authoritative-ref-monotonicity.observe")
        .vectors
        .into_iter()
        .find(|vector| {
            vector.vector_id == "github-authoritative-ref-monotonicity.observe.malformed-binding"
        })
        .unwrap();
    let binding = &vector.fixture["binding"];
    let materialized = fixture::materialize(&json!({
        "kind":"repository_plan","steps":[{"op":"write_hex","path":binding["path"],"hex":binding["bytes_hex"]}],
        "invocation":{"type":"record","value":[]}
    })).unwrap();
    let path = materialized
        .root()
        .unwrap()
        .join(binding["path"].as_str().unwrap());
    let (output, response) = invoke_json(
        &json!({"transport_version":1,"operation":"github-authoritative-ref-monotonicity.observe","input":{"binding_path":path}}),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(response["outcome"], "result");
    assert_eq!(
        response["value"],
        transport_json(&vector.expected_observation.value)
    );
}

#[test]
fn rgs_compose_matches_corpus_and_is_read_only() {
    let vector = matrix("repository-governance-state.compose")
        .vectors
        .into_iter()
        .find(|vector| vector.vector_id == "repository-governance-state.compose.minimal-v2")
        .unwrap();
    let materialized = fixture::materialize(&vector.fixture).unwrap();
    let root = materialized.root().unwrap();
    let before = repository_snapshot(root);
    let (_, response) = invoke_json(
        &json!({"transport_version":1,"operation":"repository-governance-state.compose","input":{"repository":root,"binding_ids":["provider"]}}),
    );
    let expected = &vector.expected_observation.value;
    let scope = fixture::sequence(fixture::record_field(expected, "scope_paths").unwrap())
        .unwrap()
        .iter()
        .map(|value| fixture::string(value).unwrap())
        .collect::<Vec<_>>();
    let optional = fixture::record_field(expected, "optional_components").unwrap();
    let present = |name| {
        matches!(
            fixture::record_field(optional, name),
            Ok(TransportValue::Boolean(true))
        )
    };
    let value = json!({"stage":"composition","identity_present":true,"scope_paths":scope,"components":{"repository_governance_model":true,"governance_authority":true,"governed_objects":present("governed_objects"),"governance_bindings":true,"repository_integrity":present("repository_integrity"),"projection_registry":present("projection_registry"),"evidence_requirements":present("evidence_requirements")},"bindings":{"provider":{"present":true,"kind":"executable_provider","scope":{"kind":"logical_provider","capability":null,"interface":null,"object":null},"identity":{"kind":"executable_provider","repository":"provider/repository","commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"authority":{"responsibility":"provider-binding","source":"bindings"}}}});
    assert_eq!(
        response,
        json!({"transport_version":1,"operation":"repository-governance-state.compose","outcome":"result","value":value})
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
    fixture::materialize(&json!({"kind":"repository_plan","environment":{},"faults":[],"steps":[git_step(&["init","-q","-b","main"]),git_step(&["config","user.name","Compatibility Test"]),git_step(&["config","user.email","compat@example.invalid"]),{"op":"write_utf8","path":"AGENTS.md","text":AGENTS},{"op":"write_utf8","path":"governance-authority.md","text":AUTHORITY},{"op":"write_utf8","path":"governance-bindings.md","text":BINDINGS},{"op":"write_utf8","path":"repository-integrity.md","text":profile},git_step(&["add","-A"]),git_step(&["commit","-q","-m","governed fixture"])],"invocation":{"type":"record","value":[]}})).unwrap()
}

fn evaluate_request(
    root: &Path,
    prefix: Vec<String>,
    environment: BTreeMap<String, String>,
) -> Value {
    json!({"transport_version":1,"operation":"repository-integrity.evaluate","input":{"repository":root,"evaluation_context":{"realizations":[{"environment_id":"native_test","identity_utf8":"native-test-environment-v1","command_prefix":prefix,"process_environment":environment}]}}})
}

fn assert_pass(response: &Value, validation_id: &str) {
    let obligation =
        json!({"id":validation_id,"status":"SATISFIED","executed":true,"exit_code":"0"});
    assert_eq!(
        response,
        &json!({"transport_version":1,"operation":"repository-integrity.evaluate","outcome":"result","value":{"verdict":"PASS","state_relation":"EQUAL","state_coherence_halt":false,"validations":[{"id":validation_id,"status":"SATISFIED","obligations":[obligation.clone()]}],"obligations":[obligation]}})
    );
}

#[test]
fn repository_integrity_evaluates_native_command() {
    let materialized = ri_fixture("native-smoke", &["--version"]);
    let environment = BTreeMap::from([("PATH".to_owned(), std::env::var("PATH").unwrap())]);
    let (_, response) = invoke_json(&evaluate_request(
        materialized.root().unwrap(),
        vec!["git".to_owned()],
        environment,
    ));
    assert_pass(&response, "native-smoke");
}

#[test]
fn repository_integrity_uses_only_explicit_environment() {
    let materialized = ri_fixture(
        "environment-smoke",
        &["--exact", "environment_probe", "--nocapture"],
    );
    let mut environment = BTreeMap::from([
        (PROBE_MARKER.to_owned(), "1".to_owned()),
        (EXPLICIT_MARKER.to_owned(), "present".to_owned()),
    ]);
    if let Ok(system_root) = std::env::var("SystemRoot") {
        environment.insert("SystemRoot".to_owned(), system_root);
    }
    let request = evaluate_request(
        materialized.root().unwrap(),
        vec![
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ],
        environment,
    );
    let (_, response) = invoke_json_with(&request, &[(AMBIENT_MARKER, "must-not-propagate")], None);
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
