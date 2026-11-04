use proto_ring_engine::governance_authority as engine;

use super::{
    declaration_fields::{id, path},
    governance_model::route,
};
use crate::{
    AuthorityProjection, AuthorityRole, AuthoritySource, ModelError, Precedence, Responsibility,
    VersionOne,
};

pub(crate) fn authority(
    value: &engine::GovernanceAuthorityProfile,
) -> Result<AuthorityProjection, ModelError> {
    Ok(AuthorityProjection {
        model_version: VersionOne::try_from(u64::from(value.model_version))?,
        sources: value
            .sources
            .iter()
            .map(|(key, source)| {
                Ok((
                    id(key)?,
                    AuthoritySource {
                        repository_target: source
                            .repository_target
                            .as_ref()
                            .map(route)
                            .transpose()?,
                    },
                ))
            })
            .collect::<Result<_, ModelError>>()?,
        responsibilities: value
            .responsibilities
            .iter()
            .map(|(key, responsibility)| {
                Ok((
                    id(key)?,
                    Responsibility {
                        roles: responsibility
                            .roles
                            .iter()
                            .map(|(source, role)| {
                                Ok((
                                    id(source)?,
                                    match role {
                                        engine::SourceRole::Authority => AuthorityRole::Authority,
                                        engine::SourceRole::SecondaryRepresentation => {
                                            AuthorityRole::SecondaryRepresentation
                                        }
                                        engine::SourceRole::NonAuthoritative => {
                                            AuthorityRole::NonAuthoritative
                                        }
                                    },
                                ))
                            })
                            .collect::<Result<_, ModelError>>()?,
                        precedence: responsibility
                            .precedence
                            .iter()
                            .map(|edge| {
                                Ok(Precedence {
                                    higher_source: id(&edge.higher_source)?,
                                    lower_source: id(&edge.lower_source)?,
                                })
                            })
                            .collect::<Result<_, ModelError>>()?,
                    },
                ))
            })
            .collect::<Result<_, ModelError>>()?,
        carrier: path(&value.carrier)?,
    })
}
