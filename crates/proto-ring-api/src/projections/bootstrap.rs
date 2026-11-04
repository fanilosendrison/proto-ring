use proto_ring_engine::repository_governance_state::RepositoryGovernanceState;

use super::{
    declaration_fields::path,
    structured_value::{structured_mapping, structured_value},
};
use crate::{BootstrapProjection, ModelError};

pub(crate) fn bootstrap(
    state: &RepositoryGovernanceState,
) -> Result<BootstrapProjection, ModelError> {
    Ok(BootstrapProjection {
        metadata: structured_value(&state.bootstrap.metadata)?,
        repository_governance: structured_mapping(&state.bootstrap.repository_governance)?,
        // The loaded bootstrap's specified location; no reread or resolution.
        carrier: path(&state.repository.join("AGENTS.md"))?,
    })
}
