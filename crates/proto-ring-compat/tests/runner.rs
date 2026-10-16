use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use proto_ring_conformance::fixture::{self, MaterializedFixture};
use proto_ring_conformance::loader::load_corpus;
use proto_ring_conformance::model::{CaseMatrix, ObservationKind, TransportValue, Vector};
use serde_json::{Map, Value, json};

const EXPLICIT_MARKER: &str = "PROTO_RING_COMPAT_EXPLICIT_MARKER";
const AMBIENT_MARKER: &str = "PROTO_RING_COMPAT_AMBIENT_ONLY";
const PROBE_MARKER: &str = "PROTO_RING_COMPAT_ENVIRONMENT_PROBE";

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_proto-ring-compat")
}
#[rustfmt::skip]
fn invoke(bytes: &[u8], environment: &[(&str, &str)], path: Option<&Path>) -> Output { let mut command = Command::new(binary()); command.stdin(Stdio::piped()).stdout(Stdio::piped()).envs(environment.iter().copied()); if let Some(path) = path { command.env("PATH", path); } let mut child = command.spawn().expect("compatibility binary must start"); child.stdin.take().unwrap().write_all(bytes).unwrap(); child.wait_with_output().expect("runner must complete") }
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
fn assert_transport_error(request: &Value) {
    assert_transport_error_output(invoke(&serde_json::to_vec(request).unwrap(), &[], None));
}

#[rustfmt::skip]
#[test]
fn transport_v1_is_strict() {
    assert_eq!(invoke_json(&json!({"transport_version":1,"operation":"engine.identity","input":{}})).0.status.code(), Some(0));
    for bytes in [br#"{"transport_version":1,"operation":"engine.identity","input":{},"extra":true}"#.as_slice(), br#"{"transport_version":1,"operation":"engine.identity","input":{"extra":true}}"#, br#"{"transport_version":2,"operation":"engine.identity","input":{}}"#, br#"{"transport_version":1,"operation":"unknown","input":{}}"#, br#"{"transport_version":1,"# , b"\xff".as_slice()] { assert_transport_error_output(invoke(bytes, &[], None)); }
    let invalid = [
        json!({"transport_version":1,"operation":"accepted-adr-body.immutability","input":{}}), json!({"transport_version":1,"operation":"accepted-adr-body.immutability","input":{"repository":"","extra":true}}),
        json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":"."}}), json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":".","environment":{"NAME":1}}}),
        json!({"transport_version":1,"operation":"github-authoritative-ref-monotonicity.observe","input":{"binding_path":""}}), json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{}}),
        json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":"","requirement_ids":[]}}), json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":".","requirement_ids":"gate"}}),
        json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":".","requirement_ids":[""]}}), json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":".","requirement_ids":["gate","gate"]}}),
        json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":".","requirement_ids":[1]}}), json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":".","extra":true}}),
        json!({"transport_version":1,"operation":"exact-evidence-binding.evaluate","input":{"mode":"unknown"}}), json!({"transport_version":1,"operation":"exact-evidence-binding.evaluate","input":{"mode":"evaluate_binding","comparisons":[]}}),
        json!({"transport_version":1,"operation":"exact-evidence-binding.evaluate","input":{"mode":"evaluate_requirement_history","comparisons":[],"extra":true}}), json!({"transport_version":1,"operation":"exact-evidence-binding.evaluate","input":{"mode":"evaluate_binding","requirement":{},"binding":null}}),
    ]; for request in invalid { assert_transport_error(&request); }
    for invalid_hex in ["a", "AA", "ag"] { let mut request = valid_exact_request(); request["input"]["requirement"]["subject_hex"] = json!(invalid_hex); assert_transport_error(&request); }
    assert_transport_error_output(Command::new(binary()).arg("--repository").output().unwrap());
}
fn valid_exact_request() -> Value {
    json!({"transport_version":1,"operation":"exact-evidence-binding.evaluate","input":{"mode":"evaluate_binding","requirement":{"admitted_classes":["check"],"subject_hex":"7375626a656374","context_hex":null,"context_required":false},"binding":{"evidence_class":"check","subject_hex":"7375626a656374","context_hex":null}}})
}

