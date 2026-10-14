use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use super::{PRESENTATION_DELIMITERS, unicode14};
const CUES: [&str; 10] = [
    "is defined as",
    "is defined by",
    "are defined as",
    "are defined by",
    "refer to",
    "refers to",
    "means",
    "mean",
    "are",
    "is",
];

struct FoldedText {
    original: Vec<char>,
    folded: Vec<char>,
    boundaries: Vec<Option<usize>>,
}

impl FoldedText {
    fn new(value: &str) -> Self {
        let original: Vec<_> = value.chars().collect();
        let mut folded = Vec::new();
        let mut boundaries = vec![Some(0)];
        for (index, scalar) in original.iter().copied().enumerate() {
            let mapped = unicode14::case_fold_scalar(scalar);
            let mapped_len = mapped.len();
            for (offset, folded_scalar) in mapped.into_iter().enumerate() {
                folded.push(folded_scalar);
                boundaries.push((offset + 1 == mapped_len).then_some(index + 1));
            }
        }
        Self {
            original,
            folded,
            boundaries,
        }
    }

    fn original_spans(&self, expression: &str) -> Vec<(usize, usize)> {
        let needle: Vec<_> = expression
            .chars()
            .flat_map(unicode14::case_fold_scalar)
            .collect();
        if needle.is_empty() || needle.len() > self.folded.len() {
            return Vec::new();
        }
        let mut spans = Vec::new();
        for start in 0..=self.folded.len() - needle.len() {
            let end = start + needle.len();
            if self.folded[start..end] != needle {
                continue;
            }
            let (Some(original_start), Some(original_end)) =
                (self.boundaries[start], self.boundaries[end])
            else {
                continue;
            };
            if expression_boundaries(&self.original, original_start, original_end) {
                spans.push((original_start, original_end));
            }
        }
        spans
    }
}

pub fn definition_concepts(text: &str, expressions: &[(String, String)]) -> Vec<String> {
    let folded = FoldedText::new(text);
    let mut concepts = BTreeSet::new();
    for (expression, concept) in expressions {
        if folded
            .original_spans(expression)
            .into_iter()
            .any(|span| definition_context(&folded.original, span))
        {
            concepts.insert(concept.clone());
        }
    }
    concepts.into_iter().collect()
}

pub fn normalize_text(value: &str) -> String {
    unicode14::split_whitespace(value).join(" ")
}

pub fn fingerprint_text(value: &str) -> String {
    let digest = Sha256::digest(normalize_text(value).as_bytes());
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

fn expression_boundaries(original: &[char], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|index| original.get(index));
    let after = original.get(end);
    !before.is_some_and(|value| value.is_ascii_alphanumeric())
        && !after.is_some_and(|value| value.is_ascii_alphanumeric())
}

fn definition_context(original: &[char], span: (usize, usize)) -> bool {
    prose_context(original, span.1) || equation_context(original, span)
}

fn prose_context(original: &[char], mut position: usize) -> bool {
    while original
        .get(position)
        .is_some_and(|value| PRESENTATION_DELIMITERS.contains(value))
    {
        position += 1;
    }
    let whitespace_start = position;
    while original
        .get(position)
        .is_some_and(|value| unicode14::is_whitespace(*value))
    {
        position += 1;
    }
    position > whitespace_start && CUES.iter().any(|cue| cue_matches(original, position, cue))
}

fn cue_matches(original: &[char], mut position: usize, cue: &str) -> bool {
    let cue: Vec<_> = cue.chars().collect();
    let mut index = 0;
    while index < cue.len() {
        if cue[index] == ' ' {
            let start = position;
            while original
                .get(position)
                .is_some_and(|value| unicode14::is_whitespace(*value))
            {
                position += 1;
            }
            if position == start {
                return false;
            }
            index += 1;
            continue;
        }
        let Some(actual) = original.get(position).copied() else {
            return false;
        };
        if !actual.is_ascii() || ascii_lower(actual) != cue[index] {
            return false;
        }
        position += 1;
        index += 1;
    }
    !original
        .get(position)
        .is_some_and(|value| value.is_ascii_alphanumeric() || *value == '_')
}

fn ascii_lower(value: char) -> char {
    if value.is_ascii_uppercase() {
        char::from_u32(value as u32 + 32).expect("ASCII lowercase is a scalar")
    } else {
        value
    }
}

fn equation_context(original: &[char], span: (usize, usize)) -> bool {
    let line_start = original[..span.0]
        .iter()
        .rposition(|value| *value == '\n')
        .map_or(0, |index| index + 1);
    if original[line_start..span.0]
        .iter()
        .any(|value| !unicode14::is_whitespace(*value) && !PRESENTATION_DELIMITERS.contains(value))
    {
        return false;
    }
    let mut position = span.1;
    while original.get(position).is_some_and(|value| {
        unicode14::is_whitespace(*value) || PRESENTATION_DELIMITERS.contains(value)
    }) {
        position += 1;
    }
    match original.get(position..position + 2) {
        Some([':', '=']) => true,
        Some(['=', '=']) => false,
        _ => original.get(position) == Some(&'=') && original.get(position + 1) != Some(&'='),
    }
}
