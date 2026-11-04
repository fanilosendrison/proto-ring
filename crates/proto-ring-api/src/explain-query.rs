use serde::{Deserialize, Serialize};

use crate::{Identifier, Target};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExplainQuery {
    AuthorityRole {
        responsibility: Identifier,
        source: Identifier,
    },
    AuthorityPrecedence {
        responsibility: Identifier,
        higher_source: Identifier,
        lower_source: Identifier,
    },
    Capability {
        id: Identifier,
    },
    Validation {
        id: Identifier,
    },
    Binding {
        id: Identifier,
    },
    GovernedObject {
        target: Target,
    },
    Projection {
        id: Identifier,
    },
    EvidenceRequirement {
        id: Identifier,
    },
}
