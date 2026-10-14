use std::collections::{BTreeMap, BTreeSet};

use super::{
    ConsumerPolicy, DiagnosticCategory, MaintainedOccurrence, Occurrence, OccurrenceSignature,
    RegistryEntry, TerminologyDiagnostic, TerminologyError, validate_allowed_roles,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryStatus {
    pub pass: bool,
    pub diagnostics: Vec<TerminologyDiagnostic>,
}

pub fn reconcile_inventory<P: ConsumerPolicy + ?Sized>(
    entries: &[RegistryEntry],
    discovered: &[Occurrence],
    maintained: &[MaintainedOccurrence],
    policy: &P,
) -> Result<InventoryStatus, TerminologyError> {
    validate_allowed_roles(policy)?;
    let allowed: BTreeSet<_> = policy.allowed_roles().iter().map(String::as_str).collect();
    let known: BTreeSet<_> = entries.iter().map(|entry| entry.key.as_str()).collect();
    let mut diagnostics = Vec::new();

    for occurrence in maintained {
        if occurrence.concepts.is_empty() || occurrence.concepts.iter().any(String::is_empty) {
            push(&mut diagnostics, DiagnosticCategory::InventoryShape);
        }
        if occurrence
            .concepts
            .iter()
            .any(|concept| !known.contains(concept.as_str()))
        {
            push(&mut diagnostics, DiagnosticCategory::UnknownConcept);
        }
        if occurrence.role.is_empty() || !allowed.contains(occurrence.role.as_str()) {
            push(&mut diagnostics, DiagnosticCategory::InvalidRole);
        }
        if !valid_fingerprint(&occurrence.fingerprint) {
            push(&mut diagnostics, DiagnosticCategory::MalformedFingerprint);
        }
    }

    let actual_counts = signature_counts(discovered.iter().map(Occurrence::signature));
    let maintained_counts =
        signature_counts(maintained.iter().map(MaintainedOccurrence::signature));
    for (signature, count) in &actual_counts {
        if maintained_counts.get(signature).copied().unwrap_or(0) < *count {
            push(&mut diagnostics, DiagnosticCategory::UnreviewedOccurrence);
        }
    }
    for (signature, count) in &maintained_counts {
        if actual_counts.get(signature).copied().unwrap_or(0) < *count {
            push(&mut diagnostics, DiagnosticCategory::StaleOccurrence);
        }
    }

    let mut expected_roles: BTreeMap<OccurrenceSignature, BTreeMap<String, usize>> =
        BTreeMap::new();
    for occurrence in discovered {
        let role = policy.role_for(occurrence)?;
        if role.is_empty() || !allowed.contains(role.as_str()) {
            push(&mut diagnostics, DiagnosticCategory::InvalidRole);
        }
        *expected_roles
            .entry(occurrence.signature())
            .or_default()
            .entry(role)
            .or_default() += 1;
    }
    let mut maintained_roles: BTreeMap<OccurrenceSignature, BTreeMap<String, usize>> =
        BTreeMap::new();
    for occurrence in maintained {
        *maintained_roles
            .entry(occurrence.signature())
            .or_default()
            .entry(occurrence.role.clone())
            .or_default() += 1;
    }
    if expected_roles != maintained_roles {
        push(&mut diagnostics, DiagnosticCategory::RoleMismatch);
    }

    Ok(InventoryStatus {
        pass: diagnostics.is_empty(),
        diagnostics,
    })
}

fn signature_counts(
    signatures: impl Iterator<Item = OccurrenceSignature>,
) -> BTreeMap<OccurrenceSignature, usize> {
    let mut counts = BTreeMap::new();
    for signature in signatures {
        *counts.entry(signature).or_default() += 1;
    }
    counts
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn push(diagnostics: &mut Vec<TerminologyDiagnostic>, category: DiagnosticCategory) {
    diagnostics.push(TerminologyDiagnostic::new(category));
}
