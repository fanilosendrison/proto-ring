#[path = "support/git-governance.rs"]
mod git_governance;

use git_governance::GitGovernance;
use proto_ring_api::{
    ComponentProjection, ExplainQuery, Explanation, InspectSelector, InspectionResult,
    MappingEntry, NativePath, Response, ResultTag, StructuredValue, SubjectProjection,
    SuccessResponse, VersionOne, encode_response, open, process_json,
};
use serde_json::{Value, json};
use std::{ffi::OsString, io::Write, path::Path, process::Command};

#[derive(Clone, Copy)]
enum Shape {
    Sequence,
    Mapping,
}

// Independent fixture, typed and wire oracles; no production projector reuse
// and no reparsing of deeply tagged output with a depth-limited deserializer.
fn raw_configuration(shape: Shape) -> Value {
    (0..80).fold(Value::Null, |child, _| match shape {
        Shape::Sequence => json!([child]),
        Shape::Mapping => json!({"":child}),
    })
}
fn typed_configuration(shape: Shape) -> StructuredValue {
    (0..80).fold(StructuredValue::Null {}, |child, _| match shape {
        Shape::Sequence => StructuredValue::Sequence { items: vec![child] },
        Shape::Mapping => StructuredValue::Mapping {
            entries: vec![MappingEntry {
                key: String::new(),
                value: child,
            }]
            .try_into()
            .unwrap(),
        },
    })
}
fn tagged_configuration(shape: Shape) -> Value {
    (0..80).fold(json!({"kind":"null"}), |child, _| match shape {
        Shape::Sequence => json!({"kind":"sequence","items":[child]}),
        Shape::Mapping => json!({"kind":"mapping","entries":[{"key":"","value":child}]}),
    })
}
fn depth(root: &Value) -> usize {
    let mut pending = vec![(root, 0)];
    let mut maximum = 0;
    while let Some((value, level)) = pending.pop() {
        maximum = maximum.max(level);
        match value {
            Value::Array(items) => pending.extend(items.iter().map(|v| (v, level + 1))),
            Value::Object(items) => pending.extend(items.values().map(|v| (v, level + 1))),
            _ => {}
        }
    }
    maximum
}
fn cli(arguments: &[OsString], input: &[u8], cwd: &Path) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_proto-ring"));
    for (key, _) in std::env::vars_os() {
        if key
            .to_str()
            .is_some_and(|k| k.to_ascii_uppercase().starts_with("GIT_"))
        {
            command.env_remove(key);
        }
    }
    let mut child = command
        .args(arguments)
        .current_dir(cwd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
fn capability(result: &InspectionResult) -> &proto_ring_api::CapabilityDeclaration {
    let id: proto_ring_api::Identifier = "governance_authority".try_into().unwrap();
    match result {
        InspectionResult::All(components) => &components.model.data.capabilities[&id],
        InspectionResult::Component(component) => {
            let ComponentProjection::Model(model) = component.as_ref() else {
                panic!("model")
            };
            &model.capabilities[&id]
        }
        InspectionResult::Subject(subject) => {
            let SubjectProjection::Capability { declaration, .. } = subject.as_ref() else {
                panic!("capability")
            };
            declaration
        }
    }
}
fn exercise(shape: Shape) {
    let fixture = match shape {
        Shape::Sequence => GitGovernance::minimal(),
        Shape::Mapping => GitGovernance::full(),
    };
    let mut metadata = fixture.read("AGENTS.md");
    metadata["repository_governance"]["capabilities"]["governance_authority"]["configuration"] =
        json!({"deep":raw_configuration(shape)});
    fixture.write("AGENTS.md", metadata);
    fixture.git(&["add", "-A"]);
    let index = std::fs::read(fixture.root.join(".git/index")).unwrap();
    let carrier = std::fs::read(fixture.root.join("AGENTS.md")).unwrap();
    let observation = open(&fixture.root).unwrap();
    assert_eq!(
        git_governance::native_units(observation.repository()),
        git_governance::native_units(&fixture.root)
    );
    let expected = typed_configuration(shape);
    let tagged = tagged_configuration(shape);
    let declaration = json!({"configuration":{"deep":tagged},"routes":{"profile":{
        "segments":["capabilities","governance_authority","routes","profile"],
        "declared_path":"authority.md",
        "resolved_target":NativePath::from_path(&fixture.root.join("authority.md")).unwrap()
    }}});
    let repository = NativePath::from_path(&fixture.root).unwrap();
    let mut cases = Vec::new();
    for selector in [
        InspectSelector::All {},
        InspectSelector::Model {},
        InspectSelector::Capability {
            id: "governance_authority".try_into().unwrap(),
        },
    ] {
        let inspection = observation.inspect(selector.clone()).unwrap();
        assert_eq!(inspection.provenance(), observation.provenance());
        assert_eq!(
            capability(inspection.result()).configuration["deep"],
            expected
        );
        let pointer = match selector {
            InspectSelector::All {} => "/components/model/data/capabilities/governance_authority",
            InspectSelector::Model {} => "/component/data/capabilities/governance_authority",
            _ => "/subject/declaration",
        };
        let value = serde_json::to_value(&inspection).unwrap();
        assert_eq!(value.pointer(pointer).unwrap(), &declaration);
        cases.push((
            "inspect",
            json!({"selector":selector}),
            Response::Success(SuccessResponse::Inspect {
                api_version: VersionOne::V1,
                outcome: ResultTag::Result,
                value: Box::new(inspection),
            }),
        ));
    }
    let query = ExplainQuery::Capability {
        id: "governance_authority".try_into().unwrap(),
    };
    let explanation = observation.explain(query.clone()).unwrap();
    assert_eq!(explanation.provenance(), observation.provenance());
    let Explanation::Declaration(subject) = explanation.explanation() else {
        panic!("declaration")
    };
    let SubjectProjection::Capability {
        declaration: actual,
        ..
    } = subject.as_ref()
    else {
        panic!("capability")
    };
    assert_eq!(actual.configuration["deep"], expected);
    assert_eq!(
        serde_json::to_value(&explanation)
            .unwrap()
            .pointer("/explanation/declaration/declaration")
            .unwrap(),
        &declaration
    );
    cases.push((
        "explain",
        json!({"query":query}),
        Response::Success(SuccessResponse::Explain {
            api_version: VersionOne::V1,
            outcome: ResultTag::Result,
            value: Box::new(explanation),
        }),
    ));
    let cwd = tempfile::tempdir().unwrap();
    for (operation, mut input, rust) in cases {
        input["repository"] = serde_json::to_value(&repository).unwrap();
        let request = serde_json::to_vec(&json!({"api_version":1,
            "operation":operation,"input":input}))
        .unwrap();
        let oracle = serde_json::to_value(&rust).unwrap();
        assert!(depth(&oracle) > 128);
        let bytes = serde_json::to_vec(&rust).unwrap();
        assert!(bytes.len() < 32_768);
        let mut framed = bytes.clone();
        framed.push(b'\n');
        let mut args = vec![
            operation.into(),
            "--repository".into(),
            fixture.root.clone().into_os_string(),
        ];
        let field = if operation == "inspect" {
            "selector"
        } else {
            "query"
        };
        args.extend([
            format!("--{field}-json").into(),
            input[field].to_string().into(),
            "--json".into(),
        ]);
        // E2E first: the uncorrected binary exits 70 and emits no response here.
        for (arguments, stdin) in [(args, &b""[..]), (vec!["json".into()], request.as_slice())] {
            let output = cli(&arguments, stdin, cwd.path());
            assert_eq!(
                output.status.code(),
                Some(0),
                "{operation}: Rust JSON bytes={}, depth={}, stdout bytes={}, stderr={}",
                bytes.len(),
                depth(&oracle),
                output.stdout.len(),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty());
            assert_eq!(output.stdout, framed);
            assert_eq!(
                output.stdout.iter().filter(|byte| **byte == b'\n').count(),
                1
            );
        }
        let response = process_json(&request);
        assert_eq!(serde_json::to_value(&response).unwrap(), oracle);
        assert_eq!(encode_response(&response).unwrap(), bytes);
    }
    assert_eq!(
        std::fs::read(fixture.root.join(".git/index")).unwrap(),
        index
    );
    assert_eq!(
        std::fs::read(fixture.root.join("AGENTS.md")).unwrap(),
        carrier
    );
    assert!(!fixture.root.join("EFFECT-MARKER").exists());
}

#[test]
fn eighty_nested_sequences_survive_rust_json_and_both_cli_adapters() {
    exercise(Shape::Sequence);
}
#[test]
fn eighty_nested_mappings_survive_rust_json_and_both_cli_adapters() {
    exercise(Shape::Mapping);
}
