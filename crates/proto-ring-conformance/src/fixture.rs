#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

use crate::model::{RecordEntry, TransportValue};

#[derive(Debug)]
pub struct FixtureError(String);

impl FixtureError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for FixtureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for FixtureError {}

pub struct MaterializedFixture {
    temporary: Option<TempDir>,
    arguments: TransportValue,
}

impl MaterializedFixture {
    pub fn root(&self) -> Option<&Path> {
        self.temporary.as_ref().map(|temporary| temporary.path())
    }

    pub fn arguments(&self) -> &TransportValue {
        &self.arguments
    }
}

pub fn materialize(fixture: &Value) -> Result<MaterializedFixture, FixtureError> {
    let object = fixture
        .as_object()
        .ok_or_else(|| failure("fixture must be an object"))?;
    let kind = string_field(object, "kind")?;
    match kind {
        "inline" => Ok(MaterializedFixture {
            temporary: None,
            arguments: decode_transport(required(object, "input")?)?,
        }),
        "repository_plan" => {
            let temporary = tempfile::tempdir()
                .map_err(|error| failure(format!("cannot create fixture root: {error}")))?;
            let steps = required(object, "steps")?
                .as_array()
                .ok_or_else(|| failure("steps must be an array"))?;
            let mut captures = HashMap::new();
            for raw_step in steps {
                let step = interpolate(raw_step, &captures)?;
                apply_step(temporary.path(), &step, &mut captures)?;
            }
            let invocation = interpolate(required(object, "invocation")?, &captures)?;
            Ok(MaterializedFixture {
                temporary: Some(temporary),
                arguments: decode_transport(&invocation)?,
            })
        }
        _ => Err(failure(format!("unsupported fixture kind: {kind}"))),
    }
}

pub fn record_field<'a>(
    value: &'a TransportValue,
    name: &str,
) -> Result<&'a TransportValue, FixtureError> {
    match value {
        TransportValue::Record(entries) => entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| &entry.value)
            .ok_or_else(|| failure(format!("missing fixture field: {name}"))),
        _ => Err(failure("fixture value must be a record")),
    }
}

pub fn string(value: &TransportValue) -> Result<&str, FixtureError> {
    match value {
        TransportValue::String(value) => Ok(value),
        _ => Err(failure("fixture value must be a string")),
    }
}

pub fn sequence(value: &TransportValue) -> Result<&[TransportValue], FixtureError> {
    match value {
        TransportValue::Sequence(values) => Ok(values),
        _ => Err(failure("fixture value must be a sequence")),
    }
}

pub fn record(value: &TransportValue) -> Result<&[RecordEntry], FixtureError> {
    match value {
        TransportValue::Record(entries) => Ok(entries),
        _ => Err(failure("fixture value must be a record")),
    }
}

fn apply_step(
    root: &Path,
    step: &Value,
    captures: &mut HashMap<String, String>,
) -> Result<(), FixtureError> {
    let object = step
        .as_object()
        .ok_or_else(|| failure("fixture step must be an object"))?;
    let operation = string_field(object, "op")?;
    match operation {
        "mkdir" => {
            let path = checked_path(root, string_field(object, "path")?)?;
            fs::create_dir_all(&path).map_err(|error| failure(format!("mkdir failed: {error}")))
        }
        "write_utf8" => {
            let path = checked_path(root, string_field(object, "path")?)?;
            let text = string_field(object, "text")?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| failure(format!("write_utf8 parent failed: {error}")))?;
            }
            fs::write(&path, text.as_bytes())
                .map_err(|error| failure(format!("write_utf8 failed: {error}")))
        }
        "symlink" => {
            let path = checked_path(root, string_field(object, "path")?)?;
            let target = string_field(object, "target")?;
            let target_is_directory = required(object, "target_is_directory")?
                .as_bool()
                .ok_or_else(|| failure("target_is_directory must be boolean"))?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| failure(format!("symlink parent failed: {error}")))?;
            }
            create_symlink(target, &path, target_is_directory)
        }
        "git" => apply_git_step(root, object, captures),
        _ => Err(failure(format!(
            "unsupported fixture operation: {operation}"
        ))),
    }
}

fn checked_path(root: &Path, relative: &str) -> Result<PathBuf, FixtureError> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(failure("fixture path escapes root"));
    }
    Ok(root.join(path))
}

fn interpolate(value: &Value, captures: &HashMap<String, String>) -> Result<Value, FixtureError> {
    Ok(match value {
        Value::String(value) => Value::String(interpolate_string(value, captures)?),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| interpolate(value, captures))
                .collect::<Result<_, _>>()?,
        ),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| Ok((key.clone(), interpolate(value, captures)?)))
                .collect::<Result<_, FixtureError>>()?,
        ),
        _ => value.clone(),
    })
}

