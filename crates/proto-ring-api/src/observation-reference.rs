use serde::{Deserialize, Serialize};

use crate::{HexDigest, NativePath, NativeWordBits};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Unix,
    Windows,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationFormat {
    #[serde(rename = "proto-ring-rgs-ref-v1")]
    V1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoherenceBasis {
    OrdinaryCapture,
    HostEstablished,
}

/// Public digest reference, not an authorization or proof of a coherent cut.
/// It contains no original engine identity bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationReference {
    pub format: ObservationFormat,
    pub platform: Platform,
    pub repository: HexDigest,
    pub scope: HexDigest,
    pub state: HexDigest,
    pub coherence_basis: CoherenceBasis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileFormat {
    #[serde(rename = "ri-profile-v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileReference {
    pub format: ProfileFormat,
    pub sha256: HexDigest,
    pub native_word_bits: NativeWordBits,
}

/// Concrete paths accompany, rather than substitute for, public digest references.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationProvenance {
    pub observation: ObservationReference,
    pub repository: NativePath,
    pub scope_paths: Vec<NativePath>,
}
