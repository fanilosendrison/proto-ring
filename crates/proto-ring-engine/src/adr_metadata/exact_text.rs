use super::{AdrMetadataError, failure};

const UTF8_BOM: &[u8] = &[0xef, 0xbb, 0xbf];
const CONTEXT_HEADING: &[u8] = b"## Context";

pub(crate) fn checked_adr_text(bytes: &[u8]) -> Result<&str, AdrMetadataError> {
    if bytes.starts_with(UTF8_BOM) {
        return Err(failure("UTF-8 BOM is forbidden"));
    }
    if bytes.contains(&b'\r') {
        return Err(failure("carriage return is forbidden"));
    }
    std::str::from_utf8(bytes).map_err(|_| failure("ADR bytes are not valid UTF-8"))
}

pub(crate) fn decision_body(bytes: &[u8]) -> Result<Vec<u8>, AdrMetadataError> {
    checked_adr_text(bytes)?;
    let start = unique_line_start(bytes, is_context_heading, "decision-body boundary")?;
    Ok(bytes[start..].to_vec())
}

pub(crate) fn h1_text(bytes: &[u8]) -> Result<String, AdrMetadataError> {
    checked_adr_text(bytes)?;
    let start = unique_line_start(bytes, is_h1, "level-one heading")?;
    let end = line_end(bytes, start);
    String::from_utf8(bytes[start + 2..end].to_vec())
        .map_err(|_| failure("level-one heading is not valid UTF-8"))
}

pub(crate) fn preserved_payload(bytes: &[u8]) -> Result<Vec<u8>, AdrMetadataError> {
    checked_adr_text(bytes)?;
    unique_line_start(bytes, is_context_heading, "decision-body boundary")?;
    let start = unique_line_start(bytes, is_h1, "level-one heading")?;
    Ok(bytes[start..].to_vec())
}

fn unique_line_start(
    bytes: &[u8],
    predicate: fn(&[u8]) -> bool,
    label: &str,
) -> Result<usize, AdrMetadataError> {
    let mut found = None;
    let mut start = 0;
    while start < bytes.len() {
        let end = line_end(bytes, start);
        if predicate(&bytes[start..end]) {
            if found.is_some() {
                return Err(failure(format!("multiple {label} lines")));
            }
            found = Some(start);
        }
        start = if end < bytes.len() {
            end + 1
        } else {
            bytes.len()
        };
    }
    found.ok_or_else(|| failure(format!("missing {label}")))
}

fn line_end(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |relative| start + relative)
}

fn is_context_heading(line: &[u8]) -> bool {
    line.starts_with(CONTEXT_HEADING)
        && (line.len() == CONTEXT_HEADING.len() || line.get(CONTEXT_HEADING.len()) == Some(&b' '))
}

fn is_h1(line: &[u8]) -> bool {
    line.starts_with(b"# ") && line.len() > 2
}
