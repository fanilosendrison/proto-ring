use std::collections::BTreeSet;

use proto_ring_engine::normative_terminology::{
    self as terminology, ConsumerPolicy, DiagnosticCategory, MaintainedOccurrence, MarkdownBlock,
    Occurrence, RegistryEntry, RegistryFormat, TerminologyDiagnostic, TerminologyError,
};

use crate::protocol::*;

const FINGERPRINT_OPERATION: &str = "normative-terminology.fingerprint";
const REGISTRY_OPERATION: &str = "normative-terminology.registry";
const MARKDOWN_OPERATION: &str = "normative-terminology.markdown";
const INVENTORY_OPERATION: &str = "normative-terminology.inventory";

#[rustfmt::skip]
pub(crate) fn execute(request: TerminologyRequest) -> Response {
    match request {
        TerminologyRequest::Fingerprint(input) => fingerprint(input),
        TerminologyRequest::Registry(input) => registry(input),
        TerminologyRequest::Markdown(input) => markdown(input),
        TerminologyRequest::Inventory(input) => inventory(input),
    }
}

#[rustfmt::skip]
fn fingerprint(input: FingerprintInput) -> Response {
    let value = match input {
        FingerprintInput::NormalizeText { text } => FingerprintValue::Text { text: terminology::normalize_text(&text) },
        FingerprintInput::FingerprintText { text } => FingerprintValue::Digest { sha256: terminology::fingerprint_text(&text) },
    };
    result(FINGERPRINT_OPERATION, TerminologyValue::Fingerprint(value))
}

fn registry(input: RegistryInput) -> Response {
    let value = build_format(input.format).map(|format| {
        let parsed = terminology::parse_registry(&input.markdown, &format);
        TerminologyValue::Registry(RegistryValue {
            entries: parsed.entries.iter().map(project_entry).collect(),
            diagnostics: project_diagnostics(&parsed.diagnostics),
        })
    });
    finish(REGISTRY_OPERATION, value)
}

#[rustfmt::skip]
fn markdown(input: MarkdownInput) -> Response {
    let value = match input {
        MarkdownInput::ParseBlocks { markdown, format, policy } => parse_blocks(markdown, format, policy),
        MarkdownInput::DiscoverOccurrences { markdown, format, policy } => discover_occurrences(markdown, format, policy),
    };
    finish(MARKDOWN_OPERATION, value.map(TerminologyValue::Markdown))
}

#[rustfmt::skip]
fn parse_blocks(markdown: String, format: RegistryFormatInput, policy: ConsumerPolicyInput)
    -> Result<MarkdownValue, TerminologyError>
{
    let format = build_format(format)?; let policy = CompatTerminologyPolicy::new(policy)?;
    let registry = terminology::parse_registry(&markdown, &format);
    let parsed = terminology::parse_markdown(&markdown, &format, &policy, Some(&registry))?;
    let blocks = parsed.blocks.into_iter().map(|block| MarkdownBlockValue {
        line: block.line, section: block.section, heading: block.heading, text: block.text,
    }).collect();
    let anchors = parsed.anchors.into_iter().map(|(destination, values)| {
        let values = values.into_iter().map(|anchor| CanonicalAnchorValue {
            line: anchor.line, section: anchor.section, heading: anchor.heading,
        }).collect(); (destination, values)
    }).collect();
    Ok(MarkdownValue::Blocks(MarkdownBlocksValue {
        blocks, anchors, diagnostics: project_diagnostics(&parsed.diagnostics),
    }))
}

fn discover_occurrences(
    markdown: String,
    format: RegistryFormatInput,
    policy: ConsumerPolicyInput,
) -> Result<MarkdownValue, TerminologyError> {
    let format = build_format(format)?;
    let policy = CompatTerminologyPolicy::new(policy)?;
    let discovery = terminology::discover_occurrences(&markdown, &format, &policy)?;
    Ok(MarkdownValue::Discovery(DiscoveryValue {
        occurrences: discovery
            .occurrences
            .iter()
            .map(project_occurrence)
            .collect(),
        diagnostics: project_diagnostics(&discovery.diagnostics),
    }))
}