#[rustfmt::skip]
#[test]
fn b4_transport_is_strict() {
    let parse = |input| json!({"transport_version":1,"operation":"adr-metadata.parse","input":input});
    for input in [json!({}), json!({"mode":"unknown","bytes_hex":""}), json!({"mode":"parse_adr","bytes_hex":1}), json!({"mode":"parse_adr","bytes_hex":"gg"}), json!({"mode":"parse_adr","bytes_hex":"a"}), json!({"mode":"parse_adr","bytes_hex":"AA"}), json!({"mode":"parse_adr","bytes_hex":"","extra":true})] { assert_transport_error(&parse(input)); }
    let malformed = [json!({"type":"bytes","value":"00"}), json!({"type":"integer","value":1}), json!({"type":"null","value":true}), json!({"type":"sequence","value":{}})];
    for value in malformed { assert_transport_error(&json!({"transport_version":1,"operation":"adr-metadata.validation","input":{"mode":"require_mapping","value":value}})); }
    let duplicate = json!({"type":"record","value":[{"name":"x","value":{"type":"null","value":null}},{"name":"x","value":{"type":"string","value":"y"}}]});
    assert_transport_error(&json!({"transport_version":1,"operation":"adr-metadata.validation","input":{"mode":"require_mapping","value":duplicate}}));
    for request in [json!({"transport_version":1,"operation":"adr-metadata.repository-path","input":{"repository":".","relative":"x","extra":true}}), json!({"transport_version":1,"operation":"adr-metadata.validation","input":{"mode":"unknown"}}), json!({"transport_version":1,"operation":"adr-metadata.validation","input":{"mode":"require_string","value":{"type":"string","value":"x"},"extra":true}}), json!({"transport_version":1,"operation":"canonical-adr.resolve","input":{"mode":"unknown","repository":"."}}), json!({"transport_version":1,"operation":"canonical-adr.resolve","input":{"mode":"resolve_adr","repository":".","adr_id":"ADR-001","include_decision_body":"false"}}), json!({"transport_version":1,"operation":"canonical-adr.resolve","input":{"mode":"configured_profile_path","repository":".","adr_id":"ADR-001"}})] { assert_transport_error(&request); }
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
        .find(|item| item.responsibility_id == responsibility)
        .unwrap()
}
fn vector(responsibility: &str, id: &str) -> Vector {
    matrix(responsibility)
        .vectors
        .into_iter()
        .find(|item| item.vector_id == id)
        .unwrap()
}

