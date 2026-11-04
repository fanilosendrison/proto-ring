use serde::{Deserialize, Serialize};

use crate::{ModelError, SignedDecimal};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StructuredValue {
    Null {},
    Boolean { value: bool },
    Integer { decimal: SignedDecimal },
    String { value: String },
    Sequence { items: Vec<StructuredValue> },
    Mapping { entries: MappingEntries },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingEntry {
    // Opaque configuration keys are not identifiers: the empty string is valid.
    pub key: String,
    pub value: StructuredValue,
}

/// Entries in unique Rust string lexical order. Construction never silently
/// sorts or deduplicates an incoming observation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<MappingEntry>")]
pub struct MappingEntries(Vec<MappingEntry>);

impl MappingEntries {
    pub fn as_slice(&self) -> &[MappingEntry] {
        &self.0
    }
}

impl TryFrom<Vec<MappingEntry>> for MappingEntries {
    type Error = ModelError;

    fn try_from(entries: Vec<MappingEntry>) -> Result<Self, Self::Error> {
        if entries.windows(2).all(|pair| pair[0].key < pair[1].key) {
            Ok(Self(entries))
        } else {
            Err(ModelError::Inconsistent(
                "mapping entries must have unique sorted keys",
            ))
        }
    }
}

impl From<MappingEntries> for Vec<MappingEntry> {
    fn from(entries: MappingEntries) -> Self {
        entries.0
    }
}
