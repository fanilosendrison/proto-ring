#![forbid(unsafe_code)]

#[rustfmt::skip]
mod projection {
use std::collections::BTreeMap;
use proto_ring_engine::normative_terminology as terminology;
use terminology::{ConsumerPolicy, DiagnosticCategory, MaintainedOccurrence, MarkdownBlock,
    Occurrence, RegistryEntry, RegistryFormat, TerminologyError};
use crate::fixture::{self, MaterializedFixture};
use crate::model::{Observation, RecordEntry, TransportValue};
use crate::rust_candidate::transport::{exact_record, fixture_failure, record, rejection, result,
    string_value, unique_dynamic_record};

type R<T> = Result<T, ()>;
type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;

pub(crate) fn execute(responsibility: &str, fixture: &MaterializedFixture)
    -> Result<Observation, fixture::FixtureError>
{
    let arguments = fixture.arguments();
    let operation = fixture::string(fixture::record_field(arguments, "operation")?)?;
    let projected = match operation {
        "definition_concepts" => definition(arguments),
        "normalize_text" | "fingerprint_text" => fingerprint(operation, arguments),
        "parse_registry" | "parse_registry_diagnostics" => registry(operation, arguments),
        "parse_blocks" => blocks(arguments),
        "discover_occurrence" | "discover_error_categories" | "discover_location_summary" => discovery(operation, arguments),
        "occurrence_signatures" => signatures(arguments),
        "reconcile_inventory" => inventory(arguments),
        _ => return Err(fixture_failure("unsupported terminology operation")),
    };
    Ok(projected.unwrap_or_else(|_| rejection(responsibility)))
}

fn definition(value: &TransportValue) -> R<Observation> {
    let fields = exact(value, &["expressions", "operation", "text"])?;
    let expressions = dynamic(get(&fields, "expressions")?)?.into_iter()
        .map(|(expression, concept)| Ok((expression.to_owned(), text(concept)?.to_owned())))
        .collect::<R<Vec<_>>>()?;
    let concepts = terminology::definition_concepts(text(get(&fields, "text")?)?, &expressions);
    Ok(result(record(vec![("concepts", strings(&concepts))])))
}

fn fingerprint(operation: &str, value: &TransportValue) -> R<Observation> {
    let fields = exact(value, &["operation", "text"])?;
    let source = text(get(&fields, "text")?)?;
    let (name, projected) = if operation == "normalize_text" {
        ("text", terminology::normalize_text(source))
    } else { ("sha256", terminology::fingerprint_text(source)) };
    Ok(result(record(vec![(name, string_value(&projected))])))
}

fn registry(operation: &str, value: &TransportValue) -> R<Observation> {
    let fields = dynamic(value)?;
    let format = parse_format(get(&fields, "format")?)?;
    if operation == "parse_registry" {
        names(&fields, &["format", "markdown", "operation"])?;
        let parsed = terminology::parse_registry(text(get(&fields, "markdown")?)?, &format);
        if !parsed.diagnostics.is_empty() { return Err(()); }
        return Ok(result(record(vec![("entries", entries_transport(&parsed.entries)),
            ("errors", TransportValue::Sequence(Vec::new()))])));
    }
    let documents = if fields.contains_key("markdown") {
        names(&fields, &["format", "markdown", "operation"])?;
        vec![text(get(&fields, "markdown")?)?.to_owned()]
    } else {
        names(&fields, &["documents", "format", "operation"])?;
        text_list(get(&fields, "documents")?)?
    };
    let variants = documents.iter().map(|markdown| {
        let parsed = terminology::parse_registry(markdown, &format);
        record(vec![("categories", strings(&diagnostic_categories(&parsed.diagnostics))),
            ("entries", TransportValue::Integer(parsed.entries.len().to_string()))])
    }).collect();
    Ok(result(record(vec![("variants", TransportValue::Sequence(variants))])))
}

fn blocks(value: &TransportValue) -> R<Observation> {
    let fields = exact(value, &["format", "markdown", "operation", "policy"])?;
    let format = parse_format(get(&fields, "format")?)?;
    let policy = parse_policy(get(&fields, "policy")?)?;
    let markdown = text(get(&fields, "markdown")?)?;
    let registry = terminology::parse_registry(markdown, &format);
    let parsed = terminology::parse_markdown(markdown, &format, &policy, Some(&registry)).map_err(drop_term)?;
    let anchors = TransportValue::Record(parsed.anchors.iter().map(|(destination, occurrences)| RecordEntry {
        name: destination.clone(),
        value: TransportValue::Sequence(occurrences.iter().map(|anchor| TransportValue::Sequence(vec![
            TransportValue::Integer(anchor.line.to_string()), string_value(&anchor.section),
            string_value(&anchor.heading)])).collect()),
    }).collect());
    let blocks = TransportValue::Sequence(parsed.blocks.iter().map(|block| record(vec![
        ("heading", string_value(&block.heading)),
        ("line", TransportValue::Integer(block.line.to_string())),
        ("section", string_value(&block.section)), ("text", string_value(&block.text)),
    ])).collect());
    Ok(result(record(vec![("anchors", anchors), ("blocks", blocks)])))
}

fn discovery(operation: &str, value: &TransportValue) -> R<Observation> {
    let fields = dynamic(value)?;
    let expected = match operation {
        "discover_occurrence" => &["format", "markdown", "operation", "policy", "query"][..],
        "discover_error_categories" => &["category_filter", "format", "markdown", "operation", "policy"][..],
        _ => &["format", "markdown", "operation", "policy"][..],
    };
    names(&fields, expected)?;
    let format = parse_format(get(&fields, "format")?)?;
    let policy = parse_policy(get(&fields, "policy")?)?;
    let report = terminology::discover_occurrences(text(get(&fields, "markdown")?)?, &format, &policy)
        .map_err(drop_term)?;
    if operation == "discover_error_categories" {
        let category = text(get(&fields, "category_filter")?)?;
        let present = report.diagnostics.iter().any(|error| category_name(error.category) == category);
        return Ok(result(record(vec![("category", string_value(category)),
            ("present", TransportValue::Boolean(present))])));
    }
    if operation == "discover_location_summary" {
        let canonical: Vec<_> = report.occurrences.iter().filter(|item| item.canonical).collect();
        let sections: Vec<_> = canonical.iter().map(|item| item.section.clone()).collect();
        return Ok(result(record(vec![
            ("canonical_count", TransportValue::Integer(canonical.len().to_string())),
            ("canonical_sections", strings(&sections)),
            ("errors", errors_transport(&report.diagnostics)),
        ])));
    }
    let query = parse_query(get(&fields, "query")?)?;
    let matched: Vec<_> = report.occurrences.iter().filter(|item| item.canonical == query.0
        && item.concepts == query.1 && item.section == query.2).collect();
    let occurrence = matched.first().map_or(TransportValue::Null(()), |item| occurrence_transport(item));
    Ok(result(record(vec![
        ("count", TransportValue::Integer(matched.len().to_string())),
        ("errors", errors_transport(&report.diagnostics)), ("occurrence", occurrence),
    ])))
}

fn signatures(value: &TransportValue) -> R<Observation> {
    let fields = exact(value, &["format", "occurrences", "operation"])?;
    let _format = parse_format(get(&fields, "format")?)?;
    let occurrences = parse_occurrences(get(&fields, "occurrences")?)?;
    let signatures: Vec<_> = occurrences.iter().map(Occurrence::signature).collect();
    let projected = signatures.iter().map(|item| TransportValue::Sequence(vec![
        strings(&item.concepts), string_value(&item.section), string_value(&item.heading),
        string_value(&item.fingerprint)])).collect();
    Ok(result(record(vec![
        ("a_equals_b", TransportValue::Boolean(signatures.first() == signatures.get(1))),
        ("a_equals_c", TransportValue::Boolean(signatures.first() == signatures.get(2))),
        ("signatures", TransportValue::Sequence(projected)),
    ])))
}

fn inventory(value: &TransportValue) -> R<Observation> {
    let fields = exact(value, &["allowed_roles", "entries", "format", "inventory", "occurrences",
        "operation", "policy", "role"])?;
    let _format = parse_format(get(&fields, "format")?)?;
    let mut policy = parse_policy(get(&fields, "policy")?)?;
    let allowed = text_list(get(&fields, "allowed_roles")?)?;
    if policy.allowed_roles != allowed { return Err(()); }
    policy.allowed_roles = allowed;
    let _role = text(get(&fields, "role")?)?;
    let status = terminology::reconcile_inventory(
        &parse_entries(get(&fields, "entries")?)?,
        &parse_occurrences(get(&fields, "occurrences")?)?,
        &parse_maintained(get(&fields, "inventory")?)?, &policy,
    ).map_err(drop_term)?;
    Ok(result(record(vec![("status", string_value(if status.pass { "PASS" } else { "NON_PASS" }))])))
}

#[derive(Clone)]
struct FixturePolicy {
    allowed_roles: Vec<String>, section_kind: String, canonical_kind: String,
    section_prefix: String, canonical_role: String,
    noncanonical: BTreeMap<String, String>, other_role: String,
}

impl ConsumerPolicy for FixturePolicy {
    fn allowed_roles(&self) -> &[String] { &self.allowed_roles }
    fn section_from_heading(&self, heading: &str) -> Result<String, TerminologyError> {
        match self.section_kind.as_str() {
            "identity" => Ok(heading.to_owned()),
            "numbered_dotted_prefix" => Ok(numbered_prefix(heading)),
            _ => Err(TerminologyError::invalid_policy("unsupported section policy")),
        }
    }
    fn canonical_location(&self, entry: &RegistryEntry, block: &MarkdownBlock)
        -> Result<bool, TerminologyError>
    {
        match self.canonical_kind.as_str() {
            "section_prefix" => Ok(block.section.starts_with(&self.section_prefix)),
            "anchor_matches_key" => Ok(entry.anchor == format!("term-{}", entry.key)),
            _ => Err(TerminologyError::invalid_policy("unsupported canonical-location policy")),
        }
    }
    fn role_for(&self, occurrence: &Occurrence) -> Result<String, TerminologyError> {
        if occurrence.canonical { return Ok(self.canonical_role.clone()); }
        Ok(self.noncanonical.iter().filter(|(prefix, _)| occurrence.section.starts_with(prefix.as_str()))
            .max_by_key(|(prefix, _)| prefix.len()).map_or_else(
                || self.other_role.clone(), |(_, role)| role.clone()))
    }
}

fn parse_format(value: &TransportValue) -> R<RegistryFormat> {
    let fields = exact(value, &["anchor_link_pattern", "anchor_pattern", "end_marker", "headers",
        "key_pattern", "start_marker"])?;
    RegistryFormat::new(
        text(get(&fields, "start_marker")?)?.to_owned(),
        text(get(&fields, "end_marker")?)?.to_owned(), text_list(get(&fields, "headers")?)?,
        text(get(&fields, "key_pattern")?)?.to_owned(),
        text(get(&fields, "anchor_link_pattern")?)?.to_owned(),
        text(get(&fields, "anchor_pattern")?)?.to_owned(),
    ).map_err(drop_term)
}

fn parse_policy(value: &TransportValue) -> R<FixturePolicy> {
    let fields = exact(value, &["allowed_roles", "canonical_location", "role_policy", "section_from_heading"])?;
    let location = dynamic(get(&fields, "canonical_location")?)?;
    let canonical_kind = text(get(&location, "kind")?)?.to_owned();
    match canonical_kind.as_str() {
        "section_prefix" => names(&location, &["kind", "section_prefix"] )?,
        "anchor_matches_key" => names(&location, &["kind"] )?,
        _ => return Err(()),
    }
    let section_prefix = location.get("section_prefix").map(|value| text(value).map(str::to_owned))
        .transpose()?.unwrap_or_default();
    let section = exact(get(&fields, "section_from_heading")?, &["kind"])?;
    let roles = exact(get(&fields, "role_policy")?,
        &["canonical", "noncanonical_section_prefixes", "other_noncanonical"])?;
    let noncanonical = dynamic(get(&roles, "noncanonical_section_prefixes")?)?.into_iter()
        .map(|(prefix, role)| Ok((prefix.to_owned(), text(role)?.to_owned()))).collect::<R<_>>()?;
    Ok(FixturePolicy {
        allowed_roles: text_list(get(&fields, "allowed_roles")?)?,
        section_kind: text(get(&section, "kind")?)?.to_owned(), canonical_kind, section_prefix,
        canonical_role: text(get(&roles, "canonical")?)?.to_owned(), noncanonical,
        other_role: text(get(&roles, "other_noncanonical")?)?.to_owned(),
    })
}

fn parse_entries(value: &TransportValue) -> R<Vec<RegistryEntry>> {
    sequence(value)?.iter().map(|item| {
        let fields = exact(item, &["aliases", "anchor", "canonical_term", "deprecated", "key", "term_structure"])?;
        Ok(RegistryEntry {
            key: text(get(&fields, "key")?)?.to_owned(),
            canonical_term: text(get(&fields, "canonical_term")?)?.to_owned(),
            anchor: text(get(&fields, "anchor")?)?.to_owned(),
            aliases: text_list(get(&fields, "aliases")?)?, deprecated: text_list(get(&fields, "deprecated")?)?,
            term_structure: text(get(&fields, "term_structure")?)?.to_owned(),
        })
    }).collect()
}

fn parse_occurrences(value: &TransportValue) -> R<Vec<Occurrence>> {
    sequence(value)?.iter().map(|item| {
        let fields = exact(item, &["canonical", "concepts", "fingerprint", "heading", "line", "section"])?;
        let TransportValue::Boolean(canonical) = get(&fields, "canonical")? else { return Err(()); };
        let line = integer(get(&fields, "line")?)?.parse().map_err(|_| ())?;
        Ok(Occurrence::new(text_list(get(&fields, "concepts")?)?,
            text(get(&fields, "section")?)?.to_owned(), text(get(&fields, "heading")?)?.to_owned(),
            text(get(&fields, "fingerprint")?)?.to_owned(), line, *canonical))
    }).collect()
}

fn parse_maintained(value: &TransportValue) -> R<Vec<MaintainedOccurrence>> {
    sequence(value)?.iter().map(|item| {
        let fields = exact(item, &["concepts", "fingerprint", "heading", "role", "section"])?;
        Ok(MaintainedOccurrence {
            concepts: text_list(get(&fields, "concepts")?)?, section: text(get(&fields, "section")?)?.to_owned(),
            heading: text(get(&fields, "heading")?)?.to_owned(), role: text(get(&fields, "role")?)?.to_owned(),
            fingerprint: text(get(&fields, "fingerprint")?)?.to_owned(),
        })
    }).collect()
}

fn parse_query(value: &TransportValue) -> R<(bool, Vec<String>, String)> {
    let fields = exact(value, &["canonical", "concepts", "section"])?;
    let TransportValue::Boolean(canonical) = get(&fields, "canonical")? else { return Err(()); };
    Ok((*canonical, text_list(get(&fields, "concepts")?)?, text(get(&fields, "section")?)?.to_owned()))
}

fn entries_transport(entries: &[RegistryEntry]) -> TransportValue {
    TransportValue::Sequence(entries.iter().map(|entry| record(vec![
        ("aliases", strings(&entry.aliases)), ("anchor", string_value(&entry.anchor)),
        ("canonical_term", string_value(&entry.canonical_term)), ("deprecated", strings(&entry.deprecated)),
        ("key", string_value(&entry.key)), ("term_structure", string_value(&entry.term_structure)),
    ])).collect())
}

fn occurrence_transport(item: &Occurrence) -> TransportValue {
    record(vec![("canonical", TransportValue::Boolean(item.canonical)), ("concepts", strings(&item.concepts)),
        ("fingerprint", string_value(&item.fingerprint)), ("heading", string_value(&item.heading)),
        ("section", string_value(&item.section))])
}

fn errors_transport(errors: &[terminology::TerminologyDiagnostic]) -> TransportValue {
    strings(&diagnostic_categories(errors))
}
fn diagnostic_categories(errors: &[terminology::TerminologyDiagnostic]) -> Vec<String> {
    let mut values: Vec<_> = errors.iter().map(|error| category_name(error.category).to_owned()).collect();
    values.sort(); values.dedup(); values
}
fn category_name(category: DiagnosticCategory) -> &'static str {
    match category {
        DiagnosticCategory::RegistryMarkerCardinality => "registry_marker_cardinality",
        DiagnosticCategory::EmptyCanonicalDestination => "empty_canonical_destination",
        DiagnosticCategory::EmptyCanonicalTerm => "empty_canonical_term",
        DiagnosticCategory::DuplicateRegisteredAnchor => "duplicate_registered_anchor",
        DiagnosticCategory::UnregisteredCanonicalAnchor => "unregistered_canonical_anchor",
        DiagnosticCategory::InterveningCanonicalAnchor => "intervening_canonical_anchor",
        _ => "other_controlled_error",
    }
}