fn interpolate_string(
    value: &str,
    captures: &HashMap<String, String>,
) -> Result<String, FixtureError> {
    let mut result = value.to_owned();
    while let Some(start) = result.find("${") {
        let relative_end = result[start + 2..]
            .find('}')
            .ok_or_else(|| failure("unterminated capture interpolation"))?;
        let end = start + 2 + relative_end;
        let name = &result[start + 2..end];
        let captured = captures
            .get(name)
            .ok_or_else(|| failure(format!("unbound capture: {name}")))?;
        result.replace_range(start..=end, captured);
    }
    Ok(result)
}

fn apply_git_step(
    root: &Path,
    object: &serde_json::Map<String, Value>,
    captures: &mut HashMap<String, String>,
) -> Result<(), FixtureError> {
    let argv = required(object, "argv")?
        .as_array()
        .ok_or_else(|| failure("git argv must be a sequence"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| failure("git argv values must be strings"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let stdin = nullable_string(object, "stdin_hex")?
        .map(decode_hex)
        .transpose()?;
    let environment = required(object, "environment")?
        .as_object()
        .ok_or_else(|| failure("git environment must be a mapping"))?;
    let capture = nullable_string(object, "capture_stdout_as")?;
    if capture.is_some_and(str::is_empty) {
        return Err(failure("git capture_stdout_as must be nonempty"));
    }

    let mut command = Command::new("git");
    command
        .current_dir(root)
        .args(argv)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in git_environment() {
        command.env(name, value);
    }
    for (name, value) in environment {
        command.env(
            name,
            value
                .as_str()
                .ok_or_else(|| failure("git environment values must be strings"))?,
        );
    }
    command.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let mut child = command
        .spawn()
        .map_err(|error| failure(format!("git execution failed: {error}")))?;
    if let Some(bytes) = stdin {
        child
            .stdin
            .take()
            .ok_or_else(|| failure("git stdin was unavailable"))?
            .write_all(&bytes)
            .map_err(|error| failure(format!("git stdin failed: {error}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| failure(format!("git execution failed: {error}")))?;
    if !output.status.success() {
        return Err(failure(format!(
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    if let Some(name) = capture {
        if !output.stdout.is_ascii() {
            return Err(failure("captured Git stdout is not ASCII"));
        }
        let value = std::str::from_utf8(&output.stdout)
            .expect("ASCII is valid UTF-8")
            .trim_matches([' ', '\t', '\r', '\n', '\u{000b}', '\u{000c}'])
            .to_owned();
        captures.insert(name.to_owned(), value);
    }
    Ok(())
}

fn nullable_string<'a>(
    object: &'a serde_json::Map<String, Value>,
    name: &str,
) -> Result<Option<&'a str>, FixtureError> {
    match required(object, name)? {
        Value::Null => Ok(None),
        Value::String(value) => Ok(Some(value)),
        _ => Err(failure(format!("{name} must be null or a string"))),
    }
}

fn decode_hex(value: &str) -> Result<Vec<u8>, FixtureError> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(failure("hex input has odd length"));
    }
    pairs
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|_| failure("invalid hex input"))?;
            u8::from_str_radix(text, 16).map_err(|_| failure("invalid hex input"))
        })
        .collect()
}

fn git_environment() -> [(&'static str, &'static str); 6] {
    [
        ("GIT_AUTHOR_NAME", "Corpus Author"),
        ("GIT_AUTHOR_EMAIL", "corpus@example.invalid"),
        ("GIT_COMMITTER_NAME", "Corpus Committer"),
        ("GIT_COMMITTER_EMAIL", "corpus@example.invalid"),
        ("GIT_AUTHOR_DATE", "2001-02-03T04:05:06+0000"),
        ("GIT_COMMITTER_DATE", "2001-02-03T04:05:06+0000"),
    ]
}

#[cfg(unix)]
fn create_symlink(target: &str, path: &Path, _directory: bool) -> Result<(), FixtureError> {
    std::os::unix::fs::symlink(target, path)
        .map_err(|error| failure(format!("symlink failed: {error}")))
}

#[cfg(windows)]
fn create_symlink(target: &str, path: &Path, directory: bool) -> Result<(), FixtureError> {
    let result = if directory {
        std::os::windows::fs::symlink_dir(target, path)
    } else {
        std::os::windows::fs::symlink_file(target, path)
    };
    result.map_err(|error| failure(format!("symlink failed: {error}")))
}

fn decode_transport(value: &Value) -> Result<TransportValue, FixtureError> {
    serde_json::from_value(value.clone())
        .map_err(|error| failure(format!("invalid fixture transport: {error}")))
}

fn required<'a>(
    object: &'a serde_json::Map<String, Value>,
    name: &str,
) -> Result<&'a Value, FixtureError> {
    object
        .get(name)
        .ok_or_else(|| failure(format!("missing fixture property: {name}")))
}

fn string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    name: &str,
) -> Result<&'a str, FixtureError> {
    required(object, name)?
        .as_str()
        .ok_or_else(|| failure(format!("fixture property {name} must be a string")))
}

fn failure(message: impl Into<String>) -> FixtureError {
    FixtureError(message.into())
}
