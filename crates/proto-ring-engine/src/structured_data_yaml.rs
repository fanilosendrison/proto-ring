#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use yaml_rust2::parser::{Event, EventReceiver, Parser};
use yaml_rust2::scanner::TScalarStyle;

use crate::structured_data::{StructuredDataError, StructuredValue, canonical_integer, rejected};

struct EventSink(Vec<Event>);

impl EventReceiver for EventSink {
    fn on_event(&mut self, event: Event) {
        self.0.push(event);
    }
}

pub(crate) fn parse(text: &str) -> Result<StructuredValue, StructuredDataError> {
    let mut sink = EventSink(Vec::new());
    Parser::new_from_str(text)
        .load(&mut sink, true)
        .map_err(|error| rejected(format!("invalid YAML syntax: {error}")))?;
    let document_count = sink
        .0
        .iter()
        .filter(|event| matches!(event, Event::DocumentStart))
        .count();
    if document_count != 1 {
        return Err(rejected("exactly one YAML document is required"));
    }
    let start = sink
        .0
        .iter()
        .position(|event| matches!(event, Event::DocumentStart))
        .ok_or_else(|| rejected("document start is missing"))?
        + 1;
    let mut cursor = start;
    let value = construct(&sink.0, &mut cursor)?;
    if !matches!(sink.0.get(cursor), Some(Event::DocumentEnd)) {
        return Err(rejected("document has residual content"));
    }
    Ok(value)
}

fn construct(events: &[Event], cursor: &mut usize) -> Result<StructuredValue, StructuredDataError> {
    let event = events
        .get(*cursor)
        .ok_or_else(|| rejected("unexpected end of YAML events"))?;
    *cursor += 1;
    match event {
        Event::Scalar(value, style, anchor, tag) => {
            require_unadorned(*anchor, tag.is_some())?;
            scalar(value, *style)
        }
        Event::SequenceStart(anchor, tag) => {
            require_unadorned(*anchor, tag.is_some())?;
            let mut values = Vec::new();
            while !matches!(events.get(*cursor), Some(Event::SequenceEnd)) {
                values.push(construct(events, cursor)?);
            }
            *cursor += 1;
            Ok(StructuredValue::Sequence(values))
        }
        Event::MappingStart(anchor, tag) => {
            require_unadorned(*anchor, tag.is_some())?;
            mapping(events, cursor)
        }
        Event::Alias(_) => Err(rejected("aliases are forbidden")),
        _ => Err(rejected("unexpected YAML event")),
    }
}

fn mapping(events: &[Event], cursor: &mut usize) -> Result<StructuredValue, StructuredDataError> {
    let mut values = BTreeMap::new();
    while !matches!(events.get(*cursor), Some(Event::MappingEnd)) {
        let key = match construct(events, cursor)? {
            StructuredValue::String(value) => value,
            _ => return Err(rejected("mapping keys must construct strings")),
        };
        if key == "<<" {
            return Err(rejected("merge keys are forbidden"));
        }
        let value = construct(events, cursor)?;
        if values.insert(key.clone(), value).is_some() {
            return Err(rejected(format!("duplicate mapping key: {key}")));
        }
    }
    *cursor += 1;
    Ok(StructuredValue::Mapping(values))
}

fn scalar(value: &str, style: TScalarStyle) -> Result<StructuredValue, StructuredDataError> {
    match style {
        TScalarStyle::Literal | TScalarStyle::Folded => {
            Err(rejected("block scalars are forbidden"))
        }
        TScalarStyle::SingleQuoted | TScalarStyle::DoubleQuoted => {
            Ok(StructuredValue::String(value.to_owned()))
        }
        TScalarStyle::Plain => {
            if value.is_empty() {
                return Err(rejected("omitted or empty plain scalar is forbidden"));
            }
            Ok(match value {
                "true" => StructuredValue::Boolean(true),
                "false" => StructuredValue::Boolean(false),
                "null" => StructuredValue::Null,
                value if canonical_integer(value) => StructuredValue::Integer(value.to_owned()),
                value => StructuredValue::String(value.to_owned()),
            })
        }
    }
}

fn require_unadorned(anchor: usize, tagged: bool) -> Result<(), StructuredDataError> {
    if anchor != 0 {
        return Err(rejected("anchors are forbidden"));
    }
    if tagged {
        return Err(rejected("explicit tags are forbidden"));
    }
    Ok(())
}
