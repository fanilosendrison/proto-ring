use crate::fixture;
use crate::model::{Observation, TransportValue};
use crate::rust_candidate::transport::{exact_record, record, rejection, result, string_value};
use proto_ring_engine::portable_pattern::{PortablePattern, PortablePatternMatch};

const RESPONSIBILITY: &str = "portable-pattern.full-match";

pub(super) fn execute(arguments: &TransportValue) -> Result<Observation, fixture::FixtureError> {
    let fields = exact_record(
        arguments,
        "Portable Pattern invocation",
        &["candidate", "operation", "pattern"],
    )?;
    let operation = fixture::string(fields["operation"])?;
    if operation != "full_match" {
        return Err(fixture_failure(
            "Portable Pattern operation must be full_match",
        ));
    }
    let source = fixture::string(fields["pattern"])?;
    let candidate = fixture::string(fields["candidate"])?;
    let pattern = match PortablePattern::compile(source) {
        Ok(pattern) => pattern,
        Err(_) => return Ok(rejection(RESPONSIBILITY)),
    };
    match pattern.full_match(candidate) {
        Ok(Some(matched)) => Ok(success(&pattern, &matched)),
        Ok(None) => Ok(result(record(vec![
            ("captures", TransportValue::Sequence(Vec::new())),
            ("matched", TransportValue::Boolean(false)),
        ]))),
        Err(_) => Ok(rejection(RESPONSIBILITY)),
    }
}

fn success(pattern: &PortablePattern, matched: &PortablePatternMatch) -> Observation {
    let captures = (1..=pattern.capture_count())
        .map(|index| capture_value(matched.positional_capture(index)))
        .collect();
    let named_captures = pattern
        .capture_names()
        .map(|(name, index)| crate::model::RecordEntry {
            name: name.to_owned(),
            value: capture_value(matched.positional_capture(index)),
        })
        .collect();
    result(record(vec![
        ("captures", TransportValue::Sequence(captures)),
        ("matched", TransportValue::Boolean(true)),
        ("named_captures", TransportValue::Record(named_captures)),
    ]))
}

fn capture_value(capture: Option<&str>) -> TransportValue {
    capture.map_or(TransportValue::Null(()), string_value)
}

fn fixture_failure(message: &str) -> fixture::FixtureError {
    fixture::FixtureError::new(message)
}
