mod identity;
mod load;
mod model;
mod selectors;

use std::error::Error;
use std::fmt::{Display, Formatter};

pub use load::load;
pub use model::{
    CommandBinding, ConsumerIntegrityProfile, ProfileAuthority, ProfileIdentity,
    RepositoryPathSelector, RepositoryPathsMode, SelectorKind, ValidationDefinition,
    ValidationInstances,
};

#[derive(Debug)]
pub struct RepositoryIntegrityError(String);
impl Display for RepositoryIntegrityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl Error for RepositoryIntegrityError {}
pub(super) fn failure(message: impl Into<String>) -> RepositoryIntegrityError {
    RepositoryIntegrityError(message.into())
}
