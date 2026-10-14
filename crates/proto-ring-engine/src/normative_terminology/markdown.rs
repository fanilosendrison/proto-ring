use std::collections::{BTreeMap, BTreeSet};

use super::matching::{definition_concepts, fingerprint_text};
use super::unicode14;
use super::{
    CanonicalAnchor, ConsumerPolicy, DiagnosticCategory, Discovery, MarkdownBlock, MarkdownParse,
    Occurrence, RegistryEntry, RegistryFormat, RegistryParse, TerminologyDiagnostic,
    TerminologyError, failure, parse_registry, validate_allowed_roles,
};

pub fn parse_markdown<P: ConsumerPolicy + ?Sized>(
    markdown: &str,
    format: &RegistryFormat,
    policy: &P,
    registry: Option<&RegistryParse>,
) -> Result<MarkdownParse, TerminologyError> {
    let lines: Vec<_> = markdown.split('\n').collect();
    let marker_range = registry.and_then(|value| value.marker_range);
    let registered: Option<BTreeSet<&str>> = registry.map(|value| {
        value
            .entries
            .iter()
            .map(|entry| entry.anchor.as_str())
            .collect()
    });
    let mut blocks = Vec::new();
    let mut anchors: BTreeMap<String, Vec<CanonicalAnchor>> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let mut heading = String::new();
    let mut section = String::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut paragraph_line = 0;
    let mut index = 0;
    while index < lines.len() {
        if marker_range.is_some_and(|(start, end)| (start..=end).contains(&index)) {
            flush_paragraph(
                &mut paragraph,
                paragraph_line,
                &section,
                &heading,
                &mut blocks,
            );
            index += 1;
            continue;
        }
        let line = lines[index];
        if let Some(text) = heading_text(line) {
            flush_paragraph(
                &mut paragraph,
                paragraph_line,
                &section,
                &heading,
                &mut blocks,
            );
            heading = text;
            section = policy.section_from_heading(&heading)?;
            index += 1;
            continue;
        }
        if let Some(delimiter) = opening_fence(line) {
            flush_paragraph(
                &mut paragraph,
                paragraph_line,
                &section,
                &heading,
                &mut blocks,
            );
            let first_line = index + 1;
            let mut fenced = vec![line];
            index += 1;
            while index < lines.len() {
                let fence_line = lines[index];
                fenced.push(fence_line);
                index += 1;
                if unicode14::lstrip(fence_line).starts_with(delimiter) {
                    break;
                }
            }
            blocks.push(MarkdownBlock {
                line: first_line,
                section: section.clone(),
                heading: heading.clone(),
                text: fenced.join("\n"),
            });
            continue;
        }
        if let Some(destination) = anchor_destination(line, format)? {
            flush_paragraph(
                &mut paragraph,
                paragraph_line,
                &section,
                &heading,
                &mut blocks,
            );
            if registered
                .as_ref()
                .is_some_and(|values| !values.contains(destination.as_str()))
            {
                diagnostics.push(TerminologyDiagnostic::new(
                    DiagnosticCategory::UnregisteredCanonicalAnchor,
                ));
            }
            anchors
                .entry(destination)
                .or_default()
                .push(CanonicalAnchor {
                    line: index + 1,
                    section: section.clone(),
                    heading: heading.clone(),
                });
            index += 1;
            continue;
        }
        if unicode14::strip(line).is_empty() {
            flush_paragraph(
                &mut paragraph,
                paragraph_line,
                &section,
                &heading,
                &mut blocks,
            );
        } else {
            if paragraph.is_empty() {
                paragraph_line = index + 1;
            }
            paragraph.push(line);
        }
        index += 1;
    }
    flush_paragraph(
        &mut paragraph,
        paragraph_line,
        &section,
        &heading,
        &mut blocks,
    );
    if let Some(registered) = registered {
        for destination in registered {
            if anchors
                .get(destination)
                .is_some_and(|occurrences| occurrences.len() > 1)
            {
                diagnostics.push(TerminologyDiagnostic::new(
                    DiagnosticCategory::DuplicateRegisteredAnchor,
                ));
            }
        }
    }
    Ok(MarkdownParse {
        blocks,
        anchors,
        diagnostics,
    })
}

