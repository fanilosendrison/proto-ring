use std::fmt;

/// Exact path conversion failure. Diagnostics never contain rejected path units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePathError {
    EmptyPath,
    NulUnit,
    ForeignPlatform,
    OddWindowsByteLength,
    InvalidBase64,
    UnsupportedPlatform,
}

impl fmt::Display for NativePathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyPath => "native path is empty",
            Self::NulUnit => "native path contains a NUL unit",
            Self::ForeignPlatform => "native path variant belongs to another platform",
            Self::OddWindowsByteLength => "Windows native path has an odd byte length",
            Self::InvalidBase64 => "native path Base64 is not canonical",
            Self::UnsupportedPlatform => "native path platform is unsupported",
        })
    }
}

impl std::error::Error for NativePathError {}
