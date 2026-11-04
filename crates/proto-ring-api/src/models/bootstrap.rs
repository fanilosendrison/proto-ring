use serde::{Deserialize, Serialize};

use crate::{NativePath, StructuredValue};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapProjection {
    pub metadata: StructuredValue,
    pub repository_governance: StructuredValue,
    pub carrier: NativePath,
}
