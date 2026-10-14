use std::collections::BTreeMap;
use std::sync::OnceLock;

use sha2::{Digest, Sha256};

use super::{TerminologyError, failure};

pub const UNICODE_VERSION: &str = "14.0.0";
pub const CASEFOLD_DATA_SHA256: &str =
    "a566cd48687b2cd897e02501118b2413c14ae86d318f9abbbba97feb84189f0f";
pub const CONTRACT_WHITESPACE_SCALARS: usize = 29;

const CASE_FOLDING: &[u8] = include_bytes!("../../data/unicode-14.0.0/CaseFolding.txt");
const WHITESPACE: [char; CONTRACT_WHITESPACE_SCALARS] = [
    '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{001C}', '\u{001D}', '\u{001E}',
    '\u{001F}', '\u{0020}', '\u{0085}', '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}',
    '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}',
    '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}', '\u{3000}',
];

static FOLDING_TABLE: OnceLock<BTreeMap<char, Box<[char]>>> = OnceLock::new();

pub fn casefold_data_sha256() -> String {
    let digest = Sha256::digest(CASE_FOLDING);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

pub fn is_whitespace(value: char) -> bool {
    WHITESPACE.binary_search(&value).is_ok()
}

pub fn lstrip(value: &str) -> &str {
    value.trim_start_matches(is_whitespace)
}

pub fn strip(value: &str) -> &str {
    value.trim_matches(is_whitespace)
}

pub fn split_whitespace(value: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (index, scalar) in value.char_indices() {
        if is_whitespace(scalar) {
            if let Some(token_start) = start.take() {
                tokens.push(&value[token_start..index]);
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(token_start) = start {
        tokens.push(&value[token_start..]);
    }
    tokens
}

pub fn case_fold_scalar(value: char) -> Vec<char> {
    folding_table()
        .get(&value)
        .map_or_else(|| vec![value], |mapped| mapped.to_vec())
}

pub fn case_fold(value: &str) -> String {
    value.chars().flat_map(case_fold_scalar).collect()
}

pub fn folding_status_counts() -> (usize, usize, usize, usize) {
    let text = std::str::from_utf8(CASE_FOLDING).expect("pinned CaseFolding data is UTF-8");
    let mut counts = (0, 0, 0, 0);
    for line in text.lines() {
        let fields: Vec<_> = line.split('#').next().unwrap_or("").split(';').collect();
        if fields.len() < 3 {
            continue;
        }
        match fields[1].trim() {
            "C" => counts.0 += 1,
            "F" => counts.1 += 1,
            "S" => counts.2 += 1,
            "T" => counts.3 += 1,
            _ => {}
        }
    }
    counts
}

fn folding_table() -> &'static BTreeMap<char, Box<[char]>> {
    FOLDING_TABLE.get_or_init(|| {
        parse_folding_table().expect("pinned Unicode-14 CaseFolding data must remain valid")
    })
}

fn parse_folding_table() -> Result<BTreeMap<char, Box<[char]>>, TerminologyError> {
    if casefold_data_sha256() != CASEFOLD_DATA_SHA256 {
        return Err(failure("Unicode-14 CaseFolding data hash mismatch"));
    }
    let text = std::str::from_utf8(CASE_FOLDING)
        .map_err(|_| failure("Unicode-14 CaseFolding data is not UTF-8"))?;
    if text.lines().next() != Some("# CaseFolding-14.0.0.txt") {
        return Err(failure("Unicode-14 CaseFolding data header mismatch"));
    }
    let mut table = BTreeMap::new();
    for line in text.lines() {
        let fields: Vec<_> = line.split('#').next().unwrap_or("").split(';').collect();
        if fields.len() < 3 || !matches!(fields[1].trim(), "C" | "F") {
            continue;
        }
        let source = parse_scalar(fields[0].trim())?;
        let mapping = fields[2]
            .split_ascii_whitespace()
            .map(parse_scalar)
            .collect::<Result<Vec<_>, _>>()?;
        if mapping.is_empty() || table.insert(source, mapping.into_boxed_slice()).is_some() {
            return Err(failure("invalid Unicode-14 C/F mapping"));
        }
    }
    Ok(table)
}

fn parse_scalar(value: &str) -> Result<char, TerminologyError> {
    let scalar = u32::from_str_radix(value, 16)
        .map_err(|_| failure("invalid Unicode scalar in CaseFolding data"))?;
    char::from_u32(scalar).ok_or_else(|| failure("non-scalar in CaseFolding data"))
}
