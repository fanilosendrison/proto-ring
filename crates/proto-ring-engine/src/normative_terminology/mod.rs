#![forbid(unsafe_code)]

mod inventory;
mod markdown;
mod matching;
mod registry;
pub mod unicode14;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::portable_pattern::PortablePattern;

pub(crate) const PRESENTATION_DELIMITERS: [char; 10] =
    ['`', '*', '_', '~', '\'', '"', '‘', '’', '“', '”'];

pub use inventory::{InventoryStatus, reconcile_inventory};
pub use markdown::{discover_occurrences, parse_markdown};
pub use matching::{definition_concepts, fingerprint_text, normalize_text};
pub use registry::parse_registry;

pub struct RegistryFormat {
    pub start_marker: String,
    pub end_marker: String,
    pub headers: Vec<String>,
    pub key_pattern_source: String,
    pub anchor_link_pattern_source: String,
    pub anchor_pattern_source: String,
    pub(crate) key_pattern: PortablePattern,
    pub(crate) anchor_link_pattern: PortablePattern,
    pub(crate) anchor_pattern: PortablePattern,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryEntry {
    pub key: String,
    pub canonical_term: String,
    pub anchor: String,
    pub aliases: Vec<String>,
    pub deprecated: Vec<String>,
    pub term_structure: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticCategory {
    RegistryMarkerCardinality,
    RegistryStructure,
    EmptyCanonicalDestination,
    EmptyCanonicalTerm,
    DuplicateRegisteredAnchor,
    UnregisteredCanonicalAnchor,
    InterveningCanonicalAnchor,
    MissingCanonicalAnchor,
    CanonicalSectionMismatch,
    CanonicalLocationRejected,
    MissingCanonicalDefinition,
    InventoryShape,
    UnknownConcept,
    InvalidRole,
    MalformedFingerprint,
    UnreviewedOccurrence,
    StaleOccurrence,
    RoleMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminologyDiagnostic {
    pub category: DiagnosticCategory,
}

impl TerminologyDiagnostic {
    pub(crate) fn new(category: DiagnosticCategory) -> Self {
        Self { category }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryParse {
    pub entries: Vec<RegistryEntry>,
    pub diagnostics: Vec<TerminologyDiagnostic>,
    pub(crate) marker_range: Option<(usize, usize)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownBlock {
    pub line: usize,
    pub section: String,
    pub heading: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalAnchor {
    pub line: usize,
    pub section: String,
    pub heading: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownParse {
    pub blocks: Vec<MarkdownBlock>,
    pub anchors: BTreeMap<String, Vec<CanonicalAnchor>>,
    pub diagnostics: Vec<TerminologyDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Occurrence {
    pub concepts: Vec<String>,
    pub section: String,
    pub heading: String,
    pub fingerprint: String,
    pub line: usize,
    pub canonical: bool,
}

impl Occurrence {
    pub fn new(
        mut concepts: Vec<String>,
        section: String,
        heading: String,
        fingerprint: String,
        line: usize,
        canonical: bool,
    ) -> Self {
        concepts.sort();
        Self {
            concepts,
            section,
            heading,
            fingerprint,
            line,
            canonical,
        }
    }

    pub fn signature(&self) -> OccurrenceSignature {
        OccurrenceSignature {
            concepts: self.concepts.clone(),
            section: self.section.clone(),
            heading: self.heading.clone(),
            fingerprint: self.fingerprint.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OccurrenceSignature {
    pub concepts: Vec<String>,
    pub section: String,
    pub heading: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintainedOccurrence {
    pub concepts: Vec<String>,
    pub section: String,
    pub heading: String,
    pub role: String,
    pub fingerprint: String,
}

impl MaintainedOccurrence {
    pub fn signature(&self) -> OccurrenceSignature {
        let mut concepts = self.concepts.clone();
        concepts.sort();
        OccurrenceSignature {
            concepts,
            section: self.section.clone(),
            heading: self.heading.clone(),
            fingerprint: self.fingerprint.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Discovery {
    pub occurrences: Vec<Occurrence>,
    pub diagnostics: Vec<TerminologyDiagnostic>,
}

pub trait ConsumerPolicy {
    fn allowed_roles(&self) -> &[String];

    fn section_from_heading(&self, heading: &str) -> Result<String, TerminologyError>;

    fn canonical_location(
        &self,
        entry: &RegistryEntry,
        block: &MarkdownBlock,
    ) -> Result<bool, TerminologyError>;

    fn role_for(&self, occurrence: &Occurrence) -> Result<String, TerminologyError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminologyError(String);

impl Display for TerminologyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for TerminologyError {}

impl TerminologyError {
    pub fn invalid_policy(message: impl Into<String>) -> Self {
        failure(message)
    }
}

pub(crate) fn failure(message: impl Into<String>) -> TerminologyError {
    TerminologyError(message.into())
}

pub(crate) fn validate_allowed_roles<P: ConsumerPolicy + ?Sized>(
    policy: &P,
) -> Result<(), TerminologyError> {
    let roles = policy.allowed_roles();
    if roles.is_empty() || roles.iter().any(String::is_empty) {
        return Err(failure("allowed roles must be nonempty strings"));
    }
    let mut unique = roles.to_vec();
    unique.sort();
    unique.dedup();
    if unique.len() != roles.len() {
        return Err(failure("allowed roles must be unique"));
    }
    Ok(())
}
