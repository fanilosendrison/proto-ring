#![forbid(unsafe_code)]

use crate::model::{ComparisonKind, ComparisonRule, Observation, RecordEntry, TransportValue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonOutcome {
    Match,
    Mismatch,
}

pub fn compare_observation(
    actual: &Observation,
    expected: &Observation,
    rule: &ComparisonRule,
) -> ComparisonOutcome {
    match rule.kind {
        ComparisonKind::Exact => {
            if actual.kind == expected.kind && transport_equal(&actual.value, &expected.value) {
                ComparisonOutcome::Match
            } else {
                ComparisonOutcome::Mismatch
            }
        }
    }
}

fn transport_equal(left: &TransportValue, right: &TransportValue) -> bool {
    match (left, right) {
        (TransportValue::Null(()), TransportValue::Null(())) => true,
        (TransportValue::Boolean(left), TransportValue::Boolean(right)) => left == right,
        (TransportValue::Integer(left), TransportValue::Integer(right)) => left == right,
        (TransportValue::String(left), TransportValue::String(right)) => left == right,
        (TransportValue::Bytes(left), TransportValue::Bytes(right)) => left == right,
        (TransportValue::Sequence(left), TransportValue::Sequence(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| transport_equal(left, right))
        }
        (TransportValue::Record(left), TransportValue::Record(right)) => {
            record_entries_equal(left, right)
        }
        _ => false,
    }
}

fn record_entries_equal(left: &[RecordEntry], right: &[RecordEntry]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.name == right.name && transport_equal(&left.value, &right.value)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ObservationKind, RecordEntry};

    fn observation(kind: ObservationKind, value: TransportValue) -> Observation {
        Observation { kind, value }
    }

    fn exact(left: Observation, right: Observation) -> ComparisonOutcome {
        compare_observation(
            &left,
            &right,
            &ComparisonRule {
                kind: ComparisonKind::Exact,
            },
        )
    }

    #[test]
    fn integer_and_string_are_distinct() {
        assert_eq!(
            exact(
                observation(ObservationKind::Result, TransportValue::Integer("1".into())),
                observation(ObservationKind::Result, TransportValue::String("1".into())),
            ),
            ComparisonOutcome::Mismatch
        );
    }

    #[test]
    fn bytes_and_string_are_distinct() {
        assert_eq!(
            exact(
                observation(ObservationKind::Result, TransportValue::Bytes("31".into())),
                observation(ObservationKind::Result, TransportValue::String("1".into())),
            ),
            ComparisonOutcome::Mismatch
        );
    }

    #[test]
    fn sequence_order_is_observable() {
        let left = TransportValue::Sequence(vec![
            TransportValue::String("a".into()),
            TransportValue::String("b".into()),
        ]);
        let right = TransportValue::Sequence(vec![
            TransportValue::String("b".into()),
            TransportValue::String("a".into()),
        ]);
        assert_eq!(
            exact(
                observation(ObservationKind::Result, left),
                observation(ObservationKind::Result, right),
            ),
            ComparisonOutcome::Mismatch
        );
    }

    fn entry(name: &str, value: &str) -> RecordEntry {
        RecordEntry {
            name: name.into(),
            value: TransportValue::String(value.into()),
        }
    }

    #[test]
    fn record_order_is_observable() {
        let left = TransportValue::Record(vec![entry("a", "1"), entry("b", "2")]);
        let right = TransportValue::Record(vec![entry("b", "2"), entry("a", "1")]);
        assert_eq!(
            exact(
                observation(ObservationKind::Result, left),
                observation(ObservationKind::Result, right),
            ),
            ComparisonOutcome::Mismatch
        );
    }

    #[test]
    fn record_multiplicity_is_observable() {
        let left = TransportValue::Record(vec![entry("a", "1"), entry("a", "1")]);
        let right = TransportValue::Record(vec![entry("a", "1")]);
        assert_eq!(
            exact(
                observation(ObservationKind::Result, left),
                observation(ObservationKind::Result, right),
            ),
            ComparisonOutcome::Mismatch
        );
    }

    #[test]
    fn observation_kind_is_observable() {
        let value = TransportValue::Null(());
        assert_eq!(
            exact(
                observation(ObservationKind::Result, value.clone()),
                observation(ObservationKind::ControlledRejection, value),
            ),
            ComparisonOutcome::Mismatch
        );
    }

    #[test]
    fn identical_typed_observations_match() {
        let value = TransportValue::Record(vec![entry("answer", "same")]);
        assert_eq!(
            exact(
                observation(ObservationKind::Result, value.clone()),
                observation(ObservationKind::Result, value),
            ),
            ComparisonOutcome::Match
        );
    }
}
