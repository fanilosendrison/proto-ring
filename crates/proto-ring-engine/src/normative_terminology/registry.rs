use std::collections::BTreeSet;

use crate::portable_pattern::PortablePattern;

use super::unicode14;
use super::{
    DiagnosticCategory, PRESENTATION_DELIMITERS, RegistryEntry, RegistryFormat, RegistryParse,
    TerminologyDiagnostic, TerminologyError, failure,
};

impl RegistryFormat {
    pub fn new(
        start_marker: String,
        end_marker: String,
        headers: Vec<String>,
        key_pattern_source: String,
        anchor_link_pattern_source: String,
        anchor_pattern_source: String,
    ) -> Result<Self, TerminologyError> {
        if start_marker.is_empty()
            || end_marker.is_empty()
            || start_marker == end_marker
            || headers.len() != 6
            || headers.iter().any(String::is_empty)
        {
            return Err(failure("invalid terminology registry format"));
        }
        let key_pattern = PortablePattern::compile(&key_pattern_source)
            .map_err(|error| failure(error.to_string()))?;
        let anchor_link_pattern = PortablePattern::compile(&anchor_link_pattern_source)
            .map_err(|error| failure(error.to_string()))?;
        let anchor_pattern = PortablePattern::compile(&anchor_pattern_source)
            .map_err(|error| failure(error.to_string()))?;
        if anchor_link_pattern.capture_count() == 0 || anchor_pattern.capture_count() == 0 {
            return Err(failure("anchor patterns require a destination capture"));
        }
        Ok(Self {
            start_marker,
            end_marker,
            headers,
            key_pattern_source,
            anchor_link_pattern_source,
            anchor_pattern_source,
            key_pattern,
            anchor_link_pattern,
            anchor_pattern,
        })
    }
}

pub fn parse_registry(markdown: &str, format: &RegistryFormat) -> RegistryParse {
    let lines: Vec<_> = markdown.split('\n').collect();
    let starts = marker_lines(&lines, &format.start_marker);
    let ends = marker_lines(&lines, &format.end_marker);
    if starts.len() != 1 || ends.len() != 1 || starts[0] >= ends[0] {
        return rejected(DiagnosticCategory::RegistryMarkerCardinality, None);
    }
    let range = (starts[0], ends[0]);
    let table: Vec<_> = lines[range.0 + 1..range.1]
        .iter()
        .map(|line| unicode14::strip(line))
        .filter(|line| !line.is_empty())
        .collect();
    if table.len() < 2 {
        return rejected(DiagnosticCategory::RegistryStructure, Some(range));
    }
    let Some(headers) = parse_cells(table[0]) else {
        return rejected(DiagnosticCategory::RegistryStructure, Some(range));
    };
    if headers != format.headers {
        return rejected(DiagnosticCategory::RegistryStructure, Some(range));
    }
    let Some(separator) = parse_cells(table[1]) else {
        return rejected(DiagnosticCategory::RegistryStructure, Some(range));
    };
    if separator.iter().any(|cell| !separator_cell(cell)) {
        return rejected(DiagnosticCategory::RegistryStructure, Some(range));
    }

    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    for line in &table[2..] {
        let result = parse_cells(line)
            .ok_or(DiagnosticCategory::RegistryStructure)
            .and_then(|cells| parse_entry(&cells, format));
        match result {
            Ok(entry) => entries.push(entry),
            Err(category) => diagnostics.push(TerminologyDiagnostic::new(category)),
        }
    }
    validate_registry_uniqueness(&entries, &mut diagnostics);
    if diagnostics.is_empty() {
        RegistryParse {
            entries,
            diagnostics,
            marker_range: Some(range),
        }
    } else {
        RegistryParse {
            entries: Vec::new(),
            diagnostics,
            marker_range: Some(range),
        }
    }
}

fn marker_lines(lines: &[&str], marker: &str) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (unicode14::strip(line) == marker).then_some(index))
        .collect()
}

fn parse_cells(line: &str) -> Option<Vec<String>> {
    let line = unicode14::strip(line);
    if line.len() < 2
        || !line.starts_with('|')
        || !line.ends_with('|')
        || line.starts_with("||")
        || line.ends_with("||")
    {
        return None;
    }
    let cells: Vec<_> = line[1..line.len() - 1]
        .split('|')
        .map(|cell| unicode14::strip(cell).to_owned())
        .collect();
    (cells.len() == 6).then_some(cells)
}

fn separator_cell(value: &str) -> bool {
    let bytes = value.as_bytes();
    let start = usize::from(bytes.first() == Some(&b':'));
    let end = bytes
        .len()
        .saturating_sub(usize::from(bytes.last() == Some(&b':')));
    end >= start + 3 && bytes[start..end].iter().all(|byte| *byte == b'-')
}