fn numbered_prefix(heading: &str) -> String {
    let bytes = heading.as_bytes();
    let mut end = bytes.iter().take_while(|byte| byte.is_ascii_digit()).count();
    if end == 0 { return String::new(); }
    loop {
        let dot = end;
        if bytes.get(dot) != Some(&b'.') { break; }
        let digits = bytes[dot + 1..].iter().take_while(|byte| byte.is_ascii_digit()).count();
        if digits == 0 { break; }
        end = dot + 1 + digits;
    }
    heading[..end].to_owned()
}

fn exact<'a>(value: &'a TransportValue, expected: &[&str]) -> R<Fields<'a>> {
    exact_record(value, "terminology value", expected).map_err(|_| ())
}
fn dynamic(value: &TransportValue) -> R<Fields<'_>> {
    Ok(unique_dynamic_record(value, "terminology value").map_err(|_| ())?.into_iter().collect())
}
fn names(fields: &Fields<'_>, expected: &[&str]) -> R<()> {
    (fields.len() == expected.len() && expected.iter().all(|name| fields.contains_key(name))).then_some(()).ok_or(())
}
fn get<'a>(fields: &'a Fields<'a>, name: &str) -> R<&'a TransportValue> { fields.get(name).copied().ok_or(()) }
fn text(value: &TransportValue) -> R<&str> { fixture::string(value).map_err(|_| ()) }
fn sequence(value: &TransportValue) -> R<&[TransportValue]> { fixture::sequence(value).map_err(|_| ()) }
fn integer(value: &TransportValue) -> R<&str> {
    match value { TransportValue::Integer(value) => Ok(value), _ => Err(()) }
}
fn text_list(value: &TransportValue) -> R<Vec<String>> {
    sequence(value)?.iter().map(|item| text(item).map(str::to_owned)).collect()
}
fn strings(values: &[String]) -> TransportValue {
    TransportValue::Sequence(values.iter().map(|value| string_value(value)).collect())
}
fn drop_term(_error: TerminologyError) {}
}

pub(super) use projection::execute;