#[rustfmt::skip]
#[derive(Debug, Eq, PartialEq)]
struct RepositorySnapshot { files: Vec<(String, Vec<u8>)>, head: Vec<u8>, index: Option<Vec<u8>>, config: Option<Vec<u8>> }
#[rustfmt::skip]
fn repository_snapshot(root: &Path) -> RepositorySnapshot {
    fn collect(root: &Path, directory: &Path, files: &mut Vec<(String, Vec<u8>)>) { for entry in fs::read_dir(directory).unwrap() { let entry = entry.unwrap(); if directory == root && entry.file_name() == ".git" { continue; } let path = entry.path(); let kind = entry.file_type().unwrap(); if kind.is_dir() { collect(root, &path, files); } else { let bytes = if kind.is_symlink() { fs::read_link(&path).unwrap().to_string_lossy().as_bytes().to_vec() } else { fs::read(&path).unwrap() }; files.push((path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"), bytes)); } } }
    let mut files = Vec::new(); collect(root, root, &mut files); files.sort_by(|left, right| left.0.cmp(&right.0)); let git = root.join(".git"); let head = Command::new("git").args(["-C", root.to_str().unwrap(), "rev-parse", "HEAD"]).output().unwrap().stdout;
    RepositorySnapshot { files, head, index: fs::read(git.join("index")).ok(), config: fs::read(git.join("config")).ok() }
}
#[rustfmt::skip]
fn transport_json(value: &TransportValue) -> Value { match value { TransportValue::Null(()) => Value::Null, TransportValue::Boolean(value) => Value::Bool(*value), TransportValue::Integer(value) => serde_json::from_str(value).unwrap(), TransportValue::String(value) | TransportValue::Bytes(value) => Value::String(value.clone()), TransportValue::Sequence(values) => Value::Array(values.iter().map(transport_json).collect()), TransportValue::Record(entries) => Value::Object(entries.iter().map(|entry| (entry.name.clone(), transport_json(&entry.value))).collect()) } }

#[rustfmt::skip]
#[test]
fn accepted_adr_body_matches_all_vectors_and_is_read_only() { let matrix = matrix("accepted-adr-body.immutability"); assert_eq!(matrix.vectors.len(), 22); for vector in matrix.vectors { let fixture = fixture::materialize(&vector.fixture).unwrap(); let root = fixture.root().unwrap(); let before = repository_snapshot(root); let (output, response) = invoke_json(&json!({"transport_version":1,"operation":"accepted-adr-body.immutability","input":{"repository":root}})); assert_eq!(output.status.code(), Some(0), "{}", vector.vector_id); assert_eq!(response["outcome"], "result", "{}", vector.vector_id); assert_eq!(response["value"], transport_json(&vector.expected_observation.value), "{}", vector.vector_id); assert_eq!(repository_snapshot(root), before, "{}", vector.vector_id); } }
#[rustfmt::skip]
fn whitespace_environment(materialized: &MaterializedFixture) -> BTreeMap<String, String> { let mut environment = fixture::record(fixture::record_field(materialized.arguments(), "environment").unwrap()).unwrap().iter().map(|entry| (entry.name.clone(), fixture::string(&entry.value).unwrap().to_owned())).collect::<BTreeMap<_, _>>(); if let Some(path) = environment.get_mut("GITHUB_EVENT_PATH") && Path::new(path).is_relative() { *path = materialized.root().unwrap().join(&*path).to_string_lossy().into_owned(); } environment }
fn empty_directory() -> MaterializedFixture {
    fixture::materialize(
        &json!({"kind":"repository_plan","steps":[],"invocation":{"type":"record","value":[]}}),
    )
    .unwrap()
}
#[rustfmt::skip]
#[test]
fn git_whitespace_matches_all_vectors_and_is_read_only() {
    let matrix = matrix("git-whitespace.validate"); assert_eq!(matrix.vectors.len(), 15); for vector in &matrix.vectors { let unavailable = vector.vector_id == "git-whitespace.validate.git-unavailable"; let fixture = fixture::materialize(if unavailable { &vector.fixture["base"] } else { &vector.fixture }).unwrap(); let root = fixture.root().unwrap(); let environment = whitespace_environment(&fixture); let empty = unavailable.then(empty_directory); let before = repository_snapshot(root); let request = json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":root,"environment":environment}}); let (output, response) = invoke_json_with(&request, &[], empty.as_ref().and_then(MaterializedFixture::root)); assert_eq!(output.status.code(), Some(0), "{}", vector.vector_id); assert_eq!(response["value"], transport_json(&vector.expected_observation.value), "{}", vector.vector_id); assert_eq!(repository_snapshot(root), before, "{}", vector.vector_id); }
    let clean = fixture::materialize(&matrix.vectors[0].fixture).unwrap(); let request = json!({"transport_version":1,"operation":"git-whitespace.validate","input":{"repository":clean.root().unwrap(),"environment":{}}}); let (_, response) = invoke_json_with(&request, &[("GITHUB_EVENT_NAME", "push"), ("GITHUB_EVENT_PATH", "ambient-must-not-be-read")], None); assert_eq!(response["value"], json!({"status":"PASS","domains":[]}));
}
#[rustfmt::skip]
#[test]
fn github_arm_malformed_binding_is_a_violated_result() { let vector = vector("github-authoritative-ref-monotonicity.observe", "github-authoritative-ref-monotonicity.observe.malformed-binding"); let binding = &vector.fixture["binding"]; let materialized = fixture::materialize(&json!({"kind":"repository_plan","steps":[{"op":"write_hex","path":binding["path"],"hex":binding["bytes_hex"]}],"invocation":{"type":"record","value":[]}})).unwrap(); let path = materialized.root().unwrap().join(binding["path"].as_str().unwrap()); let (output, response) = invoke_json(&json!({"transport_version":1,"operation":"github-authoritative-ref-monotonicity.observe","input":{"binding_path":path}})); assert_eq!(output.status.code(), Some(0)); assert_eq!(response["outcome"], "result"); assert_eq!(response["value"], transport_json(&vector.expected_observation.value)); }
#[rustfmt::skip]
#[test]
fn rgs_compose_matches_corpus_and_is_read_only() {
    let vector = vector("repository-governance-state.compose", "repository-governance-state.compose.minimal-v2"); let materialized = fixture::materialize(&vector.fixture).unwrap(); let root = materialized.root().unwrap(); let before = repository_snapshot(root); let (_, response) = invoke_json(&json!({"transport_version":1,"operation":"repository-governance-state.compose","input":{"repository":root,"binding_ids":["provider"]}})); let expected = &vector.expected_observation.value; let scope = fixture::sequence(fixture::record_field(expected, "scope_paths").unwrap()).unwrap().iter().map(|value| fixture::string(value).unwrap()).collect::<Vec<_>>(); let optional = fixture::record_field(expected, "optional_components").unwrap(); let present = |name| matches!(fixture::record_field(optional, name), Ok(TransportValue::Boolean(true))); let value = json!({"stage":"composition","identity_present":true,"scope_paths":scope,"components":{"repository_governance_model":true,"governance_authority":true,"governed_objects":present("governed_objects"),"governance_bindings":true,"repository_integrity":present("repository_integrity"),"projection_registry":present("projection_registry"),"evidence_requirements":present("evidence_requirements")},"bindings":{"provider":{"present":true,"kind":"executable_provider","scope":{"kind":"logical_provider","capability":null,"interface":null,"object":null},"identity":{"kind":"executable_provider","repository":"provider/repository","commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"authority":{"responsibility":"provider-binding","source":"bindings"}}}}); assert_eq!(response, json!({"transport_version":1,"operation":"repository-governance-state.compose","outcome":"result","value":value})); assert_eq!(repository_snapshot(root), before);
}
#[rustfmt::skip]
#[test]
fn evidence_registry_projects_rgs_fixture_and_failures_without_mutation() {
    let evidence = vector("repository-governance-state.compose", "repository-governance-state.compose.evidence"); let materialized = fixture::materialize(&evidence.fixture).unwrap(); let root = materialized.root().unwrap(); let before = repository_snapshot(root); let request = json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":root,"requirement_ids":["gate"]}}); let (output, response) = invoke_json(&request); assert_eq!(output.status.code(), Some(0)); let requirement = json!({"id":"gate","responsibility":"evidence-required","instances":{"kind":"single","source":null},"evidence_classes":{"kind":"explicit","classes":["qualification"],"source":null},"subject_source":"subject","context":{"required":false,"source":null},"candidate_source":"candidates","target":null}); let value = json!({"authority_source":"evidence-registry","model_version":1,"registry_authority":{"responsibility":"evidence-registry","source":"evidence-registry"},"requirement_ids":["gate"],"requirements":{"gate":requirement}}); assert_eq!(response, json!({"transport_version":1,"operation":"evidence-requirements.registry","outcome":"result","value":value})); assert_eq!(repository_snapshot(root), before); let (_, missing) = invoke_json(&json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":root,"requirement_ids":["missing"]}})); assert_eq!(missing["error"]["category"], "evidence-requirement-not-found"); let minimal = vector("repository-governance-state.compose", "repository-governance-state.compose.minimal-v2"); let minimal = fixture::materialize(&minimal.fixture).unwrap(); let (_, absent) = invoke_json(&json!({"transport_version":1,"operation":"evidence-requirements.registry","input":{"repository":minimal.root().unwrap()}})); assert_eq!(absent["error"]["category"], "evidence-requirements-absent");
}
fn exact_request(vector: &Vector) -> Value {
    let materialized = fixture::materialize(&vector.fixture).unwrap();
    let mut input = transport_json(materialized.arguments());
    let object = input.as_object_mut().unwrap();
    let mode = object.remove("operation").unwrap();
    object.insert("mode".to_owned(), mode);
    json!({"transport_version":1,"operation":"exact-evidence-binding.evaluate","input":input})
}
fn exact_vector(id: &str) -> Vector {
    vector("exact-evidence-binding.evaluate", id)
}
#[rustfmt::skip]
#[test]
fn exact_evidence_binding_matches_all_vectors() { let matrix = matrix("exact-evidence-binding.evaluate"); assert_eq!(matrix.vectors.len(), 44); for vector in &matrix.vectors { let (output, response) = invoke_json(&exact_request(vector)); assert_eq!(output.status.code(), Some(0), "{}", vector.vector_id); let expected = transport_json(&vector.expected_observation.value); match vector.expected_observation.kind { ObservationKind::Result => { assert_eq!(response["outcome"], "result", "{}", vector.vector_id); assert_eq!(response["value"], expected, "{}", vector.vector_id); } ObservationKind::ControlledRejection => { assert_eq!(response["outcome"], "controlled_failure", "{}", vector.vector_id); assert_eq!(response["error"]["category"], expected["category"], "{}", vector.vector_id); } } } }
#[rustfmt::skip]
#[test]
fn exact_evidence_transport_rejects_eight_malformed_classes() { let base = exact_request(&exact_vector("exact-evidence-binding.evaluate.requirement-duplicate-classes")); let cases = [("/input/binding/evidence_class", json!(true)), ("/input/binding/subject_hex", json!(1)), ("/input/binding/subject_hex", json!("zz")), ("/input/binding/context_hex", json!("zz")), ("/input/requirement/subject_hex", json!(1)), ("/input/requirement/subject_hex", json!("zz")), ("/input/requirement/context_required", json!("false")), ("/input/requirement/context_hex", json!("zz"))]; for (path, value) in cases { let mut request = base.clone(); *request.pointer_mut(path).unwrap() = value; assert_transport_error(&request); } }
#[rustfmt::skip]
#[test]
fn exact_evidence_history_validates_full_transport_before_semantics() { let mut request = exact_request(&exact_vector("exact-evidence-binding.evaluate.historical-change")); let classes = request.pointer_mut("/input/comparisons/0/historical_requirement/admitted_classes").unwrap().as_array_mut().unwrap(); classes.push(classes[0].clone()); *request.pointer_mut("/input/comparisons/1/binding/subject_hex").unwrap() = json!("zz"); assert_transport_error(&request); }

fn structured_plain(value: &Value) -> Value {
    match value["type"].as_str().unwrap() {
        "null" => Value::Null,
        "boolean" | "string" => value["value"].clone(),
        "integer" => serde_json::from_str(value["value"].as_str().unwrap()).unwrap(),
        "sequence" => Value::Array(
            value["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(structured_plain)
                .collect(),
        ),
        "record" => Value::Object(
            value["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| {
                    (
                        entry["name"].as_str().unwrap().to_owned(),
                        structured_plain(&entry["value"]),
                    )
                })
                .collect(),
        ),
        other => panic!("unexpected structured type {other}"),
    }
}
#[rustfmt::skip]
fn b4_request(responsibility: &str, materialized: &MaterializedFixture) -> (String, Value) {
    let entries = fixture::record(materialized.arguments()).unwrap(); let mode = fixture::string(&entries.iter().find(|entry| entry.name == "operation").unwrap().value).unwrap().to_owned(); let mut input = Map::new();
    if responsibility != "adr-metadata.repository-path" { input.insert("mode".to_owned(), json!(mode)); }
    for entry in entries { if entry.name == "operation" { continue; } let value = if responsibility == "adr-metadata.validation" { serde_json::to_value(&entry.value).unwrap() } else { transport_json(&entry.value) }; input.insert(entry.name.clone(), value); }
    if matches!(responsibility, "adr-metadata.repository-path" | "canonical-adr.resolve") { input.insert("repository".to_owned(), json!(materialized.root().unwrap())); }
    (mode, json!({"transport_version":1,"operation":responsibility,"input":input}))
}
fn normalized_b4_value(responsibility: &str, mode: &str, value: &Value) -> Value {
    match (responsibility, mode) {
        ("adr-metadata.parse", "load_yaml" | "load_json") => structured_plain(&value["structured"]),
        ("adr-metadata.parse", "parse_adr") => {
            json!({"body_hex":value["body_hex"],"metadata":structured_plain(&value["metadata"])})
        }
        ("adr-metadata.validation", "require_mapping") => structured_plain(&value["value"]),
        ("canonical-adr.resolve", _) if value.get("decision_body_hex").is_some() => {
            json!({"adr_id":value["adr_id"],"decision_body":value["decision_body_hex"],"relative_path":value["relative_path"]})
        }
        _ => value.clone(),
    }
}
#[rustfmt::skip]
#[test]
fn b4_matches_all_71_vectors_and_is_read_only() {
    let responsibilities = [("adr-metadata.parse", 24), ("adr-metadata.repository-path", 7), ("adr-metadata.validation", 18), ("canonical-adr.resolve", 22)]; let mut total = 0;
    for (responsibility, count) in responsibilities { let matrix = matrix(responsibility); assert_eq!(matrix.vectors.len(), count); for vector in &matrix.vectors { let materialized = fixture::materialize(&vector.fixture).unwrap(); let before = materialized.root().map(repository_snapshot); let (mode, request) = b4_request(responsibility, &materialized); let (output, response) = invoke_json(&request); assert_eq!(output.status.code(), Some(0), "{}", vector.vector_id); let expected = transport_json(&vector.expected_observation.value); match vector.expected_observation.kind { ObservationKind::Result => { assert_eq!(response["outcome"], "result", "{}", vector.vector_id); assert_eq!(normalized_b4_value(responsibility, &mode, &response["value"]), expected, "{}", vector.vector_id); } ObservationKind::ControlledRejection => { assert_eq!(response["outcome"], "controlled_failure", "{}", vector.vector_id); assert_eq!(response["error"]["category"], expected["category"], "{}", vector.vector_id); } } if let (Some(root), Some(before)) = (materialized.root(), before) { assert_eq!(repository_snapshot(root), before, "{}", vector.vector_id); } total += 1; } } assert_eq!(total, 71);
}
#[rustfmt::skip]
#[test]
fn configured_profile_path_uses_rgm_fixture_and_is_read_only() { let vector = vector("canonical-adr.resolve", "canonical-adr.resolve.valid-resolution"); let materialized = fixture::materialize(&vector.fixture).unwrap(); let root = materialized.root().unwrap(); let before = repository_snapshot(root); let request = json!({"transport_version":1,"operation":"canonical-adr.resolve","input":{"mode":"configured_profile_path","repository":root}}); let (output, response) = invoke_json(&request); assert_eq!(output.status.code(), Some(0)); assert_eq!(response["value"], json!({"relative_path":"config/adr-profile.yaml"})); assert_eq!(repository_snapshot(root), before); }
#[rustfmt::skip]
#[test]
fn b4_semantic_rejections_remain_controlled_failures() {
    for (responsibility, id) in [("adr-metadata.parse", "adr-metadata.parse.invalid-encoding"), ("adr-metadata.repository-path", "adr-metadata.repository-path.empty-rejected"), ("adr-metadata.repository-path", "adr-metadata.repository-path.nonstring-rejected"), ("canonical-adr.resolve", "canonical-adr.resolve.requested-shape-mismatch")] { let vector = vector(responsibility, id); let materialized = fixture::materialize(&vector.fixture).unwrap(); let (_, request) = b4_request(responsibility, &materialized); let (output, response) = invoke_json(&request); assert_eq!(output.status.code(), Some(0)); assert_eq!(response["outcome"], "controlled_failure"); assert_eq!(response["error"]["category"], format!("{responsibility}.rejected")); }
    let integer = |value| json!({"type":"integer","value":value}); let null = json!({"type":"null","value":null});
    for request in [json!({"transport_version":1,"operation":"adr-metadata.validation","input":{"mode":"require_string","value":integer("1")}}), json!({"transport_version":1,"operation":"adr-metadata.validation","input":{"mode":"schema_errors","value":integer("01"),"schema":null,"overlay_schema":null}})] { let mut request = request; if request["input"]["mode"] == "schema_errors" { request["input"]["schema"] = null.clone(); request["input"]["overlay_schema"] = null.clone(); } let (output, response) = invoke_json(&request); assert_eq!(output.status.code(), Some(0)); assert_eq!(response["outcome"], "controlled_failure"); assert_eq!(response["error"]["category"], "adr-metadata.validation.rejected"); }
}

const AGENTS: &str = "---\nrepository_governance:\n  model_version: 2\n  provider:\n    id: proto-ring\n    binding:\n      capability: shared_governance_provider\n      route: registry\n  capabilities:\n    shared_governance_provider:\n      configuration: {}\n      routes:\n        registry: governance-bindings.md\n    governance_authority:\n      configuration: {}\n      routes:\n        profile: governance-authority.md\n    repository_integrity:\n      configuration: {}\n      routes:\n        profile: repository-integrity.md\n---\n# Carrier\n";
const AUTHORITY: &str = "---\ngovernance_authority:\n  model_version: 1\n  sources:\n    bindings:\n      repository_target: governance-bindings.md\n    integrity:\n      repository_target: repository-integrity.md\n  responsibilities:\n    provider-binding:\n      roles:\n        bindings: authority\n      precedence: []\n    integrity-profile:\n      roles:\n        integrity: authority\n      precedence: []\n    repository-validation:\n      roles:\n        integrity: authority\n      precedence: []\n---\n# Carrier\n";
const BINDINGS: &str = "---\ngovernance_bindings:\n  model_version: 1\n  source: bindings\n  bindings:\n    provider:\n      kind: executable_provider\n      scope:\n        kind: logical_provider\n      identity:\n        repository: provider/repository\n        commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n      authority:\n        responsibility: provider-binding\n        source: bindings\n---\n# Carrier\n";
fn git_step(arguments: &[&str]) -> Value {
    json!({"op":"git","argv":arguments,"environment":{},"stdin_hex":null,"capture_stdout_as":null})
}
#[rustfmt::skip]
fn ri_fixture(validation_id: &str, arguments: &[&str]) -> MaterializedFixture { let args = arguments.iter().map(|argument| format!("        - {argument}\n")).collect::<String>(); let profile = format!("---\nrepository_integrity:\n  model_version: 1\n  authority:\n    responsibility: integrity-profile\n    source: integrity\n  environments:\n  - native_test\n  continue_after_non_satisfied: true\n  validations:\n    {validation_id}:\n      responsibility: repository-validation\n      prerequisites: []\n      instances:\n        kind: single\n      command:\n        kind: command\n        environment: native_test\n        arguments:\n{args}        undetermined_exit_codes: []\n  order:\n  - {validation_id}\n---\n# Carrier\n"); fixture::materialize(&json!({"kind":"repository_plan","environment":{},"faults":[],"steps":[git_step(&["init","-q","-b","main"]),git_step(&["config","user.name","Compatibility Test"]),git_step(&["config","user.email","compat@example.invalid"]),{"op":"write_utf8","path":"AGENTS.md","text":AGENTS},{"op":"write_utf8","path":"governance-authority.md","text":AUTHORITY},{"op":"write_utf8","path":"governance-bindings.md","text":BINDINGS},{"op":"write_utf8","path":"repository-integrity.md","text":profile},git_step(&["add","-A"]),git_step(&["commit","-q","-m","governed fixture"])],"invocation":{"type":"record","value":[]}})).unwrap() }
fn evaluate_request(
    root: &Path,
    prefix: Vec<String>,
    environment: BTreeMap<String, String>,
) -> Value {
    json!({"transport_version":1,"operation":"repository-integrity.evaluate","input":{"repository":root,"evaluation_context":{"realizations":[{"environment_id":"native_test","identity_utf8":"native-test-environment-v1","command_prefix":prefix,"process_environment":environment}]}}})
}
fn assert_pass(response: &Value, id: &str) {
    let obligation = json!({"id":id,"status":"SATISFIED","executed":true,"exit_code":"0"});
    assert_eq!(
        response,
        &json!({"transport_version":1,"operation":"repository-integrity.evaluate","outcome":"result","value":{"verdict":"PASS","state_relation":"EQUAL","state_coherence_halt":false,"validations":[{"id":id,"status":"SATISFIED","obligations":[obligation.clone()]}],"obligations":[obligation]}})
    );
}
#[test]
fn repository_integrity_evaluates_native_command() {
    let fixture = ri_fixture("native-smoke", &["--version"]);
    let environment = BTreeMap::from([("PATH".to_owned(), std::env::var("PATH").unwrap())]);
    let (_, response) = invoke_json(&evaluate_request(
        fixture.root().unwrap(),
        vec!["git".to_owned()],
        environment,
    ));
    assert_pass(&response, "native-smoke");
}
#[test]
fn repository_integrity_uses_only_explicit_environment() {
    let fixture = ri_fixture(
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
        fixture.root().unwrap(),
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
