use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::InvalidRequest;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryFormatInput {
    pub start_marker: String,
    pub end_marker: String,
    pub headers: Vec<String>,
    pub key_pattern: String,
    pub anchor_link_pattern: String,
    pub anchor_pattern: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConsumerPolicyInput {
    pub allowed_roles: Vec<String>,
    pub section_from_heading: SectionFromHeadingInput,
    pub canonical_location: CanonicalLocationInput,
    pub role_policy: RolePolicyInput,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SectionFromHeadingInput {
    Identity,
    NumberedDottedPrefix,
    NumberedDottedPrefixOptionalUppercaseWordBoundary,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum CanonicalLocationInput {
    AnchorMatchesKey,
    SectionPrefix { prefix: String },
    SectionHierarchy { root: String },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RolePolicyInput {
    pub canonical: String,
    pub noncanonical_rules: Vec<RoleRuleInput>,
    pub other_noncanonical: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleRuleInput {
    pub matcher: RoleMatcherInput,
    pub role: String,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RoleMatcherInput {
    SectionPrefix { value: String },
    SectionHierarchy { value: String },
}

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum FingerprintInput {
    NormalizeText { text: String },
    FingerprintText { text: String },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryInput {
    pub markdown: String,
    pub format: RegistryFormatInput,
}

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum MarkdownInput {
    ParseBlocks {
        markdown: String,
        format: RegistryFormatInput,
        policy: ConsumerPolicyInput,
    },
    DiscoverOccurrences {
        markdown: String,
        format: RegistryFormatInput,
        policy: ConsumerPolicyInput,
    },
}

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum InventoryInput {
    OccurrenceRecords {
        occurrences: Vec<OccurrenceInput>,
        policy: ConsumerPolicyInput,
    },
    ReconcileInventory {
        entries: Vec<RegistryEntryInput>,
        occurrences: Vec<OccurrenceInput>,
        inventory: Vec<MaintainedOccurrenceInput>,
        policy: ConsumerPolicyInput,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryEntryInput {
    pub key: String,
    pub canonical_term: String,
    pub anchor: String,
    pub aliases: Vec<String>,
    pub deprecated: Vec<String>,
    pub term_structure: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OccurrenceInput {
    pub concepts: Vec<String>,
    pub section: String,
    pub heading: String,
    pub fingerprint: String,
    pub line: usize,
    pub canonical: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MaintainedOccurrenceInput {
    pub concepts: Vec<String>,
    pub section: String,
    pub heading: String,
    pub role: String,
    pub fingerprint: String,
}

pub(crate) enum TerminologyRequest {
    Fingerprint(FingerprintInput),
    Registry(RegistryInput),
    Markdown(MarkdownInput),
    Inventory(InventoryInput),
}

pub(crate) fn parse_operation(
    operation: &str,
    input: Value,
) -> Result<TerminologyRequest, InvalidRequest> {
    match operation {
        "normative-terminology.fingerprint" => decode(input).map(TerminologyRequest::Fingerprint),
        "normative-terminology.registry" => decode(input).map(TerminologyRequest::Registry),
        "normative-terminology.markdown" => decode(input).map(TerminologyRequest::Markdown),
        "normative-terminology.inventory" => decode(input).map(TerminologyRequest::Inventory),
        _ => Err(InvalidRequest),
    }
}

fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, InvalidRequest> {
    serde_json::from_value(value).map_err(|_| InvalidRequest)
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum FingerprintValue {
    Text { text: String },
    Digest { sha256: String },
}

#[derive(Serialize)]
pub(crate) struct RegistryValue {
    pub entries: Vec<RegistryEntryValue>,
    pub diagnostics: Vec<&'static str>,
}

#[derive(Serialize)]
pub(crate) struct RegistryEntryValue {
    pub key: String,
    pub canonical_term: String,
    pub anchor: String,
    pub aliases: Vec<String>,
    pub deprecated: Vec<String>,
    pub term_structure: String,
}

#[derive(Serialize)]
pub(crate) struct MarkdownBlocksValue {
    pub blocks: Vec<MarkdownBlockValue>,
    pub anchors: BTreeMap<String, Vec<CanonicalAnchorValue>>,
    pub diagnostics: Vec<&'static str>,
}

#[derive(Serialize)]
pub(crate) struct MarkdownBlockValue {
    pub line: usize,
    pub section: String,
    pub heading: String,
    pub text: String,
}

#[derive(Serialize)]
pub(crate) struct CanonicalAnchorValue {
    pub line: usize,
    pub section: String,
    pub heading: String,
}

#[derive(Serialize)]
pub(crate) struct DiscoveryValue {
    pub occurrences: Vec<OccurrenceValue>,
    pub diagnostics: Vec<&'static str>,
}

#[derive(Serialize)]
pub(crate) struct OccurrenceValue {
    pub concepts: Vec<String>,
    pub section: String,
    pub heading: String,
    pub fingerprint: String,
    pub line: usize,
    pub canonical: bool,
}

#[derive(Serialize)]
pub(crate) struct OccurrenceRecordsValue {
    pub records: Vec<OccurrenceRecordValue>,
}

#[derive(Serialize)]
pub(crate) struct OccurrenceRecordValue {
    pub concepts: Vec<String>,
    pub section: String,
    pub heading: String,
    pub role: String,
    pub fingerprint: String,
}

#[derive(Serialize)]
pub(crate) struct InventoryStatusValue {
    pub status: &'static str,
    pub diagnostics: Vec<&'static str>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum MarkdownValue {
    Blocks(MarkdownBlocksValue),
    Discovery(DiscoveryValue),
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum InventoryValue {
    Records(OccurrenceRecordsValue),
    Status(InventoryStatusValue),
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum TerminologyValue {
    Fingerprint(FingerprintValue),
    Registry(RegistryValue),
    Markdown(MarkdownValue),
    Inventory(InventoryValue),
}
