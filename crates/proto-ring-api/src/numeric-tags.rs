use serde::{Deserialize, Serialize};

use crate::ModelError;

macro_rules! numeric_enum {
    ($name:ident { $($variant:ident = $value:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "u64", into = "u64")]
        pub enum $name { $($variant),+ }

        impl TryFrom<u64> for $name {
            type Error = ModelError;

            fn try_from(value: u64) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(ModelError::InvalidScalar(stringify!($name))),
                }
            }
        }

        impl From<$name> for u64 {
            fn from(value: $name) -> Self {
                match value { $($name::$variant => $value),+ }
            }
        }
    };
}

numeric_enum!(VersionOne { V1 = 1 });
numeric_enum!(ModelVersion { V1 = 1, V2 = 2 });
numeric_enum!(NativeWordBits { Bits32 = 32, Bits64 = 64 });

/// Output request size, represented as an exact JSON integer, not decimal text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct ReadLimit(u32);

impl TryFrom<u32> for ReadLimit {
    type Error = ModelError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (1..=65_536).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ModelError::InvalidScalar("output max_bytes"))
        }
    }
}

impl From<ReadLimit> for u32 {
    fn from(value: ReadLimit) -> Self {
        value.0
    }
}
