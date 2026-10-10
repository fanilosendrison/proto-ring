use std::collections::{BTreeMap, BTreeSet};

use proto_ring_engine::exact_evidence_binding::{EvidenceBinding, EvidenceRequirement};

use crate::fixture;
use crate::model::{Observation, TransportValue};

use super::transport::{
    exact_record, fixture_failure, record, rejection, result, string_value, unique_dynamic_record,
};

type Fields<'a> = BTreeMap<&'a str, &'a TransportValue>;

pub(super) fn execute(value: &TransportValue) -> Result<Observation, fixture::FixtureError> {
    let fields = unique_dynamic_record(value, "Exact Evidence Binding invocation")?;
    let operation = fields
        .iter()
        .find(|(name, _)| *name == "operation")
        .ok_or_else(|| fixture_failure("invocation is missing operation"))?;
    match fixture::string(operation.1)? {
        "evaluate_binding" => evaluate_binding(value),
        "evaluate_requirement_history" => evaluate_history(value),
        _ => Err(fixture_failure(
            "unsupported Exact Evidence Binding operation",
        )),
    }
}

fn evaluate_binding(value: &TransportValue) -> Result<Observation, fixture::FixtureError> {
    let fields = exact_record(
        value,
        "evaluate_binding",
        &["binding", "operation", "requirement"],
    )?;
    require_operation(&fields, "evaluate_binding")?;
    let parsed_requirement = requirement(field(&fields, "requirement"))?;
    let parsed_binding = binding(field(&fields, "binding"))?;
    let Some(requirement) = parsed_requirement else {
        return Ok(rejection("exact-evidence-binding.evaluate"));
    };
    Ok(result(record(vec![(
        "status",
        string_value(requirement.evaluate(parsed_binding.as_ref()).serialized()),
    )])))
}

fn evaluate_history(value: &TransportValue) -> Result<Observation, fixture::FixtureError> {
    let fields = exact_record(
        value,
        "evaluate_requirement_history",
        &["comparisons", "operation"],
    )?;
    require_operation(&fields, "evaluate_requirement_history")?;
    let parsed_comparisons = fixture::sequence(field(&fields, "comparisons"))?
        .iter()
        .map(|comparison| {
            let item = exact_record(
                comparison,
                "historical comparison",
                &[
                    "binding",
                    "current_requirement",
                    "historical_requirement",
                    "label",
                ],
            )?;
            Ok((
                string_field(&item, "label")?.to_owned(),
                requirement(field(&item, "historical_requirement"))?,
                requirement(field(&item, "current_requirement"))?,
                binding(field(&item, "binding"))?,
            ))
        })
        .collect::<Result<Vec<_>, fixture::FixtureError>>()?;

    let mut observations = Vec::new();
    for (label, historical, current, candidate) in parsed_comparisons {
        let (Some(historical), Some(current)) = (historical, current) else {
            return Ok(rejection("exact-evidence-binding.evaluate"));
        };
        observations.push(record(vec![
            (
                "current",
                string_value(current.evaluate(candidate.as_ref()).serialized()),
            ),
            (
                "historical",
                string_value(historical.evaluate(candidate.as_ref()).serialized()),
            ),
            ("label", string_value(&label)),
        ]));
    }
    Ok(result(record(vec![(
        "comparisons",
        TransportValue::Sequence(observations),
    )])))
}

fn requirement(
    value: &TransportValue,
) -> Result<Option<EvidenceRequirement>, fixture::FixtureError> {
    let fields = exact_record(
        value,
        "evidence requirement",
        &[
            "admitted_classes",
            "context_hex",
            "context_required",
            "subject_hex",
        ],
    )?;
    let serialized_classes = fixture::sequence(field(&fields, "admitted_classes"))?
        .iter()
        .map(fixture::string)
        .collect::<Result<Vec<_>, _>>()?;
    let mut unique_classes = BTreeSet::new();
    let duplicate_classes = !serialized_classes
        .iter()
        .all(|class| unique_classes.insert(*class));
    let subject = optional_hex(field(&fields, "subject_hex"))?;
    let context = optional_hex(field(&fields, "context_hex"))?;
    let context_required = boolean(field(&fields, "context_required"), "context_required")?;
    if duplicate_classes {
        return Ok(None);
    }
    let classes = serialized_classes.into_iter().map(str::to_owned).collect();
    Ok(EvidenceRequirement::new(classes, subject, context, context_required).ok())
}

