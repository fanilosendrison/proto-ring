use proto_ring_engine::governed_objects as engine;

use super::declaration_fields::{id, ids, path, target};
use crate::{
    GovernedInterface, GovernedObjectDeclaration, ModelError, ObjectRelation, ObjectsProjection,
    VersionOne,
};

pub(crate) fn governed_object(
    value: &engine::GovernedObject,
) -> Result<GovernedObjectDeclaration, ModelError> {
    Ok(GovernedObjectDeclaration {
        responsibilities: ids(&value.responsibilities)?,
        relations: value
            .relations
            .iter()
            .map(|relation| {
                Ok(ObjectRelation {
                    relation: id(&relation.relation)?,
                    responsibility: id(&relation.responsibility)?,
                    target: target(&relation.target)?,
                })
            })
            .collect::<Result<_, ModelError>>()?,
    })
}

pub(crate) fn objects(
    value: &engine::GovernedObjectCatalog,
) -> Result<ObjectsProjection, ModelError> {
    Ok(ObjectsProjection {
        model_version: VersionOne::try_from(u64::from(value.model_version))?,
        interfaces: value
            .interfaces
            .iter()
            .map(|(key, interface)| {
                Ok((
                    id(key)?,
                    GovernedInterface {
                        objects: interface
                            .objects
                            .iter()
                            .map(|(key, value)| Ok((id(key)?, governed_object(value)?)))
                            .collect::<Result<_, ModelError>>()?,
                    },
                ))
            })
            .collect::<Result<_, ModelError>>()?,
        carrier: path(&value.carrier)?,
    })
}