#[rustfmt::skip]
fn inventory(input: InventoryInput) -> Response {
    let value = match input {
        InventoryInput::OccurrenceRecords { occurrences, policy } => occurrence_records(occurrences, policy),
        InventoryInput::ReconcileInventory { entries, occurrences, inventory, policy } => reconcile(entries, occurrences, inventory, policy),
    };
    finish(INVENTORY_OPERATION, value.map(TerminologyValue::Inventory))
}

fn occurrence_records(
    occurrences: Vec<OccurrenceInput>,
    policy: ConsumerPolicyInput,
) -> Result<InventoryValue, TerminologyError> {
    let policy = CompatTerminologyPolicy::new(policy)?;
    let mut records = Vec::with_capacity(occurrences.len());
    for input in occurrences {
        let occurrence = build_occurrence(input);
        let role = policy.role_for(&occurrence)?;
        if role.is_empty() || !policy.allowed_roles().contains(&role) {
            return Err(TerminologyError::invalid_policy(
                "resolved role is not allowed",
            ));
        }
        records.push(OccurrenceRecordValue {
            concepts: occurrence.concepts,
            section: occurrence.section,
            heading: occurrence.heading,
            role,
            fingerprint: occurrence.fingerprint,
        });
    }
    Ok(InventoryValue::Records(OccurrenceRecordsValue { records }))
}

fn reconcile(
    entries: Vec<RegistryEntryInput>,
    occurrences: Vec<OccurrenceInput>,
    inventory: Vec<MaintainedOccurrenceInput>,
    policy: ConsumerPolicyInput,
) -> Result<InventoryValue, TerminologyError> {
    let policy = CompatTerminologyPolicy::new(policy)?;
    let entries: Vec<_> = entries.into_iter().map(build_entry).collect();
    let occurrences: Vec<_> = occurrences.into_iter().map(build_occurrence).collect();
    let maintained: Vec<_> = inventory.into_iter().map(build_maintained).collect();
    let status = terminology::reconcile_inventory(&entries, &occurrences, &maintained, &policy)?;
    Ok(InventoryValue::Status(InventoryStatusValue {
        status: if status.pass { "PASS" } else { "NON_PASS" },
        diagnostics: project_diagnostics(&status.diagnostics),
    }))
}

#[rustfmt::skip]
fn build_format(input: RegistryFormatInput) -> Result<RegistryFormat, TerminologyError> {
    RegistryFormat::new(input.start_marker, input.end_marker, input.headers,
        input.key_pattern, input.anchor_link_pattern, input.anchor_pattern)
}
#[rustfmt::skip]
fn build_entry(input: RegistryEntryInput) -> RegistryEntry { RegistryEntry {
    key: input.key, canonical_term: input.canonical_term, anchor: input.anchor,
    aliases: input.aliases, deprecated: input.deprecated, term_structure: input.term_structure,
} }
#[rustfmt::skip]
fn build_occurrence(input: OccurrenceInput) -> Occurrence { Occurrence::new(
    input.concepts, input.section, input.heading, input.fingerprint, input.line, input.canonical,
) }
#[rustfmt::skip]
fn build_maintained(input: MaintainedOccurrenceInput) -> MaintainedOccurrence { MaintainedOccurrence {
    concepts: input.concepts, section: input.section, heading: input.heading,
    role: input.role, fingerprint: input.fingerprint,
} }
#[rustfmt::skip]
fn project_entry(entry: &RegistryEntry) -> RegistryEntryValue { RegistryEntryValue {
    key: entry.key.clone(), canonical_term: entry.canonical_term.clone(), anchor: entry.anchor.clone(),
    aliases: entry.aliases.clone(), deprecated: entry.deprecated.clone(), term_structure: entry.term_structure.clone(),
} }
#[rustfmt::skip]
fn project_occurrence(value: &Occurrence) -> OccurrenceValue { OccurrenceValue {
    concepts: value.concepts.clone(), section: value.section.clone(), heading: value.heading.clone(),
    fingerprint: value.fingerprint.clone(), line: value.line, canonical: value.canonical,
} }
#[rustfmt::skip]
fn project_diagnostics(values: &[TerminologyDiagnostic]) -> Vec<&'static str> {
    values.iter().map(|value| category_name(value.category)).collect()
}

