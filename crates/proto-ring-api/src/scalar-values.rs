use std::fmt;

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::ModelError;

// The newtypes preserve wire strings, including decimal precision, without coercion.
macro_rules! string_scalar {
    ($name:ident, $description:literal, $validate:expr) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String")]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = ModelError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                if ($validate)(&value) {
                    Ok(Self(value))
                } else {
                    Err(ModelError::InvalidScalar($description))
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = ModelError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_from(value.to_owned())
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

string_scalar!(Identifier, "nonempty identifier", |s: &str| !s.is_empty());
string_scalar!(HexDigest, "lowercase SHA-256 hex", |s: &str| {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
});
string_scalar!(SignedDecimal, "canonical signed decimal", |s: &str| {
    let magnitude = s.strip_prefix('-').unwrap_or(s);
    s == "0"
        || (magnitude
            .as_bytes()
            .first()
            .is_some_and(|c| (b'1'..=b'9').contains(c))
            && magnitude.bytes().all(|c| c.is_ascii_digit()))
});
string_scalar!(U64Decimal, "canonical u64 decimal", |s: &str| {
    (s == "0"
        || s.as_bytes()
            .first()
            .is_some_and(|c| (b'1'..=b'9').contains(c)))
        && s.bytes().all(|c| c.is_ascii_digit())
        && s.parse::<u64>().is_ok()
});
string_scalar!(Base64Text, "canonical RFC 4648 Base64", |s: &str| {
    STANDARD
        .decode(s)
        .is_ok_and(|bytes| STANDARD.encode(bytes) == s)
});
