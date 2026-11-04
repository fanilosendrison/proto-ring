use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, de::Visitor};

/// Exactly JSON null, including when nested in Serde's tagged-enum content.
/// A derived unit deserializer can also accept an empty map in that context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExplicitNull;

impl<'de> Deserialize<'de> for ExplicitNull {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct NullVisitor;

        impl Visitor<'_> for NullVisitor {
            type Value = ExplicitNull;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("null")
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(ExplicitNull)
            }
        }

        deserializer.deserialize_any(NullVisitor)
    }
}