#[rustfmt::skip]
fn category_name(category: DiagnosticCategory) -> &'static str { match category {
    DiagnosticCategory::RegistryMarkerCardinality => "registry_marker_cardinality",
    DiagnosticCategory::RegistryStructure => "registry_structure",
    DiagnosticCategory::EmptyCanonicalDestination => "empty_canonical_destination",
    DiagnosticCategory::EmptyCanonicalTerm => "empty_canonical_term",
    DiagnosticCategory::DuplicateRegisteredAnchor => "duplicate_registered_anchor",
    DiagnosticCategory::UnregisteredCanonicalAnchor => "unregistered_canonical_anchor",
    DiagnosticCategory::InterveningCanonicalAnchor => "intervening_canonical_anchor",
    DiagnosticCategory::MissingCanonicalAnchor => "missing_canonical_anchor",
    DiagnosticCategory::CanonicalSectionMismatch => "canonical_section_mismatch",
    DiagnosticCategory::CanonicalLocationRejected => "canonical_location_rejected",
    DiagnosticCategory::MissingCanonicalDefinition => "missing_canonical_definition",
    DiagnosticCategory::InventoryShape => "inventory_shape",
    DiagnosticCategory::UnknownConcept => "unknown_concept",
    DiagnosticCategory::InvalidRole => "invalid_role",
    DiagnosticCategory::MalformedFingerprint => "malformed_fingerprint",
    DiagnosticCategory::UnreviewedOccurrence => "unreviewed_occurrence",
    DiagnosticCategory::StaleOccurrence => "stale_occurrence",
    DiagnosticCategory::RoleMismatch => "role_mismatch",
} }

fn result(operation: &'static str, value: TerminologyValue) -> Response {
    Response::result(operation, ResultValue::Terminology(value))
}
#[rustfmt::skip]
fn finish(operation: &'static str, value: Result<TerminologyValue, TerminologyError>) -> Response {
    match value {
        Ok(value) => result(operation, value),
        Err(error) => Response::controlled_failure(operation, rejection_category(operation), None, error.to_string()),
    }
}
#[rustfmt::skip]
fn rejection_category(operation: &str) -> &'static str { match operation {
    REGISTRY_OPERATION => "normative-terminology.registry.rejected",
    MARKDOWN_OPERATION => "normative-terminology.markdown.rejected",
    INVENTORY_OPERATION => "normative-terminology.inventory.rejected",
    _ => unreachable!("terminology rejection operation is closed"),
} }

struct CompatTerminologyPolicy {
    allowed_roles: Vec<String>,
    section_from_heading: SectionFromHeadingInput,
    canonical_location: CanonicalLocationInput,
    role_policy: RolePolicyInput,
}

impl CompatTerminologyPolicy {
    fn new(input: ConsumerPolicyInput) -> Result<Self, TerminologyError> {
        let mut roles = BTreeSet::new();
        if input.allowed_roles.is_empty()
            || input.allowed_roles.iter().any(String::is_empty)
            || input
                .allowed_roles
                .iter()
                .any(|role| !roles.insert(role.clone()))
        {
            return Err(TerminologyError::invalid_policy(
                "allowed roles must be unique nonempty strings",
            ));
        }
        if matches!(&input.canonical_location,
            CanonicalLocationInput::SectionHierarchy { root } if root.is_empty())
        {
            return Err(TerminologyError::invalid_policy(
                "section hierarchy root must be nonempty",
            ));
        }
        let mut matchers = BTreeSet::new();
        for rule in &input.role_policy.noncanonical_rules {
            if matches!(&rule.matcher,
                RoleMatcherInput::SectionHierarchy { value } if value.is_empty())
                || !matchers.insert(rule.matcher.identity())
            {
                return Err(TerminologyError::invalid_policy(
                    "role matchers must be valid and unique",
                ));
            }
        }
        Ok(Self {
            allowed_roles: input.allowed_roles,
            section_from_heading: input.section_from_heading,
            canonical_location: input.canonical_location,
            role_policy: input.role_policy,
        })
    }
}

