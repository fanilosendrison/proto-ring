use super::{REQUEST_LIMIT, bounded_buffer::BoundedBuffer, failure::protocol};
use crate::{ErrorCode, FailureResponse, NativePath, Operation};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io::Write,
    path::Path,
};

pub(super) enum Invocation {
    Stdin,
    Wire(Vec<u8>),
}
enum Field {
    Text(String),
    Raw(String),
    Path(NativePath),
}

fn command(name: &OsStr) -> Option<Operation> {
    match name.to_str()? {
        "discover" => Some(Operation::Discover),
        "inspect" => Some(Operation::Inspect),
        "explain" => Some(Operation::Explain),
        "check" => Some(Operation::CheckRepositoryIntegrity),
        "output" => Some(Operation::ReadExecutionOutput),
        _ => None,
    }
}

/// Syntax only. Native paths never pass through a Unicode conversion, and JSON
/// fragments remain raw until the common duplicate-safe decoder sees them.
pub(super) fn adapt(args: &[OsString]) -> Result<Invocation, FailureResponse> {
    let operation = args.first().and_then(|arg| command(arg));
    let invalid = || protocol(operation, ErrorCode::InvalidRequest);
    let name = args
        .first()
        .and_then(|arg| arg.to_str())
        .ok_or_else(invalid)?;
    let allowed: &[&str] = match name {
        "json" => &["--local-context-file"],
        "discover" => &["--json"],
        "inspect" => &["--repository", "--selector", "--selector-json", "--json"],
        "explain" => &["--repository", "--query-json", "--json"],
        "check" => &[
            "--repository",
            "--local-context-file",
            "--expected-observation-json",
            "--json",
        ],
        "output" => &[
            "--receipt",
            "--validation",
            "--instance",
            "--stream",
            "--offset",
            "--max-bytes",
            "--json",
        ],
        _ => return Err(invalid()),
    };
    let mut flags = BTreeMap::new();
    let mut index = 1;
    while index < args.len() {
        let flag = args[index]
            .to_str()
            .filter(|flag| allowed.contains(flag))
            .ok_or_else(invalid)?;
        if flags.contains_key(flag) {
            return Err(invalid());
        }
        index += 1;
        let value = if flag == "--json" {
            OsString::new()
        } else {
            let value = args.get(index).ok_or_else(invalid)?.clone();
            index += 1;
            value
        };
        flags.insert(flag, value);
    }
    let text = |key| {
        flags
            .get(key)
            .and_then(|value| value.to_str())
            .map(str::to_owned)
            .ok_or_else(invalid)
    };
    let native = |key| {
        flags
            .get(key)
            .and_then(|value| NativePath::from_path(Path::new(value)).ok())
            .ok_or_else(invalid)
    };
    if flags.contains_key("--local-context-file") {
        let _ = native("--local-context-file")?;
    }
    if name == "json" {
        return Ok(Invocation::Stdin);
    }
    if !flags.contains_key("--json") {
        return Err(invalid());
    }
    let operation = operation.ok_or_else(invalid)?;
    let fields = match operation {
        Operation::Discover => vec![],
        Operation::Inspect => {
            let simple = flags.contains_key("--selector");
            let structured = flags.contains_key("--selector-json");
            if simple == structured {
                return Err(invalid());
            }
            let selector = if simple {
                // This option admits only the nine simple selector forms. The
                // common typed decoder rejects non-simple forms with no ID.
                serde_json::to_string(&serde_json::json!({"kind":text("--selector")?}))
                    .map_err(|_| invalid())?
            } else {
                text("--selector-json")?
            };
            vec![
                ("repository", Field::Path(native("--repository")?)),
                ("selector", Field::Raw(selector)),
            ]
        }
        Operation::Explain => vec![
            ("repository", Field::Path(native("--repository")?)),
            ("query", Field::Raw(text("--query-json")?)),
        ],
        Operation::CheckRepositoryIntegrity => vec![
            ("repository", Field::Path(native("--repository")?)),
            (
                "expected_observation",
                Field::Raw(if flags.contains_key("--expected-observation-json") {
                    text("--expected-observation-json")?
                } else {
                    "null".into()
                }),
            ),
        ],
        Operation::ReadExecutionOutput => vec![
            ("receipt_id", Field::Text(text("--receipt")?)),
            ("validation_id", Field::Text(text("--validation")?)),
            ("instance_index", Field::Text(text("--instance")?)),
            ("stream", Field::Text(text("--stream")?)),
            ("offset", Field::Text(text("--offset")?)),
            ("max_bytes", Field::Raw(text("--max-bytes")?)),
        ],
    };
    wire(operation, fields)
        .map(Invocation::Wire)
        .map_err(|()| invalid())
}

fn wire(operation: Operation, fields: Vec<(&str, Field)>) -> Result<Vec<u8>, ()> {
    let mut buffer = BoundedBuffer::new(REQUEST_LIMIT);
    buffer
        .write_all(b"{\"api_version\":1,\"operation\":")
        .map_err(|_| ())?;
    serde_json::to_writer(&mut buffer, &operation).map_err(|_| ())?;
    buffer.write_all(b",\"input\":{").map_err(|_| ())?;
    for (index, (key, value)) in fields.into_iter().enumerate() {
        if index > 0 {
            buffer.write_all(b",").map_err(|_| ())?;
        }
        serde_json::to_writer(&mut buffer, key).map_err(|_| ())?;
        buffer.write_all(b":").map_err(|_| ())?;
        match value {
            Field::Text(text) => serde_json::to_writer(&mut buffer, &text).map_err(|_| ())?,
            Field::Path(path) => serde_json::to_writer(&mut buffer, &path).map_err(|_| ())?,
            Field::Raw(raw) => {
                // A fragment cannot escape its input property and inject root
                // fields. RawValue preserves duplicate keys for common decoding.
                serde_json::from_str::<&serde_json::value::RawValue>(&raw).map_err(|_| ())?;
                buffer.write_all(raw.as_bytes()).map_err(|_| ())?;
            }
        }
    }
    buffer.write_all(b"}}").map_err(|_| ())?;
    Ok(buffer.finish())
}