fn parse_entry(
    cells: &[String],
    format: &RegistryFormat,
) -> Result<RegistryEntry, DiagnosticCategory> {
    let key = decode_atom(&cells[0]).ok_or(DiagnosticCategory::RegistryStructure)?;
    if key.is_empty()
        || !PortablePattern::full_match(&format.key_pattern, &key)
            .ok()
            .flatten()
            .is_some()
    {
        return Err(DiagnosticCategory::RegistryStructure);
    }
    let canonical_term = decode_atom(&cells[1]).ok_or(DiagnosticCategory::RegistryStructure)?;
    if canonical_term.is_empty() {
        return Err(DiagnosticCategory::EmptyCanonicalTerm);
    }
    let matched = PortablePattern::full_match(&format.anchor_link_pattern, &cells[2])
        .ok()
        .flatten()
        .ok_or(DiagnosticCategory::RegistryStructure)?;
    let anchor = matched
        .positional_capture(1)
        .filter(|value| !value.is_empty())
        .ok_or(DiagnosticCategory::EmptyCanonicalDestination)?
        .to_owned();
    let aliases = decode_list(&cells[3]).ok_or(DiagnosticCategory::RegistryStructure)?;
    let deprecated = decode_list(&cells[4]).ok_or(DiagnosticCategory::RegistryStructure)?;
    let term_structure = decode_term_structure(&cells[5]);
    if term_structure.is_empty() {
        return Err(DiagnosticCategory::RegistryStructure);
    }
    let expressions = std::iter::once(&canonical_term)
        .chain(aliases.iter())
        .chain(deprecated.iter());
    let mut folded = BTreeSet::new();
    if expressions
        .map(|value| unicode14::case_fold(value))
        .any(|value| !folded.insert(value))
    {
        return Err(DiagnosticCategory::RegistryStructure);
    }
    Ok(RegistryEntry {
        key,
        canonical_term,
        anchor,
        aliases,
        deprecated,
        term_structure,
    })
}

fn decode_atom(value: &str) -> Option<String> {
    let value = unicode14::strip(value);
    let decoded = if value.starts_with('`') || value.ends_with('`') {
        if value.len() < 2 || !value.starts_with('`') || !value.ends_with('`') {
            return None;
        }
        let inner = &value[1..value.len() - 1];
        if inner.contains('`') {
            return None;
        }
        inner
    } else {
        if value.contains('`') {
            return None;
        }
        value
    };
    Some(unicode14::strip(decoded).to_owned())
}

fn decode_term_structure(value: &str) -> String {
    let value = unicode14::strip(value);

    let decoded = if value.starts_with('`') && value.ends_with('`') {
        if value.len() == 1 {
            ""
        } else {
            &value[1..value.len() - 1]
        }
    } else {
        value
    };

    unicode14::strip(decoded).to_owned()
}

fn decode_list(value: &str) -> Option<Vec<String>> {
    let value = unicode14::strip(value);
    if value.is_empty() || value == "—" {
        return Some(Vec::new());
    }
    value
        .split(';')
        .map(|item| {
            let raw = unicode14::strip(item);
            if is_em_dash_form(raw) {
                return None;
            }
            let decoded = decode_atom(raw)?;
            (!decoded.is_empty()).then_some(decoded)
        })
        .collect()
}

fn is_em_dash_form(value: &str) -> bool {
    let undecorated =
        unicode14::strip(value).trim_matches(|scalar| PRESENTATION_DELIMITERS.contains(&scalar));
    unicode14::strip(undecorated) == "—"
}

fn validate_registry_uniqueness(
    entries: &[RegistryEntry],
    diagnostics: &mut Vec<TerminologyDiagnostic>,
) {
    let mut keys = BTreeSet::new();
    let mut anchors = BTreeSet::new();
    for entry in entries {
        if !keys.insert(&entry.key) || !anchors.insert(&entry.anchor) {
            diagnostics.push(TerminologyDiagnostic::new(
                DiagnosticCategory::RegistryStructure,
            ));
        }
    }
}

fn rejected(category: DiagnosticCategory, range: Option<(usize, usize)>) -> RegistryParse {
    RegistryParse {
        entries: Vec::new(),
        diagnostics: vec![TerminologyDiagnostic::new(category)],
        marker_range: range,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_list, parse_cells};

    #[test]
    fn decorated_em_dash_is_not_an_expression_or_empty_list_sentinel() {
        for value in ["`—`", "*—*", "_—_", "~~—~~", "\"—\"", "‘—’", "`“—”`"] {
            assert_eq!(decode_list(value), None, "accepted {value}");
        }
        assert_eq!(decode_list("—"), Some(Vec::new()));
        assert_eq!(decode_list("`one`; —"), None);
    }

    #[test]
    fn single_outer_pipe_is_rejected_without_slicing() {
        assert_eq!(parse_cells("|"), None);
        assert_eq!(parse_cells(" | "), None);
    }
}