impl ConsumerPolicy for CompatTerminologyPolicy {
    fn allowed_roles(&self) -> &[String] {
        &self.allowed_roles
    }
    fn section_from_heading(&self, heading: &str) -> Result<String, TerminologyError> {
        Ok(match self.section_from_heading {
            SectionFromHeadingInput::Identity => heading.to_owned(),
            SectionFromHeadingInput::NumberedDottedPrefix => numbered_prefix(heading),
            SectionFromHeadingInput::NumberedDottedPrefixOptionalUppercaseWordBoundary => {
                numbered_prefix_with_optional_uppercase_boundary(heading)
            }
        })
    }
    fn canonical_location(
        &self,
        entry: &RegistryEntry,
        block: &MarkdownBlock,
    ) -> Result<bool, TerminologyError> {
        Ok(match &self.canonical_location {
            CanonicalLocationInput::AnchorMatchesKey => {
                entry.anchor == format!("term-{}", entry.key)
            }
            CanonicalLocationInput::SectionPrefix { prefix } => block.section.starts_with(prefix),
            CanonicalLocationInput::SectionHierarchy { root } => in_hierarchy(&block.section, root),
        })
    }
    fn role_for(&self, occurrence: &Occurrence) -> Result<String, TerminologyError> {
        if occurrence.canonical {
            return Ok(self.role_policy.canonical.clone());
        }
        let mut best: Option<&RoleRuleInput> = None;
        for rule in &self.role_policy.noncanonical_rules {
            if rule.matcher.matches(&occurrence.section)
                && best.is_none_or(|current| {
                    rule.matcher.value().len() > current.matcher.value().len()
                })
            {
                best = Some(rule);
            }
        }
        Ok(best.map_or_else(
            || self.role_policy.other_noncanonical.clone(),
            |rule| rule.role.clone(),
        ))
    }
}

impl RoleMatcherInput {
    #[rustfmt::skip]
    fn identity(&self) -> (u8, String) { match self {
        Self::SectionPrefix { value } => (0, value.clone()),
        Self::SectionHierarchy { value } => (1, value.clone()),
    } }
    #[rustfmt::skip]
    fn value(&self) -> &str { match self {
        Self::SectionPrefix { value } | Self::SectionHierarchy { value } => value,
    } }
    #[rustfmt::skip]
    fn matches(&self, section: &str) -> bool { match self {
        Self::SectionPrefix { value } => section.starts_with(value),
        Self::SectionHierarchy { value } => in_hierarchy(section, value),
    } }
}

fn numbered_prefix(heading: &str) -> String {
    numbered_prefix_end(heading).map_or_else(String::new, |end| heading[..end].to_owned())
}
fn numbered_prefix_with_optional_uppercase_boundary(heading: &str) -> String {
    let Some(mut end) = numbered_prefix_end(heading) else {
        return String::new();
    };
    if heading
        .as_bytes()
        .get(end)
        .is_some_and(u8::is_ascii_uppercase)
    {
        end += 1;
    }
    if heading[end..]
        .chars()
        .next()
        .is_some_and(|scalar| scalar.is_alphanumeric() || scalar == '_')
    {
        String::new()
    } else {
        heading[..end].to_owned()
    }
}
fn numbered_prefix_end(heading: &str) -> Option<usize> {
    let bytes = heading.as_bytes();
    let mut end = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if end == 0 {
        return None;
    }
    loop {
        let dot = end;
        if bytes.get(dot) != Some(&b'.') {
            break;
        }
        let digits = bytes[dot + 1..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if digits == 0 {
            break;
        }
        end = dot + 1 + digits;
    }
    Some(end)
}
fn in_hierarchy(section: &str, root: &str) -> bool {
    section == root
        || section
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with('.'))
}
