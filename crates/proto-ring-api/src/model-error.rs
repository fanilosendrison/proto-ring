use std::fmt;

/// Invalid public model construction; not a governance verdict or wire error code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelError {
    InvalidScalar(&'static str),
    Inconsistent(&'static str),
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScalar(name) => write!(formatter, "invalid {name}"),
            Self::Inconsistent(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for ModelError {}