pub fn discover_occurrences<P: ConsumerPolicy + ?Sized>(
    markdown: &str,
    format: &RegistryFormat,
    policy: &P,
) -> Result<Discovery, TerminologyError> {
    validate_allowed_roles(policy)?;
    let registry = parse_registry(markdown, format);
    if !registry.diagnostics.is_empty() {
        return Ok(Discovery {
            occurrences: Vec::new(),
            diagnostics: registry.diagnostics,
        });
    }
    let parsed = parse_markdown(markdown, format, policy, Some(&registry))?;
    let mut diagnostics = parsed.diagnostics.clone();
    let expressions = registry_expressions(&registry.entries);
    let all_anchor_lines: Vec<_> = parsed
        .anchors
        .values()
        .flatten()
        .map(|anchor| anchor.line)
        .collect();
    let mut canonical_bindings = BTreeSet::new();
    let mut occurrences = Vec::new();
    for entry in &registry.entries {
        let Some(anchor_occurrences) = parsed.anchors.get(&entry.anchor) else {
            diagnostics.push(TerminologyDiagnostic::new(
                DiagnosticCategory::MissingCanonicalAnchor,
            ));
            continue;
        };
        if anchor_occurrences.len() != 1 {
            continue;
        }
        let anchor = &anchor_occurrences[0];
        let Some(block) = parsed.blocks.iter().find(|block| block.line > anchor.line) else {
            diagnostics.push(TerminologyDiagnostic::new(
                DiagnosticCategory::MissingCanonicalDefinition,
            ));
            continue;
        };
        if block.section != anchor.section {
            diagnostics.push(TerminologyDiagnostic::new(
                DiagnosticCategory::CanonicalSectionMismatch,
            ));
            continue;
        }
        if all_anchor_lines
            .iter()
            .any(|line| anchor.line < *line && *line < block.line)
        {
            diagnostics.push(TerminologyDiagnostic::new(
                DiagnosticCategory::InterveningCanonicalAnchor,
            ));
            continue;
        }
        if !policy.canonical_location(entry, block)? {
            diagnostics.push(TerminologyDiagnostic::new(
                DiagnosticCategory::CanonicalLocationRejected,
            ));
            continue;
        }
        let concepts = definition_concepts(&block.text, &expressions);
        if !concepts.contains(&entry.key) {
            diagnostics.push(TerminologyDiagnostic::new(
                DiagnosticCategory::MissingCanonicalDefinition,
            ));
            continue;
        }
        canonical_bindings.insert((block.line, entry.key.clone()));
        occurrences.push(occurrence(block, vec![entry.key.clone()], true));
    }
    for block in &parsed.blocks {
        let mut concepts = definition_concepts(&block.text, &expressions);
        concepts.retain(|key| !canonical_bindings.contains(&(block.line, key.clone())));
        if !concepts.is_empty() {
            occurrences.push(occurrence(block, concepts, false));
        }
    }
    Ok(Discovery {
        occurrences,
        diagnostics,
    })
}

fn occurrence(block: &MarkdownBlock, concepts: Vec<String>, canonical: bool) -> Occurrence {
    Occurrence::new(
        concepts,
        block.section.clone(),
        block.heading.clone(),
        fingerprint_text(&block.text),
        block.line,
        canonical,
    )
}

fn registry_expressions(entries: &[RegistryEntry]) -> Vec<(String, String)> {
    entries
        .iter()
        .flat_map(|entry| {
            std::iter::once(&entry.canonical_term)
                .chain(entry.aliases.iter())
                .chain(entry.deprecated.iter())
                .map(|expression| (expression.clone(), entry.key.clone()))
        })
        .collect()
}

fn flush_paragraph(
    paragraph: &mut Vec<&str>,
    line: usize,
    section: &str,
    heading: &str,
    blocks: &mut Vec<MarkdownBlock>,
) {
    if paragraph.is_empty() {
        return;
    }
    blocks.push(MarkdownBlock {
        line,
        section: section.to_owned(),
        heading: heading.to_owned(),
        text: paragraph.join("\n"),
    });
    paragraph.clear();
}

fn anchor_destination(
    line: &str,
    format: &RegistryFormat,
) -> Result<Option<String>, TerminologyError> {
    let Some(matched) = crate::portable_pattern::PortablePattern::full_match(
        &format.anchor_pattern,
        unicode14::strip(line),
    )
    .map_err(|error| failure(error.to_string()))?
    else {
        return Ok(None);
    };
    matched
        .positional_capture(1)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .map(Some)
        .ok_or_else(|| failure("canonical anchor match has no destination capture"))
}

fn opening_fence(line: &str) -> Option<&'static str> {
    let line = unicode14::lstrip(line);
    if line.starts_with("```") {
        Some("```")
    } else if line.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

fn heading_text(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let spaces = bytes.iter().take_while(|byte| **byte == b' ').count();
    if spaces > 3 {
        return None;
    }
    let hashes = bytes[spaces..]
        .iter()
        .take_while(|byte| **byte == b'#')
        .count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let after = spaces + hashes;
    if bytes
        .get(after)
        .is_some_and(|byte| !matches!(byte, b' ' | b'\t'))
    {
        return None;
    }
    let mut text = unicode14::strip(&line[after..]);
    let closing_start = text.trim_end_matches('#').len();
    if closing_start < text.len()
        && (closing_start == 0
            || text[..closing_start]
                .chars()
                .next_back()
                .is_some_and(unicode14::is_whitespace))
    {
        text = unicode14::strip(&text[..closing_start]);
    }
    Some(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::heading_text;

    #[test]
    fn whitespace_separated_closing_hashes_admit_empty_heading_text() {
        assert_eq!(heading_text("## ###"), Some(String::new()));
        assert_eq!(
            heading_text("   ## Heading ###"),
            Some("Heading".to_owned())
        );
        assert_eq!(
            heading_text("## ###heading###"),
            Some("###heading###".to_owned())
        );
    }
}
