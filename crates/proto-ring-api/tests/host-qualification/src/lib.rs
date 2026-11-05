#![forbid(unsafe_code)]
//! Isolated experiment instrument. No product crate depends on this package.
#[path = "campaign-context.rs"]
pub mod campaign_context;
#[path = "case-verification.rs"]
pub mod case_verification;
pub mod cleanup;
pub mod coverage;
pub mod docker;
#[path = "environment-evidence.rs"]
pub mod environment_evidence;
pub mod evidence;
#[path = "filesystem-identity.rs"]
pub mod filesystem_identity;
#[path = "launch-policy.rs"]
pub mod launch_policy;
pub mod lifetime;
#[path = "mutation-evidence.rs"]
pub mod mutation_evidence;
pub mod policy;
pub mod probes;
#[path = "process-evidence.rs"]
pub mod process_evidence;
pub mod rendezvous;
#[path = "scope-evidence.rs"]
pub mod scope_evidence;
pub mod source;
#[path = "source-state.rs"]
pub mod source_state;
pub mod subject;
pub mod transport;
pub mod verification;
#[path = "write-probes.rs"]
mod write_probes;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
}