fn binding(value: &TransportValue) -> Result<Option<EvidenceBinding>, fixture::FixtureError> {
    if matches!(value, TransportValue::Null(())) {
        return Ok(None);
    }
    let entries = unique_dynamic_record(value, "evidence binding")?;
    for (name, _) in &entries {
        if !["context_hex", "evidence_class", "subject_hex"].contains(name) {
            return Err(fixture_failure(format!(
                "evidence binding has unknown field: {name}"
            )));
        }
    }
    let fields: Fields<'_> = entries.into_iter().collect();
    let evidence_class = fields
        .get("evidence_class")
        .map(|value| match value {
            TransportValue::Null(()) => Ok(None),
            TransportValue::String(value) => Ok(Some(value.clone())),
            _ => Err(fixture_failure(
                "candidate evidence class must be string or null",
            )),
        })
        .transpose()?
        .flatten();
    let subject_identity = fields
        .get("subject_hex")
        .map(|value| optional_hex(value))
        .transpose()?
        .flatten();
    let context_identity = fields
        .get("context_hex")
        .map(|value| optional_hex(value))
        .transpose()?
        .flatten();
    Ok(Some(EvidenceBinding {
        evidence_class,
        subject_identity,
        context_identity,
    }))
}

fn optional_hex(value: &TransportValue) -> Result<Option<Vec<u8>>, fixture::FixtureError> {
    let TransportValue::String(value) = value else {
        return if matches!(value, TransportValue::Null(())) {
            Ok(None)
        } else {
            Err(fixture_failure("identity hex must be string or null"))
        };
    };
    if value.len() % 2 != 0 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(fixture_failure("identity hex has invalid syntax"));
    }
    let bytes = value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("hex is ASCII");
            u8::from_str_radix(text, 16).expect("validated hexadecimal pair")
        })
        .collect();
    Ok(Some(bytes))
}

fn field<'a>(fields: &Fields<'a>, name: &str) -> &'a TransportValue {
    fields.get(name).copied().expect("exact field exists")
}

fn string_field<'a>(fields: &Fields<'a>, name: &str) -> Result<&'a str, fixture::FixtureError> {
    fixture::string(field(fields, name))
}

fn boolean(value: &TransportValue, label: &str) -> Result<bool, fixture::FixtureError> {
    match value {
        TransportValue::Boolean(value) => Ok(*value),
        _ => Err(fixture_failure(format!("{label} must be boolean"))),
    }
}

fn require_operation(fields: &Fields<'_>, expected: &str) -> Result<(), fixture::FixtureError> {
    if string_field(fields, "operation")? == expected {
        Ok(())
    } else {
        Err(fixture_failure("unexpected operation"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invocation_with_invalid_binding_field(
        field_name: &str,
        invalid_value: TransportValue,
    ) -> TransportValue {
        let mut evidence_class = string_value("check");
        let mut subject_hex = string_value("7375626a656374");
        let mut context_hex = TransportValue::Null(());
        match field_name {
            "evidence_class" => evidence_class = invalid_value,
            "subject_hex" => subject_hex = invalid_value,
            "context_hex" => context_hex = invalid_value,
            _ => unreachable!("test field inventory is closed"),
        }
        record(vec![
            (
                "binding",
                record(vec![
                    ("context_hex", context_hex),
                    ("evidence_class", evidence_class),
                    ("subject_hex", subject_hex),
                ]),
            ),
            ("operation", string_value("evaluate_binding")),
            (
                "requirement",
                record(vec![
                    (
                        "admitted_classes",
                        TransportValue::Sequence(vec![string_value("check")]),
                    ),
                    ("context_hex", TransportValue::Null(())),
                    ("context_required", TransportValue::Boolean(false)),
                    ("subject_hex", string_value("7375626a656374")),
                ]),
            ),
        ])
    }

    #[test]
    fn invalid_binding_transport_fails_before_semantic_evaluation() {
        let cases = [
            (
                "evidence_class",
                TransportValue::Boolean(true),
                "wrong evidence-class transport kind",
            ),
            (
                "subject_hex",
                TransportValue::Integer("1".to_owned()),
                "wrong subject transport kind",
            ),
            (
                "context_hex",
                TransportValue::Integer("1".to_owned()),
                "wrong ignored-context transport kind",
            ),
            (
                "subject_hex",
                string_value("zz"),
                "invalid subject hex serialization",
            ),
            (
                "context_hex",
                string_value("zz"),
                "invalid ignored-context hex serialization",
            ),
        ];
        for (field_name, invalid_value, label) in cases {
            let result = execute(&invocation_with_invalid_binding_field(
                field_name,
                invalid_value,
            ));
            assert!(result.is_err(), "{label} must remain a harness failure");
        }
        eprintln!("ISSUE66_RUST_TRANSPORT_BOUNDARY_CASES=5/5");
    }
}
