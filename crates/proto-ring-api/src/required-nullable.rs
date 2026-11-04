use serde::{Deserialize, Deserializer};

/// A nullable wire property must still be present. Serde's ordinary `Option`
/// handling otherwise treats an omitted field as if it were explicitly null.
pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}
