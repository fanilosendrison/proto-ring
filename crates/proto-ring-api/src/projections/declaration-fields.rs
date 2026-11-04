use std::path::Path;

use proto_ring_engine::governed_objects::GovernedObjectRef;

use crate::{AuthorityReference, Identifier, ModelError, NativePath, Target};

pub(super) fn id(value: &str) -> Result<Identifier, ModelError> {
    Identifier::try_from(value)
}

pub(super) fn ids<'a>(
    values: impl IntoIterator<Item = &'a String>,
) -> Result<Vec<Identifier>, ModelError> {
    values.into_iter().map(|value| id(value)).collect()
}

pub(crate) fn path(value: &Path) -> Result<NativePath, ModelError> {
    NativePath::from_path(value).map_err(|_| ModelError::InvalidScalar("loaded carrier path"))
}

pub(crate) fn authority(
    responsibility: &str,
    source: &str,
) -> Result<AuthorityReference, ModelError> {
    Ok(AuthorityReference {
        responsibility: id(responsibility)?,
        source: id(source)?,
    })
}

pub(super) fn target(value: &GovernedObjectRef) -> Result<Target, ModelError> {
    target_ids(&value.interface_id, &value.object_id)
}

pub(super) fn target_ids(interface: &str, object: &str) -> Result<Target, ModelError> {
    Ok(Target {
        interface: id(interface)?,
        object: id(object)?,
    })
}
