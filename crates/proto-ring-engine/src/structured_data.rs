#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::structured_data_yaml;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructuredValue {
    Mapping(BTreeMap<String, StructuredValue>),
    Sequence(Vec<StructuredValue>),
    String(String),
    Boolean(bool),
    Integer(String),
    Null,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontmatterEnvelope {
    pub payload: Vec<u8>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedFrontmatter {
    pub metadata: StructuredValue,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredDataError(pub(crate) String);

impl Display for StructuredDataError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for StructuredDataError {}

pub fn parse_document_bytes(bytes: &[u8]) -> Result<StructuredValue, StructuredDataError> {
    let text = checked_utf8(bytes)?;
    if text.lines().any(|line| line.starts_with('%')) {
        return Err(rejected("YAML directives are forbidden"));
    }
    structured_data_yaml::parse(text)
}

pub fn extract_frontmatter_bytes(bytes: &[u8]) -> Result<FrontmatterEnvelope, StructuredDataError> {
    checked_utf8(bytes)?;
    if !bytes.starts_with(b"---\n") {
        return Err(rejected("frontmatter opening delimiter is missing"));
    }
    let payload_start = 4;
    let mut line_start = payload_start;
    while line_start < bytes.len() {
        let relative_end = bytes[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .ok_or_else(|| rejected("frontmatter closing delimiter is missing"))?;
        let line_end = line_start + relative_end + 1;
        if &bytes[line_start..line_end] == b"---\n" {
            return Ok(FrontmatterEnvelope {
                payload: bytes[payload_start..line_start].to_vec(),
                body: bytes[line_end..].to_vec(),
            });
        }
        line_start = line_end;
    }
    Err(rejected("frontmatter closing delimiter is missing"))
}

pub fn parse_frontmatter_bytes(bytes: &[u8]) -> Result<ParsedFrontmatter, StructuredDataError> {
    let envelope = extract_frontmatter_bytes(bytes)?;
    let metadata = parse_document_bytes(&envelope.payload)?;
    Ok(ParsedFrontmatter {
        metadata,
        body: envelope.body,
    })
}

pub(crate) fn canonical_integer(value: &str) -> bool {
    let bytes = value.as_bytes();
    match bytes {
        [b'0'] => true,
        [b'-', first, rest @ ..] => {
            matches!(first, b'1'..=b'9') && rest.iter().all(u8::is_ascii_digit)
        }
        [first, rest @ ..] => matches!(first, b'1'..=b'9') && rest.iter().all(u8::is_ascii_digit),
        _ => false,
    }
}

pub(crate) fn rejected(message: impl Into<String>) -> StructuredDataError {
    StructuredDataError(message.into())
}

fn checked_utf8(bytes: &[u8]) -> Result<&str, StructuredDataError> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(rejected("UTF-8 BOM is forbidden"));
    }
    if bytes.contains(&b'\r') {
        return Err(rejected("carriage return is forbidden"));
    }
    std::str::from_utf8(bytes).map_err(|_| rejected("input is not valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_grammar_never_parses_magnitude() {
        for accepted in ["0", "42", "-42", &format!("1{}", "0".repeat(4300))] {
            assert!(canonical_integer(accepted));
        }
        for rejected in ["-0", "+1", "01", "1_000", "0x10", "1.0", "1e3"] {
            assert!(!canonical_integer(rejected));
        }
    }
}
