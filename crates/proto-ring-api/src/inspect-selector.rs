use serde::{Deserialize, Serialize};

use crate::{ExplainQuery, Identifier, ModelError, Target};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InspectSelector {
    All {},
    Bootstrap {},
    Model {},
    Authority {},
    GovernedObjects {},
    Bindings {},
    RepositoryIntegrity {},
    Projections {},
    EvidenceRequirements {},
    Capability { id: Identifier },
    Validation { id: Identifier },
    Binding { id: Identifier },
    GovernedObject { target: Target },
}

impl TryFrom<ExplainQuery> for InspectSelector {
    type Error = ModelError;

    fn try_from(query: ExplainQuery) -> Result<Self, Self::Error> {
        match query {
            ExplainQuery::Capability { id } => Ok(Self::Capability { id }),
            ExplainQuery::Validation { id } => Ok(Self::Validation { id }),
            ExplainQuery::Binding { id } => Ok(Self::Binding { id }),
            ExplainQuery::GovernedObject { target } => Ok(Self::GovernedObject { target }),
            _ => Err(ModelError::Inconsistent(
                "subject has no v1 inspection selector",
            )),
        }
    }
}
